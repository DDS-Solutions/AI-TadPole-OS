//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / conductor
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::{AgentRunner, RunContext};
use crate::agent::types::TokenUsage;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

fn invalid_conductor_plan(detail: impl Into<String>) -> AppError {
    AppError::InfrastructureError {
        provider_id: crate::error::ProviderId::Runner,
        kind: crate::error::InfrastructureErrorKind::Other,
        detail: detail.into(),
        help_link: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConductorStep {
    pub step_id: u32,
    pub subtask: String,
    pub target_agent: String,
    pub access_list: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConductorPlan {
    pub steps: Vec<ConductorStep>,
}

impl AgentRunner {
    /// Calls the model to decompose the mission query into a structured Conductor plan (DAG).
    pub(crate) async fn generate_conductor_plan(
        &self,
        ctx: &RunContext,
        message: &str,
    ) -> Result<(ConductorPlan, Option<TokenUsage>), (AppError, Option<TokenUsage>)> {
        tracing::info!(
            "🧠 [Conductor] Generating structured execution plan (DAG) for mission: {}",
            ctx.mission_id
        );

        let system_prompt = "You are the Conductor. Decompose the user's primary goal into a sequence of steps to be executed by specialized agents.\n\
                             You MUST respond with a valid JSON object matching this schema:\n\
                             {\n\
                               \"steps\": [\n\
                                 {\n\
                                   \"stepId\": 1,\n\
                                   \"subtask\": \"Analyze the repository structure.\",\n\
                                   \"targetAgent\": \"explorer-scout\",\n\
                                   \"accessList\": []\n\
                                 },\n\
                                 {\n\
                                   \"stepId\": 2,\n\
                                   \"subtask\": \"Compile and run tests based on step 1 observations.\",\n\
                                   \"targetAgent\": \"tester\",\n\
                                   \"accessList\": [1]\n\
                                 }\n\
                               ]\n\
                             }\n\
                             Do not include any thought tags or extra text. Output JSON only.";

        let user_message = format!("Primary Goal: {}", message);

        // We use the default planning slot model to run this planning query
        let (plan_text, _, planning_usage) = self
            .call_provider(ctx, system_prompt, &user_message, None)
            .await
            .map_err(|error| (error, None))?;

        // Extract JSON string robustly by isolating everything between the first '{' and last '}'
        let clean_json = plan_text.trim();
        let plan_json_to_parse = if let (Some(start_idx), Some(end_idx)) =
            (clean_json.find('{'), clean_json.rfind('}'))
        {
            if start_idx <= end_idx {
                &clean_json[start_idx..=end_idx]
            } else {
                clean_json
            }
        } else {
            clean_json
        };

        // Parse JSON output
        let plan: ConductorPlan = serde_json::from_str(plan_json_to_parse).map_err(|e| {
            (
                AppError::InfrastructureError {
                    provider_id: crate::error::ProviderId::Runner,
                    kind: crate::error::InfrastructureErrorKind::Other,
                    detail: format!(
                        "Failed to parse Conductor plan JSON: {}. Response excerpt: {}",
                        e, plan_json_to_parse
                    ),
                    help_link: None,
                },
                planning_usage.clone(),
            )
        })?;

        // Validate and sort steps
        let sorted_steps = Self::topological_sort_conductor_steps(&plan.steps)
            .map_err(|error| (error, planning_usage.clone()))?;

        Ok((
            ConductorPlan {
                steps: sorted_steps,
            },
            planning_usage,
        ))
    }

    /// Verifies topological sort and returns a sorted vector of steps or error on cycle or missing dependency.
    pub(crate) fn topological_sort_conductor_steps(
        steps: &[ConductorStep],
    ) -> Result<Vec<ConductorStep>, AppError> {
        if steps.is_empty() || steps.len() > crate::agent::runner::swarm::MAX_CONDUCTOR_STEPS {
            return Err(invalid_conductor_plan(format!(
                "Conductor plan must contain 1..={} steps",
                crate::agent::runner::swarm::MAX_CONDUCTOR_STEPS
            )));
        }

        let mut unique_step_ids = std::collections::HashSet::new();
        for step in steps {
            if step.step_id == 0 || !unique_step_ids.insert(step.step_id) {
                return Err(invalid_conductor_plan(format!(
                    "Conductor step IDs must be unique positive integers; invalid ID {}",
                    step.step_id
                )));
            }
            if step.subtask.trim().is_empty() || step.subtask.len() > 8_000 {
                return Err(invalid_conductor_plan(format!(
                    "Conductor step {} has an empty or oversized subtask",
                    step.step_id
                )));
            }
            if step.target_agent.trim().is_empty()
                || step.target_agent.len() > 128
                || step.target_agent.chars().any(char::is_control)
            {
                return Err(invalid_conductor_plan(format!(
                    "Conductor step {} has an invalid target agent",
                    step.step_id
                )));
            }
            if step.access_list.contains(&step.step_id) {
                return Err(invalid_conductor_plan(format!(
                    "Conductor step {} cannot depend on itself",
                    step.step_id
                )));
            }
        }

        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Map steps for O(1) dependency lookups
        let step_map: std::collections::HashMap<u32, &ConductorStep> =
            steps.iter().map(|s| (s.step_id, s)).collect();

        // Recursion safety invariant: DFS recursion depth is strictly bounded by
        // `MAX_CONDUCTOR_STEPS` (<= 12), ensuring complete stack-safety without heap trampoline overhead.
        fn visit(
            step_id: u32,
            step_map: &std::collections::HashMap<u32, &ConductorStep>,
            visited: &mut std::collections::HashSet<u32>,
            temp: &mut std::collections::HashSet<u32>,
            sorted: &mut Vec<ConductorStep>,
        ) -> Result<(), String> {
            if temp.contains(&step_id) {
                return Err(format!("Cycle detected at step {}", step_id));
            }
            if !visited.contains(&step_id) {
                temp.insert(step_id);
                if let Some(step) = step_map.get(&step_id) {
                    for &dep in &step.access_list {
                        // Explicitly validate that all referenced steps exist in the plan
                        if !step_map.contains_key(&dep) {
                            return Err(format!(
                                "Step {} depends on step {} which does not exist in the plan",
                                step_id, dep
                            ));
                        }
                        visit(dep, step_map, visited, temp, sorted)?;
                    }
                    visited.insert(step_id);
                    temp.remove(&step_id);
                    sorted.push((*step).clone());
                } else {
                    // Defensive fallback (unreachable given the preceding key validation)
                    temp.remove(&step_id);
                    visited.insert(step_id);
                }
            }
            Ok(())
        }

        for step in steps {
            if !visited.contains(&step.step_id) {
                let mut temp_set = std::collections::HashSet::new();
                visit(
                    step.step_id,
                    &step_map,
                    &mut visited,
                    &mut temp_set,
                    &mut sorted,
                )
                .map_err(|e| AppError::InfrastructureError {
                    provider_id: crate::error::ProviderId::Runner,
                    kind: crate::error::InfrastructureErrorKind::Other,
                    detail: format!("Topological sort cycle or logic failure: {}", e),
                    help_link: None,
                })?;
            }
        }

        Ok(sorted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(step_id: u32, access_list: Vec<u32>) -> ConductorStep {
        ConductorStep {
            step_id,
            subtask: format!("step {step_id}"),
            target_agent: format!("agent-{step_id}"),
            access_list,
        }
    }

    #[test]
    fn sorts_valid_dag_by_dependencies() {
        let sorted = AgentRunner::topological_sort_conductor_steps(&[
            step(3, vec![1, 2]),
            step(2, vec![1]),
            step(1, vec![]),
        ])
        .unwrap();
        assert_eq!(
            sorted.iter().map(|item| item.step_id).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn sorts_diamond_dag_topology() {
        // Diamond DAG: 1 is base, 2 & 3 depend on 1, 4 depends on 2 & 3
        let sorted = AgentRunner::topological_sort_conductor_steps(&[
            step(4, vec![2, 3]),
            step(3, vec![1]),
            step(2, vec![1]),
            step(1, vec![]),
        ])
        .unwrap();

        let ids: Vec<u32> = sorted.iter().map(|s| s.step_id).collect();
        assert_eq!(ids[0], 1);
        assert_eq!(ids[3], 4);
        assert!(ids[1] == 2 || ids[1] == 3);
        assert!(ids[2] == 2 || ids[2] == 3);
    }

    #[test]
    fn rejects_self_dependency() {
        assert!(AgentRunner::topological_sort_conductor_steps(&[step(1, vec![1])]).is_err());
    }

    #[test]
    fn rejects_duplicate_step_ids() {
        assert!(AgentRunner::topological_sort_conductor_steps(
            &[step(1, vec![]), step(1, vec![]),]
        )
        .is_err());
    }

    #[test]
    fn rejects_cycles_and_missing_dependencies() {
        assert!(AgentRunner::topological_sort_conductor_steps(&[
            step(1, vec![2]),
            step(2, vec![1]),
        ])
        .is_err());
        assert!(AgentRunner::topological_sort_conductor_steps(&[step(1, vec![99])]).is_err());
    }

    #[test]
    fn rejects_empty_and_oversized_plans() {
        assert!(AgentRunner::topological_sort_conductor_steps(&[]).is_err());
        let oversized = (1..=(crate::agent::runner::swarm::MAX_CONDUCTOR_STEPS as u32 + 1))
            .map(|id| step(id, vec![]))
            .collect::<Vec<_>>();
        assert!(AgentRunner::topological_sort_conductor_steps(&oversized).is_err());
    }

    #[test]
    fn rejects_invalid_subtasks_and_target_agents() {
        // Empty subtask
        let mut s1 = step(1, vec![]);
        s1.subtask = "".to_string();
        assert!(AgentRunner::topological_sort_conductor_steps(&[s1]).is_err());

        // Oversized subtask (> 8000 chars)
        let mut s2 = step(1, vec![]);
        s2.subtask = "a".repeat(8001);
        assert!(AgentRunner::topological_sort_conductor_steps(&[s2]).is_err());

        // Target agent with control characters
        let mut s3 = step(1, vec![]);
        s3.target_agent = "agent\nattacker".to_string();
        assert!(AgentRunner::topological_sort_conductor_steps(&[s3]).is_err());
    }
}
