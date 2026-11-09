//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / slot
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::runner::{AgentRunner, RunContext};

impl AgentRunner {
    pub(crate) fn activate_model_slot(
        &self,
        ctx: &RunContext,
        active_ctx: &mut RunContext,
        preferred_kind: super::super::service_traits::SlotKind,
        phase_label: &str,
    ) -> super::super::service_traits::SlotKind {
        let effective_cluster_id = ctx
            .mission_state
            .cluster_id
            .as_deref()
            .or(Some(&ctx.mission_state.mission_id));
        let selection = self.model_router.select_model_slot(
            &self.state,
            &ctx.agent_models,
            preferred_kind,
            effective_cluster_id,
        );
        if selection.kind != super::super::service_traits::SlotKind::Default
            || selection.privacy_local_override
        {
            active_ctx.model_config = selection.config.clone();
            active_ctx.provider_name = selection.config.provider.to_string().to_lowercase();
            let msg = if selection.privacy_local_override {
                format!(
                    "Orchestrator: Privacy Shield active; using Local Slot ({}) for {}",
                    selection.config.model_id, phase_label
                )
            } else {
                format!(
                    "Orchestrator: Switching slot ({}) for {}",
                    selection.config.model_id, phase_label
                )
            };
            self.broadcast_agent(ctx, &msg, "pulse");
        }
        selection.kind
    }

    pub(crate) fn apply_slot_override(&self, active_ctx: &mut RunContext, slot: &str) {
        let Some(agent_entry) = self
            .state
            .registry
            .agents
            .get(&active_ctx.identity.agent_id)
        else {
            return;
        };
        let a = agent_entry.value();
        let slot_cfg = match slot {
            "planning"
                if a.models
                    .planning_slot
                    .as_ref()
                    .is_some_and(|s| !s.model_id.trim().is_empty()) =>
            {
                a.models.planning_slot.as_ref()
            }
            "execution"
                if a.models
                    .execution_slot
                    .as_ref()
                    .is_some_and(|s| !s.model_id.trim().is_empty()) =>
            {
                a.models.execution_slot.as_ref()
            }
            _ => Some(&a.models.model),
        };
        if let Some(cfg) = slot_cfg {
            active_ctx.model_config = cfg.clone();
            active_ctx.provider_name = cfg.provider.to_string().to_lowercase();
            tracing::info!("🔄 [Builder-Debugger Swap] Active context model config updated to slot '{}' (model: {})", slot, cfg.model_id);
        }
    }
}
