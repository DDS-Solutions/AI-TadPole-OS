//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[error]` in tracing logs.
//!
//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! Localized RunnerError enum for agent execution and reasoning loop exceptions.
//!

use crate::error::{ErrorMetadata, HasErrorMetadata};
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

use crate::error::Severity;

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

// Metadata: [error]
