//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / dispatcher
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::runner::swarm::{
    SwarmBranchOutcome, SwarmBranchStatus, SwarmDispatchReport, MAX_RECRUITMENT_DEPTH,
    MAX_SWARM_TARGETS_PER_CALL,
};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};
use futures::stream::{self, StreamExt};
use std::collections::{BTreeMap, HashMap};

const MAX_DEP_OUTPUT_LEN: usize = 4_000;

impl AgentRunner {
    pub(crate) async fn execute_swarm_branch(
        &self,
        branch_id: String,
        instruction: String,
        ctx: &RunContext,
        branch_count: usize,
        step_id: Option<u32>,
    ) -> SwarmBranchOutcome {
        // Lineage cycle guard
        if ctx.lineage.contains(&branch_id) || ctx.agent_id == branch_id {
            tracing::warn!(
                "[Swarm] Recursion block triggered for {} vs {:?}",
                branch_id,
                ctx.lineage
            );
            return SwarmBranchOutcome {
                output: format!(
                    "PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT - '{}' is already in the recruitment lineage.",
                    branch_id
                ),
                branch_id,
                step_id,
                status: SwarmBranchStatus::Blocked,
            };
        }

        // Max depth guard
        if ctx.lineage.len() >= MAX_RECRUITMENT_DEPTH {
            tracing::warn!(
                "[Swarm] Max recruitment depth ({}) exceeded for '{}' (lineage: {:?})",
                MAX_RECRUITMENT_DEPTH,
                branch_id,
                ctx.lineage
            );
            return SwarmBranchOutcome {
                output: format!(
                    "PROTOCOL_VIOLATION: MAX_RECRUITMENT_DEPTH_EXCEEDED - Branch depth limit of {} reached.",
                    MAX_RECRUITMENT_DEPTH
                ),
                branch_id,
                step_id,
                status: SwarmBranchStatus::Blocked,
            };
        }

        let payload = ctx.derive_fanout_subtask_payload(instruction, branch_count);
        match Box::pin(self.clone().run(branch_id.clone(), payload)).await {
            Ok(output) => {
                let trimmed = output.trim();
                let status = if trimmed.is_empty() {
                    SwarmBranchStatus::Blocked
                } else if output.starts_with("SWARM_PARTIAL_FAILURE")
                    || output.starts_with("SWARM_REVIEW_REQUIRED")
                {
                    SwarmBranchStatus::Blocked
                } else {
                    SwarmBranchStatus::Completed
                };
                SwarmBranchOutcome {
                    branch_id,
                    step_id,
                    status,
                    output,
                }
            }
            Err(error) => {
                tracing::error!(
                    "[Swarm] Branch '{}' failed in mission {}: {}",
                    branch_id,
                    ctx.mission_id,
                    error
                );
                SwarmBranchOutcome {
                    branch_id,
                    step_id,
                    status: SwarmBranchStatus::Failed,
                    output: error.to_string(),
                }
            }
        }
    }

