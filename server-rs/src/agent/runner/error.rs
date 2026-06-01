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

// Metadata: [runner_error]

// Metadata: [error]
