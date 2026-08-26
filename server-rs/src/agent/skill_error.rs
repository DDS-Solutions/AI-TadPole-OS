//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / skill_error
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
            SkillError::ValidationError(_) => Severity::Error,
            SkillError::RecruitmentFailure { .. } => Severity::Error,
            SkillError::SanitizationViolation(_) => Severity::Critical,
        };
        let status_code = match self {
            SkillError::ValidationError(_) => axum::http::StatusCode::BAD_REQUEST,
            SkillError::RecruitmentFailure { .. } => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            SkillError::SanitizationViolation(_) => axum::http::StatusCode::FORBIDDEN,
        };
        let type_slug = match self {
            SkillError::ValidationError(_) => "validation-error".to_string(),
            SkillError::RecruitmentFailure { role, .. } => {
                format!("recruitment:{}", role).to_lowercase()
            }
            SkillError::SanitizationViolation(_) => "sanitization-violation".to_string(),
        };
        let error_code = match self {
            SkillError::ValidationError(_) => None,
            SkillError::RecruitmentFailure {
                recipe_id, role, ..
            } => Some(format!("RECRUITMENT_FAILED:{}:{}", recipe_id, role)),
            SkillError::SanitizationViolation(_) => None,
        };

        let resolved_code =
            error_code.unwrap_or_else(|| type_slug.to_uppercase().replace('-', "_"));

        ErrorMetadata {
            status_code,
            type_slug,
            help_link: None,
            error_code: Some(resolved_code),
            severity,
        }
    }
}
