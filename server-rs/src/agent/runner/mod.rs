//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **The Heartbeat**: This is the core `run()` loop. Orchestrates the
//! full mission lifecycle: **Setup -> Initialization -> ContextResolution
//! -> IntelligenceLoop -> Finalization**. Manages the `RunContext`
//! (state bag) and hierarchical OTel tracing for real-time "God View"
//! visualization. Supports **Deterministic Workflows** and **Recursive Swarm**
//! recruitment.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Budget exhaustion, recursion depth limit (SEC-01),
//!   prompt injection detection, or API provider timeouts.
//! - **Trace Scope**: `server-rs::agent::runner` (Check for `AgentExecution` span)
//! - **Telemetry**: Emits `agent:status` and `agent:message` events to
//!   the global telemetry bus.

use crate::agent::backlog::MissionBacklog;
use crate::agent::types::{ModelConfig, RoleAuthorityLevel, TaskPayload};
use crate::error::AppError;
use crate::state::AppState;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// ─────────────────────────────────────────────────────────
//  SUBMODULES
// ─────────────────────────────────────────────────────────
pub mod a2a_ledger;
pub mod a2a_mailbox;
pub mod a2a_router;
pub mod a2a_types;
mod analysis;
pub(crate) mod conductor;
mod context;
pub(crate) mod error;
mod evolution_tools;
mod external_tools;
mod finalize;
mod fs_tools;
mod intelligence;
pub mod turn_compactor;
mod lifecycle;
mod metrics_tools;
mod mission_tools;
mod oversight;
pub(crate) mod prompt_renderer;
mod provider;
mod refinement;
pub(crate) mod service_traits;
pub(crate) mod swarm;
mod swarm_persistence;
pub mod synthesis;
pub mod tools;
mod workflow;

fn safe_truncate_str(s: &str, limit: usize) -> String {
    if s.len() <= limit {
        return s.to_string();
    }
    let mut end = limit;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}... [TRUNCATED]", &s[..end])
}

// ─────────────────────────────────────────────────────────
//  CORE TYPES
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod a2a_tests;

/// Static identity properties for an agent.
#[derive(Clone, Debug, Default)]
pub(crate) struct AgentIdentity {
    pub agent_id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub description: String,
    pub authority_level: RoleAuthorityLevel,
}

/// Dynamic mission state for a execution context.
#[derive(Clone, Debug, Default)]
pub(crate) struct MissionState {
    pub mission_id: String,
    pub cluster_id: Option<String>,
    pub user_id: Option<String>,
    pub depth: u32,
    pub lineage: Vec<String>,
    pub primary_goal: Option<String>,
    pub budget_usd: f64,
    pub current_cost_usd: f64,
    pub sub_budget_usd: Option<f64>,
}

/// System environment parameters for an execution context.
#[derive(Clone)]
pub(crate) struct Environment {
    pub workspace_root: std::path::PathBuf,
    pub fs_adapter: crate::adapter::filesystem::FilesystemAdapter,
    pub base_dir: std::path::PathBuf,
}

