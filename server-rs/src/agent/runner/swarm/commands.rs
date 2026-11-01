//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - Keep public swarm tool entrypoints thin and preserve assignment idempotency and branch evidence.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / swarm commands
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `ToolExecutionError::Validation`, `ToolExecutionError::ExecutionFailed`, `AppError`
//! - **Telemetry Targets**: `[Swarm]`

use super::{
    SubAgentOptions, SwarmBranchOutcome, SwarmBranchStatus, SwarmDispatchReport,
    MAX_CONDUCTOR_STEPS, MAX_RECRUITMENT_DEPTH, MAX_SWARM_TARGETS_PER_CALL, TADPOLE_ALPHA_ID,
    TADPOLE_CEO_ID,
};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::{TokenUsage, ToolCall};
use crate::error::AppError;
use std::collections::HashMap;

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
            ctx.mission_state
                .primary_goal
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
    pub(super) async fn synthesize_swarm_results(
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
                ctx.mission_state.mission_id
            );
        }

        let pooled_results = report.render_markdown();
        let review_instruction = if requires_human_review {
            "Some branches failed, were blocked, or were cancelled. Preserve those statuses explicitly, do not claim full completion, and recommend a concrete human retry/resume decision."
        } else {
            "All branches returned results. Preserve attribution and evidence references. Findings remain candidates until independently reproduced. Counter-check whether each proposed correction changes its stated failing example. Label missing or inconsistent evidence explicitly. A regression idea is not an executed passing test."
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
        // A text-free provider response must not erase completed branch evidence.
        let final_text = if final_text.trim().is_empty() {
            pooled_results
        } else {
            final_text
        };
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
        crate::agent::runner::swarm_persistence::execute_assignment_once(
            &self.state.resources.pool,
            ctx,
            fc,
            || self.handle_new_spawn_subagent(ctx, fc, usage),
        )
        .await
    }

    async fn handle_new_spawn_subagent(
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
            // Defect #6: Enforce executive root check and target cap on conductor plans as well
            if targets.iter().any(|id| id == TADPOLE_CEO_ID) {
                return Err(ToolExecutionError::Validation(
                    "Executive root agent ('1') cannot be recruited as a sub-agent worker"
                        .to_string(),
                ));
            }
            if targets.len() > MAX_SWARM_TARGETS_PER_CALL {
                return Err(ToolExecutionError::Validation(format!(
                    "Conductor plan accepts at most {MAX_SWARM_TARGETS_PER_CALL} unique targets per call"
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
        if ctx.mission_state.depth >= max_swarm_depth
            || ctx.mission_state.lineage.len() >= MAX_RECRUITMENT_DEPTH
        {
            tracing::warn!("🐝 [Swarm] Swarm depth limit exceeded (current: {}, max: {}). Blocking recruitment.", ctx.mission_state.depth, max_swarm_depth);
            return Ok(format!("PROTOCOL_VIOLATION: Swarm depth limit exceeded (current depth: {}). You cannot spawn more sub-agents.", ctx.mission_state.depth));
        }

        tracing::info!(
            "🐝 [Swarm] Agent {} spawning {} sub-agent(s): {:?}...",
            ctx.identity.agent_id,
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
        let mut resolved_by_target = HashMap::new();
        let mut staged_agents = Vec::new();
        for sub_agent_id in &target_ids {
            // Proactive circular recruitment guard
            if ctx.mission_state.lineage.contains(sub_agent_id)
                || ctx.identity.agent_id == *sub_agent_id
            {
                tracing::warn!(
                    "[Swarm] Circular recruitment blocked for '{}' vs lineage {:?}",
                    sub_agent_id,
                    ctx.mission_state.lineage
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
                        parent_agent_id: Some(&ctx.identity.agent_id),
                        mission_id: Some(&ctx.mission_state.mission_id),
                        parent_config: &ctx.model_config,
                        extra_skills: extra_skills.map(|v| v.as_slice()),
                        extra_workflows: extra_workflows.map(|v| v.as_slice()),
                        role_override,
                    },
                )
                .await?;

            // Defect #4: Guard circular recruitment on resolved ID as well
            if ctx.mission_state.lineage.contains(&resolved.id)
                || ctx.identity.agent_id == resolved.id
            {
                tracing::warn!(
                    "[Swarm] Circular recruitment blocked for resolved ID '{}' vs lineage {:?}",
                    resolved.id,
                    ctx.mission_state.lineage
                );
                return Ok(format!(
                    "PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT - Resolved agent '{}' is already in the recruitment lineage.",
                    resolved.id
                ));
            }

            resolved_by_target.insert(sub_agent_id.clone(), resolved.id.clone());
            resolved_ids.push(resolved.id);
            if let Some(agent) = resolved.staged_agent {
                staged_agents.push(agent);
            }
        }

        // Commit transaction atomically
        tx.commit()
            .await
            .map_err(|e| ToolExecutionError::ExecutionFailed(e.to_string()))?;

        // Publish committed agents to live registry (Defect #13: merge targeted fields for pre-existing agents)
        for agent in staged_agents {
            self.state
                .registry
                .capability_map
                .write()
                .update_agent(&agent);

            if let Some(mut existing) = self.state.registry.agents.get_mut(&agent.identity.id) {
                // Pre-existing agent: update active mission, parent, and status without clobbering concurrent state
                existing.value_mut().metadata.insert(
                    "parent_agent_id".to_string(),
                    serde_json::Value::String(ctx.identity.agent_id.clone()),
                );
                existing.value_mut().state.active_mission = Some(serde_json::json!({
                    "id": ctx.mission_state.mission_id,
                    "parent_agent_id": ctx.identity.agent_id
                }));
                existing.value_mut().health.status = "working".to_string();
                existing.value_mut().capabilities = agent.capabilities;
            } else {
                // Newly fabricated agent: insert complete entry
                self.state
                    .registry
                    .agents
                    .insert(agent.identity.id.clone(), agent);
            }
        }

        // Also update any remaining resolved targets so their pulse edge connects to parent
        for target_id in &resolved_ids {
            if let Some(mut entry) = self.state.registry.agents.get_mut(target_id) {
                entry.value_mut().metadata.insert(
                    "parent_agent_id".to_string(),
                    serde_json::Value::String(ctx.identity.agent_id.clone()),
                );
                entry.value_mut().state.active_mission = Some(serde_json::json!({
                    "id": ctx.mission_state.mission_id,
                    "parent_agent_id": ctx.identity.agent_id
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

        // Dispatch execution with resolved_by_target map (Defect #4)
        let results = self
            .execute_swarm_dispatch(resolved_ids, resolved_by_target, sub_message, ctx)
            .await?;

        // Synthesize results
        self.synthesize_swarm_results(results, ctx, usage).await
    }

    /// Handles `handle_alpha_directive`: delegates to Tadpole Alpha (ID: 2) with full guard enforcement.
    pub(crate) async fn handle_alpha_directive(
        &self,
        ctx: &RunContext,
        fc: &ToolCall,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, AppError> {
        crate::agent::runner::swarm_persistence::execute_assignment_once(
            &self.state.resources.pool,
            ctx,
            fc,
            || async {
                self.handle_new_alpha_directive(ctx, fc, usage)
                    .await
                    .map_err(ToolExecutionError::from)
            },
        )
        .await
        .map_err(|error| match error {
            ToolExecutionError::AppError(error) => error,
            ToolExecutionError::Validation(msg) => AppError::BadRequest(msg),
            other => AppError::InternalServerError(other.to_string()),
        })
    }

    async fn handle_new_alpha_directive(
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
            ctx.identity.agent_id
        );
        self.broadcast_agent(ctx, "🧬 Issuing directive to Tadpole Alpha...", "info");

        // Lineage cycle guard
        if ctx
            .mission_state
            .lineage
            .contains(&TADPOLE_ALPHA_ID.to_string())
            || ctx.identity.agent_id == TADPOLE_ALPHA_ID
        {
            tracing::warn!(
                "[Swarm] Circular delegation blocked: Tadpole Alpha is already in lineage {:?}",
                ctx.mission_state.lineage
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
        if ctx.mission_state.depth >= max_swarm_depth
            || ctx.mission_state.lineage.len() >= MAX_RECRUITMENT_DEPTH
        {
            tracing::warn!(
                "[Swarm] Depth limit exceeded during alpha directive (depth: {}, max: {})",
                ctx.mission_state.depth,
                max_swarm_depth
            );
            return Ok(format!(
                "PROTOCOL_VIOLATION: Swarm depth limit exceeded (current depth: {}). Cannot delegate to Alpha.",
                ctx.mission_state.depth
            ));
        }

        // Mission cancellation check
        if crate::agent::mission::is_mission_completed(
            &self.state.resources.pool,
            &ctx.mission_state.mission_id,
        )
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
            ctx.mission_state.mission_id, outcome.output
        ))
    }
}

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────
