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

use super::{AgentRunner, RunContext};

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
        let entry = self.state.registry.agents.get(agent_id)
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;
        let a = entry.value();

        // 1. Resolve Target Model ID
        let target_model_id = payload.model_id.as_ref()
            .or(match a.models.active_model_slot {
                Some(2) => a.models.model_2.as_ref(),
                Some(3) => a.models.model_3.as_ref(),
                _ => a.models.model_id.as_ref(),
            })
            .unwrap_or(&a.models.model.model_id)
            .clone();

        // 2. Resolve Base Configuration
        let slot_cfg = match a.models.active_model_slot {
            Some(2) => a.models.model_config2.as_ref().unwrap_or(&a.models.model),
            Some(3) => a.models.model_config3.as_ref().unwrap_or(&a.models.model),
            _ => &a.models.model,
        };
        let mut resolved_config = self.resolve_base_config(a, &target_model_id, slot_cfg)?;

        // 3. Apply Payload Overrides
        self.merge_payload_overrides(&mut resolved_config, payload);

        // 4. Resolve Workspace Root
        let workspace_root = self.resolve_workspace_paths(payload.cluster_id.as_deref())?;
        let fs_adapter = crate::adapter::filesystem::FilesystemAdapter::new(workspace_root.clone());

        // 5. Capability Merging & Security Gates
        let mut skills = resolved_config.skills.clone().unwrap_or_else(|| a.capabilities.skills.clone());
        let mut workflows = resolved_config.workflows.clone().unwrap_or_else(|| a.capabilities.workflows.clone());
        let mcp_tools = resolved_config.mcp_tools.clone().unwrap_or_else(|| a.capabilities.mcp_tools.clone());

        let safe_mode = payload.safe_mode.unwrap_or(false);
        self.apply_security_gates(safe_mode, &mut skills, &mut workflows);

        // 6. Mission Financials
        let (budget_usd, current_cost_usd) = if mission_id != "system-internal" && !mission_id.is_empty() {
            match crate::agent::mission::get_mission_by_id(&self.state.resources.pool, mission_id).await {
                Ok(Some(m)) => (m.budget_usd, m.cost_usd),
                _ => (payload.budget_usd.unwrap_or(0.0), 0.0),
            }
        } else {
            (0.0, 0.0)
        };

        Ok(RunContext {
            agent_id: agent_id.to_string(),
            name: a.identity.name.clone(),
            role: a.identity.role.clone(),
            department: a.identity.department.clone(),
            description: a.identity.description.clone(),
            model_config: resolved_config.clone(),
            skills,
            workflows,
            mcp_tools,
            mission_id: mission_id.to_string(),
            user_id: payload.user_id.clone(),
            depth,
            lineage: lineage.to_vec(),
            provider_name: resolved_config.provider.to_string().to_lowercase(),
            workspace_root,
            fs_adapter,
            safe_mode,
            analysis: payload.analysis.unwrap_or(false),
            traceparent: payload.traceparent.clone(),
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: payload.recent_findings.clone(),
            working_memory: a.state.working_memory.clone(),
            base_dir: self.state.base_dir.clone(),
            summarized_history: None,
            structured_output: false,
            backlog: None,
            primary_goal: payload.primary_goal.clone().or_else(|| Some(payload.message.clone())),
            budget_usd,
            current_cost_usd,
            reasoning_depth: resolved_config.reasoning_depth.unwrap_or(1),
            act_threshold: resolved_config.act_threshold.unwrap_or(0.9),
            max_turns: resolved_config.max_turns.unwrap_or(20),
            authority_level: crate::agent::types::RoleAuthorityLevel::from_role(&a.identity.role),
            resource_weights: crate::utils::data_weighting::DataWeighting::default_weights(),
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
        let mut ctx = self.resolve_agent_context(agent_id, payload, mission_id, depth, lineage).await?;

        ctx.structured_output = payload.structured_output.unwrap_or(false);
        ctx.backlog = None; // Reserved for global backlog injection

        // 1. Process Mission History & Summarization (4k Threshold)
        self.process_mission_summarization(&mut ctx, mission_id).await;

        // 2. Synchronize Remote Memory (Hybrid RAG)
        self.synchronize_rag_memory(&ctx, &payload.message).await;

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
        } else if let Some(found_entry) = self.state.registry.models.iter().find(|kv| {
            kv.value().name.to_lowercase() == target_model_id.to_lowercase()
        }) {
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
        let provider_config = self.state.registry.providers.get(&m.provider_id).ok_or_else(|| {
            AppError::NotFound(format!("Provider {} not found for model {}", m.provider_id, m.name))
        })?;

        let model_id = if provider_config.protocol == crate::agent::types::ModelProvider::Ollama {
            // For Ollama, we prefer names/tags over UUIDs if possible
            if Uuid::parse_str(&m.id).is_ok() { m.name.clone() } else { m.id.clone() }
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
                if payload.api_key.is_none() { config.api_key = provider_config.api_key.clone(); }
                if payload.base_url.is_none() { config.base_url = provider_config.base_url.clone(); }
                if payload.external_id.is_none() { config.external_id = provider_config.external_id.clone(); }
            } else {
                config.provider = p;
            }
        } else {
            // Fallback: Ensure secrets are pulled from backend registry if protocol matches
            let current_p = config.provider.to_string();
            if let Some(provider_config) = self.state.registry.providers.get(&current_p) {
                if config.api_key.is_none() { config.api_key = provider_config.api_key.clone(); }
                if config.base_url.is_none() { config.base_url = provider_config.base_url.clone(); }
                if config.external_id.is_none() { config.external_id = provider_config.external_id.clone(); }
            }
        }

        if let Some(key) = &payload.api_key { config.api_key = Some(key.clone()); }
        if let Some(url) = &payload.base_url { config.base_url = Some(url.clone()); }
        if let Some(eid) = &payload.external_id { config.external_id = Some(eid.clone()); }
        if let Some(m) = &payload.model_id { config.model_id = m.clone(); }
    }

    /// Resolves and sanitizes the workspace root path.
    fn resolve_workspace_paths(&self, cluster_id: Option<&str>) -> Result<std::path::PathBuf, AppError> {
        let workspace_id = cluster_id.unwrap_or("executive-core");
        let mut workspace_root = self.state.base_dir.join("data/workspaces");
        
        // SEC: Whitelist sanitization — adhere to typical filesystem limitations
        let sanitized_id: String = workspace_id
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();

        if sanitized_id.is_empty() {
            return Err(AppError::BadRequest(format!("Invalid workspace ID: '{}'", workspace_id)));
        }
        workspace_root.push(sanitized_id);
        Ok(workspace_root)
    }

    /// Applies security restrictions to the tool list if safe_mode is active.
    fn apply_security_gates(&self, safe_mode: bool, skills: &mut Vec<String>, workflows: &mut Vec<String>) {
        if safe_mode {
            let blacklisted_skills = [
                "issue_alpha_directive", "spawn_subagent", "execute_bash",
                "write_file", "delete_file", "append_file", "deploy",
                "list_files", "fetch_url", "search_mission_knowledge",
                "script_builder", "read_codebase_file",
            ];
            skills.retain(|s| !blacklisted_skills.contains(&s.as_str()));
            workflows.clear();
        }
    }

    /// Processes history summarization if the token count exceeds the threshold.
    async fn process_mission_summarization(&self, ctx: &mut RunContext, mission_id: &str) {
        const CONTEXT_LOG_LIMIT: i64 = 250;
        const TOKEN_SUMMARIZATION_THRESHOLD: usize = 4000;

        let logs = match crate::agent::mission::get_recent_mission_logs(&self.state.resources.pool, mission_id, CONTEXT_LOG_LIMIT).await {
            Ok(logs) => logs,
            Err(e) => {
                tracing::warn!("⚠️ [Runner] Failed to fetch mission logs for summarization: {}", e);
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
            tracing::info!("💡 [Runner] Mission history exceeds 4k tokens for {}. Summarizing...", mission_id);
            match ContextManager::summarize_history(self, ctx, &history_text).await {
                Ok(summary) => {
                    ctx.summarized_history = Some(summary);
                    tracing::info!("✅ [Runner] Context summarized for {}.", mission_id);
                }
                Err(e) => {
                    // SEC-FAILSAFE: Summarization failure should not block the mission.
                    // We log a warning and continue, accepting potential token cutoff.
                    tracing::warn!("⚠️ [Runner] Context summarization failed (continuing without summary): {}", e);
                }
            }
        }
    }

    /// Synchronizes remote memory via hybrid search and reranking.
    async fn synchronize_rag_memory(&self, ctx: &RunContext, initial_prompt: &str) {
        let _ = (ctx, initial_prompt);
        #[cfg(feature = "vector-memory")]
        {
            let (_, agent_memory_dir, mission_scope_dir) = ctx.resolve_paths();
            let prompt_words: std::collections::HashSet<String> = initial_prompt
                .to_lowercase()
                .split_whitespace()
                .map(str::to_string)
                .collect();

            let agent_mem = match crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories").await {
                Ok(m) => m,
                Err(_) => return,
            };
            let mission_mem = match crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await {
                Ok(m) => m,
                Err(_) => return,
            };

            let client = (*self.state.resources.http_client).clone();
            let provider = self.resolve_provider(&ctx, client);

            if let Ok(vec) = provider.embed(&initial_prompt).await {
                let vec: Vec<f32> = vec;
                // Phase 1: Hybrid Search (Vector + Keywords)
                if let Ok(results) = agent_mem.search_knowledge_hybrid(&initial_prompt, vec.clone(), 10).await {
                    // Phase 2: Reranking Pass
                    // Heuristic: Weighted word intersection count to prioritize contextual relevance.
                    let mut scored_results: Vec<(f32, String)> = results
                        .into_iter()
                        .map(|text| {
                            let mut score = 0.0;
                            let text_lower = text.to_lowercase();
                            let text_words: std::collections::HashSet<&str> = text_lower.split_whitespace().collect();
                            let intersection = text_words.iter().filter(|word| prompt_words.contains(**word)).count();
                            score += intersection as f32 * 0.1;
                            (score, text)
                        })
                        .collect();

                    scored_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

                    let top_results: Vec<String> = scored_results.into_iter().take(5).map(|(_, t)| t).collect();
                    let count = top_results.len();

                    for (i, text) in top_results.into_iter().enumerate() {
                        let _ = mission_mem.add_memory(&format!("mem-{}", i), &text, &ctx.mission_id, vec.clone()).await;
                    }
                    tracing::info!("🧠 [RAG] Hybrid Search + Rerank: Injected {} refined findings into mission scope", count);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_mock_runner() -> AgentRunner {
        let state = Arc::new(AppState::new_minimal_mock().await);
        AgentRunner {
            state,
        }
    }

    #[tokio::test]
    async fn test_workspace_path_sanitization() {
        let runner = setup_mock_runner().await;
        
        // 1. Valid ID
        let path = runner.resolve_workspace_paths(Some("cluster-123")).unwrap();
        assert!(path.to_string_lossy().contains("cluster-123"));

        // 2. Path Traversal Attempt
        let path = runner.resolve_workspace_paths(Some("../../../etc/passwd")).unwrap();
        let path_str = path.to_string_lossy();
        assert!(!path_str.contains(".."));
        assert!(path_str.contains("etcpasswd")); // Sanitized version
    }

    #[tokio::test]
    async fn test_security_gates_safe_mode() {
        let runner = setup_mock_runner().await;
        let mut skills = vec!["read_file".to_string(), "write_file".to_string(), "custom_skill".to_string()];
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

// Metadata: [context]
