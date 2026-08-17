//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Run Context**: The source of truth for an active mission. Resolves agent
//! identities, model configs, tool registries, and workspace sandboxes.
//! Implements **Hybrid RAG (Vector + Keyword)** injection and
//! **Context Summarization** (4k token threshold) to optimize prompt windows.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Agent/Model/Provider lookup failure, invalid workspace ID,
//!   summarization error, or RAG connection timeout.
//! - **Trace Scope**: `server-rs::agent::runner::context`
//!

use crate::agent::context_manager::ContextManager;
use crate::agent::types::{EngineAgent, ModelConfig, ModelEntry, TaskPayload};
use crate::error::AppError;
use uuid::Uuid;

use super::{AgentIdentity, AgentRunner, Environment, MissionState, RunContext};

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  CONTEXT RESOLUTION
    // ─────────────────────────────────────────────────────────

    /// Resolves the full agent context from registries, applying payload overrides.
    pub(crate) async fn resolve_agent_context(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
        mission_id: &str,
        depth: u32,
        lineage: &[String],
    ) -> Result<RunContext, AppError> {
        let a = self
            .state
            .registry
            .agents
            .get(agent_id)
            .map(|e| e.value().clone())
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;

        let active_slot = payload
            .active_model_slot
            .as_deref()
            .or(a.models.active_model_slot.as_deref());

        // 1. Resolve Target Model ID
        let target_model_id = payload
            .model_id
            .as_ref()
            .or_else(|| match active_slot {
                Some("planning") => a.models.planning_slot.as_ref().map(|s| &s.model_id),
                Some("execution") => a.models.execution_slot.as_ref().map(|s| &s.model_id),
                _ => a.models.model_id.as_ref(),
            })
            .unwrap_or(&a.models.model.model_id)
            .clone();

        // 2. Resolve Base Configuration
        let slot_cfg = match active_slot {
            Some("planning") => a.models.planning_slot.as_ref().unwrap_or(&a.models.model),
            Some("execution") => a.models.execution_slot.as_ref().unwrap_or(&a.models.model),
            _ => &a.models.model,
        };
        let mut resolved_config = self.resolve_base_config(&a, &target_model_id, slot_cfg)?;

        // 3. Apply Payload Overrides
        self.merge_payload_overrides(&mut resolved_config, payload);

        // Model-Provider Alignment
        if resolved_config.provider == crate::agent::types::ModelProvider::Openai {
            if let Some(detected) =
                crate::agent::types::ModelProvider::from_model_id(&resolved_config.model_id)
            {
                resolved_config.provider = detected;
            }
        }

        // 4. Resolve Workspace Root
        let workspace_root = self.resolve_workspace_paths(payload.cluster_id.as_deref())?;
        let fs_adapter = crate::adapter::filesystem::FilesystemAdapter::new(workspace_root.clone());

        // 5. Capability Merging & Security Gates
        let mut skills = resolved_config
            .skills
            .clone()
            .unwrap_or_else(|| a.capabilities.skills.clone());
        let mut workflows = resolved_config
            .workflows
            .clone()
            .unwrap_or_else(|| a.capabilities.workflows.clone());
        let mcp_tools = resolved_config
            .mcp_tools
            .clone()
            .unwrap_or_else(|| a.capabilities.mcp_tools.clone());

        let safe_mode = payload.safe_mode.unwrap_or(false);
        self.apply_security_gates(safe_mode, &mut skills, &mut workflows);

        // 6. Mission Financials
        let (mut budget_usd, current_cost_usd) = if mission_id != "system-internal"
            && !mission_id.is_empty()
        {
            match crate::agent::mission::get_mission_by_id(&self.state.resources.pool, mission_id)
                .await
            {
                Ok(Some(m)) => (m.budget_usd, m.cost_usd),
                _ => (payload.budget_usd.unwrap_or(0.0), 0.0),
            }
        } else {
            (0.0, 0.0)
        };

        // SEC-02: Synchronize with persistent budget_guard (mission_quotas)
        if mission_id != "system-internal" && !mission_id.is_empty() {
            if let Ok(Some(quota_budget)) = sqlx::query_scalar::<_, f64>(
                "SELECT budget_usd FROM mission_quotas WHERE cluster_id = ?1",
            )
            .bind(mission_id)
            .fetch_optional(&self.state.resources.pool)
            .await
            {
                // Convert micro-dollars to float (unified to micro-dollars in database)
                let normalized_budget = quota_budget / 1_000_000.0;
                if normalized_budget > 0.0 && normalized_budget > budget_usd {
                    budget_usd = normalized_budget;
                }
            }
        }

        let authority_level = crate::agent::types::RoleAuthorityLevel::from_role(&a.identity.role);
        let primary_goal = payload
            .primary_goal
            .clone()
            .or_else(|| Some(payload.message.clone()));

        let identity = AgentIdentity {
            agent_id: agent_id.to_string(),
            name: a.identity.name.clone(),
            role: a.identity.role.clone(),
            department: a.identity.department.clone(),
            description: a.identity.description.clone(),
            authority_level,
        };

        let mission_state = MissionState {
            mission_id: mission_id.to_string(),
            cluster_id: payload.cluster_id.clone(),
            user_id: payload.user_id.clone(),
            depth,
            lineage: lineage.to_vec(),
            primary_goal: primary_goal.clone(),
            budget_usd,
            current_cost_usd,
            sub_budget_usd: payload.sub_budget_usd,
        };

        let env = Environment {
            workspace_root: workspace_root.clone(),
            fs_adapter: fs_adapter.clone(),
            base_dir: self.state.base_dir.clone(),
        };

        Ok(RunContext {
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
            model_config: resolved_config.clone(),
            skills,
            workflows,
            agent_models: a.models.clone(),
            mcp_tools,
            provider_name: resolved_config.provider.to_string().to_lowercase(),
            safe_mode,
            analysis: payload.analysis.unwrap_or(false),
            traceparent: payload.traceparent.clone(),
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            modified_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            commands_run: std::sync::Arc::new(parking_lot::Mutex::new(
                std::collections::HashSet::new(),
            )),
            current_dir: std::sync::Arc::new(parking_lot::Mutex::new(None)),
            allowed_files: payload.allowed_files.clone(),
            recent_findings: payload.recent_findings.clone(),
            working_memory: a.state.working_memory.clone(),
            summarized_history: None,
            structured_output: false,
            backlog: None,
            visible_transcript: Some(std::sync::Arc::new(parking_lot::Mutex::new(
                payload.visible_transcript.clone().unwrap_or_default(),
            ))),
            conductor_plan: None,
            reasoning_depth: resolved_config.reasoning_depth.unwrap_or(1),
            act_threshold: resolved_config.act_threshold.unwrap_or(0.9),
            max_turns: resolved_config.max_turns.unwrap_or(20),
            resource_weights: {
                #[cfg(feature = "vector-memory")]
                {
                    crate::utils::data_weighting::DataWeighting::default_weights()
                }
                #[cfg(not(feature = "vector-memory"))]
                {
                    std::collections::HashMap::new()
                }
            },
            graph_context: None,
            verification_passed: false,
        })
    }

    /// Prepares the runtime context, including remote memory (RAG) synchronization.
    pub(crate) async fn prepare_run_context(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
        mission_id: &str,
        depth: u32,
        lineage: &[String],
    ) -> Result<RunContext, AppError> {
        let mut ctx = self
            .resolve_agent_context(agent_id, payload, mission_id, depth, lineage)
            .await?;

        // Sync the new budget back to mission_history so frontend/logs stay aligned.
        // Moved here from resolve_agent_context to keep context resolution strictly read-only (SRP).
        if mission_id != "system-internal" && !mission_id.is_empty() {
            if let Ok(Some(m)) =
                crate::agent::mission::get_mission_by_id(&self.state.resources.pool, mission_id)
                    .await
            {
                if ctx.budget_usd > m.budget_usd {
                    let _ = sqlx::query("UPDATE mission_history SET budget_usd = ?1 WHERE id = ?2")
                        .bind(ctx.budget_usd)
                        .bind(mission_id)
                        .execute(&self.state.resources.pool)
                        .await;
                }
            }
        }

        ctx.structured_output = payload.structured_output.unwrap_or(false);

        // Auto-mount fresh playbooks to workspace on execution
        if let Err(e) = crate::system::okf_gate::mount_playbooks_to_workspace(
            &self.state.resources.pool,
            &ctx.mission_id,
            &ctx.workspace_root,
        )
        .await
        {
            tracing::warn!(
                "⚠️ [OKF Gate] Failed to mount playbooks to workspace: {:?}",
                e
            );
        }
        let backlog = self
            .state
            .registry
            .mission_backlogs
            .entry(mission_id.to_string())
            .or_insert_with(|| {
                std::sync::Arc::new(parking_lot::Mutex::new(
                    crate::agent::backlog::MissionBacklog::new(mission_id),
                ))
            })
            .clone();
        ctx.backlog = Some(backlog);

        // --- 🏗️ [System 2] Relational Code Indexing ---
        // We only perform deep indexing for non-safe missions to preserve resources.
        if !ctx.safe_mode {
            let blueprint = self
                .state
                .resources
                .get_blueprint(ctx.workspace_root.clone())
                .await?;
            ctx.recent_findings = Some(format!(
                "{}\n\n### 🏗️ Codebase Blueprint (System 2 Index)\n{}",
                ctx.recent_findings.clone().unwrap_or_default(),
                serde_json::to_string_pretty(&*blueprint).unwrap_or_default()
            ));
        }

        // 1. Process Mission History & Summarization (4k Threshold)
        self.process_mission_summarization(&mut ctx, mission_id)
            .await;

        // 2. Synchronize Remote Memory (Hybrid RAG)
        self.synchronize_rag_memory(&mut ctx, &payload.message)
            .await;

        if ctx.depth == 0 && !ctx.safe_mode && !Self::is_fast_path_query(&payload.message) {
            match self.generate_conductor_plan(&ctx, &payload.message).await {
                Ok(plan) => {
                    ctx.conductor_plan = Some(plan);
                }
                Err(e) => {
                    tracing::warn!("⚠️ [Conductor] Failed to generate Conductor plan: {:?}", e);
                }
            }
        }

        Ok(ctx)
    }
    // ─────────────────────────────────────────────────────────
    //  PRIVATE HELPERS
    // ─────────────────────────────────────────────────────────

    /// Resolves the base model configuration from registries or agent defaults.
    fn resolve_base_config(
        &self,
        a: &EngineAgent,
        target_model_id: &str,
        slot_cfg: &ModelConfig,
    ) -> Result<ModelConfig, AppError> {
        if let Some(model_entry) = self.state.registry.models.get(target_model_id) {
            self.construct_registry_config(a, &model_entry, slot_cfg)
        } else if let Some(found_entry) = self
            .state
            .registry
            .models
            .iter()
            .find(|kv| kv.value().name.to_lowercase() == target_model_id.to_lowercase())
        {
            self.construct_registry_config(a, found_entry.value(), slot_cfg)
        } else {
            // FALLBACK: Use agent's internal model config for the active slot
            let mut cfg = slot_cfg.clone();
            cfg.model_id = target_model_id.to_string();
            cfg.skills = Some(a.capabilities.skills.clone());
            cfg.workflows = Some(a.capabilities.workflows.clone());
            cfg.mcp_tools = Some(a.capabilities.mcp_tools.clone());
            Ok(cfg)
        }
    }

    /// Helper to build a ModelConfig from registry entries.
    fn construct_registry_config(
        &self,
        a: &EngineAgent,
        m: &ModelEntry,
        slot_cfg: &ModelConfig,
    ) -> Result<ModelConfig, AppError> {
        let provider_config = self
            .state
            .registry
            .providers
            .get(&m.provider_id)
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Provider {} not found for model {}",
                    m.provider_id, m.name
                ))
            })?;

        let model_id = if let Some(pm_id) = &m.provider_model_id {
            pm_id.clone()
        } else if Uuid::parse_str(&m.id).is_ok() {
            m.name.clone()
        } else {
            m.id.clone()
        };

        let mut config = ModelConfig {
            provider: provider_config.protocol,
            model_id,
            api_key: provider_config.api_key.clone(),
            base_url: provider_config.base_url.clone(),
            system_prompt: if provider_config.supports_steering_vectors {
                Some("".to_string())
            } else {
                slot_cfg.system_prompt.clone()
            },
            temperature: slot_cfg.temperature,
            max_tokens: slot_cfg.max_tokens,
            external_id: provider_config.external_id.clone(),
            rpm: m.rpm,
            rpd: m.rpd,
            tpm: m.tpm,
            tpd: m.tpd,
            skills: Some(a.capabilities.skills.clone()),
            workflows: Some(a.capabilities.workflows.clone()),
            mcp_tools: Some(a.capabilities.mcp_tools.clone()),
            steering_vectors: if provider_config.supports_steering_vectors {
                Some(vec![format!("persona:{}", a.identity.role)])
            } else {
                None
            },
            reasoning_depth: slot_cfg.reasoning_depth,
            act_threshold: slot_cfg.act_threshold,
            max_turns: slot_cfg.max_turns,
            connector_configs: None,
            extra_parameters: None,
        };

        if let Some(default) = &provider_config.default_config {
            config = default.merge(&config);
        }

        Ok(config)
    }

    /// Merges task payload overrides into the resolved configuration.
    fn merge_payload_overrides(&self, config: &mut ModelConfig, payload: &TaskPayload) {
        if let Some(p) = payload.provider {
            // SEC-02: Resolve Provider ID to protocol if it exists in registry
            if let Some(provider_config) = self.state.registry.providers.get(&p.to_string()) {
                config.provider = provider_config.protocol;
                if payload.api_key.is_none() {
                    config.api_key = provider_config.api_key.clone();
                }
                if payload.base_url.is_none() {
                    config.base_url = provider_config.base_url.clone();
                }
                if payload.external_id.is_none() {
                    config.external_id = provider_config.external_id.clone();
                }
            } else {
                config.provider = p;
            }
        } else {
            // Fallback: Ensure secrets are pulled from backend registry if protocol matches
            let current_p = config.provider.to_string();
            if let Some(provider_config) = self.state.registry.providers.get(&current_p) {
                if config.api_key.is_none() {
                    config.api_key = provider_config.api_key.clone();
                }
                if config.base_url.is_none() {
                    config.base_url = provider_config.base_url.clone();
                }
                if config.external_id.is_none() {
                    config.external_id = provider_config.external_id.clone();
                }
            }
        }

        if let Some(key) = &payload.api_key {
            config.api_key = Some(key.clone());
        }
        if let Some(url) = &payload.base_url {
            config.base_url = Some(url.clone());
        }
        if let Some(eid) = &payload.external_id {
            config.external_id = Some(eid.clone());
        }
        if let Some(m) = &payload.model_id {
            config.model_id = m.clone();
        }
    }

    /// Resolves and sanitizes the workspace root path.
    fn resolve_workspace_paths(
        &self,
        cluster_id: Option<&str>,
    ) -> Result<std::path::PathBuf, AppError> {
        let workspace_id = cluster_id.unwrap_or("executive-core");
        let mut workspace_root = self.state.base_dir.join("data/workspaces");

        // SEC: Whitelist sanitization — adhere to typical filesystem limitations
        let sanitized_id: String = workspace_id
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();

        if sanitized_id.is_empty() {
            return Err(AppError::BadRequest(format!(
                "Invalid workspace ID: '{}'",
                workspace_id
            )));
        }
        workspace_root.push(sanitized_id);

        // Dynamically ensure that the workspace root directory exists
        if !workspace_root.exists() {
            std::fs::create_dir_all(&workspace_root).map_err(|e| {
                AppError::InternalServerError(format!(
                    "Failed to create workspace directory: {}. Error: {}",
                    workspace_root.display(),
                    e
                ))
            })?;
        }

        Ok(workspace_root)
    }

    /// Applies security restrictions to the tool list if safe_mode is active.
    fn apply_security_gates(
        &self,
        safe_mode: bool,
        skills: &mut Vec<String>,
        workflows: &mut Vec<String>,
    ) {
        if safe_mode {
            skills.retain(|s| {
                if let Some(tool) = self.state.registry.tool_registry.get(s) {
                    !tool.is_dangerous()
                } else if let Some(skill_manifest) = self.state.registry.skills.skills.get(s) {
                    !skill_manifest.oversight_required
                } else {
                    // Block unknown tools in safe mode by default
                    false
                }
            });
            workflows.clear();
        }
    }

    /// Processes history summarization if the token count exceeds the threshold.
    async fn process_mission_summarization(&self, ctx: &mut RunContext, mission_id: &str) {
        const CONTEXT_LOG_LIMIT: i64 = 250;
        const TOKEN_SUMMARIZATION_THRESHOLD: usize = 4000;

        let logs = match crate::agent::mission::get_recent_mission_logs(
            &self.state.resources.pool,
            mission_id,
            CONTEXT_LOG_LIMIT,
        )
        .await
        {
            Ok(logs) => logs,
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Runner] Failed to fetch mission logs for summarization: {}",
                    e
                );
                return;
            }
        };

        let history_text: String = logs
            .iter()
            .filter(|l| l.severity != "debug")
            .map(|l| format!("[{}]: {}", l.source, l.text))
            .collect::<Vec<_>>()
            .join("\n");

        if ContextManager::calculate_tokens(&history_text) > TOKEN_SUMMARIZATION_THRESHOLD {
            tracing::info!(
                "💡 [Runner] Mission history exceeds 4k tokens for {}. Summarizing...",
                mission_id
            );
            match ContextManager::summarize_history(self, ctx, &history_text).await {
                Ok(summary) => {
                    ctx.summarized_history = Some(summary);
                    tracing::info!("✅ [Runner] Context summarized for {}.", mission_id);
                }
                Err(e) => {
                    // SEC-FAILSAFE: Summarization failure should not block the mission.
                    // We log a warning and continue, accepting potential token cutoff.
                    tracing::warn!(
                        "⚠️ [Runner] Context summarization failed (continuing without summary): {}",
                        e
                    );
                }
            }
        }
    }

    /// Synchronizes remote memory via hybrid search, TrustGraph BFS traversal, and multi-factor reranking.
    async fn synchronize_rag_memory(&self, ctx: &mut RunContext, initial_prompt: &str) {
        #[cfg(not(feature = "vector-memory"))]
        let _ = (ctx, initial_prompt);

        #[cfg(feature = "vector-memory")]
        {
            let (_, agent_memory_dir, mission_scope_dir) = ctx.resolve_paths();
            let agent_mem =
                match crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories")
                    .await
                {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!("⚠️ [RAG] Failed to connect to agent VectorMemory: {}", e);
                        return;
                    }
                };
            let mission_mem = match crate::agent::memory::VectorMemory::connect(
                &mission_scope_dir,
                "scope",
            )
            .await
            {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [RAG] Failed to connect to mission VectorMemory scope: {}",
                        e
                    );
                    return;
                }
            };

            let client = (*self.state.resources.http_client).clone();
            let provider = self.resolve_provider(ctx, client).await;

            let vec = match provider.embed(&initial_prompt).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(
                        "🚨 [RAG] Failed to generate initial prompt embedding: {}",
                        e
                    );
                    return;
                }
            };

            // 1. High-Fidelity Vector Retrieval
            let raw_entries = match agent_mem.search_knowledge_full(vec.clone(), 15).await {
                Ok(entries) => entries,
                Err(e) => {
                    tracing::error!("🚨 [RAG] Failed to retrieve vector memories: {}", e);
                    return;
                }
            };

            // 2. Multi-Factor Reranking (modality source-aware)
            let scoring_config = crate::types::rag_scoring::ScoringConfig::default();
            let ranked_entries = crate::utils::data_weighting::DataWeighting::rank_memories(
                raw_entries,
                Some(&ctx.mission_id),
                &scoring_config,
            );

            // 3. Extract BFS adjacent context from SQLite TrustGraph
            let seed_ids: Vec<String> = ranked_entries
                .iter()
                .take(5)
                .map(|e| e.id.clone())
                .collect();
            let graph_engine =
                crate::agent::trustgraph::TrustGraphEngine::new(self.state.resources.pool.clone());

            let graph_context = match graph_engine.traverse_subgraph(&seed_ids, 2).await {
                Ok(subgraph) => {
                    let mut ctx_str = String::new();
                    if !subgraph.nodes.is_empty() {
                        ctx_str.push_str("#### Semantic Nodes:\n");
                        for node in &subgraph.nodes {
                            ctx_str.push_str(&format!(
                                "- Entity [{}] ({}) - Name: {}, Description: {}\n",
                                node.id,
                                node.r#type,
                                node.name,
                                node.description
                                    .as_deref()
                                    .unwrap_or("No details available")
                            ));
                        }
                    }
                    if !subgraph.relations.is_empty() {
                        ctx_str.push_str("\n#### Semantic Relations:\n");
                        for rel in &subgraph.relations {
                            ctx_str.push_str(&format!(
                                "- Node [{}] --({})--> Node [{}] (Weight: {})\n",
                                rel.source_entity_id,
                                rel.relation_type,
                                rel.target_entity_id,
                                rel.weight
                            ));
                        }
                    }
                    ctx_str
                }
                Err(e) => {
                    tracing::error!("🚨 [RAG] TrustGraph traversal failed: {}", e);
                    String::new()
                }
            };

            if !graph_context.is_empty() {
                ctx.graph_context = Some(graph_context);
            }

            // 4. Synthesize context and inject into active mission scope vector memory
            let top_results: Vec<_> = ranked_entries.into_iter().take(5).collect();
            let mut embeddings = Vec::new();
            let mut final_results = Vec::new();

            // Use bounded concurrency to avoid triggering provider rate limits (429s)
            use futures::StreamExt;
            let embedding_futures = top_results.iter().map(|entry| provider.embed(&entry.text));
            let results: Vec<_> = futures::stream::iter(embedding_futures)
                .buffer_unordered(3)
                .collect()
                .await;

            for (entry, res) in top_results.into_iter().zip(results.into_iter()) {
                if let Ok(v) = res {
                    embeddings.push(v);
                    final_results.push(entry);
                } else {
                    tracing::warn!("⚠️ [RAG] Failed to generate embedding for entry, skipping to avoid semantic pollution");
                }
            }
            let deduplicated_results =
                crate::utils::deduplicator::SwarmDeduplicator::deduplicate_semantic(
                    &final_results,
                    &embeddings,
                    0.95,
                );
            let top_results_len = deduplicated_results.len();

            // Sliding Window Eviction
            const MAX_SCOPE_ENTRIES: usize = 25;
            if let Ok(existing) = mission_mem.get_all_memories(&ctx.mission_id).await {
                if existing.len() + top_results_len > MAX_SCOPE_ENTRIES {
                    let to_delete_count =
                        (existing.len() + top_results_len).saturating_sub(MAX_SCOPE_ENTRIES);
                    let to_delete_ids: Vec<String> = existing
                        .iter()
                        .take(to_delete_count)
                        .map(|(id, _)| id.clone())
                        .collect();
                    if !to_delete_ids.is_empty() {
                        let _ = mission_mem.delete_memories(to_delete_ids).await;
                    }
                }
            }

            let mut count = 0;
            for entry in deduplicated_results {
                let final_context = entry.text;
                let unique_id = format!("mem-{}-{}", ctx.mission_id, uuid::Uuid::new_v4());
                if mission_mem
                    .add_memory(&unique_id, &final_context, &ctx.mission_id, vec.clone())
                    .await
                    .is_ok()
                {
                    count += 1;
                }
            }
            tracing::info!("🧠 [RAG] Hybrid GraphRAG + Multi-Factor Rerank: Injected {} refined findings into mission scope", count);
        }
    }
}

