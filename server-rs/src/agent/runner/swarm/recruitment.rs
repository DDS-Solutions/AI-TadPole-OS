//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / recruitment
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `ToolExecutionError::Validation`, `ToolExecutionError::ExecutionFailed`
//! - **Telemetry Targets**: `[Swarm]`

use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{swarm::SubAgentOptions, AgentRunner};
use crate::agent::types::EngineAgent;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedSubAgent {
    pub id: String,
    pub staged_agent: Option<EngineAgent>,
}

/// Normalizes and validates raw agent recruitment tokens / identifiers.
pub(crate) fn sanitize_agent_id(raw: &str) -> Result<String, ToolExecutionError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(ToolExecutionError::Validation(format!(
            "Invalid agent ID or recruitment query '{}': length must be between 1 and 64 characters.",
            raw
        )));
    }
    let sanitized: String = trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('-').to_string();
    if sanitized.is_empty() {
        return Err(ToolExecutionError::Validation(format!(
            "Invalid agent ID or recruitment query '{}': must contain alphanumeric characters.",
            raw
        )));
    }
    Ok(sanitized)
}

/// Evaluates candidate alignment against recruitment query tokens and mission availability.
pub(crate) fn score_candidate(
    a: &EngineAgent,
    requested_token: &str,
    query_tokens: &[&str],
    mission_id: Option<&str>,
) -> Result<Option<u32>, ToolExecutionError> {
    let is_exact_id = a.identity.id.eq_ignore_ascii_case(requested_token);

    // Privilege check: Executive root agent ("1") cannot be recruited as a sub-agent worker
    if a.identity.id == crate::agent::runner::swarm::TADPOLE_CEO_ID {
        if is_exact_id {
            return Err(ToolExecutionError::Validation(
                "Executive root agent ('1') cannot be recruited as a sub-agent worker".to_string(),
            ));
        }
        return Ok(None);
    }

    // Skip Alpha and COO unless targeted by exact ID
    if !is_exact_id
        && (a.identity.id == crate::agent::runner::swarm::TADPOLE_ALPHA_ID
            || a.identity.id == crate::agent::runner::swarm::TADPOLE_COO_ID)
    {
        return Ok(None);
    }

    // Mission availability check: guard against clobbering in-flight missions
    if let Some(ref active) = a.state.active_mission {
        if let Some(active_id) = active.get("id").and_then(|v| v.as_str()) {
            if Some(active_id) != mission_id {
                if is_exact_id {
                    return Err(ToolExecutionError::Validation(format!(
                        "Agent '{}' is currently engaged in mission '{}' and cannot be recruited for mission '{}'.",
                        a.identity.id,
                        active_id,
                        mission_id.unwrap_or("unknown")
                    )));
                } else {
                    tracing::debug!(
                        "⏩ [Swarm] Candidate '{}' skipped: busy with mission '{}'",
                        a.identity.id,
                        active_id
                    );
                    return Ok(None);
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
        } else if a
            .capabilities
            .skills
            .iter()
            .any(|s| s.eq_ignore_ascii_case(requested_token))
            || a.capabilities
                .workflows
                .iter()
                .any(|w| w.eq_ignore_ascii_case(requested_token))
        {
            base_score = 50;
        } else if requested_token.len() >= 3 {
            let role_words: Vec<&str> = role_lower
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .collect();
            // Single word boundary match
            if role_words.contains(&requested_token) {
                base_score = 40;
            } else if query_tokens.len() > 1
                && query_tokens
                    .iter()
                    .all(|qt| role_words.iter().any(|rw| rw == qt))
            {
                // Multi-word token agreement (e.g. "code reviewer" matching "Senior Code Reviewer")
                base_score = 45;
            }
        }
    }

    if base_score > 0 {
        let category_bonus = if a.identity.category == "user" { 10 } else { 0 };
        Ok(Some(base_score + category_bonus))
    } else {
        Ok(None)
    }
}

impl AgentRunner {
    /// ### 🧠 Orchestration: Tiered Recruitment (ensure_sub_agent_exists)
    /// Guarantees that a sub-agent exists and is staged with active mission context
    /// before mission dispatch. Implements an inverted capability index search with
    /// prioritized scoring to optimize for specialist continuity and eliminate
    /// non-deterministic substring hijacking.
    ///
    /// ### 🧬 Search Strategy: Scored Priority Tiers
    /// 1. **Exact ID Match** (Score 100): Direct targeted recruitment.
    /// 2. **Exact Name Match** (Score 80): Case-insensitive match on agent name.
    /// 3. **Exact Role Match** (Score 60): Full match on defined specialty.
    /// 4. **Exact Skill / Workflow Match** (Score 50): Match on declared capabilities.
    /// 5. **Word-bounded Role Match** (Score 40-45): Word-delimited token match (>= 3 chars) or multi-word agreement.
    /// + **Category Weight** (+10 for "user"): Prioritizes user-customized specialists.
    /// + **Fabrication**: Atomically created, dependency-validated, and staged if no match meets threshold.
    pub(crate) async fn ensure_sub_agent_exists(
        &self,
        conn: &mut sqlx::SqliteConnection,
        opts: SubAgentOptions<'_>,
    ) -> Result<ResolvedSubAgent, ToolExecutionError> {
        let requested_id_lower = opts.agent_id.to_lowercase();
        let requested_token = requested_id_lower.trim();
        let query_tokens: Vec<&str> = requested_token
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();

        // 1. O(1) Candidate Retrieval via CapabilityMap and Direct ID Lookup
        let mut candidate_ids: std::collections::HashSet<String> = self
            .state
            .registry
            .capability_map
            .read()
            .find_candidates(requested_token);

        if let Some(agent_ref) = self.state.registry.agents.get(requested_token) {
            candidate_ids.insert(agent_ref.identity.id.clone());
        }
        if let Some(agent_ref) = self.state.registry.agents.get(opts.agent_id.trim()) {
            candidate_ids.insert(agent_ref.identity.id.clone());
        }
        for qt in &query_tokens {
            if qt.len() >= 3 {
                let sub_c = self
                    .state
                    .registry
                    .capability_map
                    .read()
                    .find_candidates(qt);
                candidate_ids.extend(sub_c);
            }
        }

        // 2. Score candidates by reference
        let mut scored_candidates: Vec<(u32, EngineAgent)> = Vec::new();

        if !candidate_ids.is_empty() {
            for id in candidate_ids {
                if let Some(agent_entry) = self.state.registry.agents.get(&id) {
                    if let Some(score) = score_candidate(
                        agent_entry.value(),
                        requested_token,
                        &query_tokens,
                        opts.mission_id,
                    )? {
                        scored_candidates.push((score, agent_entry.value().clone()));
                    }
                }
            }
        } else {
            // Zero-clone fallback by reference: inspect agents in registry without bulk cloning
            for kv in self.state.registry.agents.iter() {
                if let Some(score) =
                    score_candidate(kv.value(), requested_token, &query_tokens, opts.mission_id)?
                {
                    scored_candidates.push((score, kv.value().clone()));
                }
            }
        }

        // Sort deterministically: highest score first, tie-break by user category, then agent ID
        scored_candidates.sort_by(|(score_a, agent_a), (score_b, agent_b)| {
            score_b
                .cmp(score_a)
                .then_with(|| {
                    (agent_b.identity.category == "user")
                        .cmp(&(agent_a.identity.category == "user"))
                })
                .then_with(|| agent_a.identity.id.cmp(&agent_b.identity.id))
        });

        // 3. Resolve target agent: matched from registry or fabricated from template
        let mut target_agent = if let Some((_, matched)) = scored_candidates.into_iter().next() {
            tracing::info!(
                "🎯 [Swarm] Priority Match: Found specialist '{}' ({}) in category '{}' to fulfill request for '{}'",
                matched.identity.name,
                matched.identity.id,
                matched.identity.category,
                opts.agent_id
            );
            matched
        } else {
            let sanitized_id = sanitize_agent_id(opts.agent_id)?;

            const RESERVED_FABRICATION_IDS: &[&str] = &[
                crate::agent::runner::swarm::TADPOLE_CEO_ID,
                crate::agent::runner::swarm::TADPOLE_ALPHA_ID,
                crate::agent::runner::swarm::TADPOLE_COO_ID,
                "ceo",
                "alpha",
                "coo",
                "overlord",
            ];

            if RESERVED_FABRICATION_IDS
                .iter()
                .any(|r| r.eq_ignore_ascii_case(&sanitized_id))
            {
                return Err(ToolExecutionError::Validation(format!(
                    "Identity '{}' is reserved for sovereign executive governance and cannot be dynamically fabricated.",
                    sanitized_id
                )));
            }

            tracing::info!(
                "🛠️ [Swarm] Fabricating missing sub-agent specialist: {}",
                sanitized_id
            );
            self.fabricate_sub_agent(&sanitized_id, &opts)?
        };

        // 4. Validate and merge capabilities (Validate-Before-Mutate)
        // 4a. Validate ambient skills
        let mut valid_skills = Vec::new();
        let mut unfulfilled = Vec::new();
        for skill in &target_agent.capabilities.skills {
            if let Err(errs) = crate::security::dependency_guard::check_skill_dependencies(
                std::slice::from_ref(skill),
            ) {
                unfulfilled.push((skill.clone(), errs));
            } else if !valid_skills.contains(skill) {
                valid_skills.push(skill.clone());
            }
        }

        // 4b. Validate and merge explicitly requested extra skills
        if let Some(extra) = opts.extra_skills {
            for req in extra {
                if let Some(name) = req.get("name").and_then(|v| v.as_str()) {
                    if let Err(errs) =
                        crate::security::dependency_guard::check_skill_dependencies(&[
                            name.to_string()
                        ])
                    {
                        let err_msg = format!(
                            "Explicitly requested skill '{}' failed dependency check for recruited agent '{}': {:?}",
                            name, target_agent.identity.id, errs
                        );
                        tracing::warn!("⚠️ [Swarm] {}", err_msg);
                        return Err(ToolExecutionError::Validation(err_msg));
                    }
                    if !valid_skills.contains(&name.to_string()) {
                        valid_skills.push(name.to_string());
                    }
                }
            }
        }

        if !unfulfilled.is_empty() {
            tracing::warn!(
                "⚠️ [Swarm] Recruited agent '{}' has unfulfilled optional ambient skills: {:?}. Decoupling unfulfilled skills from runtime capabilities.",
                target_agent.identity.id,
                unfulfilled
            );
        }

        // 4c. Merge explicitly requested extra workflows
        let mut valid_workflows = target_agent.capabilities.workflows.clone();
        if let Some(extra_wf) = opts.extra_workflows {
            for req in extra_wf {
                if let Some(name) = req.get("name").and_then(|v| v.as_str()) {
                    if !valid_workflows.contains(&name.to_string()) {
                        valid_workflows.push(name.to_string());
                    }
                } else if let Some(s) = req.as_str() {
                    if !valid_workflows.contains(&s.to_string()) {
                        valid_workflows.push(s.to_string());
                    }
                }
            }
        }

        // Defect #14: Decouple declared persistent capabilities from transient ambient environment exclusions.
        // If an ambient skill has missing env vars (e.g. DISCORD_WEBHOOK_URL), retain its declaration in SQLite
        // rather than permanently erasing it from the agent record.
        let mut persisted_skills = target_agent.capabilities.skills.clone();
        for s in &valid_skills {
            if !persisted_skills.contains(s) {
                persisted_skills.push(s.clone());
            }
        }
        target_agent.capabilities.skills = persisted_skills;
        target_agent.capabilities.workflows = valid_workflows;

        if !unfulfilled.is_empty() {
            let unfulfilled_names: Vec<String> =
                unfulfilled.iter().map(|(s, _)| s.clone()).collect();
            target_agent.metadata.insert(
                "runtime_unfulfilled_skills".to_string(),
                serde_json::json!(unfulfilled_names),
            );
        }

        // 5. Provider configuration check & surgical fallback
        let is_configured = match target_agent.models.model.provider {
            crate::agent::types::ModelProvider::Ollama => true,
            ref p => {
                let env_var = p.default_env_key();
                target_agent
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
                target_agent.models.model.provider,
                target_agent.identity.id,
                opts.parent_config.provider
            );
            target_agent.models.model.provider = opts.parent_config.provider;
            target_agent.models.model.model_id = opts.parent_config.model_id.clone();
            target_agent.models.model.base_url = opts.parent_config.base_url.clone();
            target_agent.models.model_id = Some(opts.parent_config.model_id.clone());
        }

        // 6. Apply persistent mission lineage and health status
        target_agent.metadata.insert(
            "has_participated_in_swarm".to_string(),
            serde_json::Value::Bool(true),
        );
        if let Some(parent) = opts.parent_agent_id {
            target_agent.metadata.insert(
                "parent_agent_id".to_string(),
                serde_json::Value::String(parent.to_string()),
            );
        }
        if let Some(mission) = opts.mission_id {
            target_agent.state.active_mission = Some(serde_json::json!({
                "id": mission,
                "parent_agent_id": opts.parent_agent_id
            }));
        }
        target_agent.health.status = "working".to_string();

        // 7. Single atomic persistence to SQLite inside transaction
        let next_ver =
            crate::agent::persistence::save_agent_db_in_tx(&mut *conn, &mut target_agent)
                .await
                .map_err(ToolExecutionError::from)?;
        target_agent.version = next_ver;

        // Active runtime skills filtering: the returned staged agent must only have fulfilled skills for execution
        target_agent.capabilities.skills = valid_skills;

        // 8. Register discovered capabilities
        if let Some(skills) = opts.extra_skills {
            for s in skills {
                if let Err(e) = self
                    .state
                    .registry
                    .skills
                    .register_capability("skill", s.clone(), "ai")
                    .await
                {
                    tracing::warn!("⚠️ [Swarm] Failed to register capability skill: {:?}", e);
                }
            }
        }
        if let Some(workflows) = opts.extra_workflows {
            for w in workflows {
                if let Err(e) = self
                    .state
                    .registry
                    .skills
                    .register_capability("workflow", w.clone(), "ai")
                    .await
                {
                    tracing::warn!("⚠️ [Swarm] Failed to register capability workflow: {:?}", e);
                }
            }
        }

        Ok(ResolvedSubAgent {
            id: target_agent.identity.id.clone(),
            staged_agent: Some(target_agent),
        })
    }

    /// Fabricates an initialized `EngineAgent` template for a new specialist node.
    fn fabricate_sub_agent(
        &self,
        target_id: &str,
        opts: &SubAgentOptions<'_>,
    ) -> Result<EngineAgent, ToolExecutionError> {
        let (initial_role, initial_dept, initial_desc) = if let Some(r) = opts.role_override {
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
                "researcher" | "searcher" => (
                    "Swarm Research Specialist".to_string(),
                    "Intelligence".to_string(),
                    "Expert in web discovery, data extraction, and information synthesis.".to_string(),
                ),
                "coder" | "developer" => (
                    "Swarm Code Specialist".to_string(),
                    "Engineering".to_string(),
                    "Expert in Rust, TypeScript, and system architecture.".to_string(),
                ),
                "auditor" | "compliance" => (
                    "Swarm Security Auditor".to_string(),
                    "Compliance".to_string(),
                    "Expert in vulnerability scanning, budget enforcement, and protocol verification.".to_string(),
                ),
                _ => (
                    "AI-General Intelligence Node".to_string(),
                    "Swarm Core".to_string(),
                    "Autonomous sub-agent spawned for specific task resolution.".to_string(),
                ),
            }
        };

        Ok(crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: target_id.to_string(),
                name: target_id.to_string(),
                role: initial_role,
                department: initial_dept,
                description: initial_desc,
                category: "ai".to_string(),
                theme_color: Some("#4fd1c5".to_string()),
            },
            models: crate::agent::types::AgentModels {
                model_id: Some(opts.parent_config.model_id.clone()),
                model: crate::agent::types::ModelConfig {
                    provider: opts.parent_config.provider,
                    model_id: opts.parent_config.model_id.clone(),
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
                    custom_headers: None,
                    context_window: opts.parent_config.context_window,
                    input_cost_per_m: opts.parent_config.input_cost_per_m,
                    output_cost_per_m: opts.parent_config.output_cost_per_m,
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
                skills: Self::get_default_skills(),
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
            metadata: std::collections::HashMap::new(),
            ..Default::default()
        })
    }
}
