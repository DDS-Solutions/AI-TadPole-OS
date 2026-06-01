//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[skill_error]` in tracing logs.
//! 
//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! Localized SkillError enum for agent capability, manifest validation, and safety sanitization exceptions.
//!

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillError {
    #[error("Validation Error: {0}")]
    ValidationError(String),

    #[error("Recruitment Failure ({role}): {detail}")]
    RecruitmentFailure {
        recipe_id: String,
        role: String,
        detail: String,
    },

    #[error("Sanitization Violation: {0}")]
    SanitizationViolation(String),
}

// Metadata: [skill_error]

// Metadata: [skill_error]
