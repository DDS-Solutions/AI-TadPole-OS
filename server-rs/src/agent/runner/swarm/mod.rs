//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Swarm]`
//! - **Witness Tests**: none declared

pub(crate) mod dispatcher;
pub(crate) mod governance;
pub(crate) mod recruitment;

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::types::{ModelConfig, TokenUsage, ToolCall};
use crate::error::AppError;

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
                format!(
                    "### {step}Branch [{}] — {}\n```text\n{}\n```",
                    outcome.branch_id,
                    outcome.status.as_str().to_uppercase(),
                    outcome.output.trim()
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
    pub parent_config: &'a ModelConfig,
    pub extra_skills: Option<&'a [serde_json::Value]>,
    pub extra_workflows: Option<&'a [serde_json::Value]>,
    pub role_override: Option<&'a str>,
}

impl AgentRunner {
    /// Safe default capability grant for newly fabricated swarm agents.
    /// Excludes destructive file mutations (delete_file) and mission termination (complete_mission).
    pub(crate) fn get_default_skills() -> Vec<String> {
        vec![
            "fetch_url".to_string(),
            "read_file".to_string(),
            "write_file".to_string(),
            "list_files".to_string(),
            "get_file_contents".to_string(),
            "grep_search".to_string(),
            "get_agent_metrics".to_string(),
        ]
    }

    /// Extracts sub-agent IDs from the tool call arguments with strict validation.
    fn resolve_target_agents(&self, fc: &ToolCall) -> Result<Vec<String>, ToolExecutionError> {
        let mut target_ids = Vec::new();
        if let Some(ids) = fc.args.get("agent_ids").and_then(|v| v.as_array()) {
            for id in ids {
                if let Some(s) = id.as_str() {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        target_ids.push(trimmed.to_string());
                    }
                }
            }
        } else if let Some(single_id) = fc.args.get("agent_id").and_then(|v| v.as_str()) {
            let trimmed = single_id.trim();
            if !trimmed.is_empty() {
                target_ids.push(trimmed.to_string());
            }
        }

        let mut seen = std::collections::HashSet::new();
        target_ids.retain(|id| seen.insert(id.clone()));
        if target_ids.is_empty() {
            return Err(ToolExecutionError::Validation(
                "spawn_subagent requires at least one valid, non-empty agent ID".to_string(),
            ));
        }

        // Privilege tier check: root executive CEO ("1") cannot be spawned as a sub-agent worker
        if target_ids.iter().any(|id| id == TADPOLE_CEO_ID) {
            return Err(ToolExecutionError::Validation(
                "Executive root agent ('1') cannot be recruited as a sub-agent worker".to_string(),
            ));
        }

