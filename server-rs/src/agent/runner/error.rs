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
//! - **Witness Tests**: none declared

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
