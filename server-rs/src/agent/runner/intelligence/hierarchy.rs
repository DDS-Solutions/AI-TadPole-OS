//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Hierarchy**: Decouples role-based authority levels (CEO, COO, Alpha) and provides
//! a reasoning turn guard.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Authority level mismatches or reasoning turn state leaks.
//!

use std::sync::Arc;
use crate::agent::constants::*;

/// RAII Guard to ensure reasoning turn state is always reset in the registry.
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
        if agent_id == AGENT_CEO || role.to_lowercase().contains("ceo") {
            "CEO (Strategic Intelligence Lead)"
        } else if agent_id == AGENT_COO || role.to_lowercase().contains("coo") {
            "COO (Operations Director)"
        } else {
            "ALPHA NODE (Swarm Mission Commander)"
        }
    } else {
        "AGENT (Task Specialist)"
    }
}
