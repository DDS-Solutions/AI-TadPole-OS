//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / hierarchy
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::constants::*;
use std::sync::Arc;

/// RAII Guard to ensure reasoning turn state is always reset in the registry upon completion or panic.
pub struct ReasoningTurnGuard {
    agent_id: String,
    state: Arc<crate::state::AppState>,
}

impl ReasoningTurnGuard {
    pub fn new(agent_id: String, state: Arc<crate::state::AppState>) -> Self {
        Self { agent_id, state }
    }
}

impl Drop for ReasoningTurnGuard {
    fn drop(&mut self) {
        if let Some(mut entry) = self.state.registry.agents.get_mut(&self.agent_id) {
            entry.value_mut().state.current_reasoning_turn = 0;
        }
    }
}

/// Pure function for hierarchy label resolution (Fix 14).
/// Extracted from execute_intelligence_loop for isolated testability.
pub fn resolve_hierarchy_label(agent_id: &str, role: &str) -> &'static str {
    if crate::agent::runner::service_traits::IdentityService::is_orchestrator(agent_id) {
        let role_lower = role.to_lowercase();
        if agent_id == AGENT_CEO || role_lower.contains("ceo") {
            "CEO (Strategic Intelligence Lead)"
        } else if agent_id == AGENT_COO || role_lower.contains("coo") {
            "COO (Operations Director)"
        } else {
            "ALPHA NODE (Swarm Mission Commander)"
        }
    } else {
        "AGENT (Task Specialist)"
    }
}