    async fn execute_conductor_dispatch(
        &self,
        ctx: &RunContext,
        sub_message: &str,
    ) -> Result<SwarmDispatchReport, ToolExecutionError> {
        let plan = ctx.conductor_plan.as_ref().ok_or_else(|| {
            ToolExecutionError::ExecutionFailed("Conductor plan disappeared before dispatch".into())
        })?;
        let sorted = Self::topological_sort_conductor_steps(&plan.steps)
            .map_err(|error| ToolExecutionError::Validation(error.to_string()))?;
        let branch_count = sorted.len();
        let mut pending: BTreeMap<u32, _> = sorted
            .into_iter()
            .map(|step| (step.step_id, step))
            .collect();
        let mut outcomes_by_step: HashMap<u32, SwarmBranchOutcome> = HashMap::new();
        let mut wave_index: usize = 0;

        while !pending.is_empty() {
            // Check cancellation status resiliently without throwing away completed work
            let is_cancelled = match crate::agent::mission::is_mission_completed(
                &self.state.resources.pool,
                &ctx.mission_id,
            )
            .await
            {
                Ok(val) => val,
                Err(err) => {
                    tracing::warn!(
                        "⚠️ [Swarm] Failed to query mission cancellation status: {}. Continuing with remaining DAG steps.",
                        err
                    );
                    false
                }
            };

            if is_cancelled {
                for (_, step) in std::mem::take(&mut pending) {
                    outcomes_by_step.insert(
                        step.step_id,
                        SwarmBranchOutcome {
                            branch_id: step.target_agent,
                            step_id: Some(step.step_id),
                            status: SwarmBranchStatus::Cancelled,
                            output: "Mission was closed by the operator before this step started."
                                .to_string(),
                        },
                    );
                }
                break;
            }

            let ready_ids: Vec<u32> = pending
                .iter()
                .filter(|(_, step)| {
                    step.access_list
                        .iter()
                        .all(|dependency| outcomes_by_step.contains_key(dependency))
                })
                .map(|(step_id, _)| *step_id)
                .collect();
            if ready_ids.is_empty() {
                return Err(ToolExecutionError::ExecutionFailed(
                    "Conductor DAG could not make progress after validation".to_string(),
                ));
            }

            let mut executable = Vec::new();
            for step_id in ready_ids {
                let step = pending.remove(&step_id).expect("ready step must exist");
                let failed_dep = step.access_list.iter().find_map(|dependency| {
                    outcomes_by_step.get(dependency).and_then(|outcome| {
                        if outcome.status != SwarmBranchStatus::Completed {
                            Some((*dependency, outcome.status))
                        } else {
                            None
                        }
                    })
                });

                if let Some((dependency, dep_status)) = failed_dep {
                    let msg = match dep_status {
                        SwarmBranchStatus::Cancelled => {
                            format!(
                                "Dependency step {} was cancelled by the operator.",
                                dependency
                            )
                        }
                        SwarmBranchStatus::Blocked => {
                            format!(
                                "Dependency step {} was blocked and requires review.",
                                dependency
                            )
                        }
                        _ => format!("Dependency step {} failed to complete.", dependency),
                    };
                    outcomes_by_step.insert(
                        step.step_id,
                        SwarmBranchOutcome {
                            branch_id: step.target_agent,
                            step_id: Some(step.step_id),
                            status: SwarmBranchStatus::Blocked,
                            output: format!("{}; human retry or replanning is required.", msg),
                        },
                    );
                } else {
                    executable.push(step);
                }
            }

            if executable.is_empty() {
                continue;
            }

            let prepared: Vec<_> = executable
                .into_iter()
                .map(|step| {
                    let dependency_context = step
                        .access_list
                        .iter()
                        .filter_map(|dependency| {
                            outcomes_by_step.get(dependency).map(|outcome| {
                                let truncated = if outcome.output.len() > MAX_DEP_OUTPUT_LEN {
                                    format!(
                                        "{}... [truncated]",
                                        &outcome.output[..MAX_DEP_OUTPUT_LEN]
                                    )
                                } else {
                                    outcome.output.clone()
                                };
                                format!(
                                    "=== BEGIN UNTRUSTED DATA FROM DEPENDENCY STEP {} ===\n{}\n=== END UNTRUSTED DATA ===\n",
                                    dependency, truncated
                                )
                            })
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    (step, dependency_context)
                })
                .collect();

            let parallelism = prepared.len().min(MAX_SWARM_TARGETS_PER_CALL).max(1);
            let orchestrator_preamble = if !sub_message.trim().is_empty() {
                format!("### ORCHESTRATOR INSTRUCTION:\n{}\n\n", sub_message.trim())
            } else {
                String::new()
            };

            let tasks = prepared.into_iter().map(|(step, dependency_context)| {
                let primary_goal_block = Self::mission_goal_block(ctx);
                let instruction = format!(
                    "{}{}Subtask: {}\n\n{}\n\nComplete only this DAG step.",
                    orchestrator_preamble, dependency_context, step.subtask, primary_goal_block
                );
                self.execute_swarm_branch(
                    step.target_agent,
                    instruction,
                    ctx,
                    branch_count,
                    Some(step.step_id),
                )
            });

            wave_index += 1;
            let wave: Vec<_> = stream::iter(tasks)
                .buffer_unordered(parallelism)
                .collect()
                .await;

            let completed_count = wave
                .iter()
                .filter(|o| o.status == SwarmBranchStatus::Completed)
                .count();
            let blocked_count = wave
                .iter()
                .filter(|o| o.status == SwarmBranchStatus::Blocked)
                .count();
            let failed_count = wave
                .iter()
                .filter(|o| o.status == SwarmBranchStatus::Failed)
                .count();

            tracing::info!(
                "🌊 [Swarm Wave {}] Executed {} steps (Completed: {}, Blocked: {}, Failed: {})",
                wave_index,
                wave.len(),
                completed_count,
                blocked_count,
                failed_count
            );

            for outcome in wave {
                let step_id = outcome.step_id.unwrap_or(0);
                outcomes_by_step.insert(step_id, outcome);
            }
        }

        let mut outcomes: Vec<_> = outcomes_by_step.into_values().collect();
        outcomes.sort_by_key(|outcome| outcome.step_id);
        Ok(SwarmDispatchReport { outcomes })
    }

    /// Executes direct swarm fan-out or dependency-aware Conductor waves.
    pub(crate) async fn execute_swarm_dispatch(
        &self,
        target_ids: Vec<String>,
        sub_message: &str,
        ctx: &RunContext,
    ) -> Result<SwarmDispatchReport, ToolExecutionError> {
        if ctx.conductor_plan.is_some() {
            tracing::info!(
                "[Conductor] Executing dependency-aware plan for agent '{}'",
                ctx.agent_id
            );
            return self.execute_conductor_dispatch(ctx, sub_message).await;
        }

        let stats = self
            .state
            .security
            .system_monitor
            .get_system_defense_stats();
        let sequential = stats.memory_pressure > 0.85
            || ctx.model_config.provider == crate::agent::types::ModelProvider::Ollama;
        let branch_count = target_ids.len().max(1);
        let mut outcomes = Vec::with_capacity(target_ids.len());

        if sequential {
            for (index, branch_id) in target_ids.into_iter().enumerate() {
                let is_cancelled = match crate::agent::mission::is_mission_completed(
                    &self.state.resources.pool,
                    &ctx.mission_id,
                )
                .await
                {
                    Ok(val) => val,
                    Err(err) => {
                        tracing::warn!(
                            "⚠️ [Swarm] Failed to query mission cancellation status: {}. Continuing sequential branch.",
                            err
                        );
                        false
                    }
                };

                if is_cancelled {
                    outcomes.push(SwarmBranchOutcome {
                        branch_id,
                        step_id: None,
                        status: SwarmBranchStatus::Cancelled,
                        output: format!(
                            "Mission was closed by the operator before branch {} of {} started.",
                            index + 1,
                            branch_count
                        ),
                    });
                    continue;
                }
                let instruction = self.build_neural_handoff(sub_message, ctx);
                outcomes.push(
                    self.execute_swarm_branch(branch_id, instruction, ctx, branch_count, None)
                        .await,
                );
            }
        } else {
            let is_cancelled = match crate::agent::mission::is_mission_completed(
                &self.state.resources.pool,
                &ctx.mission_id,
            )
            .await
            {
                Ok(val) => val,
                Err(err) => {
                    tracing::warn!(
                        "⚠️ [Swarm] Failed to query mission cancellation status: {}. Proceeding with fan-out.",
                        err
                    );
                    false
                }
            };

            if is_cancelled {
                outcomes.extend(target_ids.into_iter().map(|branch_id| SwarmBranchOutcome {
                    branch_id,
                    step_id: None,
                    status: SwarmBranchStatus::Cancelled,
                    output:
                        "Mission was closed by the operator before fan-out started.".to_string(),
                }));
            } else {
                let tasks = target_ids.into_iter().map(|branch_id| {
                    let instruction = self.build_neural_handoff(sub_message, ctx);
                    self.execute_swarm_branch(branch_id, instruction, ctx, branch_count, None)
                });
                outcomes = stream::iter(tasks)
                    .buffer_unordered(MAX_SWARM_TARGETS_PER_CALL)
                    .collect()
                    .await;
            }
        }

        Ok(SwarmDispatchReport { outcomes })
    }
}
