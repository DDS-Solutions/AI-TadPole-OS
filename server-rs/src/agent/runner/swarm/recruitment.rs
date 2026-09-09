//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / recruitment
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Swarm]`
//! - **Witness Tests**: none declared

use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{swarm::SubAgentOptions, AgentRunner};
use crate::agent::types::EngineAgent;

pub(crate) struct ResolvedSubAgent {
    pub id: String,
    pub staged_agent: Option<EngineAgent>,
}

impl AgentRunner {
    /// ### 🧠 Orchestration: Tiered Recruitment (ensure_sub_agent_exists)
    /// Guarantees that a sub-agent exists in the swarm's memory registry before a
    /// mission dispatch. Implements a prioritized, scored search strategy to optimize for
    /// specialist continuity and eliminate non-deterministic substring hijacking.
    ///
    /// ### 🧬 Search Strategy: Scored Priority Tiers
    /// 1. **Exact ID Match** (Score 100): Direct targeted recruitment.
    /// 2. **Exact Name Match** (Score 80): Case-insensitive match on agent name.
    /// 3. **Exact Role Match** (Score 60): Full match on defined specialty.
    /// 4. **Word-bounded Role Match** (Score 40): Word-delimited token match ($\ge 3$ chars).
    /// + **Sector Weight** (+10 for "user"): Prioritizes user-customized specialists.
    /// + **Tier 3 (Fabrication)**: Atomically created and staged if no match meets threshold.
    pub(crate) async fn ensure_sub_agent_exists(
        &self,
        conn: &mut sqlx::SqliteConnection,
        opts: SubAgentOptions<'_>,
    ) -> Result<ResolvedSubAgent, ToolExecutionError> {
        let requested_id_lower = opts.agent_id.to_lowercase();
        let requested_token = requested_id_lower.trim();

        let mut target_id = opts.agent_id.to_string();
        let mut target_config = opts.parent_config.clone();

        // Deterministic Scored Match Search
        let mut scored_candidates: Vec<(u32, EngineAgent)> = Vec::new();

        for kv in self.state.registry.agents.iter() {
            let a = kv.value();
            let is_exact_id = a.identity.id.eq_ignore_ascii_case(requested_token);

            // Privilege check: skip executive root agents unless targeted by exact ID
            if !is_exact_id
                && (a.identity.id == crate::agent::runner::swarm::TADPOLE_CEO_ID
                    || a.identity.id == crate::agent::runner::swarm::TADPOLE_ALPHA_ID
                    || a.identity.id == crate::agent::runner::swarm::TADPOLE_COO_ID)
            {
                continue;
            }

            // Mission availability check: skip agents busy with another mission
            if let Some(ref active) = a.state.active_mission {
                if let Some(mission_id) = opts.mission_id {
                    if let Some(active_id) = active.get("id").and_then(|v| v.as_str()) {
                        if active_id != mission_id && !is_exact_id {
                            continue;
                        }
                    }
                }
            }

            let mut base_score = 0;
            if is_exact_id {
                base_score = 100;
            } else if a.identity.name.eq_ignore_ascii_case(requested_token) {
                base_score = 80;
            } else {
                let role_lower = a.identity.role.to_lowercase();
                if role_lower == requested_token {
                    base_score = 60;
                } else if requested_token.len() >= 3 {
                    // Check word-boundary match to prevent short token hijacking (e.g. "ai" matching "Maintainer")
                    let is_word_match = role_lower
                        .split(|c: char| !c.is_alphanumeric())
                        .any(|w| w == requested_token);
                    if is_word_match {
                        base_score = 40;
                    }
                }
            }

            if base_score > 0 {
                let sector_bonus = if a.identity.category == "user" { 10 } else { 0 };
                scored_candidates.push((base_score + sector_bonus, a.clone()));
            }
        }

        // Sort deterministically: highest score first, tie-break by agent ID alphabetically
        scored_candidates.sort_by(|(score_a, agent_a), (score_b, agent_b)| {
            score_b
                .cmp(score_a)
                .then_with(|| agent_a.identity.id.cmp(&agent_b.identity.id))
        });

        let registry_match = scored_candidates.into_iter().next().map(|(_, a)| a);
        let mut staged_match = None;

        if let Some(mut matched_agent) = registry_match {
            tracing::info!(
                "🎯 [Swarm] Priority Match: Found specialist '{}' ({}) in category '{}' to fulfill request for '{}'",
                matched_agent.identity.name,
                matched_agent.identity.id,
                matched_agent.identity.category,
                opts.agent_id
            );

            target_id = matched_agent.identity.id.clone();

            // Persistent Swarm Tagging and Lineage
            matched_agent.metadata.insert(
                "has_participated_in_swarm".to_string(),
                serde_json::Value::Bool(true),
            );
            if let Some(parent) = opts.parent_agent_id {
                matched_agent.metadata.insert(
                    "parent_agent_id".to_string(),
                    serde_json::Value::String(parent.to_string()),
                );
            }
            if let Some(mission) = opts.mission_id {
                matched_agent.state.active_mission = Some(serde_json::json!({
                    "id": mission,
                    "parent_agent_id": opts.parent_agent_id
                }));
            }
            matched_agent.health.status = "working".to_string();

            // Check if the matched agent's provider is properly configured
            let is_configured = match matched_agent.models.model.provider {
                crate::agent::types::ModelProvider::Ollama => true,
                ref p => {
                    let env_var = p.default_env_key();
                    matched_agent
                        .models
                        .model
                        .api_key
                        .as_ref()
                        .map(|s| !s.trim().is_empty())
                        .unwrap_or(false)
                        || std::env::var(env_var)
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                }
            };

            if !is_configured {
                tracing::warn!(
                    "⚠️ [Swarm] Resolved provider {:?} for sub-agent '{}' is unconfigured. Falling back provider coordinates to parent ({:?}).",
                    matched_agent.models.model.provider,
                    target_id,
                    opts.parent_config.provider
                );
                // Surgical fallback: swap provider/model coordinates while preserving tuned prompt/sampling settings
                matched_agent.models.model.provider = opts.parent_config.provider;
                matched_agent.models.model.model_id = opts.parent_config.model_id.clone();
                matched_agent.models.model.base_url = opts.parent_config.base_url.clone();
                matched_agent.models.model_id = Some(opts.parent_config.model_id.clone());
            }

            // Sync changes to DB mutably inside transaction
            crate::agent::persistence::save_agent_db_in_tx(&mut *conn, &mut matched_agent)
                .await
                .map_err(ToolExecutionError::from)?;

            if let Some(mid) = &matched_agent.models.model_id {
                target_config.model_id = mid.clone();
                target_config.provider = matched_agent.models.model.provider;
            }
            staged_match = Some(matched_agent);
        }

        // Verify skill and environment dependencies for the recruited agent
        let target_skills = if let Some(agent) = staged_match.as_ref() {
            agent.capabilities.skills.clone()
        } else if let Some(agent) = self.state.registry.agents.get(&target_id) {
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

        // Register discovered capabilities after dependency validation passes
        if let Some(skills) = opts.extra_skills {
            for s in skills {
                let _ = self
                    .state
                    .registry
                    .skills
                    .register_capability("skill", s.clone(), "ai")
                    .await;
            }
        }
        if let Some(workflows) = opts.extra_workflows {
            for w in workflows {
                let _ = self
                    .state
                    .registry
                    .skills
                    .register_capability("workflow", w.clone(), "ai")
                    .await;
            }
        }

        if let Some(agent) = staged_match {
            return Ok(ResolvedSubAgent {
                id: target_id,
                staged_agent: Some(agent),
            });
        }

        if let Some(existing) = self.state.registry.agents.get(&target_id) {
            return Ok(ResolvedSubAgent {
                id: target_id,
                staged_agent: Some(existing.value().clone()),
            });
        }

        tracing::info!("🛠️ [Swarm] Registering missing sub-agent: {}", target_id);
        let mut base_skills = Self::get_default_skills();

        if let Some(skills) = opts.extra_skills {
            for s in skills {
                if let Some(name) = s.get("name").and_then(|v| v.as_str()) {
                    if !base_skills.contains(&name.to_string()) {
                        base_skills.push(name.to_string());
                    }
                }
            }
        }

        let mut base_workflows = Vec::new();
        if let Some(workflows) = opts.extra_workflows {
            for w in workflows {
                if let Some(name) = w.get("name").and_then(|v| v.as_str()) {
                    if !base_workflows.contains(&name.to_string()) {
                        base_workflows.push(name.to_string());
                    }
                } else if let Some(s) = w.as_str() {
                    if !base_workflows.contains(&s.to_string()) {
                        base_workflows.push(s.to_string());
                    }
                }
            }
        }

        let (initial_role, initial_dept, initial_desc) = if let Some(r) = opts.role_override {
            // Cap authority delegation: prevent unprivileged parents from fabricating executive/management sub-agents
            let requested_auth = crate::agent::types::RoleAuthorityLevel::from_role(r);
            let parent_auth = opts
                .parent_agent_id
                .and_then(|pid| {
                    self.state.registry.agents.get(pid).map(|a| {
                        crate::agent::types::RoleAuthorityLevel::from_role(&a.identity.role)
                    })
                })
                .unwrap_or(crate::agent::types::RoleAuthorityLevel::Specialist);

            let auth_rank = |lvl: crate::agent::types::RoleAuthorityLevel| match lvl {
                crate::agent::types::RoleAuthorityLevel::Executive => 3,
                crate::agent::types::RoleAuthorityLevel::Management => 2,
                crate::agent::types::RoleAuthorityLevel::Specialist => 1,
                crate::agent::types::RoleAuthorityLevel::Observer => 0,
            };

            let sanitized_role = if auth_rank(requested_auth) > auth_rank(parent_auth) {
                tracing::warn!(
                    "⚠️ [Swarm] Blocked role escalation attempt: sub-agent requested '{:?}' while parent has '{:?}'. Downgrading to Specialist.",
                    requested_auth,
                    parent_auth
                );
                match parent_auth {
                    crate::agent::types::RoleAuthorityLevel::Management => {
                        "Tactical Coordinator".to_string()
                    }
                    _ => "Operational Specialist".to_string(),
                }
            } else {
                r.to_string()
            };

            (
                sanitized_role,
                "Tactical Operations".to_string(),
                format!("Specialized agent with role override: {}", r),
            )
        } else {
            match target_id.to_lowercase().as_str() {
                "researcher" | "searcher" => ("Swarm Research Specialist".to_string(), "Intelligence".to_string(), "Expert in web discovery, data extraction, and information synthesis.".to_string()),
                "coder" | "developer" => ("Swarm Code Specialist".to_string(), "Engineering".to_string(), "Expert in Rust, TypeScript, and system architecture.".to_string()),
                "auditor" | "compliance" => ("Swarm Security Auditor".to_string(), "Compliance".to_string(), "Expert in vulnerability scanning, budget enforcement, and protocol verification.".to_string()),
                "alpha" => ("Swarm Mission Commander".to_string(), "Operations".to_string(), "The Alpha Node, responsible for coordinating multi-agent missions.".to_string()),
                _ => ("AI-General Intelligence Node".to_string(), "Swarm Core".to_string(), "Autonomous sub-agent spawned for specific task resolution.".to_string())
            }
        };

        let mut metadata = std::collections::HashMap::new();
        if let Some(parent) = opts.parent_agent_id {
            metadata.insert(
                "parent_agent_id".to_string(),
                serde_json::Value::String(parent.to_string()),
            );
        }

        let active_mission = opts.mission_id.map(|m| {
            serde_json::json!({
                "id": m,
                "parent_agent_id": opts.parent_agent_id
            })
        });

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
                    model_id: target_config.model_id.clone(),
                    api_key: None, // Credential hygiene: do not duplicate parent plaintext keys to SQLite
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
                budget_usd: *self.state.governance.default_budget_usd.read(),
                cost_usd: 0.0,
                tokens_used: 0,
                token_usage: crate::agent::types::TokenUsage::default(),
            },
            health: crate::agent::types::AgentHealth {
                status: "working".to_string(),
                failure_count: 0,
                last_failure_at: None,
                heartbeat_at: None,
            },
            capabilities: crate::agent::types::AgentCapabilities {
                skills: base_skills,
                workflows: base_workflows,
                mcp_tools: vec![],
                skill_manifest: None,
            },
            state: crate::agent::types::AgentState {
                active_mission,
                current_task: None,
                working_memory: serde_json::json!({}),
                current_reasoning_turn: 0,
            },
            metadata,
            ..Default::default()
        };

        crate::agent::persistence::save_agent_db_in_tx(&mut *conn, &mut sub_agent)
            .await
            .map_err(ToolExecutionError::from)?;

        Ok(ResolvedSubAgent {
            id: target_id,
            staged_agent: Some(sub_agent),
        })
    }
}
