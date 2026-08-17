//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Swarm Recruitment**: Coordinates Tier 1/2/3 recruitment and fabrication
//! of specialized sub-agents into the SQLite database and registry.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Registry sync lag, database connection pool exhaustion.

use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{swarm::SubAgentOptions, AgentRunner};

impl AgentRunner {
    /// ### 🧠 Orchestration: Tiered Recruitment (ensure_sub_agent_exists)
    /// Guarantees that a sub-agent exists in the swarm's memory registry before a
    /// mission dispatch. Implements a prioritized search strategy to optimize for
    /// specialist continuity.
    ///
    /// ### 🧬 Search Strategy: Priority Tiers
    /// 1. **Tier 1 (Specialist)**: Searches for "user-sector" agents identified
    ///    by `name` or `role`. These are typically highly-refined or fine-tuned
    ///    specialists manually configured by the user.
    /// 2. **Tier 2 (Swarm Pool)**: Searches for existing "ai-sector" agents
    ///    that have participated in previous missions. These agents may have
    ///    accumulated "Experience" in their persistent working memory.
    /// 3. **Tier 3 (Fabrication)**: If no match is found, a new agent node is
    ///    atomically created, registered in the registry, and persisted to SQLite.
    pub(crate) async fn ensure_sub_agent_exists(
        &self,
        conn: &mut sqlx::SqliteConnection,
        opts: SubAgentOptions<'_>,
    ) -> Result<String, ToolExecutionError> {
        // Register any new unique capabilities into AI Services registry
        if let Some(skills) = opts.extra_skills {
            for s in skills {
                if let Ok(name) = self
                    .state
                    .registry
                    .skills
                    .register_capability("skill", s.clone(), "ai")
                    .await
                {
                    tracing::info!("Registered discovered skill: {}", name);
                }
            }
        }
        if let Some(workflows) = opts.extra_workflows {
            for w in workflows {
                if let Ok(name) = self
                    .state
                    .registry
                    .skills
                    .register_capability("workflow", w.clone(), "ai")
                    .await
                {
                    tracing::info!("Registered discovered workflow: {}", name);
                }
            }
        }

        // MANAGER PRIORITY: Tiered Recruitment Search
        // 1. Tier 1: User Sector Specialists (Highest Priority)
        // 2. Tier 2: Existing AI Swarm Brains (Experienced Pool)
        // 3. Tier 3: Spawn New (Fallback)
        let mut target_id = opts.agent_id.to_string();
        let mut target_config = opts.parent_config.clone();

        let mut tier1_match = None;
        let mut tier2_match = None;

        for kv in self.state.registry.agents.iter() {
            let a = kv.value();
            let is_match = a.identity.name.eq_ignore_ascii_case(opts.agent_id)
                || a.identity
                    .role
                    .to_lowercase()
                    .contains(&opts.agent_id.to_lowercase());

            if is_match {
                if a.identity.category == "user" {
                    tier1_match = Some(a.clone());
                    break; // Tier 1 is highest priority, stop search
                } else if a.identity.category == "ai" && tier2_match.is_none() {
                    tier2_match = Some(a.clone());
                }
            }
        }
        let registry_match = tier1_match.or(tier2_match);

        if let Some(mut matched_agent) = registry_match {
            tracing::info!("🎯 [Swarm] Priority Match: Found existing specialist '{}' ({}) in category '{}' to fulfill request for '{}'", 
                matched_agent.identity.name, matched_agent.identity.id, matched_agent.identity.category, opts.agent_id);

            target_id = matched_agent.identity.id.clone();

            // Persistent Swarm Tagging
            matched_agent.metadata.insert(
                "has_participated_in_swarm".to_string(),
                serde_json::Value::Bool(true),
            );
            matched_agent.health.status = "active".to_string(); // Force active status for UI visibility

            // If the matched agent has a specific model config, verify if it is configured/healthy.
            let is_configured = match matched_agent.models.model.provider {
                crate::agent::types::ModelProvider::Ollama => true,
                ref p => {
                    let env_var = p.default_env_key();
                    matched_agent.models.model.api_key.is_some() || std::env::var(env_var).is_ok()
                }
            };

            if !is_configured {
                tracing::warn!(
                    "⚠️ [Swarm] Resolved provider {:?} for sub-agent '{}' is unconfigured. Falling back to parent agent model config ({:?}).",
                    matched_agent.models.model.provider,
                    target_id,
                    opts.parent_config.provider
                );
                matched_agent.models.model = opts.parent_config.clone();
                matched_agent.models.model_id = Some(opts.parent_config.model_id.clone());
            }

            // Sync changes to DB mutably first
            let _ = crate::agent::persistence::save_agent_db_in_tx(&mut *conn, &mut matched_agent)
                .await
                .map_err(|e| {
                    tracing::warn!(
                        "⚠️ [Swarm] Failed to persist recruited agent {} to DB: {}",
                        target_id,
                        e
                    )
                });

            // Sync changes to Registry
            self.state
                .registry
                .agents
                .insert(target_id.clone(), matched_agent.clone());

            if let Some(mid) = &matched_agent.models.model_id {
                target_config.model_id = mid.clone();
                target_config.provider = matched_agent.models.model.provider;
            }
        }

        // Verify skill and environment dependencies for the recruited agent
        let target_skills = if let Some(agent) = self.state.registry.agents.get(&target_id) {
            agent.capabilities.skills.clone()
        } else {
            // New fabricated agent skills
            let mut s = Self::get_default_skills();
            if let Some(skills) = opts.extra_skills {
                for s_val in skills {
                    if let Some(name) = s_val.get("name").and_then(|v| v.as_str()) {
                        if !s.contains(&name.to_string()) {
                            s.push(name.to_string());
                        }
                    }
                }
            }
            s
        };

        if let Err(missing) =
            crate::security::dependency_guard::check_skill_dependencies(&target_skills)
        {
            let err_msg = format!(
                "Dependency check failed for recruited agent '{}': {:?}",
                target_id, missing
            );
            tracing::warn!("⚠️ [Swarm] {}", err_msg);
            return Err(ToolExecutionError::Validation(err_msg));
        }

        if self.state.registry.agents.contains_key(&target_id) {
            return Ok(target_id);
        }

        tracing::info!("🛠️ [Swarm] Registering missing sub-agent: {}", target_id);
        let mut base_skills = Self::get_default_skills();

        // discovered skills logic
        if let Some(skills) = opts.extra_skills {
            for s in skills {
                if let Some(name) = s.get("name").and_then(|v| v.as_str()) {
                    if !base_skills.contains(&name.to_string()) {
                        base_skills.push(name.to_string());
                    }
                }
            }
        }

        let (initial_role, initial_dept, initial_desc) = if let Some(r) = opts.role_override {
            (
                r.to_string(),
                "Tactical Operations".to_string(),
                format!("Specialized agent with role override: {}", r),
            )
        } else {
            match target_id.to_lowercase().as_str() {
                "researcher" | "searcher" => ("Swarm Research Specialist".to_string(), "Intelligence".to_string(), "Expert in web discovery, data extraction, and information synthesis.".to_string()),
                "coder" | "developer" => ("Swarm Code Specialist".to_string(), "Engineering".to_string(), "Expert in Rust, TypeScript, and system architecture.".to_string()),
                "auditor" | "compliance" => ("Swarm Security Auditor".to_string(), "Compliance".to_string(), "Expert in vulnerability scanning, budget enforcement, and protocol verification.".to_string()),
                "alpha" => ("Swarm Mission Commander".to_string(), "Operations".to_string(), "The Alpha Node, responsible for coordinating multi-agent missions.".to_string()),
                _ => (format!("AI-{}", "General Intelligence Node"), "Swarm Core".to_string(), "Autonomous sub-agent spawned for specific task resolution.".to_string())
            }
        };

        let mut sub_agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: target_id.clone(),
                name: target_id.clone(),
                role: initial_role,
                department: initial_dept,
                description: initial_desc,
                category: "ai".to_string(),
                theme_color: Some("#4fd1c5".to_string()),
            },
            models: crate::agent::types::AgentModels {
                model_id: Some(target_config.model_id.clone()),
                model: crate::agent::types::ModelConfig {
                    provider: target_config.provider,
                    model_id: opts.parent_config.model_id.clone(),
                    api_key: opts.parent_config.api_key.clone(),
                    base_url: opts.parent_config.base_url.clone(),
                    system_prompt: None,
                    temperature: None,
                    max_tokens: None,
                    external_id: None,
                    rpm: opts.parent_config.rpm,
                    rpd: opts.parent_config.rpd,
                    tpm: opts.parent_config.tpm,
                    tpd: opts.parent_config.tpd,
                    skills: None,
                    workflows: None,
                    mcp_tools: None,
                    connector_configs: None,
                    extra_parameters: None,
                    steering_vectors: None,
                    reasoning_depth: None,
                    act_threshold: None,
                    max_turns: None,
                },
                planning_slot: None,
                execution_slot: None,
                active_model_slot: None,
            },
            economics: crate::agent::types::AgentEconomics {
                budget_usd: 10.0,
                cost_usd: 0.0,
                tokens_used: 0,
                token_usage: crate::agent::types::TokenUsage::default(),
            },
            health: crate::agent::types::AgentHealth {
                status: "idle".to_string(),
                failure_count: 0,
                last_failure_at: None,
                heartbeat_at: None,
            },
            capabilities: crate::agent::types::AgentCapabilities {
                skills: base_skills,
                workflows: vec![],
                mcp_tools: vec![],
                skill_manifest: None,
            },
            state: crate::agent::types::AgentState {
                active_mission: None,
                current_task: None,
                working_memory: serde_json::json!({}),
                current_reasoning_turn: 0,
            },
            ..Default::default()
        };

        let _ = crate::agent::persistence::save_agent_db_in_tx(&mut *conn, &mut sub_agent)
            .await
            .map_err(|e| {
                tracing::warn!(
                    "⚠️ [Swarm] Failed to persist sub-agent {} to DB: {}",
                    target_id,
                    e
                )
            });
        self.state
            .registry
            .agents
            .insert(target_id.to_string(), sub_agent);

        Ok(target_id)
    }
}
