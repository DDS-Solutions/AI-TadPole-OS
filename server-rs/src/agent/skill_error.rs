/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * Localized SkillError enum for agent capability, manifest validation, and safety sanitization exceptions.
 */

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