        if target_ids.len() > MAX_SWARM_TARGETS_PER_CALL {
            return Err(ToolExecutionError::Validation(format!(
                "spawn_subagent accepts at most {MAX_SWARM_TARGETS_PER_CALL} unique targets per call"
            )));
        }
        if let Some(invalid) = target_ids
            .iter()
            .find(|id| id.is_empty() || id.len() > 128 || id.chars().any(char::is_control))
        {
            return Err(ToolExecutionError::Validation(format!(
                "Invalid sub-agent identifier: {:?}",
                invalid
            )));
        }
        Ok(target_ids)
    }

    /// Centralized mission goal framing helper across all swarm dispatch paths.
    pub(crate) fn mission_goal_block(ctx: &RunContext) -> String {
        format!(
            "### PRIMARY MISSION GOAL:\n{}",
            ctx.primary_goal
                .as_deref()
                .unwrap_or("See mission scope for details.")
        )
    }

    /// Centralized prompt building for neural handoff strategic context.
    pub(crate) fn build_neural_handoff(&self, sub_message: &str, ctx: &RunContext) -> String {
        let primary_mission = Self::mission_goal_block(ctx);
        if sub_message.trim().is_empty() {
            format!(
                "{}\n\n(Please assist with the mission goal listed above.)",
                primary_mission
            )
        } else {
            format!(
                "{}\n\n--- STRATEGIC CONTEXT ---\n{}\n--- END CONTEXT ---",
                sub_message.trim(),
                primary_mission
            )
        }
    }

    /// Synthesizes pooled sub-agent results. Tool-less to prevent recursive spawns during synthesis.
    async fn synthesize_swarm_results(
        &self,
        report: SwarmDispatchReport,
        ctx: &RunContext,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        if report.outcomes.is_empty() {
            return Err(ToolExecutionError::ExecutionFailed(
                "Swarm dispatch produced no branch outcomes".to_string(),
            ));
        }

        let requires_human_review = report.requires_human_review();
        if requires_human_review {
            ctx.swarm_partial_failure
                .store(true, std::sync::atomic::Ordering::Release);
            tracing::warn!(
                "⚠️ [Swarm] Mission {} has partial branch failures; preserving results for human review.",
                ctx.mission_id
            );
        }

        let pooled_results = report.render_markdown();
        let review_instruction = if requires_human_review {
            "Some branches failed, were blocked, or were cancelled. Preserve those statuses explicitly, do not claim full completion, and recommend a concrete human retry/resume decision."
        } else {
            "All branches completed. Synthesize their findings without dropping attribution."
        };
        let synthesis_prompt = format!(
            "Your swarm reported back with these structured branch outcomes:\n\n{}\n\n{}",
            pooled_results, review_instruction
        );

        if !report.has_completed_branch() {
            return Ok(format!(
                "SWARM_REVIEW_REQUIRED: No branch completed successfully.\n\n{}",
                pooled_results
            ));
        }

        // Tool-less synthesis to eliminate recursive spawn loopholes
        let (final_text, _, final_usage) = self
            .call_provider_for_synthesis(ctx, &synthesis_prompt, None)
            .await?;

        self.accumulate_usage(usage, final_usage);
        if let Some(ref vt) = ctx.visible_transcript {
            vt.lock()
                .push(format!("OBSERVATION (Spawn Sub-agents): {}", final_text));
        }
        if requires_human_review {
            Ok(format!(
                "SWARM_PARTIAL_FAILURE — HUMAN REVIEW REQUIRED\n\n{}",
                final_text
            ))
        } else {
            Ok(final_text)
        }
    }

    pub(crate) async fn handle_spawn_subagent(
        &self,
        ctx: &RunContext,
        fc: &ToolCall,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let target_ids = if let Some(plan) = &ctx.conductor_plan {
            if plan.steps.is_empty() || plan.steps.len() > MAX_CONDUCTOR_STEPS {
                return Err(ToolExecutionError::Validation(format!(
                    "Conductor plans require 1..={MAX_CONDUCTOR_STEPS} steps"
                )));
            }
            let mut seen = std::collections::HashSet::new();
            let targets: Vec<String> = plan
                .steps
                .iter()
                .map(|step| step.target_agent.trim().to_string())
                .filter(|target| seen.insert(target.clone()))
                .collect();
            if let Some(invalid) = targets
                .iter()
                .find(|id| id.is_empty() || id.len() > 128 || id.chars().any(char::is_control))
            {
                return Err(ToolExecutionError::Validation(format!(
                    "Invalid Conductor target agent: {:?}",
                    invalid
                )));
            }
            targets
        } else {
            self.resolve_target_agents(fc)?
        };

        let max_swarm_depth = self
            .state
            .governance
            .max_swarm_depth
            .load(std::sync::atomic::Ordering::Relaxed);
        if ctx.depth >= max_swarm_depth || ctx.lineage.len() >= MAX_RECRUITMENT_DEPTH {
            tracing::warn!("🐝 [Swarm] Swarm depth limit exceeded (current: {}, max: {}). Blocking recruitment.", ctx.depth, max_swarm_depth);
            return Ok(format!("PROTOCOL_VIOLATION: Swarm depth limit exceeded (current depth: {}). You cannot spawn more sub-agents.", ctx.depth));
        }

        tracing::info!(
            "🐝 [Swarm] Agent {} spawning {} sub-agent(s): {:?}...",
            ctx.agent_id,
            target_ids.len(),
            target_ids
        );

        // Human/COO governance may wait on remote systems, so it must complete
        // before opening the short-lived persistence transaction.
        self.validate_and_reactivate_agents(&target_ids, ctx)
            .await?;

        let mut tx = self
            .state
            .resources
            .pool
            .begin()
            .await
            .map_err(|e| ToolExecutionError::ExecutionFailed(e.to_string()))?;

        // Extract extra capabilities and role override if provided
        let extra_skills = fc.args.get("skills").and_then(|v| v.as_array());
        let extra_workflows = fc.args.get("workflows").and_then(|v| v.as_array());
        let role_override = fc.args.get("role").and_then(|v| v.as_str());

        // Ensure sub-agents exist in DB inside the same transaction
        let mut resolved_ids = Vec::new();
        let mut staged_agents = Vec::new();
        for sub_agent_id in &target_ids {
            // Proactive circular recruitment guard
            if ctx.lineage.contains(sub_agent_id) || ctx.agent_id == *sub_agent_id {
                tracing::warn!(
                    "[Swarm] Circular recruitment blocked for '{}' vs lineage {:?}",
                    sub_agent_id,
                    ctx.lineage
                );
                return Ok(format!(
                    "PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT - Agent '{}' is already in the recruitment lineage.",
                    sub_agent_id
                ));
            }

            let resolved = self
                .ensure_sub_agent_exists(
                    &mut tx,
                    SubAgentOptions {
                        agent_id: sub_agent_id,
                        parent_agent_id: Some(&ctx.agent_id),
                        mission_id: Some(&ctx.mission_id),
                        parent_config: &ctx.model_config,
                        extra_skills: extra_skills.map(|v| v.as_slice()),
                        extra_workflows: extra_workflows.map(|v| v.as_slice()),
                        role_override,
                    },
                )
                .await?;
            resolved_ids.push(resolved.id);
            if let Some(agent) = resolved.staged_agent {
                staged_agents.push(agent);
            }
        }

        // Commit transaction atomically
        tx.commit()
            .await
            .map_err(|e| ToolExecutionError::ExecutionFailed(e.to_string()))?;

        // Publish committed agents to live registry
        for agent in staged_agents {
            self.state
                .registry
                .agents
                .insert(agent.identity.id.clone(), agent);
        }

        // Also update existing registered targets so their pulse edge connects to parent
        for target_id in &resolved_ids {
            if let Some(mut entry) = self.state.registry.agents.get_mut(target_id) {
                entry.value_mut().metadata.insert(
                    "parent_agent_id".to_string(),
                    serde_json::Value::String(ctx.agent_id.clone()),
                );
                entry.value_mut().state.active_mission = Some(serde_json::json!({
                    "id": ctx.mission_id,
                    "parent_agent_id": ctx.agent_id
                }));
                entry.value_mut().health.status = "working".to_string();
            }
        }

        self.state.governance.recruit_count.fetch_add(
            resolved_ids.len() as u32,
            std::sync::atomic::Ordering::Relaxed,
        );

        let sub_message = fc
            .args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Dispatch execution
        let results = self
            .execute_swarm_dispatch(resolved_ids, sub_message, ctx)
            .await?;

        // Synthesize results
        self.synthesize_swarm_results(results, ctx, usage).await
    }

    /// Handles `handle_alpha_directive`: delegates to Tadpole Alpha (ID: 2) with full guard enforcement.
    pub(crate) async fn handle_alpha_directive(
        &self,
        ctx: &RunContext,
        fc: &ToolCall,
        _usage: &mut Option<TokenUsage>,
    ) -> Result<String, AppError> {
        let directive = fc
            .args
            .get("directive")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "🧬 [Sovereignty] Agent {} issuing directive to Tadpole Alpha...",
            ctx.agent_id
        );
        self.broadcast_agent(ctx, "🧬 Issuing directive to Tadpole Alpha...", "info");

        // Lineage cycle guard
        if ctx.lineage.contains(&TADPOLE_ALPHA_ID.to_string()) || ctx.agent_id == TADPOLE_ALPHA_ID {
            tracing::warn!(
                "[Swarm] Circular delegation blocked: Tadpole Alpha is already in lineage {:?}",
                ctx.lineage
            );
            return Ok(format!(
                "PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT - Tadpole Alpha ('{}') is already in the recruitment lineage.",
                TADPOLE_ALPHA_ID
            ));
        }

        // Swarm depth guard
        let max_swarm_depth = self
            .state
            .governance
            .max_swarm_depth
            .load(std::sync::atomic::Ordering::Relaxed);
        if ctx.depth >= max_swarm_depth || ctx.lineage.len() >= MAX_RECRUITMENT_DEPTH {
            tracing::warn!(
                "[Swarm] Depth limit exceeded during alpha directive (depth: {}, max: {})",
                ctx.depth,
                max_swarm_depth
            );
            return Ok(format!(
                "PROTOCOL_VIOLATION: Swarm depth limit exceeded (current depth: {}). Cannot delegate to Alpha.",
                ctx.depth
            ));
        }

        // Mission cancellation check
        if crate::agent::mission::is_mission_completed(&self.state.resources.pool, &ctx.mission_id)
            .await
            .unwrap_or(false)
        {
            return Ok(
                "Mission was closed by the operator before directive reached Alpha.".to_string(),
            );
        }

        // Neural handoff framing
        let primary_mission = Self::mission_goal_block(ctx);
        let final_directive = format!("{}\n\n{}", directive, primary_mission);

        // Execute through guarded swarm branch
        let outcome = self
            .execute_swarm_branch(TADPOLE_ALPHA_ID.to_string(), final_directive, ctx, 1, None)
            .await;

        if let Some(ref vt) = ctx.visible_transcript {
            vt.lock().push(format!(
                "OBSERVATION (Alpha Directive): [{}] {}",
                outcome.status.as_str().to_uppercase(),
                outcome.output
            ));
        }

        if outcome.status != SwarmBranchStatus::Completed {
            ctx.swarm_partial_failure
                .store(true, std::sync::atomic::Ordering::Release);
            return Ok(format!(
                "SWARM_PARTIAL_FAILURE: Alpha directive resulted in status '{}'.\n\nOutcome: {}",
                outcome.status.as_str(),
                outcome.output
            ));
        }

        Ok(format!(
            "Directive executed by Tadpole Alpha. Mission ID: {}\n\nResult: {}",
            ctx.mission_id, outcome.output
        ))
    }
}

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::agent::types::{ModelConfig, ToolCall};
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_test_runner() -> (AgentRunner, Arc<AppState>) {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        (runner, state)
    }

    #[tokio::test]
    async fn test_proactive_recursion_block_parent() {
        let (runner, _) = setup_test_runner().await;
        let mut ctx = RunContext {
            agent_id: "weather_api".to_string(),
            lineage: vec![
                "2".to_string(),
                "alpha".to_string(),
                "weather_agent".to_string(),
            ],
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
            agent_id: "weather_api".to_string(),
            lineage: vec!["2".to_string(), "alpha".to_string()],
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
            agent_id: "3".to_string(),
            lineage: vec!["1".to_string(), "2".to_string()],
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
            agent_id: "analyst".to_string(),
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
            primary_goal: Some("Deploy v1.2 to cluster".to_string()),
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
            agent_id: "orchestrator".to_string(),
            mission_id: "test-parallel".to_string(),
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
}
