//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - Grouped execution context owns identity, mission state and environment; retain scope across callers.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

pub(crate) mod commands;
pub(crate) mod dispatcher;
pub(crate) mod governance;
pub(crate) mod recruitment;

pub(crate) const MAX_SWARM_TARGETS_PER_CALL: usize = 8;
pub(crate) const MAX_CONDUCTOR_STEPS: usize = 16;
pub(crate) const MAX_RECRUITMENT_DEPTH: usize = 12;

pub(crate) const TADPOLE_CEO_ID: &str = "1";
pub(crate) const TADPOLE_ALPHA_ID: &str = "2";
pub(crate) const TADPOLE_COO_ID: &str = "3";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SwarmBranchStatus {
    Completed,
    Failed,
    Blocked,
    Cancelled,
}

impl SwarmBranchStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SwarmBranchOutcome {
    pub branch_id: String,
    pub step_id: Option<u32>,
    pub status: SwarmBranchStatus,
    pub output: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SwarmDispatchReport {
    pub outcomes: Vec<SwarmBranchOutcome>,
}

impl SwarmDispatchReport {
    pub fn requires_human_review(&self) -> bool {
        self.outcomes
            .iter()
            .any(|outcome| outcome.status != SwarmBranchStatus::Completed)
    }

    pub fn has_completed_branch(&self) -> bool {
        self.outcomes
            .iter()
            .any(|outcome| outcome.status == SwarmBranchStatus::Completed)
    }

