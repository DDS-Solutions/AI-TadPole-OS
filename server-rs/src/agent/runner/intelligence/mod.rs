//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Intelligence Module**: Coordinates the agent thinking turns and workflow coordinator interface.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Model failure or state rollback during workflow execution.
//!

pub mod loop_detector;
pub mod sentinel;
pub mod monologue;
pub mod hierarchy;
pub mod turn;

pub use loop_detector::DoomLoopDetector;
pub use hierarchy::resolve_hierarchy_label;

#[cfg(test)]
pub mod tests;

use crate::error::AppError;
use crate::agent::runner::{AgentRunner, IntelligenceOutput, RunContext};
use crate::agent::types::TaskPayload;

/// Result type for a single agent turn.
#[derive(Debug, Clone)]
pub enum TurnOutcome {
    /// Turn completed, continue to next turn
    Continue,
    /// Mission completed with final output
    Completed(IntelligenceOutput),
    /// Budget exceeded, halt with partial output
    BudgetExceeded(IntelligenceOutput),
}

impl AgentRunner {
    /// Handles the prompt generation, provider calls, and tool execution loop.
    pub(crate) async fn execute_intelligence_loop(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        self.workflow_coordinator
            .execute_workflow(self, ctx, payload)
            .await
    }
}

pub struct MissionWorkflowCoordinator {
    pub state: std::sync::Arc<crate::state::AppState>,
    #[allow(dead_code)]
    pub prompt_service: std::sync::Arc<dyn super::service_traits::PromptService>,
    pub mission_state_manager: std::sync::Arc<dyn super::service_traits::MissionStateManager>,
}

#[async_trait::async_trait]
impl super::service_traits::WorkflowCoordinator for MissionWorkflowCoordinator {
    async fn execute_workflow(
        &self,
        runner: &AgentRunner,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        // --- 🛡️ [Anti-Injection Escaping] ---
        let sanitized_message =
            crate::agent::sanitizer::Sanitizer::sanitize_for_prompt(&payload.message);
        let mut sanitized_payload = payload.clone();
        sanitized_payload.message = sanitized_message;

        // --- 🛡️ [StateTransaction Boundary] ---
        let mut state_tx = crate::agent::runner::service_traits::StateTransaction::new(
            self.mission_state_manager.clone(),
            self.state.clone(),
            &ctx.agent_id,
            &ctx.mission_id,
        );

        // NOTE: All state-touching operations below this point are inside the
        // StateTransaction boundary. If commit() is never reached, the
        // status/task will be rolled back automatically via Drop.

        // --- [Mythos RAII Guard] ---
        let _turn_guard = hierarchy::ReasoningTurnGuard::new(ctx.agent_id.clone(), self.state.clone());

        let hierarchy_label = resolve_hierarchy_label(&ctx.agent_id, &ctx.role);

        runner.broadcast_agent(
            ctx,
            &format!("starting task ({})...", hierarchy_label),
            "info",
        );
        runner.update_status(
            &ctx.agent_id,
            &ctx.mission_id,
            "thinking",
            Some("Consulting intelligence model..."),
        );

        // --- 📑 [System 2] Specification Check (Fix 6: AgentMissionState dispatch) ---
        let spec =
            crate::agent::mission::get_mission_context(&self.state.resources.pool, &ctx.mission_id)
                .await
                .map_err(|e| (e, None))?;

        let is_fast_path = AgentRunner::is_fast_path_query(&sanitized_payload.message);
        let mission_state = super::service_traits::AgentMissionState::resolve(&spec, ctx.safe_mode, is_fast_path);
        let output = match mission_state {
            super::service_traits::AgentMissionState::SpecificationGeneration => {
                runner
                    .handle_specification_generation_phase(ctx, &sanitized_payload, &mut state_tx)
                    .await
            }
            _ => runner.handle_reasoning_phase(ctx, &sanitized_payload).await,
        };

        match output {
            Ok(out) => {
                state_tx.commit();
                Ok(out)
            }
            Err(e) => {
                if let Err(rollback_err) = state_tx.rollback().await {
                    tracing::error!(
                        "❌ [StateTransaction] Explicit rollback failed: {:?}",
                        rollback_err
                    );
                }
                Err(e)
            }
        }
    }
}
