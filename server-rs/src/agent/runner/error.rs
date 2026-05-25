/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * Localized RunnerError enum for agent execution and reasoning loop exceptions.
 */

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
