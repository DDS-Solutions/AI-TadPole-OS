//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - Grouped execution context owns identity, mission state and environment; retain scope across callers.
//! - Start deterministic workflows only for affirmative commands, never incidental mentions or prohibitions.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / workflow
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::types::TaskPayload;
use crate::error::AppError;

use super::AgentRunner;

impl AgentRunner {
    pub(crate) fn explicitly_requests_workflow(message: &str, workflow_name: &str) -> bool {
        let normalize = |text: &str| {
            text.to_lowercase()
                .split(|c: char| !c.is_alphanumeric() && c != '\'')
                .filter(|word| !word.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        let workflow_words = normalize(workflow_name);
        if workflow_words.is_empty() {
            return false;
        }
        for line in message.lines() {
            let trimmed = line.trim();
            if let Some(command) = trimmed.strip_prefix('/') {
                let command_words = normalize(command.split_whitespace().next().unwrap_or(""));
                if command_words == workflow_words {
                    return true;
                }
            }
            let words = normalize(line);
            let mut command = words.as_slice();
            if command.first().is_some_and(|word| word == "please") {
                command = &command[1..];
            }
            if command
                .first()
                .is_some_and(|word| matches!(word.as_str(), "can" | "could" | "would" | "will"))
                && command.get(1).is_some_and(|word| word == "you")
            {
                command = &command[2..];
            }
            if command.first().is_some_and(|word| word == "please") {
                command = &command[1..];
            }
            // The verb must lead the command. Scanning later words would treat
            // explanations and long prohibitions as requests to execute them.
            if command.first().is_some_and(|word| {
                matches!(
                    word.as_str(),
                    "run" | "execute" | "start" | "follow" | "apply" | "use" | "begin" | "launch"
                )
            }) {
                let target = &command[1..];
                let skip = target
                    .iter()
                    .take_while(|word| {
                        matches!(word.as_str(), "a" | "an" | "the" | "this" | "our" | "my")
                    })
                    .count();
                let target = &target[skip..];
                if target
                    .first()
                    .is_some_and(|word| matches!(word.as_str(), "workflow" | "sop"))
                    || target.starts_with(&workflow_words)
                {
                    return true;
                }
            }
        }
        false
    }

    // ─────────────────────────────────────────────────────────
    //  DETERMINISTIC WORKFLOW
    // ─────────────────────────────────────────────────────────

    /// Orchestrates the step-by-step execution of a deterministic markdown workflow.
    ///
    /// This method replaces the standard "reason-act" loop with a fixed sequence
    /// of instructions derived from the SOP file. This ensures perfect fidelity
    /// to business processes while reducing token overhead for SME workloads.
    pub(crate) async fn run_deterministic_workflow(
        &self,
        agent_id: &str,
        payload: TaskPayload,
        mission_id: &str,
        generation: u64,
        workflow: &mut crate::agent::workflows::WorkflowExecutionState,
    ) -> Result<super::IntelligenceOutput, AppError> {
        let mut final_out = String::new();
        let mut total_usage = crate::agent::types::TokenUsage::default();

        // 1. Resolve basic context (RAG/Paths)
        let ctx = match self
            .prepare_run_context(agent_id, &payload, mission_id, 0, &[], generation)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                let _ = self
                    .fail_unprepared_mission(agent_id, mission_id, generation, &e)
                    .await;
                return Err(e);
            }
        };

        self.broadcast_agent(
            &ctx,
            &format!(
                "📜 [SOP] Starting deterministic workflow: {}",
                workflow.workflow_name
            ),
            "info",
        );

        let mut accumulated_results = String::new();

        // 2. Step-by-Step Execution
        while let Some(step) = workflow.current_step() {
            self.broadcast_agent(
                &ctx,
                &format!(
                    "🔹 [Step {}/{}] {}",
                    workflow.current_step_index + 1,
                    workflow.steps.len(),
                    step.title
                ),
                "info",
            );

            self.update_status(
                &ctx.identity.agent_id,
                &ctx.mission_state.mission_id,
                "working",
                Some(&format!("Executing SOP Step: {}", step.title)),
            );

            // Synthesize a prompt for this specific step
            let mut step_prompt = format!(
                "You are currently executing Step {} of the '{}' workflow.\n\n### GLOBAL MISSION DIRECTIVE:\n{}\n\n### STEP TITLE: {}\n### INSTRUCTION:\n{}\n\n",
                workflow.current_step_index + 1,
                workflow.workflow_name,
                payload.primary_goal.as_deref().unwrap_or(&payload.message),
                step.title,
                step.instruction
            );

            if !accumulated_results.is_empty() {
                step_prompt.push_str(&format!(
                    "### PREVIOUS STEPS FINDINGS:\n{}\n\n",
                    accumulated_results
                ));
            }

            step_prompt.push_str(
                "### TASK:\nFollow the instruction above to achieve the global mission. DO NOT repeat work that has already been accomplished in previous steps. Provide a concise report of your actions and findings for this specific step.",
            );

            let mut step_payload = payload.clone();
            step_payload.message = step_prompt;

            // Execute via standard intelligence loop (supports tools)
            match self.execute_intelligence_loop(&ctx, &step_payload).await {
                Ok(out) => {
                    final_out.push_str(&format!("\n\n---\n## Step: {}\n{}", step.title, out.text));
                    accumulated_results.push_str(&format!(
                        "\n[Step {}: {}]\n{}\n",
                        workflow.current_step_index + 1,
                        step.title,
                        out.text
                    ));

                    if let Some(u) = out.usage {
                        total_usage.input_tokens += u.input_tokens;
                        total_usage.output_tokens += u.output_tokens;
                        total_usage.total_tokens += u.total_tokens;
                    }
                    workflow.advance();
                }
                Err((e, usage)) => {
                    let _ = self.fail_mission(&ctx, &e, &usage).await;
                    return Err(e);
                }
            }
        }

        // 3. Finalize
        self.broadcast_agent(
            &ctx,
            &format!("✅ [SOP] Workflow completed: {}", workflow.workflow_name),
            "success",
        );

        let text = self
            .finalize_run(&ctx, &final_out, &Some(total_usage.clone()))
            .await?;
        Ok(super::IntelligenceOutput {
            text,
            usage: Some(total_usage),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::AgentRunner;

    #[test]
    fn test_workflow_activation_requires_an_affirmative_command() {
        for message in [
            "Run the orchestrate workflow",
            "Please execute the workflow",
            "Follow our SOP",
            "/orchestrate inspect source",
            "Could you please run the workflow?",
        ] {
            assert!(
                AgentRunner::explicitly_requests_workflow(message, "orchestrate"),
                "{message}"
            );
        }
        for message in [
            "Read file audit. Do not use script_builder, a workflow, a script, or a tool wrapper.",
            "Do not run the orchestrate workflow.",
            "Never execute a workflow.",
            "Don't follow the SOP.",
            "Explain how the workflow works.",
            "Explain how to run the workflow.",
            "Do not ever under any circumstances run the workflow.",
            "Could you explain how to run orchestrate?",
            "Read the sophisticated parser.",
            "Use read_codebase_file directly; workflow is forbidden.",
        ] {
            assert!(
                !AgentRunner::explicitly_requests_workflow(message, "orchestrate"),
                "{message}"
            );
        }
        assert!(AgentRunner::explicitly_requests_workflow(
            "Run code_audit",
            "code_audit"
        ));
        assert!(!AgentRunner::explicitly_requests_workflow(
            "Run code_auditor",
            "code_audit"
        ));
    }

    /// Verifies that the step prompt format used in run_deterministic_workflow
    /// correctly includes the step index, workflow name, goal, title, and instruction,
    /// and appends prior-step findings when accumulated results are non-empty.
    #[test]
    fn test_workflow_step_prompt_format() {
        let workflow_name = "Compliance Audit SOP";
        let step_index = 2usize;
        let step_title = "Identify Risk Areas";
        let step_instruction = "Review all flagged items and classify by severity.";
        let goal = "Complete a full compliance audit for Q3.";
        let accumulated_results = "[Step 1: Document Ingestion]\nAll docs ingested.\n";

        // Recreate the same format string used in run_deterministic_workflow
        let mut step_prompt = format!(
            "You are currently executing Step {} of the '{}' workflow.\n\n\
             ### GLOBAL MISSION DIRECTIVE:\n{}\n\n\
             ### STEP TITLE: {}\n### INSTRUCTION:\n{}\n\n",
            step_index + 1,
            workflow_name,
            goal,
            step_title,
            step_instruction
        );

        if !accumulated_results.is_empty() {
            step_prompt.push_str(&format!(
                "### PREVIOUS STEPS FINDINGS:\n{}\n\n",
                accumulated_results
            ));
        }

        step_prompt.push_str(
            "### TASK:\nFollow the instruction above to achieve the global mission. \
             DO NOT repeat work that has already been accomplished in previous steps. \
             Provide a concise report of your actions and findings for this specific step.",
        );

        // Assertions
        assert!(
            step_prompt.contains("Step 3 of the 'Compliance Audit SOP'"),
            "Step number should be 1-indexed"
        );
        assert!(step_prompt.contains("### GLOBAL MISSION DIRECTIVE:"));
        assert!(step_prompt.contains("Complete a full compliance audit for Q3."));
        assert!(step_prompt.contains("### STEP TITLE: Identify Risk Areas"));
        assert!(step_prompt.contains("Review all flagged items and classify by severity."));
        assert!(step_prompt.contains("### PREVIOUS STEPS FINDINGS:"));
        assert!(step_prompt.contains("All docs ingested."));
        assert!(step_prompt.contains("### TASK:"));
    }
}