/// Context bag for data resolved during the setup phase of a run.
#[derive(Clone)]
pub(crate) struct RunContext {
    #[allow(dead_code)]
    pub identity: AgentIdentity,
    #[allow(dead_code)]
    pub mission_state: MissionState,
    #[allow(dead_code)]
    pub env: Environment,
    pub agent_id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub description: String,
    pub model_config: ModelConfig,
    pub skills: Vec<String>,
    pub workflows: Vec<String>,
    pub agent_models: crate::agent::types::AgentModels,
    #[allow(dead_code)]
    pub mcp_tools: Vec<String>,
    pub mission_id: String,
    pub cluster_id: Option<String>,
    pub user_id: Option<String>,
    pub depth: u32,
    pub lineage: Vec<String>,
    pub provider_name: String,
    pub workspace_root: std::path::PathBuf,
    pub fs_adapter: crate::adapter::filesystem::FilesystemAdapter,
    pub safe_mode: bool,
    pub analysis: bool,
    pub traceparent: Option<String>,
    pub visible_transcript: Option<std::sync::Arc<parking_lot::Mutex<Vec<String>>>>,
    pub conductor_plan: Option<conductor::ConductorPlan>,
    pub last_accessed_files: std::sync::Arc<parking_lot::Mutex<Vec<String>>>,
    pub modified_files: std::sync::Arc<parking_lot::Mutex<Vec<String>>>,
    pub commands_run: std::sync::Arc<parking_lot::Mutex<std::collections::HashSet<String>>>,
    pub current_dir: std::sync::Arc<parking_lot::Mutex<Option<std::path::PathBuf>>>,
    pub allowed_files: Option<Vec<String>>,
    pub recent_findings: Option<String>,
    pub working_memory: serde_json::Value,
    pub base_dir: std::path::PathBuf,
    pub summarized_history: Option<String>,
    pub structured_output: bool,
    pub backlog: Option<Arc<parking_lot::Mutex<MissionBacklog>>>,
    pub primary_goal: Option<String>,
    pub budget_usd: f64,
    pub current_cost_usd: f64,
    pub sub_budget_usd: Option<f64>,
    pub reasoning_depth: u32,
    pub act_threshold: f32,
    pub max_turns: u32,
    pub authority_level: RoleAuthorityLevel,
    pub resource_weights: std::collections::HashMap<String, f32>,
    pub graph_context: Option<String>,
    #[allow(dead_code)]
    pub verification_passed: bool,
}

impl Default for RunContext {
    fn default() -> Self {
        let identity = AgentIdentity {
            agent_id: "default-agent".to_string(),
            name: "Default".to_string(),
            role: "Specialist".to_string(),
            department: "Standard".to_string(),
            description: "Default test context".to_string(),
            authority_level: RoleAuthorityLevel::Specialist,
        };
        let mission_state = MissionState {
            mission_id: "default-mission".to_string(),
            cluster_id: None,
            user_id: None,
            depth: 0,
            lineage: vec![],
            primary_goal: None,
            budget_usd: 0.0,
            current_cost_usd: 0.0,
            sub_budget_usd: None,
        };
        let env = Environment {
            workspace_root: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            base_dir: std::path::PathBuf::from("."),
        };
        Self {
            agent_id: identity.agent_id.clone(),
            name: identity.name.clone(),
            role: identity.role.clone(),
            department: identity.department.clone(),
            description: identity.description.clone(),
            authority_level: identity.authority_level,
            identity,
            mission_id: mission_state.mission_id.clone(),
            cluster_id: mission_state.cluster_id.clone(),
            user_id: mission_state.user_id.clone(),
            depth: mission_state.depth,
            lineage: mission_state.lineage.clone(),
            primary_goal: mission_state.primary_goal.clone(),
            budget_usd: mission_state.budget_usd,
            current_cost_usd: mission_state.current_cost_usd,
            sub_budget_usd: mission_state.sub_budget_usd,
            mission_state,
            workspace_root: env.workspace_root.clone(),
            fs_adapter: env.fs_adapter.clone(),
            base_dir: env.base_dir.clone(),
            env,
            model_config: ModelConfig::default(),
            skills: vec![],
            workflows: vec![],
            agent_models: crate::agent::types::AgentModels::default(),
            mcp_tools: vec![],
            provider_name: "mock".to_string(),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            modified_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            commands_run: std::sync::Arc::new(parking_lot::Mutex::new(
                std::collections::HashSet::new(),
            )),
            current_dir: std::sync::Arc::new(parking_lot::Mutex::new(None)),
            allowed_files: None,
            recent_findings: None,
            working_memory: serde_json::json!({}),
            summarized_history: None,
            structured_output: false,
            backlog: None,
            visible_transcript: None,
            conductor_plan: None,
            reasoning_depth: 1,
            act_threshold: 0.9,
            max_turns: 20,
            resource_weights: std::collections::HashMap::new(),
            graph_context: None,
            verification_passed: false,
        }
    }
}

