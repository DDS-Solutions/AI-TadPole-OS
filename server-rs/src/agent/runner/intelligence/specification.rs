//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / specification
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::runner::{AgentRunner, IntelligenceOutput, RunContext};
use crate::agent::types::TaskPayload;
use crate::error::AppError;

fn format_spec_prompt(objective: &str) -> String {
    format!(
        "Generate a Unified Technical Specification (UTS) for this mission based on the provided Codebase Map.\n\n\
         ### MISSION OBJECTIVE:\n\
         {}\n\n\
         ### TASK:\n\
         1. Review the Codebase Map in your context.\n\
         2. Outline planned changes, architectural impact, and verification plan.\n\
         3. Provide the spec in clean Markdown format.\n\
         Do NOT write code yet. Respond with the UTS only.",
        objective
    )
}

impl AgentRunner {
    fn resolve_spec_slot(
        &self,
        ctx: &RunContext,
        active_ctx: &mut RunContext,
        payload: &TaskPayload,
    ) {
        if payload.active_model_slot.as_deref() != Some("default") {
            self.activate_model_slot(
                ctx,
                active_ctx,
                super::super::service_traits::SlotKind::Planning,
                "Planning",
            );
        }
    }

    pub(crate) async fn handle_specification_generation_phase(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
        state_tx: &mut crate::agent::runner::service_traits::StateTransaction,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        let mut active_ctx = ctx.clone();
        self.resolve_spec_slot(ctx, &mut active_ctx, payload);

        let system_prompt = self
            .prompt_service
            .build_system_prompt(self, &active_ctx, &payload.message)
            .await;

        self.broadcast_agent(ctx, "Phase: Specification (System 2 Thinking)", "info");
        self.update_status(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            "thinking",
            Some("Generating Unified Technical Specification (UTS)..."),
        );

        let spec_prompt = format_spec_prompt(&payload.message);
        let mut usage = ctx
            .planning_usage
            .clone()
            .or_else(|| Some(crate::agent::types::TokenUsage::default()));
        let (spec_content, _, spec_usage) = self
            .call_provider(&active_ctx, &system_prompt, &spec_prompt, None)
            .await
            .map_err(|e| (e, usage.clone()))?;
        self.accumulate_usage(&mut usage, spec_usage);

        crate::agent::mission::set_mission_spec(
            &self.state.resources.pool,
            &ctx.mission_state.mission_id,
            &ctx.identity.agent_id,
            &spec_content,
        )
        .await
        .map_err(|e| (e, usage.clone()))?;

        state_tx.record_mission_spec_change(
            &ctx.mission_state.mission_id,
            &ctx.identity.agent_id,
            None,
        );

        Ok(IntelligenceOutput {
            text: format!("## Unified Technical Specification (UTS) Generated\n\n{}\n\n> [!IMPORTANT]\n> Mission status set to **Spec Review**. Please approve the spec before execution proceeds.", spec_content),
            usage,
        })
    }
}
