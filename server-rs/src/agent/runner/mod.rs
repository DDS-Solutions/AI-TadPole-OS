//! Agent Execution Runtime & Swarm Orchestration
//!
//! This module implements the core runner responsible for taking an agent mission,
//! resolving its context, synthesized prompts, and executing it via the selected
//! LLM provider while managing tool invocation and swarm expansion.
//!
//! Developed with a local-first, high-performance focus using Tokio.
//! The lifecycle includes context RAG, tool orchestration, and recursive recruitment.
use crate::agent::types::{ModelConfig, TaskPayload};
use crate::state::AppState;
use std::sync::Arc;

mod provider;
mod swarm;
mod synthesis;
mod tools;
mod fs_tools;
mod mission_tools;
mod external_tools;
mod analysis;

use analysis::spawn_post_mission_analysis;

/// Context bag for data resolved during the setup phase of a run.
///
/// This struct aggregates all state required for an agent execution session,
/// including identity, governance limits, workspace paths, and telemetry markers.
/// Avoids the "argument explosion" anti-pattern in internal runner methods.
#[derive(Clone)]
pub(crate) struct RunContext {
    /// Unique identifier for the agent instance.
    pub agent_id: String,
    /// Friendly name for display and logs.
    pub name: String,
    /// The agent's functional role (e.g., "Developer").
    pub role: String,
    /// The organizational unit (e.g., "Standard-Core").
    pub department: String,
    /// High-level personality and utility instructions.
    pub description: String,
    /// Config for the specific model assigned to this run.
    pub model_config: ModelConfig,
    /// List of enabled skill IDs (tools).
    pub skills: Vec<String>,
    /// List of enabled workflow IDs (procedures).
    pub workflows: Vec<String>,
    /// List of enabled MCP tool IDs.
    #[allow(dead_code)]
    pub mcp_tools: Vec<String>,
    /// The unique ID of the mission cluster.
    pub mission_id: String,
    /// Optional ID of the user who initiated the request.
    pub user_id: Option<String>,
    /// Current recursion depth in the swarm.
    pub depth: u32,
    /// Ancestry path of nodes that led to this one.
    pub lineage: Vec<String>,
    /// Normalized name of the LLM provider (e.g., "openai").
    pub provider_name: String,
    /// Physical path to the cluster's data sandbox.
    pub workspace_root: std::path::PathBuf,
    /// Secure filesystem bridge for the workspace.
    pub fs_adapter: crate::adapter::filesystem::FilesystemAdapter,
    /// If true, mutation and recruitment tools are disabled.
    pub safe_mode: bool,
    /// If true, performs additional iterative refinement.
    pub analysis: bool,
    /// OTel traceparent for cross-service tracing.
    pub traceparent: Option<String>,
    /// Tracks the last few files accessed for context propagation.
    pub last_accessed_files: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// Summary of recent findings from previous turns or parents.
    pub recent_findings: Option<String>,
}

impl RunContext {
    /// Resolves common workspace and data paths for the current mission.
    pub fn resolve_paths(&self) -> (String, String, String) {
        let cluster_name = self
            .workspace_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let agent_memory_dir = format!(
            "data/workspaces/{}/agents/{}/memory.lance",
            cluster_name, self.agent_id
        );
        let mission_scope_dir = format!(
            "data/workspaces/{}/missions/{}/scope.lance",
            cluster_name, self.mission_id
        );

        (cluster_name, agent_memory_dir, mission_scope_dir)
    }

    /// Derives a TaskPayload for a recursive sub-agent recruitment.
    /// Safely propagates traceparents and parent context.
    pub fn derive_subtask_payload(&self, message: String) -> TaskPayload {
        TaskPayload {
            message,
            cluster_id: Some(self.mission_id.clone()),
            department: None,
            provider: Some(self.model_config.provider.clone()),
            model_id: Some(self.model_config.model_id.clone()),
            api_key: self.model_config.api_key.clone(),
            base_url: self.model_config.base_url.clone(),
            rpm: self.model_config.rpm,
            tpm: self.model_config.tpm,
            budget_usd: None,
            swarm_depth: Some(self.depth + 1),
            swarm_lineage: Some({
                let mut l = self.lineage.clone();
                l.push(self.agent_id.clone());
                l
            }),
            external_id: self.model_config.external_id.clone(),
            safe_mode: Some(self.safe_mode),
            analysis: None,
            traceparent: self.traceparent.clone(),
            user_id: self.user_id.clone(),
            context_files: Some(self.last_accessed_files.lock().unwrap().clone()),
            recent_findings: self.recent_findings.clone(),
        }
    }
}


/// The primary entry point for executing autonomous agent tasks.
///
/// The `AgentRunner` wraps the shared `AppState` and provides methods for
/// orchestrating the full lifecycle of a mission, from prompt synthesis to
/// final result persistence.
#[derive(Clone)]
pub struct AgentRunner {
    /// Reference to the global application state.
    pub state: Arc<AppState>,
}

impl AgentRunner {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub(crate) fn broadcast_sys(&self, msg: &str, level: &str) {
        self.state.broadcast_sys(msg, level);
    }

    /// Centralized mission failure handler. Sets status, logs error, and updates agent health.
    pub(crate) async fn fail_mission(
        &self,
        ctx: &RunContext,
        e: &anyhow::Error,
        usage: &Option<crate::agent::types::TokenUsage>,
    ) -> anyhow::Result<()> {
        let safe_error = self.state.security.secret_redactor.redact(&format!("{}", e));
        tracing::error!(
            "❌ [Runner] Mission failure for agent {}: {}",
            ctx.agent_id,
            safe_error
        );
        self.broadcast_agent_message(&ctx.agent_id, &format!("❌ Error: {}", safe_error));

        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();
            agent.failure_count += 1;
            agent.last_failure_at = Some(chrono::Utc::now());

            let agent_data = agent.clone();
            drop(entry); // Release DashMap lock before async calls

            // Sync to DB
            let _ = crate::agent::persistence::save_agent_db(&self.state.resources.pool, &agent_data).await;

            self.state.emit_event(serde_json::json!({
                "type": "agent:update",
                "data": agent_data
            }));

            // Record final cost if usage was provided
            if let Some(u) = usage {
                let turn_cost = crate::agent::rates::calculate_cost(&ctx.model_config.model_id, u.input_tokens, u.output_tokens);
                let budget_guard = self.state.security.budget_guard.clone();
                let agent_id = ctx.agent_id.clone();
                tokio::spawn(async move {
                    let _ = budget_guard.record_usage(&agent_id, turn_cost).await;
                });
            }
        }