impl RunContext {
    #[allow(dead_code)]
    pub fn resolve_paths(&self) -> (String, String, String) {
        let cluster_name = self
            .workspace_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let agent_memory_dir = self
            .base_dir
            .join("data/workspaces")
            .join(&cluster_name)
            .join("agents")
            .join(&self.agent_id)
            .join("memory.lance")
            .to_string_lossy()
            .to_string();
        let mission_scope_dir = self
            .base_dir
            .join("data/workspaces")
            .join(&cluster_name)
            .join("missions")
            .join(&self.mission_id)
            .join("scope.lance")
            .to_string_lossy()
            .to_string();
        (cluster_name, agent_memory_dir, mission_scope_dir)
    }

    pub(crate) fn derive_subtask_payload(&self, message: String) -> TaskPayload {
        let remaining = if let Some(sub_b) = self.sub_budget_usd {
            let overall_rem = self.budget_usd - self.current_cost_usd;
            sub_b.min(overall_rem)
        } else {
            self.budget_usd - self.current_cost_usd
        };
        let derived_sub_budget = Some((remaining * 0.5).max(0.0));

        TaskPayload {
            message,
            cluster_id: Some(self.mission_id.clone()),
            provider: None,
            model_id: None,
            api_key: None,
            base_url: None,
            rpm: None,
            tpm: None,
            rpd: None,
            tpd: None,
            sub_budget_usd: derived_sub_budget,
            swarm_depth: Some(self.depth + 1),
            swarm_lineage: Some({
                let mut l = self.lineage.clone();
                l.push(self.agent_id.clone());
                l
            }),
            external_id: None,
            safe_mode: Some(self.safe_mode),
            traceparent: self.traceparent.clone(),
            user_id: self.user_id.clone(),
            context_files: Some(self.last_accessed_files.lock().clone()),
            recent_findings: self
                .recent_findings
                .as_ref()
                .map(|rf| safe_truncate_str(rf, 2048)),
            structured_output: Some(false),
            primary_goal: self
                .primary_goal
                .as_ref()
                .map(|pg| safe_truncate_str(pg, 1024)),
            allowed_files: self.allowed_files.clone(),
            visible_transcript: self.visible_transcript.as_ref().map(|vt| {
                let locked = vt.lock();
                locked
                    .iter()
                    .rev()
                    .take(6)
                    .rev()
                    .map(|msg| safe_truncate_str(msg, 4096))
                    .collect()
            }),
            ..Default::default()
        }
    }