impl RunContext {
    /// Cleans and filters raw conversation history (e.g., removing thought blocks
    /// and truncating/summarizing large tool outputs) to produce a sandboxed visible transcript.
    /// (PD-002: Delegated to `turn_compactor` module).
    #[allow(dead_code)]
    pub fn build_sandboxed_transcript(&self, raw_history: &[String]) -> Vec<String> {
        super::turn_compactor::build_sandboxed_transcript(&self.role, raw_history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_mock_runner() -> AgentRunner {
        let state = Arc::new(AppState::new_minimal_mock().await);
        AgentRunner::new(state)
    }

    #[tokio::test]
    async fn test_workspace_path_sanitization() {
        let runner = setup_mock_runner().await;

        // 1. Valid ID
        let path = runner.resolve_workspace_paths(Some("cluster-123")).unwrap();
        assert!(path.to_string_lossy().contains("cluster-123"));

        // 2. Path Traversal Attempt
        let path = runner
            .resolve_workspace_paths(Some("../../../etc/passwd"))
            .unwrap();
        let path_str = path.to_string_lossy();
        assert!(!path_str.contains(".."));
        assert!(path_str.contains("etcpasswd")); // Sanitized version
    }

    #[tokio::test]
    async fn test_security_gates_safe_mode() {
        let runner = setup_mock_runner().await;

        // Register custom_skill in the mock registry so it passes safety checks
        let skill_def = crate::agent::script_skills::SkillDefinition {
            id: None,
            name: "custom_skill".to_string(),
            description: "Test skill".to_string(),
            execution_command: "echo test".to_string(),
            schema: serde_json::json!({}),
            oversight_required: false,
            doc_url: None,
            tags: None,
            full_instructions: None,
            negative_constraints: None,
            verification_script: None,
            category: "user".to_string(),
            security_score: None,
            security_severity: None,
            security_report: None,
        };
        runner
            .state
            .registry
            .skills
            .skills
            .insert("custom_skill".to_string(), skill_def);

        let mut skills = vec![
            "read_file".to_string(),
            "write_file".to_string(),
            "custom_skill".to_string(),
        ];
        let mut workflows = vec!["deploy_flow".to_string()];

        // 1. Safe Mode OFF
        runner.apply_security_gates(false, &mut skills, &mut workflows);
        assert_eq!(skills.len(), 3);
        assert_eq!(workflows.len(), 1);

        // 2. Safe Mode ON
        runner.apply_security_gates(true, &mut skills, &mut workflows);
        assert!(skills.contains(&"read_file".to_string()));
        assert!(!skills.contains(&"write_file".to_string()));
        assert!(skills.contains(&"custom_skill".to_string()));
        assert_eq!(workflows.len(), 0); // Workflows cleared in safe mode
    }

    #[tokio::test]
    async fn test_payload_overrides() {
        let runner = setup_mock_runner().await;
        let mut base_config = ModelConfig {
            model_id: "base-model".to_string(),
            api_key: Some("base-key".to_string()),
            ..Default::default()
        };
        let payload = TaskPayload {
            model_id: Some("override-model".to_string()),
            api_key: Some("override-key".to_string()),
            ..TaskPayload::default()
        };

        runner.merge_payload_overrides(&mut base_config, &payload);
        assert_eq!(base_config.model_id, "override-model");
        assert_eq!(base_config.api_key, Some("override-key".to_string()));
    }
}

// Metadata: [context]