    pub fn render_markdown(&self) -> String {
        self.outcomes
            .iter()
            .map(|outcome| {
                let step = outcome
                    .step_id
                    .map(|id| format!("Step {id} / "))
                    .unwrap_or_default();
                // Defect #22: Escape any triple-backticks inside output so it doesn't break out of the markdown fence
                let escaped_output = outcome.output.trim().replace("```", "'''");
                format!(
                    "### {step}Branch [{}] — {}\n```text\n{}\n```",
                    outcome.branch_id,
                    outcome.status.as_str().to_uppercase(),
                    escaped_output
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }
}

pub(crate) struct SubAgentOptions<'a> {
    pub agent_id: &'a str,
    pub parent_agent_id: Option<&'a str>,
    pub mission_id: Option<&'a str>,
    pub parent_config: &'a crate::agent::types::ModelConfig,
    pub extra_skills: Option<&'a [serde_json::Value]>,
    pub extra_workflows: Option<&'a [serde_json::Value]>,
    pub role_override: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::tools::error::ToolExecutionError;
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::agent::types::{ModelConfig, ToolCall};
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_test_runner() -> (AgentRunner, Arc<AppState>) {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        (runner, state)
    }

    #[tokio::test]
    async fn test_empty_synthesis_preserves_completed_branch_evidence() {
        use crate::agent::types::{EngineAgent, ModelProvider};
        use axum::{routing::post, Json, Router};
        use std::sync::atomic::{AtomicUsize, Ordering};

        let calls = Arc::new(AtomicUsize::new(0));
        let captured = Arc::new(parking_lot::Mutex::new(serde_json::Value::Null));
        let call_counter = calls.clone();
        let request_body = captured.clone();
        let app = Router::new().route("/v1/chat/completions", post(move |Json(body): Json<serde_json::Value>| {
            let call_counter = call_counter.clone();
            let request_body = request_body.clone();
            async move {
                call_counter.fetch_add(1, Ordering::SeqCst);
                *request_body.lock() = body;
                Json(serde_json::json!({
                    "choices": [{"message": {"role": "assistant", "content": "", "tool_calls": [{
                        "id": "unexpected", "type": "function", "function": {
                            "name": "spawn_subagent", "arguments": "{}"
                        }
                    }]}, "finish_reason": "tool_calls"}],
                    "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
                }))
            }
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let (runner, state) = setup_test_runner().await;
        let mut agent = EngineAgent::default();
        agent.identity.id = "synthesis-coordinator".into();
        agent.economics.budget_usd = 1.0;
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);
        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            "synthesis-coordinator",
            "Completed evidence",
            1.0,
        )
        .await
        .unwrap();
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "synthesis-coordinator".into();
        ctx.mission_state.mission_id = mission.id;
        ctx.mission_state.budget_usd = 1.0;
        ctx.provider_name = "openai".into();
        ctx.model_config = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "gpt-4o-mini".into(),
            api_key: Some("local-test-only".into()),
            base_url: Some(format!("http://{address}/v1")),
            input_cost_per_m: Some(0.0),
            output_cost_per_m: Some(0.0),
            ..Default::default()
        };
        let report = SwarmDispatchReport {
            outcomes: vec![SwarmBranchOutcome {
                branch_id: "Alpha".into(),
                step_id: None,
                status: SwarmBranchStatus::Completed,
                output: "AUDIT_COMPLETE: verified source excerpt pub fn evidence() {}".into(),
            }],
        };
        let result = runner
            .synthesize_swarm_results(report, &ctx, &mut None)
            .await
            .unwrap();
        assert!(result.contains("Branch [Alpha]"));
        assert!(result.contains("COMPLETED"));
        assert!(result.contains("AUDIT_COMPLETE: verified source excerpt pub fn evidence() {}"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(captured.lock().get("tools").is_none());
        server.abort();
    }

    #[tokio::test]
    async fn test_proactive_recursion_block_parent() {
        let (runner, _) = setup_test_runner().await;
        let mut ctx = RunContext {
            identity: crate::agent::runner::AgentIdentity {
                agent_id: "weather_api".to_string(),
                ..Default::default()
            },
            mission_state: crate::agent::runner::MissionState {
                lineage: vec![
                    "2".to_string(),
                    "alpha".to_string(),
                    "weather_agent".to_string(),
                ],
                ..Default::default()
            },
            ..RunContext::default()
        };
        ctx.model_config = ModelConfig::default();

        let mut output = String::new();
        let mut usage = None;
        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_id": "weather_agent",
                "message": "try again"
            }),
        };

        let result = runner.handle_spawn_subagent(&ctx, &fc, &mut usage).await;

        if let Ok(res) = &result {
            output.push_str(res);
        }

        assert!(result.is_ok());
        assert!(output.contains("PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT"));
        assert!(output.contains("'weather_agent' is already in the recruitment lineage"));
    }

    #[tokio::test]
    async fn test_proactive_recursion_block_self() {
        let (runner, _) = setup_test_runner().await;
        let mut ctx = RunContext {
            identity: crate::agent::runner::AgentIdentity {
                agent_id: "weather_api".to_string(),
                ..Default::default()
            },
            mission_state: crate::agent::runner::MissionState {
                lineage: vec!["2".to_string(), "alpha".to_string()],
                ..Default::default()
            },
            ..RunContext::default()
        };
        ctx.model_config = ModelConfig::default();

        let mut output = String::new();
        let mut usage = None;
        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_id": "weather_api",
                "message": "talk to myself"
            }),
        };

        let result = runner.handle_spawn_subagent(&ctx, &fc, &mut usage).await;

        if let Ok(res) = &result {
            output.push_str(res);
        }

