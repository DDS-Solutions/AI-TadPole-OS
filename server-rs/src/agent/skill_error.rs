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
use crate::error::{HasErrorMetadata, ErrorMetadata};

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

impl HasErrorMetadata for SkillError {
    fn get_metadata(&self) -> ErrorMetadata {
        let severity = match self {
            SkillError::ValidationError(_) => "ERROR",
            SkillError::RecruitmentFailure { .. } => "ERROR",
            SkillError::SanitizationViolation(_) => "CRITICAL",
        };
        let status_code = match self {
            SkillError::ValidationError(_) => axum::http::StatusCode::BAD_REQUEST,
            SkillError::RecruitmentFailure { .. } => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            SkillError::SanitizationViolation(_) => axum::http::StatusCode::FORBIDDEN,
        };
        let type_slug = match self {
            SkillError::ValidationError(_) => "validation-error".to_string(),
            SkillError::RecruitmentFailure { role, .. } => format!("recruitment:{}", role).to_lowercase(),
            SkillError::SanitizationViolation(_) => "sanitization-violation".to_string(),
        };
        let error_code = match self {
            SkillError::ValidationError(_) => None,
            SkillError::RecruitmentFailure { recipe_id, role, .. } => {
                Some(format!("RECRUITMENT_FAILED:{}:{}", recipe_id, role))
            }
            SkillError::SanitizationViolation(_) => None,
        };

        let resolved_code = error_code.unwrap_or_else(|| type_slug.to_uppercase());

        ErrorMetadata {
            status_code,
            type_slug,
            help_link: None,
            error_code: Some(resolved_code),
            severity,
        }
    }
}

// Metadata: [skill_error]
