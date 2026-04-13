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

use uuid::Uuid;
use crate::agent::context_manager::ContextManager;
use crate::agent::types::{ModelConfig, TaskPayload};

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
    ) -> anyhow::Result<RunContext> {
        let entry = self
            .state
            .registry
            .agents
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not found", agent_id))?;
        let a = entry.value();

        // 1. RESOLVE MODEL ID
        let target_model_id = payload
            .model_id
            .clone()
            .or_else(|| a.model_id.clone())
            .unwrap_or_else(|| a.model.model_id.clone());

        // CENTRAL REGISTRY PATH: Resolve full config from model + provider registries
        let mut resolved_config = if let Some(model_entry) =
            self.state.registry.models.get(&target_model_id)
        {
            let model_id = model_entry.id.clone();
            let provider_id = model_entry.provider_id.clone();

            let provider_config =
                self.state
                    .registry
                    .providers
                    .get(&provider_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Provider {} not found for model {}",
                            provider_id,
                            target_model_id
                        )
                    })?;

            let base_config = ModelConfig {
                provider: provider_config.protocol,
                model_id: if provider_config.protocol == crate::agent::types::ModelProvider::Ollama && Uuid::parse_str(&model_id).is_ok() {
                    tracing::debug!("🔄 [Context] Resolving Ollama UUID {} to tag: {}", model_id, model_entry.name);
                    model_entry.name.clone()
                } else {
                    model_id
                },
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
                connector_configs: None,
                extra_parameters: None,
            };

            if let Some(default) = &provider_config.default_config {
                default.merge(&base_config)
            } else {
                base_config
            }
        } else if let Some(found_entry) = self
            .state
            .registry
            .models
            .iter()
            .find(|kv| kv.value().name.to_lowercase() == target_model_id.to_lowercase())
        {
            // FUZZY RESOLUTION: ID might be a friendly name from the UI
            let m = found_entry.value();
            let model_id = if m.provider_id.to_lowercase() == "ollama" {
                m.name.clone()
            } else {
                m.id.clone()
            };
            let provider_id = m.provider_id.clone();

            let provider_config =
                self.state
                    .registry
                    .providers
                    .get(&provider_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Provider {} not found for model {}", provider_id, m.name)
                    })?;

            let base_config = ModelConfig {
                provider: provider_config.protocol,
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
                connector_configs: None,
                extra_parameters: None,
            };

            if let Some(default) = &provider_config.default_config {
                default.merge(&base_config)
            } else {
                base_config
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
        if let Some(p) = payload.provider {
            // SEC-02: Check if this is a known Provider ID. If so, resolve to its protocol.
            // This prevents "unknown_provider (local)" errors when IDs are passed instead of protocols.
            let p_str = p.to_string();
            if let Some(provider_config) = self.state.registry.providers.get(&p_str) {
                resolved_config.provider = provider_config.protocol;
                // Optionally pull other defaults if they aren't provided in the payload
                if payload.api_key.is_none() {
                    resolved_config.api_key = provider_config.api_key.clone();
                }
                if payload.base_url.is_none() {
                    resolved_config.base_url = provider_config.base_url.clone();
                }
                if payload.external_id.is_none() {
                    resolved_config.external_id = provider_config.external_id.clone();
                }
            } else {
                resolved_config.provider = p;
            }
        } else {
            // No payload override: Resolve the fallback provider ID to a protocol if it exists in registry
            let current_p = &resolved_config.provider.to_string();
            if let Some(provider_config) = self.state.registry.providers.get(current_p) {
                resolved_config.provider = provider_config.protocol;
                // Ensure secrets are pulled from backend registry even if agent record is stale
                if resolved_config.api_key.is_none() {
                    resolved_config.api_key = provider_config.api_key.clone();
                }
                if resolved_config.base_url.is_none() {
                    resolved_config.base_url = provider_config.base_url.clone();
                }
                if resolved_config.external_id.is_none() {
                    resolved_config.external_id = provider_config.external_id.clone();
                }
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

        let provider_name = resolved_config.provider.to_string().to_lowercase();

        // Workspace Anchoring: Map clusterId to a physical path in ./workspaces
        let workspace_id = payload.cluster_id.as_deref().unwrap_or("executive-core"); // Default fallback

        let mut workspace_root = self.state.base_dir.join("data/workspaces");
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
            name: a.name.clone(),
            role: a.role.clone(),
            department: a.department.clone(),
            description: a.description.clone(),
            model_config: resolved_config,
            skills,
            workflows,
            mcp_tools,
            mission_id: mission_id.to_string(),
            user_id: payload.user_id.clone(),
            depth,
            lineage: lineage.to_vec(),
            provider_name,
            workspace_root,
            fs_adapter,
            safe_mode,
            analysis: payload.analysis.unwrap_or(false),
            traceparent: payload.traceparent.clone(),
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: payload.recent_findings.clone(),
            working_memory: a.working_memory.clone(),
            base_dir: self.state.base_dir.clone(),
            summarized_history: None,
            structured_output: false,
            backlog: None,
            primary_goal: payload
                .primary_goal
                .clone()
                .or_else(|| Some(payload.message.clone())),
            budget_usd,
            current_cost_usd,
        })
    }

    /// Prepares the runtime context, including remote memory (RAG) synchronization.
    ///
    /// Spawns a background task to perform semantic search across agent
    /// history and mission scope, injecting findings into the local prompt.
    pub(crate) async fn prepare_run_context(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
        mission_id: &str,
        depth: u32,
        lineage: &[String],
    ) -> anyhow::Result<RunContext> {
        const CONTEXT_LOG_LIMIT: i64 = 250;
        let mut ctx = self.resolve_agent_context(agent_id, payload, mission_id, depth, lineage).await?;

        ctx.structured_output = payload.structured_output.unwrap_or(false);
        // Backlog is currently initialized as None; in a full implementation,
        // we would fetch/create it from a global registry or state.
        ctx.backlog = None;

        // 🧠 CONTEXT SUMMARIZATION (4k Token Threshold)
        // If the mission has a long history, we summarize it to save prompt tokens
        // and reduce latency.
        let logs = crate::agent::mission::get_recent_mission_logs(
            &self.state.resources.pool,
            mission_id,
            CONTEXT_LOG_LIMIT,
        )
        .await
        .unwrap_or_default();

        let history_text: String = logs
            .iter()
            .filter(|l| l.severity != "debug") // Only meaningful logs
            .map(|l| format!("[{}]: {}", l.source, l.text))
            .collect::<Vec<_>>()
            .join("\n");

        if ContextManager::calculate_tokens(&history_text) > 4000 {
            tracing::info!(
                "💡 [Runner] Mission history exceeds 4k tokens for {}. Summarizing...",
                mission_id
            );
            match ContextManager::summarize_history(self, &ctx, &history_text).await {
                Ok(summary) => {
                    ctx.summarized_history = Some(summary);
                    tracing::info!("✅ [Runner] Context summarized for {}.", mission_id);
                }
                Err(e) => {
                    tracing::error!("❌ [Runner] Context summarization failed: {}", e);
                }
            }
        }

        // 🧠 RAG INJECTION (IDENTITY & SCOPE)
        // We resolve previous mission findings and identity context
        // synchronously before prompt synthesis to ensure first-turn parity.
        #[cfg(feature = "vector-memory")]
        {
            let (_, agent_memory_dir, mission_scope_dir) = ctx.resolve_paths();
            let initial_prompt = payload.message.clone();
            let prompt_words: std::collections::HashSet<String> = initial_prompt
                .to_lowercase()
                .split_whitespace()
                .map(str::to_string)
                .collect();

            if let Ok(agent_mem) =
                crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories").await
            {
                if let Ok(mission_mem) =
                    crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await
                {
                    let client = (*self.state.resources.http_client).clone();
                    let provider = self.resolve_provider(&ctx, client);

                    if let Ok(vec) = provider.embed(&initial_prompt).await {
                        let vec: Vec<f32> = vec;
                        // Phase 1: Use Hybrid Search (Vector + Keywords)
                        if let Ok(results) = agent_mem
                            .search_knowledge_hybrid(&initial_prompt, vec.clone(), 10)
                            .await
                        {
                            // Phase 2: Reranking Pass (Heuristic for now)
                            let mut scored_results: Vec<(f32, String)> = results
                                .into_iter()
                                .map(|text| {
                                    let mut score = 0.0;
                                    let text_lower = text.to_lowercase();
                                    let text_words: std::collections::HashSet<&str> =
                                        text_lower.split_whitespace().collect();
                                    let intersection = text_words
                                        .iter()
                                        .filter(|word| prompt_words.contains(**word))
                                        .count();
                                    score += intersection as f32 * 0.1;
                                    (score, text)
                                })
                                .collect();

                            scored_results.sort_by(|a, b| {
                                b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
                            });

                            let top_results: Vec<String> =
                                scored_results.into_iter().take(5).map(|(_, t)| t).collect();
                            let count = top_results.len();

                            for (i, text) in top_results.into_iter().enumerate() {
                                let _ = mission_mem
                                    .add_memory(&format!("mem-{}", i), &text, mission_id, vec.clone())
                                    .await;
                            }
                            tracing::info!(
                                "🧠 [RAG] Hybrid Search + Rerank: Injected {} refined findings into mission scope for {}",
                                count,
                                agent_id
                            );
                        }
                    }
                }
            }
        }

        Ok(ctx)
    }
}
