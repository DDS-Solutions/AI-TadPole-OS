//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Metrics Tools**: Provides deterministic access to an agent's own
//! live metrics, including budget limits, current costs, and swarm
//! hierarchy position. This bypasses the need for RAG searches on
//! volatile governance data.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Agent ID not found in registry (should be impossible
//!   during a run), or serialization errors.
//! - **Trace Scope**: `server-rs::agent::runner::metrics_tools`
//!

use super::{AgentRunner, RunContext};
use crate::error::AppError;
use crate::agent::runner::tools::error::ToolExecutionError;

impl AgentRunner {
    /// Handles `get_agent_metrics`: retrieves live financial and identity data from the registry.
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
            serde_json::to_string_pretty(&metrics).map_err(|e| AppError::InternalServerError(format!("Failed to serialize metrics: {}", e)))?
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

// Metadata: [metrics_tools]

// Metadata: [metrics_tools]