    /// Creates a partial RunContext from an isolated ToolContext.
    /// Used for bridging between Zero-Trust tools and legacy AgentRunner handlers.
    pub fn from_tool_ctx(ctx: &crate::agent::runner::tools::ToolContext) -> Self {
        Self {
            agent_id: ctx.agent_id.clone(),
            mission_id: ctx.mission_id.clone(),
            workspace_root: ctx.workspace_root.clone(),
            fs_adapter: ctx.fs_adapter.clone(),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct AgentRunner {
    pub state: Arc<AppState>,
    pub model_router: Arc<dyn service_traits::ModelRouter>,
    pub prompt_service: Arc<dyn service_traits::PromptService>,
    pub tool_orchestrator: Arc<dyn service_traits::ToolOrchestrator>,
    #[allow(dead_code)]
    pub mission_state_manager: Arc<dyn service_traits::MissionStateManager>,
    pub workflow_coordinator: Arc<dyn service_traits::WorkflowCoordinator>,
}

#[derive(Debug, Clone)]
pub struct IntelligenceOutput {
    pub text: String,
    pub usage: Option<crate::agent::types::TokenUsage>,
}

struct ActiveAgentGuard<'a> {
    counter: &'a AtomicU32,
}

impl<'a> ActiveAgentGuard<'a> {
    fn acquire(counter: &'a AtomicU32) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self { counter }
    }
}

impl Drop for ActiveAgentGuard<'_> {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

impl AgentRunner {
    pub fn new(state: Arc<AppState>) -> Self {
        let prompt_service = Arc::new(service_traits::DefaultPromptService);
        let mission_state_manager = Arc::new(service_traits::DefaultMissionStateManager);
        let workflow_coordinator = Arc::new(intelligence::MissionWorkflowCoordinator {
            state: state.clone(),
            prompt_service: prompt_service.clone(),
            mission_state_manager: mission_state_manager.clone(),
        });

        Self {
            state,
            model_router: Arc::new(service_traits::DefaultModelRouter),
            prompt_service,
            tool_orchestrator: Arc::new(service_traits::DefaultToolOrchestrator::default()),
            mission_state_manager,
            workflow_coordinator,
        }
    }

    /// Emits a diagnostic event to the global system terminal.
    pub(crate) fn broadcast_sys(&self, msg: &str, level: &str, mission_id: Option<String>) {
        self.state.broadcast_sys(msg, level, mission_id);
    }

    /// Emits an agent-specific personality event to the dashboard.
    pub(crate) fn broadcast_agent(&self, ctx: &RunContext, msg: &str, level: &str) {
        self.state.broadcast_agent(
            msg,
            level,
            Some(ctx.mission_id.clone()),
            &ctx.agent_id,
            &ctx.name,
        );
    }

    /// Updates the heartbeat timestamp in the registry and persistence layer.
    /// Used by the `reap_stale_agents` safety valve to detect hung missions.
    pub(crate) async fn record_heartbeat(&self, agent_id: &str) {
        let now = chrono::Utc::now();
        if let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) {
            entry.value_mut().health.heartbeat_at = Some(now);
        }
        if let Err(e) =
            crate::agent::persistence::update_agent_heartbeat(&self.state.resources.pool, agent_id)
                .await
        {
            tracing::warn!(
                "⚠️ [Runner] Failed to persist heartbeat for agent {}: {}",
                agent_id,
                e
            );
        }
    }

    /// ### 🔄 Processing Pipeline: The Intelligence Heartbeat
    /// Orchestrates the full autonomous mission lifecycle for an agent identity.
    ///
    /// ### 🧬 Mission Phases
    /// 1. **Setup**: Resolves the static `RunContext` (agent name, role, department)
    ///    and validates the incoming `TaskPayload` for circular recursion.
    /// 2. **Initialization**: Creates a persistent record in the `mission_history`
    ///    table and clears the agent's short-term working memory.
    /// 3. **ContextResolution**: Loads the workspace path, canonicalizes the
    ///    filesystem adapter, and injects semantic history (if enabled). Checks
    ///    for active **Deterministic Workflows** (SOPs).
    /// 4. **IntelligenceLoop**: The primary cognitive cycle. Alternates between
    ///    LLM inference (Reasoning) and Tool Execution (Interaction) until
    ///    the goal is met or the budget/recursion limit is hit.
    /// 5. **Finalization**: Records total mission cost, logs the final completion
    ///    text, and releases the `ActiveAgentGuard`.
    #[tracing::instrument(
        name = "AgentExecution",
        skip(self, payload),
        fields(
            agent_id = %agent_id,
            mission_id = %payload.cluster_id.as_deref().unwrap_or("unknown"),
            status = "running",
            swarm_depth = payload.swarm_depth.unwrap_or(0),
            trace_id = tracing::field::Empty
        )
    )]
    pub async fn run(
        &self,
        agent_id: String,
        mut payload: TaskPayload,
    ) -> Result<String, AppError> {
        if payload.primary_goal.is_none() {
            payload.primary_goal = Some(payload.message.clone());
        }

        self.state.yield_phase_transition(&agent_id, "Setup").await;
        self.setup_and_validate(&agent_id, &payload)?;

        self.state
            .yield_phase_transition(&agent_id, "Initialization")
            .await;
        let _active_agent_guard = ActiveAgentGuard::acquire(&self.state.governance.active_agents);
        let mission = self.initialize_mission_state(&agent_id, &payload).await?;
        let mission_id = mission.id.clone();
        self.update_status(&agent_id, &mission_id, "active", None);

        self.state
            .yield_phase_transition(&agent_id, "ContextResolution")
            .await;
        let agent_data = self
            .state
            .registry
            .agents
            .get(&agent_id)
            .map(|a| a.value().clone())
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;

        if let Some(workflow_name) = agent_data.capabilities.workflows.first() {
            let msg_lower = payload.message.to_lowercase();
            let workflow_requested = msg_lower
                .contains(&workflow_name.to_lowercase().replace("_", " "))
                || msg_lower.contains("workflow")
                || msg_lower.contains("sop");
            let is_sme = agent_data
                .identity
                .department
                .to_lowercase()
                .contains("sme")
                || agent_data
                    .identity
                    .role
                    .to_lowercase()
                    .contains("specialist");

            if workflow_requested || is_sme {
                if let Ok(mut state) = crate::agent::workflows::load_workflow(
                    self.state.base_dir.as_path(),
                    workflow_name,
                )
                .await
                {
                    return self
                        .run_deterministic_workflow(&agent_id, payload, &mut state)
                        .await;
                }
            }
        }

        let depth = payload.swarm_depth.unwrap_or(0);
        let lineage = payload.swarm_lineage.clone().unwrap_or_default();
        let ctx = self
            .prepare_run_context(&agent_id, &payload, &mission_id, depth, &lineage)
            .await?;

        // Pre-flight validation of API credentials before entering loop or spending budget
        self.verify_provider_connectivity(&ctx).await?;

        self.state
            .yield_phase_transition(&agent_id, "Specification")
            .await;

        self.state
            .yield_phase_transition(&agent_id, "IntelligenceLoop")
            .await;
        self.record_heartbeat(&ctx.agent_id).await;
        let output_res = self.execute_intelligence_loop(&ctx, &payload).await;

        match output_res {
            Ok(output) => {
                self.state
                    .yield_phase_transition(&agent_id, "Finalization")
                    .await;
                self.record_heartbeat(&ctx.agent_id).await;
                self.finalize_run(&ctx, &output.text, &output.usage).await
            }
            Err((e, usage)) => {
                let _ = self.fail_mission(&ctx, &e, &usage).await;
                Err(e)
            }
        }
    }

    /// Verifies provider connectivity pre-flight before dedicating mission resources/budgets.
    pub(crate) async fn verify_provider_connectivity(
        &self,
        ctx: &RunContext,
    ) -> Result<(), AppError> {
        use crate::agent::types::ModelProvider;
        use provider::ProviderVariant;

        // Bypass connectivity check if null_providers_test_mode is active (for unit testing)
        if self
            .state
            .governance
            .null_providers_test_mode
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Ok(());
        }

        let is_local = ctx.model_config.provider == ModelProvider::Ollama;

        let provider_name_str = format!("{:?}", ctx.model_config.provider);
        let provider_label = if is_local {
            "(local model)".to_string()
        } else if ctx.model_config.model_id.is_empty() {
            provider_name_str.clone()
        } else {
            format!("{} ({})", provider_name_str, ctx.model_config.model_id)
        };

        let success_label = provider_label.clone();

        tracing::info!(
            "🔍 [Connectivity Check] Pre-flight connection validation for provider {}...",
            provider_label
        );
        self.broadcast_sys(
            &format!(
                "🔍 Pre-flight connection checking for valid provider {}...",
                provider_label
            ),
            "info",
            Some(ctx.mission_id.clone()),
        );

        // Skip connectivity check for local Ollama to save time and prevent local offline blocks,
        // but log/broadcast success message above and below.
        if is_local {
            tracing::info!(
                "✅ [Connectivity Check] Provider {} is authenticated and responsive.",
                success_label
            );
            self.broadcast_sys(
                &format!("✅ Provider {} authenticated successfully.", success_label),
                "success",
                Some(ctx.mission_id.clone()),
            );
            return Ok(());
        }

        let client = (*self.state.resources.http_client).clone();
        let provider = self.resolve_provider(ctx, client).await;

        if let ProviderVariant::Null(ref _null_prov) = provider {
            let err_msg = format!(
                "API key configuration missing for provider {:?}",
                ctx.model_config.provider
            );
            tracing::warn!("❌ [Connectivity Check] {}", err_msg);
            return Err(AppError::Forbidden(err_msg));
        }

        // Send a minimal validation prompt directly using the resolved provider instance
        match provider.generate("ping", "", None).await {
            Ok(_) => {
                tracing::info!(
                    "✅ [Connectivity Check] Provider {} is authenticated and responsive.",
                    success_label
                );
                self.broadcast_sys(
                    &format!("✅ Provider {} authenticated successfully.", success_label),
                    "success",
                    Some(ctx.mission_id.clone()),
                );
                Ok(())
            }
            Err(e) => {
                let err_msg = format!(
                    "API Key authentication failed for provider {:?}: {}",
                    ctx.model_config.provider, e
                );
                tracing::warn!("❌ [Connectivity Check] {}", err_msg);
                self.broadcast_sys(
                    &format!(
                        "❌ API Key authentication failed for provider {:?}.",
                        ctx.model_config.provider
                    ),
                    "error",
                    Some(ctx.mission_id.clone()),
                );
                Err(AppError::Forbidden(err_msg))
            }
        }
    }

    /// Safely truncates a string to a byte limit without breaking UTF-8 boundaries.
    pub(crate) fn safe_truncate(&self, s: &str, limit: usize) -> String {
        safe_truncate_str(s, limit)
    }
}

