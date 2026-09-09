//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / metrics_tools
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::error::AppError;

impl AgentRunner {
    /// Handles `handle_get_agent_metrics`: retrieves live financial and identity data from the registry.
    pub(crate) async fn handle_get_agent_metrics(
        &self,
        ctx: &RunContext,
        _fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let agent_id = &ctx.agent_id;

        tracing::info!(
            "📊 [Governance] Agent {} fetching live metrics...",
            agent_id
        );

        // Fetch live data from the in-memory registry for maximum accuracy
        let (budget, cost, name, role, department) =
            if let Some(a) = self.state.registry.agents.get(agent_id) {
                let v = a.value();
                (
                    v.economics.budget_usd,
                    v.economics.cost_usd,
                    v.identity.name.clone(),
                    v.identity.role.clone(),
                    v.identity.department.clone(),
                )
            } else {
                // Fallback to RunContext if registry lookup fails (though this shouldn't happen during a valid run)
                (
                    0.0,
                    0.0,
                    ctx.name.clone(),
                    ctx.role.clone(),
                    ctx.department.clone(),
                )
            };

        let metrics = serde_json::json!({
            "agent_id": agent_id,
            "name": name,
            "role": role,
            "department": department,
            "budget_limit_usd": budget,
            "current_cost_usd": cost,
            "remaining_budget_usd": budget - cost,
            "status": if cost >= budget { "BREACHED" } else { "OK" },
            "mission_id": ctx.mission_id
        });

        let metrics_str = format!(
            "(AGENT METRICS RETRIEVED):\n\n{}",
            serde_json::to_string_pretty(&metrics).map_err(|e| AppError::InternalServerError(
                format!("Failed to serialize metrics: {}", e)
            ))?
        );

        self.broadcast_agent(
            ctx,
            &format!(
                "📊 Governance: reviewed live metrics (${:.4} / ${:.2})",
                cost, budget
            ),
            "info",
        );

        Ok(metrics_str)
    }
}