        assert!(output.contains("PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT"));
        assert!(output.contains("'weather_api' is already in the recruitment lineage"));
    }

    #[tokio::test]
    async fn test_alpha_directive_recursion_blocked() {
        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext {
            identity: crate::agent::runner::AgentIdentity {
                agent_id: "3".to_string(),
                ..Default::default()
            },
            mission_state: crate::agent::runner::MissionState {
                lineage: vec!["1".to_string(), "2".to_string()],
                ..Default::default()
            },
            ..RunContext::default()
        };

        let fc = ToolCall {
            name: "issue_alpha_directive".to_string(),
            args: serde_json::json!({
                "directive": "Coordinate security audit"
            }),
        };

        let mut usage = None;
        let result = runner.handle_alpha_directive(&ctx, &fc, &mut usage).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.contains("PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT"));
    }

    #[tokio::test]
    async fn test_cannot_spawn_root_ceo() {
        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext {
            identity: crate::agent::runner::AgentIdentity {
                agent_id: "analyst".to_string(),
                ..Default::default()
            },
            ..RunContext::default()
        };

        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_id": "1",
                "message": "help me with this task"
            }),
        };

        let mut usage = None;
        let result = runner.handle_spawn_subagent(&ctx, &fc, &mut usage).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Executive root agent"));
    }

    #[tokio::test]
    async fn test_build_neural_handoff_preserves_short_messages() {
        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext {
            mission_state: crate::agent::runner::MissionState {
                primary_goal: Some("Deploy v1.2 to cluster".to_string()),
                ..Default::default()
            },
            ..RunContext::default()
        };

        let handoff = runner.build_neural_handoff("deploy!", &ctx);
        assert!(handoff.contains("deploy!"));
        assert!(handoff.contains("PRIMARY MISSION GOAL"));
    }

    #[tokio::test]
    async fn test_recruitment_tier1_specialist() {
        let (runner, state) = setup_test_runner().await;

        // Setup Tier 1 Specialist (User category)
        let tier1_agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "specialist_analyst".to_string(),
                name: "Expert Analyst".to_string(),
                role: "Analyst".to_string(),
                category: "user".to_string(),
                ..Default::default()
            },
            ..crate::agent::types::EngineAgent::default()
        };
        state
            .registry
            .agents
            .insert(tier1_agent.identity.id.clone(), tier1_agent);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "analyst",
                    parent_agent_id: None,
                    mission_id: None,
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();
        if let Some(agent) = resolved.staged_agent {
            state.registry.agents.insert(resolved.id, agent);
        }

        let analyst = state.registry.agents.get("specialist_analyst").unwrap();
        assert_eq!(
            analyst.metadata.get("has_participated_in_swarm").unwrap(),
            &serde_json::Value::Bool(true)
        );
        assert_eq!(analyst.health.status, "working");
    }

    #[tokio::test]
    async fn test_recruitment_tier2_swarm_pool() {
        let (runner, state) = setup_test_runner().await;

        // Setup Tier 2 Brain (AI category)
        let tier2_agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "previous_brain".to_string(),
                name: "Experienced Coder".to_string(),
                role: "Coder".to_string(),
                category: "ai".to_string(),
                ..Default::default()
            },
            ..crate::agent::types::EngineAgent::default()
        };
        state
            .registry
            .agents
            .insert(tier2_agent.identity.id.clone(), tier2_agent);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "coder",
                    parent_agent_id: None,
                    mission_id: None,
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();
        if let Some(agent) = resolved.staged_agent {
            state.registry.agents.insert(resolved.id, agent);
        }

        let coder = state.registry.agents.get("previous_brain").unwrap();
        assert_eq!(
            coder.metadata.get("has_participated_in_swarm").unwrap(),
            &serde_json::Value::Bool(true)
        );
    }

    #[tokio::test]
    async fn test_recruitment_tier3_fabrication() {
        let (runner, state) = setup_test_runner().await;
        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "researcher",
                    parent_agent_id: None,
                    mission_id: None,
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();
        if let Some(agent) = resolved.staged_agent {
            state.registry.agents.insert(resolved.id, agent);
        }

        assert!(state.registry.agents.contains_key("researcher"));
        let researcher = state.registry.agents.get("researcher").unwrap();
        assert_eq!(researcher.identity.category, "ai");
    }

    #[tokio::test]
    async fn test_parallel_swarm_recruitment_logic() {
        let (runner, _state) = setup_test_runner().await;

        let ctx = RunContext {
            identity: crate::agent::runner::AgentIdentity {
                agent_id: "orchestrator".to_string(),
                ..Default::default()
            },
            mission_state: crate::agent::runner::MissionState {
                mission_id: "test-parallel".to_string(),
                ..Default::default()
            },
            ..RunContext::default()
        };

        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_ids": ["researcher_1", "researcher_2"],
                "message": "Verify this in parallel."
            }),
        };

        let mut output = String::new();
        let mut usage = None;

        let result = runner.handle_spawn_subagent(&ctx, &fc, &mut usage).await;
        if let Ok(res) = &result {
            output.push_str(res);
        }

        // ASSERT: The agents should have been recruited/registered in state
        assert!(
            runner.state.registry.agents.contains_key("researcher_1"),
            "researcher_1 should be in registry"
        );
        assert!(
            runner.state.registry.agents.contains_key("researcher_2"),
            "researcher_2 should be in registry"
        );
    }

    #[tokio::test]
    async fn test_exact_id_match_beats_fuzzy_role_match() {
        let (runner, state) = setup_test_runner().await;

        // User agent with role containing "coder"
        let user_agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "user_manager".to_string(),
                name: "Team Lead".to_string(),
                role: "Lead Coder Coordinator".to_string(),
                category: "user".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        state
            .registry
            .agents
            .insert(user_agent.identity.id.clone(), user_agent);

        // AI agent with exact ID "coder"
        let ai_coder = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "coder".to_string(),
                name: "Autonomous Coder".to_string(),
                role: "Software Engineer".to_string(),
                category: "ai".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        state
            .registry
            .agents
            .insert(ai_coder.identity.id.clone(), ai_coder);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "coder",
                    parent_agent_id: None,
                    mission_id: None,
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Exact ID "coder" (score 100) must beat user substring "Lead Coder Coordinator" (score 40+10=50)
        assert_eq!(resolved.id, "coder");
    }

    #[tokio::test]
    async fn test_short_token_does_not_hijack_role_substring() {
        let (runner, state) = setup_test_runner().await;

        // Agent whose role contains "ai" within a larger word ("Maintainer")
        let maintainer = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "repo_maintainer".to_string(),
                name: "Repo Maintainer".to_string(),
                role: "Maintainer".to_string(),
                category: "user".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        state
            .registry
            .agents
            .insert(maintainer.identity.id.clone(), maintainer);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "ai",
                    parent_agent_id: None,
                    mission_id: None,
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Short token "ai" (< 3 chars and not a word match for "Maintainer") must NOT hijack repo_maintainer
        assert_ne!(resolved.id, "repo_maintainer");
    }

    #[tokio::test]
    async fn test_fabricated_agent_credential_hygiene_and_workflows() {
        let (runner, state) = setup_test_runner().await;

        let mut parent_config = ModelConfig::default();
        parent_config.api_key = Some("sk-secret-key-123".to_string());

        let extra_workflows = vec![serde_json::json!({
            "name": "deep_code_review"
        })];

        let mut tx = state.resources.pool.begin().await.unwrap();
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "sec_auditor",
                    parent_agent_id: Some("1"),
                    mission_id: Some("mission-42"),
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: Some(&extra_workflows),
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let staged = resolved.staged_agent.expect("staged agent must exist");
        // Credential Hygiene: fabricated agent must have None for api_key in SQLite persistence
        assert_eq!(staged.models.model.api_key, None);
        // Workflows must be attached symmetrically
        assert!(staged
            .capabilities
            .workflows
            .contains(&"deep_code_review".to_string()));
        // Lineage must be present
        assert_eq!(
            staged.metadata.get("parent_agent_id"),
            Some(&serde_json::Value::String("1".to_string()))
        );
    }

    #[tokio::test]
    async fn test_recruitment_role_override_authority_capping() {
        let (runner, state) = setup_test_runner().await;

        // Setup a Specialist agent as parent
        let specialist_agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "specialist_parent".to_string(),
                name: "Specialist Parent".to_string(),
                role: "Engineer".to_string(),
                category: "ai".to_string(),
                ..Default::default()
            },
            ..crate::agent::types::EngineAgent::default()
        };
        state
            .registry
            .agents
            .insert(specialist_agent.identity.id.clone(), specialist_agent);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();

        // Attempt privilege escalation: specialist attempts to spawn a subagent with role "CEO"
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "rogue_subagent",
                    parent_agent_id: Some("specialist_parent"),
                    mission_id: Some("mission-escalation"),
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: Some("CEO"),
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();
        let staged = resolved.staged_agent.expect("staged agent must exist");
        // Authority escalation must be blocked: role is downgraded to "Operational Specialist"
        assert_eq!(staged.identity.role, "Operational Specialist");
        assert_eq!(
            crate::agent::types::RoleAuthorityLevel::from_role(&staged.identity.role),
            crate::agent::types::RoleAuthorityLevel::Specialist
        );
    }

    #[tokio::test]
    async fn test_recruitment_via_capability_map_and_skills() {
        let (runner, state) = setup_test_runner().await;

        let rust_specialist = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "rust-guru-01".to_string(),
                name: "Rust Systems Expert".to_string(),
                role: "Systems Architect".to_string(),
                category: "user".to_string(),
                ..Default::default()
            },
            capabilities: crate::agent::types::AgentCapabilities {
                skills: vec!["rust-pro".to_string(), "memory-safety".to_string()],
                workflows: vec!["audit-subsystem".to_string()],
                mcp_tools: vec![],
                skill_manifest: None,
            },
            ..crate::agent::types::EngineAgent::default()
        };

        state
            .registry
            .capability_map
            .write()
            .update_agent(&rust_specialist);
        state
            .registry
            .agents
            .insert(rust_specialist.identity.id.clone(), rust_specialist);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();

        // 1. Recruit by exact skill match: "rust-pro"
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "rust-pro",
                    parent_agent_id: None,
                    mission_id: None,
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(resolved.id, "rust-guru-01");
        assert!(resolved.staged_agent.is_some());
    }

    #[tokio::test]
    async fn test_recruitment_ambient_skill_dependency_decoupling() {
        let (runner, state) = setup_test_runner().await;

        // Ensure DISCORD_WEBHOOK_URL is not set so notify_discord fails dependency check
        std::env::remove_var("DISCORD_WEBHOOK_URL");

        let agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "decoupled_analyst".to_string(),
                name: "Decoupled Analyst".to_string(),
                role: "Analyst".to_string(),
                category: "user".to_string(),
                ..Default::default()
            },
            capabilities: crate::agent::types::AgentCapabilities {
                skills: vec!["clean-code".to_string(), "notify_discord".to_string()],
                workflows: vec![],
                mcp_tools: vec![],
                skill_manifest: None,
            },
            ..crate::agent::types::EngineAgent::default()
        };
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();

        // 1. Recruited with ambient unfulfilled skill -> succeeds with unfulfilled skill decoupled
        let resolved = runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "decoupled_analyst",
                    parent_agent_id: None,
                    mission_id: None,
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let staged = resolved.staged_agent.expect("staged agent must exist");
        assert!(staged
            .capabilities
            .skills
            .contains(&"clean-code".to_string()));
        assert!(!staged
            .capabilities
            .skills
            .contains(&"notify_discord".to_string()));
        state
            .registry
            .agents
            .insert(staged.identity.id.clone(), staged);

        // 2. Explicitly requested unfulfilled skill -> returns ToolExecutionError::Validation
        let mut tx2 = state.resources.pool.begin().await.unwrap();
        let extra = vec![serde_json::json!({ "name": "notify_discord" })];
        let err = runner
            .ensure_sub_agent_exists(
                &mut tx2,
                SubAgentOptions {
                    agent_id: "decoupled_analyst",
                    parent_agent_id: None,
                    mission_id: None,
                    parent_config: &parent_config,
                    extra_skills: Some(&extra),
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap_err();

        match err {
            ToolExecutionError::Validation(msg) => {
                assert!(msg.contains("notify_discord"));
            }
            other => panic!("Expected Validation error, got: {:?}", other),
        }
    }
}
