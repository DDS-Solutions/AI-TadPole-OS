//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Swarm Dispatcher**: Manages execution scheduling for sub-agents, handling
//! parallel execution, sequential throttling under memory pressure, and conductor plan execution.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Circular recursion, thread allocation exhaustion under parallel execution.

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;

impl AgentRunner {
    /// Executes the dispatch logic for Conductor vs Sequential vs Parallel modes.
    pub(crate) async fn execute_swarm_dispatch(
        &self,
        target_ids: Vec<String>,
        sub_message: &str,
        ctx: &RunContext,
    ) -> Result<Vec<String>, ToolExecutionError> {
        let mut results = Vec::new();

        if let Some(ref plan) = ctx.conductor_plan {
            tracing::info!(
                "🧠 [Conductor] Executing topologically sorted Conductor plan for agent '{}'",
                ctx.agent_id
            );
            let sorted_steps = match Self::topological_sort_conductor_steps(&plan.steps) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Conductor topological sort failed: {:?}", e);
                    return Ok(vec![format!(
                        "ERROR: Conductor plan topological sort failed: {:?}",
                        e
                    )]);
                }
            };

            let mut step_results: std::collections::HashMap<u32, String> =
                std::collections::HashMap::new();
            for step in sorted_steps {
                if ctx.lineage.contains(&step.target_agent) || ctx.agent_id == step.target_agent {
                    tracing::warn!(
                        "🛡️ [Swarm] Conductor recursion block for {} vs {:?}",
                        step.target_agent,
                        ctx.lineage
                    );
                    let res_text = format!("PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT - '{}' is in recruitment lineage.", step.target_agent);
                    step_results.insert(step.step_id, res_text.clone());
                    results.push(format!(
                        "### Step {} [{}] Result:\n{}",
                        step.step_id, step.target_agent, res_text
                    ));
                    continue;
                }

                // Build instruction injecting only output of step_id dependencies present in access_list
                let mut dependency_context = String::new();
                for &dep_id in &step.access_list {
                    if let Some(dep_output) = step_results.get(&dep_id) {
                        dependency_context.push_str(&format!(
                            "--- DEPENDENCY STEP {} OUTPUT ---\n{}\n---------------------------------\n\n",
                            dep_id, dep_output
                        ));
                    }
                }

                let primary_goal_text = ctx
                    .primary_goal
                    .as_deref()
                    .unwrap_or("See mission scope for details.");
                let final_instruction = format!(
                    "{}Subtask: {}\n\n### PRIMARY MISSION GOAL:\n{}\n\n(Please assist with the subtask and mission goal above.)",
                    dependency_context, step.subtask, primary_goal_text
                );

                let payload = ctx.derive_subtask_payload(final_instruction);
                let runner = self.clone();
                let res = match Box::pin(runner.run(step.target_agent.clone(), payload)).await {
                    Ok(r) => r,
                    Err(e) => format!("SUB-AGENT EXECUTION ERROR: {}", e),
                };

                step_results.insert(step.step_id, res.clone());
                results.push(format!(
                    "### Step {} [{}] Result:\n{}",
                    step.step_id, step.target_agent, res
                ));
            }
        } else {
            let stats = self.state.security.system_monitor.get_system_defense_stats();
            let is_memory_constrained = stats.memory_pressure > 0.85;

            let is_local_provider = ctx.model_config.provider == crate::agent::types::ModelProvider::Ollama;

            if is_memory_constrained || is_local_provider {
                if is_memory_constrained {
                    tracing::warn!(
                        "🐌 [Resource Guardian] High memory pressure ({:.1}%). Throttling concurrency: forcing sequential sub-agent execution for agent '{}'.",
                        stats.memory_pressure * 100.0,
                        ctx.agent_id
                    );
                } else {
                    tracing::info!(
                        "🐌 [Swarm Throttling] Agent '{}' running on local provider. Executing sub-agents sequentially to preserve host resources.",
                        ctx.agent_id
                    );
                }
                for sub_agent_id in target_ids {
                    // 🛡️ [Harden Proactive Lineage Guard]
                    if ctx.lineage.contains(&sub_agent_id) || ctx.agent_id == sub_agent_id {
                        tracing::warn!(
                            "🛡️ [Swarm] Recursion block triggered for {} vs {:?}",
                            sub_agent_id,
                            ctx.lineage
                        );
                        results.push(format!("### Sub-agent [{}] Result:\nPROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT - '{}' is already in your recruitment lineage. Parallel cycles are prohibited to prevent infinite loops (SEC-01).", sub_agent_id, sub_agent_id));
                        continue;
                    }

                    // Gating check: check if mission is already marked completed
                    if crate::agent::mission::is_mission_completed(&self.state.resources.pool, &ctx.mission_id).await.unwrap_or(false) {
                        tracing::info!("🏁 [Swarm Gating] Mission {} already completed. Skipping remaining sequential sub-agent '{}'.", ctx.mission_id, sub_agent_id);
                        break;
                    }

                    let runner = self.clone();
                    let final_instruction = self.build_neural_handoff(sub_message, ctx);
                    let payload = ctx.derive_subtask_payload(final_instruction);
                    let res = match Box::pin(runner.run(sub_agent_id.clone(), payload)).await {
                        Ok(r) => r,
                        Err(e) => format!("SUB-AGENT EXECUTION ERROR: {}", e),
                    };
                    results.push(format!("### Sub-agent [{}] Result:\n{}", sub_agent_id, res));
                }
            } else {
                use futures::stream::{FuturesUnordered, StreamExt};
                let mut swarm_tasks = FuturesUnordered::new();

                for sub_agent_id in target_ids {
                    // 🛡️ [Harden Phase 4: Proactive Lineage Guard]
                    if ctx.lineage.contains(&sub_agent_id) || ctx.agent_id == sub_agent_id {
                        tracing::warn!(
                            "🛡️ [Swarm] Recursion block triggered for {} vs {:?}",
                            sub_agent_id,
                            ctx.lineage
                        );
                        results.push(format!("### Sub-agent [{}] Result:\nPROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT - '{}' is already in your recruitment lineage. Parallel cycles are prohibited to prevent infinite loops (SEC-01).", sub_agent_id, sub_agent_id));
                        continue;
                    }

                    let runner = self.clone();
                    let ctx_clone = ctx.clone();
                    let final_instruction = self.build_neural_handoff(sub_message, ctx);
                    let sub_id_clone = sub_agent_id.clone();

                    swarm_tasks.push(async move {
                        let payload = ctx_clone.derive_subtask_payload(final_instruction);
                        let res = match Box::pin(runner.run(sub_id_clone.clone(), payload)).await {
                            Ok(r) => r,
                            Err(e) => format!("SUB-AGENT EXECUTION ERROR: {}", e),
                        };
                        (sub_id_clone, res)
                    });
                }

                while let Some((id, res)) = swarm_tasks.next().await {
                    results.push(format!("### Sub-agent [{}] Result:\n{}", id, res));
                    
                    // Gating check: check if mission is already marked completed
                    if crate::agent::mission::is_mission_completed(&self.state.resources.pool, &ctx.mission_id).await.unwrap_or(false) {
                        tracing::info!("🏁 [Swarm Gating] Mission {} completed. Aborting remaining parallel sub-tasks.", ctx.mission_id);
                        break;
                    }
                }
            }
        }
        Ok(results)
    }
}
