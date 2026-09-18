//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / error
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_runner_error_display_strings`, `test_runner_error_metadata_status_codes`, `test_runner_error_type_slugs`

use crate::error::{ErrorMetadata, HasErrorMetadata, Severity};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Budget Exhausted: {0}")]
    BudgetExhausted(String),

    #[error("Recursion Blocked: {0}")]
    RecursionBlocked(String),

    #[error("Sentinel Gate Failure: {0}")]
    SentinelGate(String),

    #[error("Monologue Compression Failure: {0}")]
    Compression(String),
}

impl HasErrorMetadata for RunnerError {
    fn get_metadata(&self) -> ErrorMetadata {
        let severity = match self {
            RunnerError::BudgetExhausted(_) => Severity::Error,
            RunnerError::RecursionBlocked(_) => Severity::Critical,
            RunnerError::SentinelGate(_) => Severity::Critical,
            RunnerError::Compression(_) => Severity::Critical,
        };
        let status_code = match self {
            RunnerError::BudgetExhausted(_) => axum::http::StatusCode::PAYMENT_REQUIRED,
            // 508 LOOP_DETECTED represents swarm recursion depth violations
            RunnerError::RecursionBlocked(_) => axum::http::StatusCode::LOOP_DETECTED,
            RunnerError::SentinelGate(_) => axum::http::StatusCode::FORBIDDEN,
            RunnerError::Compression(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        let type_slug = match self {
            RunnerError::BudgetExhausted(_) => "budget-exhausted",
            RunnerError::RecursionBlocked(_) => "recursion-blocked",
            RunnerError::SentinelGate(_) => "sentinel-gate-failure",
            RunnerError::Compression(_) => "compression-error",
        }
        .to_string();

        let resolved_code = type_slug.to_uppercase().replace('-', "_");

        ErrorMetadata {
            status_code,
            type_slug,
            help_link: None,
            error_code: Some(resolved_code),
            severity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::HasErrorMetadata;

    #[test]
    fn test_runner_error_display_strings() {
        assert_eq!(
            RunnerError::BudgetExhausted("mission-1".to_string()).to_string(),
            "Budget Exhausted: mission-1"
        );
        assert_eq!(
            RunnerError::RecursionBlocked("depth 5".to_string()).to_string(),
            "Recursion Blocked: depth 5"
        );
        assert_eq!(
            RunnerError::SentinelGate("denied".to_string()).to_string(),
            "Sentinel Gate Failure: denied"
        );
        assert_eq!(
            RunnerError::Compression("oom".to_string()).to_string(),
            "Monologue Compression Failure: oom"
        );
    }

    #[test]
    fn test_runner_error_metadata_status_codes() {
        let budget = RunnerError::BudgetExhausted("x".to_string());
        assert_eq!(
            budget.get_metadata().status_code,
            axum::http::StatusCode::PAYMENT_REQUIRED
        );

        let recursion = RunnerError::RecursionBlocked("x".to_string());
        assert_eq!(
            recursion.get_metadata().status_code,
            axum::http::StatusCode::LOOP_DETECTED
        );

        let sentinel = RunnerError::SentinelGate("x".to_string());
        assert_eq!(
            sentinel.get_metadata().status_code,
            axum::http::StatusCode::FORBIDDEN
        );

        let compression = RunnerError::Compression("x".to_string());
        assert_eq!(
            compression.get_metadata().status_code,
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_runner_error_type_slugs() {
        let m = RunnerError::BudgetExhausted("x".to_string()).get_metadata();
        assert_eq!(m.type_slug, "budget-exhausted");
        assert_eq!(m.error_code, Some("BUDGET_EXHAUSTED".to_string()));

        let m = RunnerError::RecursionBlocked("x".to_string()).get_metadata();
        assert_eq!(m.type_slug, "recursion-blocked");

        let m = RunnerError::SentinelGate("x".to_string()).get_metadata();
        assert_eq!(m.type_slug, "sentinel-gate-failure");

        let m = RunnerError::Compression("x".to_string()).get_metadata();
        assert_eq!(m.type_slug, "compression-error");
    }
}
