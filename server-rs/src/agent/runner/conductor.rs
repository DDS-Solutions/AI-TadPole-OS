//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / conductor
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Localized single-agent tasks bypass Conductor DAG planning to conserve tokens and reduce turn latency.
//!   - enforced_by: `test_is_conductor_plan_required`
//! - `[Behavioral]` Planning payloads from reasoning models (<think> blocks, markdown fences) are sanitized before deserialization.
//!   - enforced_by: `test_extract_json_plan_payload`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_is_conductor_plan_required`, `test_extract_json_plan_payload`

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
    /// Determines whether a mission requires a full multi-agent Conductor DAG decomposition.
    /// Returns false for localized, single-agent, or direct file operations, skipping the
    /// heavy planning model call and token budget overhead.
    pub(crate) fn is_conductor_plan_required(message: &str) -> bool {
        let lower = message.to_lowercase();

        // 1. Explicit multi-agent, swarm, or workflow indicators
        static MULTI_AGENT_TRIGGERS: &[&str] = &[
            "swarm",
            "multi-agent",
            "multi agent",
            "conductor",
            "dag",
            "orchestrate",
            "pipeline",
            "workflow",
            "end-to-end",
            "e2e",
            "full-stack",
            "full stack",
            "across all",
            "all services",
            "multi-step",
            "multi step",
            "phase 1",
            "step 1",
        ];

        for trigger in MULTI_AGENT_TRIGGERS {
            if lower.contains(trigger) {
                return true;
            }
        }

        // 2. Sequential step markers (e.g. "1. ... 2. ..." or "first ..., then ...")
        if (lower.contains("1. ") && lower.contains("2. "))
            || (lower.contains("1) ") && lower.contains("2) "))
            || (lower.contains("first ") && lower.contains("then "))
        {
            return true;
        }

        // 3. Multi-phase project directives
        static MULTI_PHASE_DIRECTIVES: &[&str] = &[
            "audit and refactor",
            "benchmark and optimize",
            "migrate and test",
            "build app",
            "create app",
            "build application",
            "create application",
        ];

        for directive in MULTI_PHASE_DIRECTIVES {
            if lower.contains(directive) {
                return true;
            }
        }

        // 4. Batch operations across multiple files or components require planning if paired with action verbs
        let is_batch_scope = (lower.contains("all ")
            || lower.contains("every ")
            || lower.contains("each ")
            || lower.contains("across "))
            && (lower.contains("file")
                || lower.contains("module")
                || lower.contains("crate")
                || lower.contains("component"));
        if is_batch_scope
            && (lower.contains("audit")
                || lower.contains("refactor")
                || lower.contains("check")
                || lower.contains("update")
                || lower.contains("test")
                || lower.contains("migrate"))
        {
            return true;
        }

        // 5. If prompt mentions a specific file directly (single-file target) and is under 200 chars,
        // it is a targeted single-agent task, not a multi-agent DAG.
        let is_batch = lower.contains("all ")
            || lower.contains("every ")
            || lower.contains("each ")
            || lower.contains("files")
            || lower.contains("*.");

        let mentions_single_file = message.contains(".rs")
            || message.contains(".ts")
            || message.contains(".tsx")
            || message.contains(".toml")
            || message.contains(".json")
            || message.contains(".css")
            || message.contains(".md")
            || message.contains(".sql");

        if mentions_single_file && !is_batch && message.len() < 200 {
            return false;
        }

        // 6. Short, localized prompts (< 100 characters) without multi-step markers are single-agent tasks
        if message.len() < 100 {
            return false;
        }

        // 7. Broad/complex prompts (> 250 characters) without single-file targeting default to DAG decomposition
        message.len() > 250
    }

    /// Extracts the JSON plan payload string from raw LLM output, cleanly stripping
    /// reasoning/thought tags (`<think>...</think>`, `<thought>...</thought>`) and markdown fences.
    pub(crate) fn extract_json_plan_payload(raw: &str) -> &str {
        let mut text = raw.trim();

        // 1. Strip reasoning / thought blocks (<think>...</think> or <thought>...</thought>)
        while let Some(start_think) = text.find("<think>") {
            if let Some(end_think) = text[start_think..].find("</think>") {
                let before = &text[..start_think];
                let after = &text[start_think + end_think + 8..];
                if after.contains('{') {
                    text = after.trim();
                } else if before.contains('{') {
                    text = before.trim();
                } else {
                    text = after.trim();
                }
            } else {
                break;
            }
        }

        while let Some(start_thought) = text.find("<thought>") {
            if let Some(end_thought) = text[start_thought..].find("</thought>") {
                let before = &text[..start_thought];
                let after = &text[start_thought + end_thought + 10..];
                if after.contains('{') {
                    text = after.trim();
                } else if before.contains('{') {
                    text = before.trim();
                } else {
                    text = after.trim();
                }
            } else {
                break;
            }
        }

        // 2. Strip markdown code fence if wrapping the payload
        if let Some(start_fence) = text.find("```") {
            if let Some(first_nl) = text[start_fence..].find('\n') {
                let content_start = start_fence + first_nl + 1;
                if let Some(end_fence) = text[content_start..].rfind("```") {
                    let inside_fence = text[content_start..content_start + end_fence].trim();
                    if inside_fence.contains('{') && inside_fence.contains('}') {
                        text = inside_fence;
                    }
                }
            }
        }

        // 3. Isolate outermost JSON object bounds: first '{' and last '}'
        if let (Some(start_idx), Some(end_idx)) = (text.find('{'), text.rfind('}')) {
            if start_idx <= end_idx {
                return &text[start_idx..=end_idx];
            }
        }

        text
    }

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

        let plan_json_to_parse = Self::extract_json_plan_payload(&plan_text);

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
    #[test]
    fn test_is_conductor_plan_required() {
        // Multi-agent / swarm triggers require DAG planning
        assert!(AgentRunner::is_conductor_plan_required(
            "Orchestrate a swarm to audit the backend"
        ));
        assert!(AgentRunner::is_conductor_plan_required(
            "Multi-agent workflow for auth migration"
        ));
        assert!(AgentRunner::is_conductor_plan_required(
            "Build an end-to-end pipeline"
        ));
        assert!(AgentRunner::is_conductor_plan_required(
            "1. Research specs 2. Generate code"
        ));
        assert!(AgentRunner::is_conductor_plan_required(
            "First inspect the files, then run tests"
        ));
        assert!(AgentRunner::is_conductor_plan_required(
            "Audit and refactor the database layer"
        ));

        // Batch operations across files/modules require planning
        assert!(AgentRunner::is_conductor_plan_required(
            "Audit every .rs file in the project"
        ));
        assert!(AgentRunner::is_conductor_plan_required(
            "Refactor all components across the frontend"
        ));

        // Single-agent / localized tasks bypass DAG planning
        assert!(!AgentRunner::is_conductor_plan_required(
            "Fix the typo in turn.rs:409"
        ));
        assert!(!AgentRunner::is_conductor_plan_required(
            "Update version to 1.2.0 in Cargo.toml"
        ));
        assert!(!AgentRunner::is_conductor_plan_required(
            "Patch user routes in routes.rs"
        ));
        assert!(!AgentRunner::is_conductor_plan_required(
            "Run cargo check and report warnings"
        ));
        assert!(!AgentRunner::is_conductor_plan_required(
            "Add a unit test for helper foo"
        ));
    }

    #[test]
    fn test_extract_json_plan_payload() {
        // 1. Clean JSON
        let clean = r#"{"steps": []}"#;
        assert_eq!(AgentRunner::extract_json_plan_payload(clean), clean);

        // 2. Markdown code fences
        let fenced = "```json\n{\n  \"steps\": []\n}\n```";
        assert_eq!(
            AgentRunner::extract_json_plan_payload(fenced),
            "{\n  \"steps\": []\n}"
        );

        // 3. Reasoning model with <think> tag containing curly braces
        let reasoning_output = "<think>\nThinking about alternatives: { \"discard\": true }.\nDecision made.\n</think>\n```json\n{\n  \"steps\": [\n    {\"stepId\": 1, \"subtask\": \"Test\", \"targetAgent\": \"tester\", \"accessList\": []}\n  ]\n}\n```";
        let extracted = AgentRunner::extract_json_plan_payload(reasoning_output);
        assert!(!extracted.contains("<think>"));
        assert!(!extracted.contains("discard"));
        assert!(extracted.starts_with('{'));
        assert!(extracted.ends_with('}'));

        // Verify deserialization succeeds on extracted output
        let plan: Result<ConductorPlan, _> = serde_json::from_str(extracted);
        assert!(plan.is_ok());
        assert_eq!(plan.unwrap().steps.len(), 1);

        // 4. Conversational wrapping
        let conversational = "Sure! Here is the plan:\n{\n  \"steps\": []\n}\nHope this helps!";
        assert_eq!(
            AgentRunner::extract_json_plan_payload(conversational),
            "{\n  \"steps\": []\n}"
        );
    }
}