// Metadata: [mod]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_subtask_payload() {
        let ctx = RunContext {
            agent_id: "parent".to_string(),
            mission_id: "mission-123".to_string(),
            depth: 1,
            lineage: vec!["grandparent".to_string()],
            ..Default::default()
        };

        let payload = ctx.derive_subtask_payload("Hello".to_string());

        assert_eq!(payload.message, "Hello");
        assert_eq!(payload.cluster_id, Some("mission-123".to_string()));
        assert_eq!(payload.swarm_depth, Some(2));
        assert_eq!(
            payload.swarm_lineage,
            Some(vec!["grandparent".to_string(), "parent".to_string()])
        );
    }

    #[tokio::test]
    async fn test_safe_truncate() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);

        let s = "Hello World";
        assert_eq!(runner.safe_truncate(s, 5), "Hello... [TRUNCATED]");
        assert_eq!(runner.safe_truncate(s, 20), "Hello World");

        // Test UTF-8 boundary
        let emoji = "👋 Hello";
        // 👋 is 4 bytes. Truncating at 2 should back off to 0.
        let truncated = runner.safe_truncate(emoji, 2);
        assert!(truncated.contains("... [TRUNCATED]"));
    }

    #[tokio::test]
    async fn test_resolve_provider_empty_base_url() {
        use crate::agent::runner::provider::ProviderVariant;
        use crate::agent::types::ModelProvider;

        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            agent_id: "test-agent".to_string(),
            provider_name: "ollama".to_string(),
            model_config: ModelConfig {
                provider: ModelProvider::Ollama,
                model_id: "gemma4:e4b".to_string(),
                base_url: Some("".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let provider = runner.resolve_provider(&ctx, client).await;

        match provider {
            ProviderVariant::OpenAI(p) => {
                let base = p.config.base_url.as_ref().unwrap();
                assert!(!base.is_empty());
                assert!(
                    base.contains("11434")
                        || base.contains("localhost")
                        || base.contains("127.0.0.1")
                        || base.contains("host.docker.internal")
                );
            }
            _ => panic!("Expected OpenAI variant for Ollama provider"),
        }
    }
}