        // 🕵️ UNIFIED STATUS SYNC
        // We use update_status AFTER the registry block to ensure the lock is released
        // and telemetry is broadcast correctly for the "Idle" state.
        self.update_status(&ctx.agent_id, "idle", None);

        crate::agent::mission::update_mission(
            &self.state.resources.pool,
            &ctx.mission_id,
            crate::agent::types::MissionStatus::Failed,
            0.0,
        )
        .await?;

        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &ctx.mission_id,
            &ctx.agent_id,
            "System",
            &format!("❌ Error: {}", safe_error),
            "error",
            None,
        )
        .await?;

        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    //  MAIN ENTRY POINT
    // ─────────────────────────────────────────────────────────

    /// The core execution loop for a mission.
    /// 
    /// This is the primary entry point for all agent actions. It orchestrates:
    /// 1. **Validation**: Checks task length, budget, and recursion depth.
    /// 2. **Context**: Gathers long-term memory, system architecture, and swarm findings.
    /// 3. **Intelligence**: Calls the LLM provider and dispatches tools.
    /// 4. **Persistence**: Saves results to SQL and Merkle-signed audit trails.
    #[tracing::instrument(
        name = "agent_run",
        skip(self, payload),
        fields(
            agent_id = %agent_id,
            mission_id = %payload.cluster_id.as_deref().unwrap_or("unknown"),
            swarm_depth = payload.swarm_depth.unwrap_or(0),
            trace_id = tracing::field::Empty
        )
    )]
    pub async fn run(&self, agent_id: String, payload: TaskPayload) -> anyhow::Result<String> {
        self.state.yield_phase_transition(&agent_id, "Setup").await;
        // 1. Setup tracing, validation and security
        self.setup_and_validate(&agent_id, &payload)?;

        self.state.yield_phase_transition(&agent_id, "Initialization").await;
        // 2. Initialize mission and monitoring state
        let mission = self.initialize_mission_state(&agent_id, &payload).await?;
        let mission_id = mission.id.clone();

        self.state.yield_phase_transition(&agent_id, "ContextResolution").await;
        // 3. Resolve context and prepare runtime environment (memory/RAG)
        let depth = payload.swarm_depth.unwrap_or(0);
        let lineage = payload.swarm_lineage.clone().unwrap_or_default();
        let ctx = self
            .prepare_run_context(&agent_id, &payload, &mission_id, depth, &lineage)
            .await?;

        self.state.yield_phase_transition(&agent_id, "IntelligenceLoop").await;
        // 4. Intelligence Loop: Prompt -> Provider -> Tool Execution
        let output_res = self.execute_intelligence_loop(&ctx, &payload).await;

        match output_res {
            Ok(output) => {
                self.state.yield_phase_transition(&agent_id, "Finalization").await;
                // 5. Finalize results and cleanup
                let final_res = self.finalize_run(&ctx, &output.text, &output.usage).await;
                self.state.governance.active_agents
                    .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                final_res
            }
            Err((e, usage)) => {
                self.fail_mission(&ctx, &e, &usage).await?;
                self.state.governance.active_agents
                    .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// Performs initial safety and tracing setup.
    /// 
    /// - Extracts OTel `traceparent` for cross-service observability.
    /// - Scans user input for prompt injection via the `Sanitizer`.
    /// - Checks global health metrics and recursion lineage.
    fn setup_and_validate(&self, agent_id: &str, payload: &TaskPayload) -> anyhow::Result<()> {
        let span = tracing::Span::current();

        // Setup OTel Trace Propagation
        if let Some(tp) = &payload.traceparent {
            use opentelemetry::global;
            use tracing_opentelemetry::OpenTelemetrySpanExt;

            let mut extractor = std::collections::HashMap::new();
            extractor.insert("traceparent".to_string(), tp.clone());
            let parent_cx = global::get_text_map_propagator(|prop| prop.extract(&extractor));
            span.set_parent(parent_cx);

            // Parse trace_id directly from traceparent (00-traceid-spanid-01)
            let parts: Vec<&str> = tp.split('-').collect();
            if parts.len() >= 4 {
                span.record("trace_id", parts[1]);
            } else {
                use opentelemetry::trace::TraceContextExt;
                let otel_trace_id = span.context().span().span_context().trace_id().to_string();
                span.record("trace_id", &otel_trace_id);
            }
        } else {
            use opentelemetry::trace::TraceContextExt;
            use tracing_opentelemetry::OpenTelemetrySpanExt;
            let otel_trace_id = span.context().span().span_context().trace_id().to_string();
            span.record("trace_id", &otel_trace_id);
        }

        // Core validation logic
        self.validate_input(agent_id, payload)?;

        // Security Sanitization
        if let crate::agent::sanitizer::SanitizationResult::Alert(msg) =
            crate::agent::sanitizer::Sanitizer::scan(&payload.message)
        {
            tracing::warn!("🛡️ [Security] Blocked potential injection: {}", msg);
            self.state
                .broadcast_sys(&format!("🛡️ Security: {}", msg), "error");
            return Err(anyhow::anyhow!("❌ Security Violation: {}", msg));
        }

        Ok(())
    }

    /// Registers a new mission in the database and enforces budget gates.
    /// 
    /// This method ensures the agent's failure rate is within healthy limits and 
    /// that sufficient USD credits remain in the persistent `BudgetGuard` 
    /// before any tokens are consumed.
    async fn initialize_mission_state(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
    ) -> anyhow::Result<crate::agent::types::Mission> {
        // 🛡️ [Resilience] Ensure agent exists in database (auto-sync if only in registry)
        let db_pool = &self.state.resources.pool;
        let db_check: Option<i64> = sqlx::query_scalar("SELECT 1 FROM agents WHERE id = ?")
            .bind(agent_id)
            .fetch_optional(db_pool)
            .await
            .unwrap_or(None);

        if db_check.is_none() {
            if let Some(agent) = self.state.registry.agents.get(agent_id) {
                tracing::info!("🔄 [Runner] Agent {} not in DB but found in registry. Self-healing...", agent_id);
                let _ = crate::agent::persistence::save_agent_db(db_pool, agent.value()).await;
            }
        }

        self.state.governance.active_agents
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let depth = payload.swarm_depth.unwrap_or(0);
        let _ = self.state.governance.max_swarm_depth
            .fetch_max(depth, std::sync::atomic::Ordering::Relaxed);

        let mission_title = payload.message.chars().take(50).collect::<String>() + "...";

        // 🏥 [Health Check] Failure Rate Throttling
        if let Some(agent) = self.state.registry.agents.get(agent_id) {
            if agent.value().failure_count >= 5 {
                let last_fail = agent.value().last_failure_at.unwrap_or_else(chrono::Utc::now);
                let cooldown = chrono::Duration::minutes(15);
                if chrono::Utc::now() - last_fail < cooldown {
                    tracing::warn!("🏥 [Health] Agent {} is degraded (Failure Count: {})", agent_id, agent.value().failure_count);
                    self.state.broadcast_sys(
                        &format!("🏥 Health: {} is in self-heal cooldown (Failure Count: {}).", agent_id, agent.value().failure_count),
                        "warning",
                    );
                    return Err(anyhow::anyhow!("❌ Agent Degraded. Self-heal cooldown active."));
                }
            }
        }

        // [Budget Gate] Persistent Budget Check (OpenFang Pattern)
        let has_budget = self.state.security.budget_guard.check_budget(agent_id, 0.0).await?;
        if !has_budget {
            tracing::warn!("🛡️ [Governance] Agent {} persistent budget exhausted", agent_id);
            self.state.broadcast_sys(
                &format!("🛡️ Budget Guard: {} has exceeded its periodic quota.", agent_id),
                "error",
            );
            return Err(anyhow::anyhow!("❌ Persistent Quota Exhausted. Check Security Dashboard."));
        }

        // Keep legacy agent-local budget for backward compatibility / double gating
        let mut agent_budget = 0.0;
        let mut agent_cost = 0.0;
        if let Some(agent) = self.state.registry.agents.get(agent_id) {
            agent_budget = agent.value().budget_usd;
            agent_cost = agent.value().cost_usd;
        }

        if agent_budget > 0.0 && agent_cost >= agent_budget {
            tracing::warn!(
                "⚠️ [Governance] Agent {} local budget exhausted (${:.4}/${:.4})",
                agent_id,
                agent_cost,
                agent_budget
            );
            return Err(anyhow::anyhow!(
                "❌ Local Agent Budget Exhausted (${:.4}/${:.4})",
                agent_cost,
                agent_budget
            ));
        }

        let mission_budget = payload.budget_usd.unwrap_or_else(|| {
            if agent_budget > 0.0 {
                (agent_budget - agent_cost).max(0.001) // Ensure at least some budget if not explicitly over
            } else {
                1.0 // Default fallback
            }
        });

        let mission = crate::agent::mission::create_mission(
            &self.state.resources.pool,
            agent_id,
            &mission_title,
            mission_budget,
        )
        .await?;

        crate::agent::mission::update_mission(
            &self.state.resources.pool,
            &mission.id,
            crate::agent::types::MissionStatus::Active,
            0.0,
        )
        .await?;
        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &mission.id,
            agent_id,
            "User",
            &payload.message,
            "info",
            None,
        )
        .await?;

        Ok(mission)
    }

    /// Prepares the runtime context, including remote memory (RAG) synchronization.
    /// 
    /// Spawns a background task to perform semantic search across agent 
    /// history and mission scope, injecting findings into the local prompt.
    async fn prepare_run_context(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
        mission_id: &str,
        depth: u32,
        lineage: &[String],
    ) -> anyhow::Result<RunContext> {
        let ctx = self.resolve_agent_context(agent_id, payload, mission_id, depth, lineage)?;

        let (_, agent_memory_dir, mission_scope_dir) = ctx.resolve_paths();

        let initial_prompt = payload.message.clone();
        // 🕵️ RAG INJECTION (IDENTITY & SCOPE)
        // We resolve previous mission findings and identity context 
        // synchronously before prompt synthesis to ensure first-turn parity.
        if let Ok(agent_mem) = crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories").await {
            if let Ok(mission_mem) = crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await {
                let client = (*self.state.resources.http_client).clone();
                let provider = self.resolve_provider(&ctx, client);
                
                if let Ok(vec) = provider.embed(&initial_prompt).await {
                    let vec: Vec<f32> = vec;
                    if let Ok(results) = agent_mem.search_knowledge(vec.clone(), 5).await {
                        let count = results.len();
                        for (i, text) in results.into_iter().enumerate() {
                            let _ = mission_mem.add_memory(
                                &format!("mem-{}", i),
                                &text,
                                mission_id,
                                vec.clone(),
                            ).await;
                        }
                        tracing::info!("🧠 [RAG] Injected {} previous findings into mission scope for {}", count, agent_id);
                    }
                }
            }
        }

        Ok(ctx)
    }

    /// Handles the prompt generation, provider calls, and tool execution loop.
    async fn execute_intelligence_loop(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (anyhow::Error, Option<crate::agent::types::TokenUsage>)> {
        let hierarchy_label = if ctx.agent_id == "1" {
            "CEO (Strategic Intelligence Lead)"
        } else if ctx.agent_id == "2" {
            "COO (Operations Director)"
        } else if ctx.agent_id == "alpha" {
            "ALPHA NODE (Swarm Mission Commander)"
        } else {
            match ctx.depth {
                0 => "OVERLORD (Strategic Intelligence Lead)",
                1 => "ORCHESTRATOR (Execution Lead)",
                2 => "CLUSTER COORDINATOR",
                _ => "AGENT (Task Specialist)",
            }
        };

        self.state.broadcast_sys(
            &format!("Agent {} starting task ({})...", ctx.name, hierarchy_label),
            "info",
        );
        self.update_status(&ctx.agent_id, "thinking", Some("Consulting intelligence model..."));

        self.state.yield_phase_transition(&ctx.agent_id, "Execution: Prompt Generation").await;

        let system_prompt = self
            .build_system_prompt(ctx, hierarchy_label, &payload.message)
            .await;
        let swarm_tool = self.build_tools(ctx);

        let result = self
            .call_provider(
                ctx,
                &system_prompt,
                &payload.message,
                Some(vec![swarm_tool]),
            )
            .await;
        let (mut output_text, function_calls, mut usage) = match result {
            Ok(data) => data,
            Err(e) => {
                // Return usage so far (none) for budget tracking
                return Err((e, None));
            }
        };

        // Budget Enforcement
        let step_cost = crate::agent::rates::calculate_cost(
            &ctx.model_config.model_id,
            usage.as_ref().map(|u| u.input_tokens).unwrap_or(0),
            usage.as_ref().map(|u| u.output_tokens).unwrap_or(0),
        );

        if let Some(budget_msg) = self.check_budget(ctx, step_cost, &output_text).await.map_err(|e| (e, usage.clone()))? {
            return Ok(IntelligenceOutput {
                text: budget_msg,
                usage,
            });
        }

        self.state.yield_phase_transition(&ctx.agent_id, "Execution: Tool Orchestration").await;

        // Tool Execution Loop
        if !function_calls.is_empty() {
            use futures::stream::{FuturesUnordered, StreamExt};
            let mut futures = FuturesUnordered::new();
            for fc in function_calls {
                let runner = self.clone();
                let ctx_clone = ctx.clone();
                let user_msg = payload.message.clone();
                futures.push(async move {
                    runner.update_status(&ctx_clone.agent_id, "working", Some(&format!("Executing tool: {}...", fc.name)));
                    let mut local_text = String::new();
                    let mut local_usage = None;
                    let result = runner
                        .execute_tool(
                            &ctx_clone,
                            &fc,
                            &mut local_text,
                            &mut local_usage,
                            &user_msg,
                        )
                        .await;
                    (result, local_text, local_usage)
                });
            }

            while let Some(item) = futures.next().await {
                let (result, local_text, local_usage) = item;
                self.accumulate_usage(&mut usage, local_usage);
                output_text.push_str(&local_text);
                
                if let Err(e) = result {
                    return Err((e, usage));
                }
                
                if let Ok(Some(early_return)) = result {
                    return Ok(IntelligenceOutput {
                        text: early_return,
                        usage,
                    });
                }
            }
        }

        Ok(IntelligenceOutput {
            text: output_text,
            usage,
        })
    }
}

struct IntelligenceOutput {
    text: String,
    usage: Option<crate::agent::types::TokenUsage>,
}

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  VALIDATION
    // ─────────────────────────────────────────────────────────

    /// Validates input constraints before execution begins.
    fn validate_input(&self, agent_id: &str, payload: &TaskPayload) -> anyhow::Result<()> {
        let max_task_length = self.state.governance.max_task_length
            .load(std::sync::atomic::Ordering::Relaxed);
        if payload.message.len() > max_task_length {
            return Err(anyhow::anyhow!(
                "❌ Task message too long ({} bytes, max {})",
                payload.message.len(),
                max_task_length
            ));
        }

        let depth = payload.swarm_depth.unwrap_or(0);
        let lineage = payload.swarm_lineage.as_deref().unwrap_or(&[]);

        // CODE-01 FIX: Use iterator instead of to_string() allocation on hot path.
        if lineage.iter().any(|id| id == agent_id) {
            let path = lineage.join(" -> ");
            return Err(anyhow::anyhow!("🐝 CIRCULAR RECURSION DETECTED: Agent '{}' has already participated in this mission chain (Lineage: {} -> {}). Recruitment aborted.", agent_id, path, agent_id));
        }

        let max_swarm_depth = self.state.governance.max_swarm_depth
            .load(std::sync::atomic::Ordering::Relaxed);
        if depth >= max_swarm_depth {
            return Err(anyhow::anyhow!("🐝 Swarm depth limit exceeded (current depth: {})! To prevent infinite recursions, this agent cannot spawn more sub-agents.", depth));
        }

        // Check max agents limit (only for new recruitment, not for existing agents)
        if !self.state.registry.agents.contains_key(agent_id) {
            let max_agents = self.state.governance.max_agents
                .load(std::sync::atomic::Ordering::Relaxed);
            if self.state.registry.agents.len() as u32 >= max_agents {
                return Err(anyhow::anyhow!(
                    "🐝 Swarm agent limit reached (max: {}). New agent recruitment denied.",
                    max_agents
                ));
            }
        }

        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    //  CONTEXT RESOLUTION
    // ─────────────────────────────────────────────────────────

    /// Resolves the full agent context from registries, applying payload overrides.
    fn resolve_agent_context(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
        mission_id: &str,
        depth: u32,
        lineage: &[String],
    ) -> anyhow::Result<RunContext> {
        let entry = self.state.registry.agents
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not found", agent_id))?;
        let a = entry.value();

        let target_model_id = payload
            .model_id
            .clone()
            .or_else(|| a.model_id.clone())
            .unwrap_or_else(|| a.model.model_id.clone());

        // CENTRAL REGISTRY PATH: Resolve full config from model + provider registries
        let mut resolved_config = if let Some(model_entry) = self.state.registry.models.get(&target_model_id)
        {
            let model_id = model_entry.id.clone();
            let provider_id = model_entry.provider_id.clone();

            let provider_config = self.state.registry.providers.get(&provider_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "Provider {} not found for model {}",
                    provider_id,
                    target_model_id
                )
            })?;

            ModelConfig {
                provider: provider_config.protocol.clone(),
                model_id,
                api_key: provider_config.api_key.clone(),
                base_url: provider_config.base_url.clone(),
                system_prompt: a.model.system_prompt.clone(),
                temperature: a.model.temperature,
                max_tokens: a.model.max_tokens,
                external_id: provider_config.external_id.clone(),
                rpm: model_entry.rpm,
                rpd: model_entry.rpd,
                tpm: model_entry.tpm,
                tpd: model_entry.tpd,
                skills: None,
                workflows: None,
                mcp_tools: None,
            }
        } else if let Some(found_entry) = self.state.registry.models
            .iter()
            .find(|kv| kv.value().name.to_lowercase() == target_model_id.to_lowercase())
        {
            // FUZZY RESOLUTION: ID might be a friendly name from the UI
            let m = found_entry.value();
            let model_id = m.id.clone();
            let provider_id = m.provider_id.clone();

            let provider_config = self.state.registry.providers.get(&provider_id).ok_or_else(|| {
                anyhow::anyhow!("Provider {} not found for model {}", provider_id, m.name)
            })?;

            ModelConfig {
                provider: provider_config.protocol.clone(),
                model_id,
                api_key: provider_config.api_key.clone(),
                base_url: provider_config.base_url.clone(),
                system_prompt: a.model.system_prompt.clone(),
                temperature: a.model.temperature,
                max_tokens: a.model.max_tokens,
                external_id: provider_config.external_id.clone(),
                rpm: m.rpm,
                rpd: m.rpd,
                tpm: m.tpm,
                tpd: m.tpd,
                skills: Some(a.skills.clone()),       // Added
                workflows: Some(a.workflows.clone()), // Added
                mcp_tools: Some(a.mcp_tools.clone()),
            }
        } else {
            // FALLBACK: Use agent's internal model config
            let mut cfg = a.model.clone();
            cfg.model_id = target_model_id;
            cfg.skills = Some(a.skills.clone());
            cfg.workflows = Some(a.workflows.clone());
            cfg
        };

        // Mission-specific overrides from payload
        if let Some(p) = &payload.provider {
            // SEC-02: Check if this is a known Provider ID. If so, resolve to its protocol.
            // This prevents "unknown_provider (local)" errors when IDs are passed instead of protocols.
            if let Some(provider_config) = self.state.registry.providers.get(p) {
                resolved_config.provider = provider_config.protocol.clone();
                // Optionally pull other defaults if they aren't provided in the payload
                if payload.api_key.is_none() { resolved_config.api_key = provider_config.api_key.clone(); }
                if payload.base_url.is_none() { resolved_config.base_url = provider_config.base_url.clone(); }
                if payload.external_id.is_none() { resolved_config.external_id = provider_config.external_id.clone(); }
            } else {
                resolved_config.provider = p.clone();
            }
        } else {
            // No payload override: Resolve the fallback provider ID to a protocol if it exists in registry
            let current_p = &resolved_config.provider;
            if let Some(provider_config) = self.state.registry.providers.get(current_p) {
                resolved_config.provider = provider_config.protocol.clone();
                // Ensure secrets are pulled from backend registry even if agent record is stale
                if resolved_config.api_key.is_none() { resolved_config.api_key = provider_config.api_key.clone(); }
                if resolved_config.base_url.is_none() { resolved_config.base_url = provider_config.base_url.clone(); }
                if resolved_config.external_id.is_none() { resolved_config.external_id = provider_config.external_id.clone(); }
            }
        }
        if let Some(key) = &payload.api_key {
            resolved_config.api_key = Some(key.clone());
        }
        if let Some(url) = &payload.base_url {
            resolved_config.base_url = Some(url.clone());
        }
        if let Some(eid) = &payload.external_id {
            resolved_config.external_id = Some(eid.clone());
        }
        if let Some(m) = &payload.model_id {
            resolved_config.model_id = m.clone();
        }

        let provider_name = resolved_config.provider.to_lowercase();

        // Workspace Anchoring: Map clusterId to a physical path in ./workspaces
        let workspace_id = payload.cluster_id.as_deref().unwrap_or("executive-core"); // Default fallback

        let mut workspace_root = std::path::PathBuf::from("data/workspaces");
        // SEC: Whitelist sanitization — only allow alphanumeric, hyphens, and underscores
        let sanitized_id: String = workspace_id
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();

        if sanitized_id.is_empty() {
            return Err(anyhow::anyhow!("Invalid workspace ID: '{}'", workspace_id));
        }
        workspace_root.push(sanitized_id);

        let mut skills = resolved_config
            .skills
            .clone()
            .unwrap_or_else(|| a.skills.clone());
        let mut workflows = resolved_config
            .workflows
            .clone()
            .unwrap_or_else(|| a.workflows.clone());
        let mcp_tools = resolved_config
            .mcp_tools
            .clone()
            .unwrap_or_else(|| a.mcp_tools.clone());

        let safe_mode = payload.safe_mode.unwrap_or(false);
        if safe_mode {
            // Strip mutation/execution tools
            let blacklisted_skills = [
                "issue_alpha_directive",
                "spawn_subagent",
                "execute_bash",
                "write_file",
                "delete_file",
                "append_file",
                "deploy",
            ];
            skills.retain(|s| !blacklisted_skills.contains(&s.as_str()));
            workflows.clear();
        }

        let fs_adapter = crate::adapter::filesystem::FilesystemAdapter::new(workspace_root.clone());

        Ok(RunContext {
            agent_id: agent_id.to_string(),
            name: a.name.clone(),
            role: a.role.clone(),
            department: a.department.clone(),
            description: a.description.clone(),
            model_config: resolved_config,
            skills,
            workflows,
            mcp_tools,
            mission_id: mission_id.to_string(),
            depth,
            lineage: lineage.to_vec(),
            provider_name,
            workspace_root,
            fs_adapter,
            safe_mode,
            analysis: payload.analysis.unwrap_or(false),
            traceparent: payload.traceparent.clone(),
            user_id: payload.user_id.clone(),
            last_accessed_files: std::sync::Arc::new(std::sync::Mutex::new(payload.context_files.clone().unwrap_or_default())),
            recent_findings: payload.recent_findings.clone(),
        })
    }

    // ─────────────────────────────────────────────────────────
    //  FINALIZATION
    // ─────────────────────────────────────────────────────────

    /// Finalizes the run: updates token usage, persists mission state, broadcasts results.
    async fn finalize_run(
        &self,
        ctx: &RunContext,
        output_text: &str,
        usage: &Option<crate::agent::types::TokenUsage>,
    ) -> anyhow::Result<String> {
        tracing::info!(
            "🔔 [DIAGNOSTIC] Finalizing run for agent {}. Content length: {}",
            ctx.agent_id,
            output_text.len()
        );
        tracing::info!(
            "✅ [Runner] Provider responded successfully ({} tokens). Output length: {}",
            usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
            output_text.len()
        );
        if output_text.is_empty() {
            tracing::warn!(
                "⚠️ [Runner] final_delivery is EMPTY for agent {}",
                ctx.agent_id
            );
        }
        tracing::debug!("DEBUG [Runner] final_delivery content: {:?}", output_text);

        // Update global agent state
        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();
            if let Some(ref u) = usage {
                agent.token_usage = u.clone(); // Use the cumulative turn usage
                agent.tokens_used += u.total_tokens;
            }

            // Re-calculate turn cost from final cumulative usage
            let turn_cost = usage.as_ref().map(|u| {
                crate::agent::rates::calculate_cost(&ctx.model_config.model_id, u.input_tokens, u.output_tokens)
            }).unwrap_or(0.0);
            agent.cost_usd += turn_cost;
            // [Health Monitoring] Reset failure count on success
            agent.failure_count = 0;
            // Status is handled by update_status() at the end of the run

            // Record to persistent budget guard
            let budget_guard = self.state.security.budget_guard.clone();
            let agent_id = ctx.agent_id.clone();
            tokio::spawn(async move {
                let _ = budget_guard.record_usage(&agent_id, turn_cost).await;
            });

            // Sync to persistence
            let pool = self.state.resources.pool.clone();
            let agent_clone = agent.clone();
            tokio::spawn(async move {
                let _ = crate::agent::persistence::save_agent_db(&pool, &agent_clone).await;
            });

            self.state.emit_event(serde_json::json!({
                "type": "agent:update",
                "agentId": ctx.agent_id,
                "data": *agent
            }));
        }

        // Delivery formatting
        let mut final_delivery = output_text.trim().to_string();
        if final_delivery.is_empty() {
            final_delivery =
                "(Agent completed its actions without a final conversational response.)"
                    .to_string();
        }

        self.broadcast_agent_message(&ctx.agent_id, &final_delivery);
        self.update_status(&ctx.agent_id, "idle", None);

        // 💾 PERSISTENCE & MEMORY
        self.finalize_mission_persistence(ctx, output_text, usage).await?;
        self.archive_lance_db_memory(ctx, output_text);

        // 🧠 MISSION ANALYSIS TRIGGER
        if ctx.analysis && ctx.agent_id != "99" {
            spawn_post_mission_analysis(self.clone(), ctx.clone(), output_text.to_string());
        }

        Ok(final_delivery)
    }

    /// Finalizes mission state and logs the steps to SQLite.
    async fn finalize_mission_persistence(
        &self,
        ctx: &RunContext,
        output_text: &str,
        usage: &Option<crate::agent::types::TokenUsage>,
    ) -> Result<(), anyhow::Error> {
        let final_cumulative_cost = crate::agent::rates::calculate_cost(
            &ctx.model_config.model_id,
            usage.as_ref().map(|u| u.input_tokens).unwrap_or(0),
            usage.as_ref().map(|u| u.output_tokens).unwrap_or(0),
        );

        crate::agent::mission::update_mission(
            &self.state.resources.pool,
            &ctx.mission_id,
            crate::agent::types::MissionStatus::Completed,
            final_cumulative_cost,
        )
        .await?;

        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &ctx.mission_id,
            &ctx.agent_id,
            "Agent",
            output_text,
            "success",
            None,
        )
        .await?;
        Ok(())
    }

    /// Saves the final output to the agent's permanent institutional knowledge in LanceDB.
    fn archive_lance_db_memory(
        &self,
        ctx: &RunContext,
        output_text: &str,
    ) {
        let (_, agent_memory_dir, _) = ctx.resolve_paths();
        let mem_output = output_text.to_string();
        let mem_mission_id = ctx.mission_id.clone();
        let api_key = ctx.model_config.api_key.clone().unwrap_or_default();
        let http_client = self.state.resources.http_client.clone();

        let dedupe_threshold = std::env::var("LANCEDB_DEDUPE_THRESHOLD")
            .unwrap_or_else(|_| "0.2".to_string())
            .parse::<f32>()
            .unwrap_or(0.2);

        tokio::spawn(async move {
            match crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories").await {
                Ok(mem) => {
                    if let Ok(vec) = crate::agent::memory::get_gemini_embedding(
                        &http_client,
                        &api_key,
                        &mem_output,
                    )
                    .await
                    {
                        match mem
                            .check_memory_duplicate(vec.clone(), dedupe_threshold)
                            .await
                            {
                            Ok(true) => {
                                tracing::info!("🧠 [Memory] Duplicate detected (dist < {}), skipping LanceDB insertion for mission {}", dedupe_threshold, mem_mission_id);
                            }
                            _ => {
                                let id = uuid::Uuid::new_v4().to_string();
                                let _ =
                                    mem.add_memory(&id, &mem_output, &mem_mission_id, vec).await;
                                tracing::info!(
                                    "🧠 [Memory] Archived final result to LanceDB for mission {}",
                                    mem_mission_id
                                );
                            }
                        }
                    }
                }
                Err(e) => tracing::error!("❌ [Memory] Failed to archive LanceDB memory: {}", e),
            }
        });
    }

    // ─────────────────────────────────────────────────────────
    //  UTILITIES
    // ─────────────────────────────────────────────────────────

    /// Submits a tool call for manual user approval.
    /// Returns true if approved, false if rejected.
    #[allow(dead_code)]
    pub async fn submit_oversight(
        &self,
        mut tool_call: crate::agent::types::ToolCall,
        mission_id: Option<String>,
    ) -> bool {
        let entry_id = uuid::Uuid::new_v4().to_string();

        tool_call.mission_id = mission_id.clone();

        let entry = crate::agent::types::OversightEntry {
            id: entry_id.clone(),
            mission_id: mission_id.clone(),
            tool_call: Some(tool_call.clone()),
            skill_proposal: None,
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // 0. [Persistence] Pre-register in SQLite for audit history
        let payload_json = serde_json::to_string(&tool_call).unwrap_or_default();
        let params_json = serde_json::to_string(&tool_call.params).unwrap_or_default();

        let _ = sqlx::query(
            "INSERT INTO oversight_log (id, mission_id, agent_id, entry_type, skill, params, status, payload) VALUES (?, ?, ?, 'tool_call', ?, ?, 'pending', ?)"
        )
        .bind(&entry_id)
        .bind(&mission_id)
        .bind(&tool_call.agent_id)
        .bind(&tool_call.skill)
        .bind(params_json)
        .bind(payload_json)
        .execute(&self.state.resources.pool)
        .await;

        // 1. Register in the queue
        self.state.comms.oversight_queue
            .insert(entry_id.clone(), entry.clone());

        // 2. Create a channel for the decision
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.state.comms.oversight_resolvers.insert(entry_id.clone(), tx);

        // 3. Notify the UI
        self.state.emit_event(serde_json::json!({
            "type": "oversight:new",
            "entry": entry
        }));

        // 4. Await the user's click in the dashboard
        rx.await.unwrap_or_default()
    }

    /// Submits a skill proposal for manual user approval.
    pub async fn submit_skill_oversight(
        &self,
        proposal: crate::agent::types::SkillProposal,
        mission_id: Option<String>,
        agent_id: &str,
        _department: &str,
    ) -> bool {
        let entry_id = uuid::Uuid::new_v4().to_string();

        let entry = crate::agent::types::OversightEntry {
            id: entry_id.clone(),
            mission_id: mission_id.clone(),
            tool_call: None,
            skill_proposal: Some(proposal.clone()),
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // 0. [Persistence] Pre-register in SQLite
        let payload_json = serde_json::to_string(&proposal).unwrap_or_default();
        let _ = sqlx::query(
            "INSERT INTO oversight_log (id, mission_id, agent_id, entry_type, skill, params, status, payload) VALUES (?, ?, ?, 'capability_proposal', ?, '{}', 'pending', ?)"
        )
        .bind(&entry_id)
        .bind(&mission_id)
        .bind(agent_id)
        .bind(&proposal.name)
        .bind(payload_json)
        .execute(&self.state.resources.pool)
        .await;

        self.state.comms.oversight_queue
            .insert(entry_id.clone(), entry.clone());
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.state.comms.oversight_resolvers.insert(entry_id.clone(), tx);

        self.state.emit_event(serde_json::json!({
            "type": "oversight:new",
            "entry": entry
        }));

        rx.await.unwrap_or_default()
    }

    // --- Telemetry Helpers ---

    pub(crate) fn broadcast_agent_status(&self, agent_id: &str, status: &str) {
        let task = self.state.registry.agents.get(agent_id).and_then(|a| a.current_task.clone());
        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:status",
            "agentId": agent_id,
            "status": status,
            "currentTask": task
        }));
    }

    /// Centralized status and task update that syncs registry AND broadcasts telemetry.
    pub(crate) fn update_status(&self, agent_id: &str, status: &str, task: Option<&str>) {
        if let Some(mut agent) = self.state.registry.agents.get_mut(agent_id) {
            agent.status = status.to_string();
            agent.current_task = task.map(|t| t.to_string());
        }
        self.broadcast_agent_status(agent_id, status);
    }

    pub(crate) fn broadcast_agent_message(&self, agent_id: &str, text: &str) {
        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:message",
            "agentId": agent_id,
            "text": text,
            "messageId": uuid::Uuid::new_v4().to_string()
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::TaskPayload;

    fn make_payload(msg: &str) -> TaskPayload {
        TaskPayload {
            message: msg.to_string(),
            cluster_id: None,
            department: None,
            provider: None,
            model_id: None,
            api_key: None,
            base_url: None,
            rpm: None,
            tpm: None,
            budget_usd: None,
            swarm_depth: None,
            swarm_lineage: None,
            external_id: None,
            safe_mode: None,
            analysis: None,
            traceparent: None,
            user_id: None,
            context_files: None,
            recent_findings: None,
        }
    }

    #[tokio::test]
    async fn test_finalize_run_fallback_on_empty_output() {
        let state = Arc::new(crate::state::AppState::new().await);
        let runner = AgentRunner::new(state.clone());

        let test_uuid = uuid::Uuid::new_v4().to_string();
        let agent_id = format!("agent-test-{}", test_uuid);
        let mission_id = format!("mission-test-{}", test_uuid);

        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Test Runner', 'tester', 'QA', 'desc', 'idle', '{}')").bind(&agent_id).execute(&state.resources.pool).await.unwrap();
        sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Test Mission', 'active')").bind(&mission_id).bind(&agent_id).execute(&state.resources.pool).await.unwrap();

        let ctx = RunContext {
            agent_id: agent_id.clone(),
            name: "Test Runner".to_string(),
            role: "tester".to_string(),
            department: "QA".to_string(),
            description: "desc".to_string(),
            mission_id: mission_id.clone(),
            model_config: crate::agent::types::ModelConfig {
                provider: "google".to_string(),
                model_id: "gemini-pro-latest".to_string(),
                api_key: None,
                base_url: None,
                system_prompt: None,
                temperature: None,
                max_tokens: None,
                external_id: None,
                rpm: None,
                rpd: None,
                tpm: None,
                tpd: None,
                skills: None,
                workflows: None,
                mcp_tools: None,
            },
            provider_name: "mock".to_string(),
            skills: vec![],
            workflows: vec![],
            mcp_tools: vec![],
            depth: 0,
            lineage: vec![],
            workspace_root: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            recent_findings: None,
        };

        let result_empty = runner
            .finalize_run(&ctx, "   \n  \t ", &None)
            .await
            .unwrap();
        assert_eq!(
            result_empty,
            "(Agent completed its actions without a final conversational response.)"
        );

        let result_normal = runner
            .finalize_run(&ctx, "  Hello Context!  ", &None)
            .await
            .unwrap();
        assert_eq!(result_normal, "Hello Context!");
    }

    #[tokio::test]
    async fn validate_input_accepts_normal_message() {
        let state = Arc::new(crate::state::AppState::new().await);
        state.registry.agents.clear(); // Ensure clean state for runner tests
        let runner = AgentRunner::new(state.clone());
        let payload = make_payload("Hello, agent!");
        let result = runner.validate_input("agent-1", &payload);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_input_rejects_oversized_message() {
        let state = Arc::new(crate::state::AppState::new().await);
        state.governance.max_task_length
            .store(1000, std::sync::atomic::Ordering::Relaxed); // Lower limit for test
        state.registry.agents.clear();
        let runner = AgentRunner::new(state.clone());
        let long_msg = "x".repeat(2000);
        let payload = make_payload(&long_msg);
        let result = runner.validate_input("agent-1", &payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[tokio::test]
    async fn validate_input_detects_circular_recursion() {
        let state = Arc::new(crate::state::AppState::new().await);
        state.registry.agents.clear(); // Ensure clean state for runner tests
        let runner = AgentRunner::new(state.clone());
        let mut payload = make_payload("test");
        payload.swarm_lineage = Some(vec!["agent-1".to_string(), "agent-2".to_string()]);
        payload.swarm_depth = Some(2);

        let result = runner.validate_input("agent-1", &payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CIRCULAR"));
    }

    #[tokio::test]
    async fn validate_input_allows_non_circular_lineage() {
        let state = Arc::new(crate::state::AppState::new().await);
        state.registry.agents.clear(); // Ensure clean state for runner tests
        let runner = AgentRunner::new(state.clone());
        let mut payload = make_payload("test");
        payload.swarm_lineage = Some(vec!["agent-1".to_string(), "agent-2".to_string()]);
        payload.swarm_depth = Some(2);

        let result = runner.validate_input("agent-3", &payload);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_input_enforces_depth_limit() {
        let state = Arc::new(crate::state::AppState::new().await);
        state.governance.max_swarm_depth
            .store(3, std::sync::atomic::Ordering::Relaxed); // Lower limit for test
        state.registry.agents.clear();
        let runner = AgentRunner::new(state.clone());
        let mut payload = make_payload("test");
        payload.swarm_depth = Some(3); // Should trigger limit if depth >= max

        let result = runner.validate_input("agent-1", &payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth limit"));
    }

    #[tokio::test]
    async fn build_system_prompt_includes_role_and_department() {
        let state = Arc::new(crate::state::AppState::new().await);
        let runner = AgentRunner::new(state);

        let ctx = RunContext {
            agent_id: "test-1".to_string(),
            name: "Test Agent".to_string(),
            role: "Researcher".to_string(),
            department: "Intelligence".to_string(),
            description: "A test agent".to_string(),
            model_config: crate::agent::types::ModelConfig {
                provider: "mock".to_string(),
                model_id: "mock".to_string(),
                api_key: None,
                base_url: None,
                system_prompt: None,
                temperature: None,
                max_tokens: None,
                external_id: None,
                rpm: None,
                rpd: None,
                tpm: None,
                tpd: None,
                skills: None,
                workflows: None,
                mcp_tools: None,
            },
            provider_name: "mock".to_string(),
            skills: vec![],
            workflows: vec![],
            mcp_tools: vec![],
            depth: 0,
            lineage: vec![],
            mission_id: "m1".to_string(),
            workspace_root: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            recent_findings: None,
        };

        let prompt = runner
            .build_system_prompt(&ctx, "OVERLORD", "Analyze the system.")
            .await;
        assert!(prompt.contains("Test Agent"));
        assert!(prompt.contains("Researcher"));
        assert!(prompt.contains("Intelligence"));
    }

    #[tokio::test]
    async fn build_system_prompt_includes_lineage_when_present() {
        let state = Arc::new(crate::state::AppState::new().await);
        let runner = AgentRunner::new(state);

        let ctx = RunContext {
            agent_id: "test-3".to_string(),
            name: "Specialist".to_string(),
            role: "Analyst".to_string(),
            department: "Research".to_string(),
            description: "A test agent".to_string(),
            model_config: crate::agent::types::ModelConfig {
                provider: "mock".to_string(),
                model_id: "mock".to_string(),
                api_key: None,
                base_url: None,
                system_prompt: None,
                temperature: None,
                max_tokens: None,
                external_id: None,
                rpm: None,
                rpd: None,
                tpm: None,
                tpd: None,
                skills: None,
                workflows: None,
                mcp_tools: None,
            },
            provider_name: "mock".to_string(),
            skills: vec![],
            workflows: vec![],
            mcp_tools: vec![],
            depth: 2,
            lineage: vec!["agent-1".to_string(), "agent-2".to_string()],
            mission_id: "m2".to_string(),
            workspace_root: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            recent_findings: None,
        };

        let prompt_small = runner.build_system_prompt(&ctx, "root", "hi").await;
        assert!(prompt_small.contains("agent-1 -> agent-2"));
    }
}
