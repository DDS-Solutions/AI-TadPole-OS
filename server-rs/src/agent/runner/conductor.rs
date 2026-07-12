//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Conductor Planner**: Decomposes a primary mission query into a topologically
//! sorted, structured execution plan (DAG) of sub-agents.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Model planning failure, invalid JSON response, or topological sorting cycle.
//! - **Trace Scope**: `server-rs::agent::runner::conductor`

use super::{AgentRunner, RunContext};
use crate::error::AppError;
use serde::{Deserialize, Serialize};

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
    ) -> Result<ConductorPlan, AppError> {
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
        let (plan_text, _, _) = self
            .call_provider(ctx, system_prompt, &user_message, None)
            .await?;

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
            AppError::InfrastructureError {
                provider_id: crate::error::ProviderId::Runner,
                kind: crate::error::InfrastructureErrorKind::Other,
                detail: format!(
                    "Failed to parse Conductor plan JSON: {}. Response excerpt: {}",
                    e, plan_json_to_parse
                ),
                help_link: None,
            }
        })?;

        // Validate and sort steps
        let sorted_steps = Self::topological_sort_conductor_steps(&plan.steps)?;

        Ok(ConductorPlan {
            steps: sorted_steps,
        })
    }

    /// Verifies topological sort and returns a sorted vector of steps or error on cycle or missing dependency.
    pub(crate) fn topological_sort_conductor_steps(
        steps: &[ConductorStep],
    ) -> Result<Vec<ConductorStep>, AppError> {
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Map steps for O(1) dependency lookups
        let step_map: std::collections::HashMap<u32, &ConductorStep> =
            steps.iter().map(|s| (s.step_id, s)).collect();

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
