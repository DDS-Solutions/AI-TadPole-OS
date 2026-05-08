//! @docs ARCHITECTURE:Registry
//! Standardized Error System for the Tadpole OS Tooling Layer.
//! Features RecoveryAction metadata to guide autonomous agent self-annealing.

use crate::error::AppError;

/// Defines the recovery path for an autonomous agent.
#[derive(Debug, Clone, serde::Serialize)]
pub enum RecoveryAction {
    /// The agent should wait and retry the exact same call.
    RetryWithBackoff { seconds: u64 },
    /// The agent should refine the input parameters (e.g., path was wrong).
    RefineInput,
    /// The agent should attempt to recruit a specialist with higher authority.
    Escalate,
    /// The mission cannot proceed; fail immediately.
    Abort,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolExecutionError {
    #[allow(dead_code)]
    #[error("Transient Failure: {message}")]
    Transient {
        message: String,
        retry_after: Option<u64>,
    },
    
    #[error("Security Violation: {0}")]
    SecurityBlocked(String),
    
    #[error("Hierarchy Violation: {0}")]
    HierarchyBlocked(String),
    
    #[allow(dead_code)]
    #[error("Parameter Validation Error: {0}")]
    Validation(String),
    
    #[error("Runtime Execution Failed: {0}")]
    ExecutionFailed(String),
    
    #[error(transparent)]
    AppError(#[from] AppError),
}

impl ToolExecutionError {
    /// Determines if the error is recoverable and provides a strategy.
    pub fn recovery_strategy(&self) -> RecoveryAction {
        match self {
            Self::Transient { retry_after, .. } => {
                RecoveryAction::RetryWithBackoff { seconds: retry_after.unwrap_or(1) }
            },
            Self::Validation(_) => RecoveryAction::RefineInput,
            Self::SecurityBlocked(_) | Self::HierarchyBlocked(_) => RecoveryAction::Escalate,
            Self::AppError(e) => match e {
                AppError::RateLimit(_) => RecoveryAction::RetryWithBackoff { seconds: 5 },
                AppError::Io(io_e) if io_e.kind() == std::io::ErrorKind::TimedOut => {
                    RecoveryAction::RetryWithBackoff { seconds: 2 }
                }
                _ => RecoveryAction::Abort,
            },
            _ => RecoveryAction::Abort,
        }
    }

    pub fn is_transient(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryAction::RetryWithBackoff { .. })
    }
}

// Metadata: [error]

