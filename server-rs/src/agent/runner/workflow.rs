//! Deterministic Workflow Execution — SOP-driven step-by-step processing
//!
//! Extracted from `runner/mod.rs` for maintainability. All methods remain
//! on `impl AgentRunner` — this is a structural split, not a behavioral change.
//!
//! @docs ARCHITECTURE:Runner

use crate::agent::types::TaskPayload;

use super::AgentRunner;

impl AgentRunner {
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
        workflow: &mut crate::agent::workflows::WorkflowExecutionState,
    ) -> anyhow::Result<String> {
        let mut final_out = String::new();
        let mut total_usage = crate::agent::types::TokenUsage::default();

        // 1. Resolve basic context (RAG/Paths)
        let mission_id = payload
            .cluster_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let ctx = self
            .prepare_run_context(agent_id, &payload, &mission_id, 0, &[])
            .await?;

        self.broadcast_sys(
            &format!(
                "📜 [SOP] Starting deterministic workflow: {}",
                workflow.workflow_name
            ),
            "info",
            Some(ctx.mission_id.clone()),
        );

        // 2. Step-by-Step Execution
        while let Some(step) = workflow.current_step() {
            self.broadcast_sys(
                &format!(
                    "🔹 [Step {}/{}] {}",
                    workflow.current_step_index + 1,
                    workflow.steps.len(),
                    step.title
                ),
                "info",
                Some(ctx.mission_id.clone()),
            );

            self.update_status(
                &ctx.agent_id,
                &ctx.mission_id,
                "working",
                Some(&format!("Executing SOP Step: {}", step.title)),
            );

            // Synthesize a prompt for this specific step
            let step_prompt = format!(
                "You are currently executing Step {} of the '{}' workflow.\n\n### STEP TITLE: {}\n### INSTRUCTION:\n{}\n\n### TASK:\nFollow the instruction above exactly. If you need to use tools, do so now. Provide a concise report of your actions.",
                workflow.current_step_index + 1,
                workflow.workflow_name,
                step.title,
                step.instruction
            );

            let mut step_payload = payload.clone();
            step_payload.message = step_prompt;

            // Execute via standard intelligence loop (supports tools)
            match self.execute_intelligence_loop(&ctx, &step_payload).await {
                Ok(out) => {
                    final_out.push_str(&format!("\n\n---\n## Step: {}\n{}", step.title, out.text));
                    if let Some(u) = out.usage {
                        total_usage.input_tokens += u.input_tokens;
                        total_usage.output_tokens += u.output_tokens;
                    }
                    workflow.advance();
                }
                Err((e, usage)) => {
                    self.fail_mission(&ctx, &e, &usage).await?;
                    return Err(e);
                }
            }
        }

        // 3. Finalize
        self.broadcast_sys(
            &format!("✅ [SOP] Workflow completed: {}", workflow.workflow_name),
            "success",
            Some(ctx.mission_id.clone()),
        );

        let _ = self
            .finalize_run(&ctx, &final_out, &Some(total_usage))
            .await?;
        Ok(final_out)
    }
}
