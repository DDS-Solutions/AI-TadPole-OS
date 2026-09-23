//! @docs ARCHITECTURE:Identity
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits / identity
//! - **Architecture**: `@docs ARCHITECTURE:Identity`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural] [ID-01]` Orchestrator predicate is a non-mutable, hardcoded check — never user-configurable.
//! - `[Structural] [Trust]` `IdentityService::is_orchestrator` is the sole trust boundary gate for elevated roles.
//!
//! ### 🔍 Debugging & Observability
//! - **Witness Tests**: `test_identity_service`

pub struct IdentityService;

impl IdentityService {
    /// Validates if an agent ID maps to a known orchestrator.
    /// Non-mutable, hardcoded check to secure trust boundaries (ID-01).
    pub fn is_orchestrator(agent_id: &str) -> bool {
        matches!(
            agent_id,
            crate::agent::constants::AGENT_CEO
                | crate::agent::constants::AGENT_COO
                | crate::agent::constants::AGENT_ALPHA
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_identity_service() {
        assert!(IdentityService::is_orchestrator(
            crate::agent::constants::AGENT_CEO
        ));
        assert!(IdentityService::is_orchestrator(
            crate::agent::constants::AGENT_COO
        ));
        assert!(IdentityService::is_orchestrator(
            crate::agent::constants::AGENT_ALPHA
        ));
        assert!(!IdentityService::is_orchestrator("agent-specialist"));
        assert!(!IdentityService::is_orchestrator("another-agent"));
    }
}
