//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / metadata
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use super::{AppError, DomainCode, ProviderId};

/// Strongly-typed severity levels for engine errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Critical,
    Error,
    Warning,
}

impl Severity {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Structured metadata extracted from any error for RFC 9457 response building.
#[derive(Debug, Clone)]
pub struct ErrorMetadata {
    pub status_code: StatusCode,
    pub type_slug: String,
    pub help_link: Option<String>,
    pub error_code: Option<String>,
    pub severity: Severity,
}

/// Trait to extract error metadata for specific domain and infrastructure errors.
pub trait HasErrorMetadata {
    fn get_metadata(&self) -> ErrorMetadata;
}

/// Helper function to generate standard Internal Server Error metadata without boilerplate redundancy.
fn internal_error_metadata() -> ErrorMetadata {
    ErrorMetadata {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        type_slug: "internal".to_string(),
        help_link: None,
        error_code: Some("INTERNAL_SERVER_ERROR".to_string()),
        severity: Severity::Critical,
    }
}

/// Helper function to construct domain error metadata.
fn domain_metadata(code: &DomainCode, help_link: &Option<String>) -> ErrorMetadata {
    let type_slug = format!("domain:{}", code).to_lowercase();
    let error_code = format!(
        "DOMAIN_ERROR:{}",
        code.to_string().to_uppercase().replace('-', "_")
    );
    let status_code = match code {
        DomainCode::InsufficientQuota => StatusCode::PAYMENT_REQUIRED,
        DomainCode::InvalidConfiguration => StatusCode::BAD_REQUEST,
        DomainCode::AgentDeactivated => StatusCode::FORBIDDEN,
        DomainCode::InvalidStateTransition => StatusCode::BAD_REQUEST,
        DomainCode::OversightViolation => StatusCode::FORBIDDEN,
        DomainCode::TaskConflict => StatusCode::CONFLICT,
        DomainCode::SystemError => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let severity = if status_code.is_server_error() {
        Severity::Critical
    } else {
        Severity::Error
    };
    ErrorMetadata {
        status_code,
        type_slug,
        help_link: help_link.clone(),
        error_code: Some(error_code),
        severity,
    }
}

/// Helper function to construct infrastructure error metadata.
fn infra_metadata(provider_id: &ProviderId, help_link: &Option<String>) -> ErrorMetadata {
    ErrorMetadata {
        status_code: StatusCode::BAD_GATEWAY,
        type_slug: "infrastructure".to_string(),
        help_link: help_link.clone(),
        error_code: Some(format!(
            "INFRA_ERROR:{}",
            provider_id.to_string().to_uppercase()
        )),
        severity: Severity::Critical,
    }
}

impl HasErrorMetadata for sqlx::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        internal_error_metadata()
    }
}

impl HasErrorMetadata for std::io::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        internal_error_metadata()
    }
}

impl HasErrorMetadata for reqwest::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        internal_error_metadata()
    }
}

impl HasErrorMetadata for serde_json::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        internal_error_metadata()
    }
}

impl HasErrorMetadata for walkdir::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        internal_error_metadata()
    }
}

impl HasErrorMetadata for anyhow::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        if let Some(err) = self.downcast_ref::<AppError>() {
            return err.get_metadata();
        }
        if let Some(err) = self.downcast_ref::<crate::agent::runner::error::RunnerError>() {
            return err.get_metadata();
        }
        if let Some(err) = self.downcast_ref::<crate::agent::skill_error::SkillError>() {
            return err.get_metadata();
        }
        if let Some(err) = self.downcast_ref::<sqlx::Error>() {
            return err.get_metadata();
        }
        if let Some(err) = self.downcast_ref::<std::io::Error>() {
            return err.get_metadata();
        }
        if let Some(err) = self.downcast_ref::<reqwest::Error>() {
            return err.get_metadata();
        }
        if let Some(err) = self.downcast_ref::<serde_json::Error>() {
            return err.get_metadata();
        }
        if let Some(err) = self.downcast_ref::<walkdir::Error>() {
            return err.get_metadata();
        }

        internal_error_metadata()
    }
}

impl HasErrorMetadata for AppError {
    fn get_metadata(&self) -> ErrorMetadata {
        match self {
            AppError::Runner(e) => e.get_metadata(),
            AppError::Skill(e) => e.get_metadata(),
            AppError::Sqlx(e) => e.get_metadata(),
            AppError::Io(e) => e.get_metadata(),
            AppError::Reqwest(e) => e.get_metadata(),
            AppError::Serde(e) => e.get_metadata(),
            AppError::WalkDir(e) => e.get_metadata(),
            AppError::Graph(_e) => ErrorMetadata {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                type_slug: "graph-error".to_string(),
                help_link: None,
                error_code: Some("GRAPH_ERROR".to_string()),
                severity: Severity::Critical,
            },
            AppError::Anyhow(e) => e.get_metadata(),

            AppError::BadRequest(_) => ErrorMetadata {
                status_code: StatusCode::BAD_REQUEST,
                type_slug: "bad-request".to_string(),
                help_link: None,
                error_code: Some("BAD_REQUEST".to_string()),
                severity: Severity::Error,
            },
            AppError::Unauthorized(_) => ErrorMetadata {
                status_code: StatusCode::UNAUTHORIZED,
                type_slug: "unauthorized".to_string(),
                help_link: None,
                error_code: Some("UNAUTHORIZED".to_string()),
                severity: Severity::Critical,
            },
            AppError::Forbidden(_) => ErrorMetadata {
                status_code: StatusCode::FORBIDDEN,
                type_slug: "forbidden".to_string(),
                help_link: None,
                error_code: Some("FORBIDDEN".to_string()),
                severity: Severity::Critical,
            },
            AppError::NotFound(_) => ErrorMetadata {
                status_code: StatusCode::NOT_FOUND,
                type_slug: "not-found".to_string(),
                help_link: None,
                error_code: Some("NOT_FOUND".to_string()),
                severity: Severity::Error,
            },
            AppError::DomainError {
                code,
                help_link: hl,
                ..
            } => domain_metadata(code, hl),
            AppError::InfrastructureError {
                provider_id,
                help_link: hl,
                ..
            } => infra_metadata(provider_id, hl),
            AppError::QuantizationFallback {
                model_id,
                suggested_quant,
                detail: _,
            } => ErrorMetadata {
                status_code: StatusCode::INSUFFICIENT_STORAGE,
                type_slug: "resource-exhaustion".to_string(),
                help_link: Some(format!(
                    "https://docs.tadpole.os/troubleshooting/quantization#{}",
                    urlencoding::encode(suggested_quant)
                )),
                error_code: Some(format!("OOM_QUANTIZATION_FALLBACK:{}", model_id)),
                severity: Severity::Error,
            },
            AppError::NotImplemented(_) => ErrorMetadata {
                status_code: StatusCode::NOT_IMPLEMENTED,
                type_slug: "not-implemented".to_string(),
                help_link: None,
                error_code: Some("NOT_IMPLEMENTED".to_string()),
                severity: Severity::Error,
            },
            AppError::RateLimit(_) => ErrorMetadata {
                status_code: StatusCode::TOO_MANY_REQUESTS,
                type_slug: "rate-limit".to_string(),
                help_link: None,
                error_code: Some("RATE_LIMIT".to_string()),
                severity: Severity::Error,
            },
            AppError::InternalServerError(_) => internal_error_metadata(),
            AppError::Conflict(_) => ErrorMetadata {
                status_code: StatusCode::CONFLICT,
                type_slug: "conflict".to_string(),
                help_link: None,
                error_code: Some("CONFLICT".to_string()),
                severity: Severity::Error,
            },
            AppError::DegradedState(_) => ErrorMetadata {
                status_code: StatusCode::SERVICE_UNAVAILABLE,
                type_slug: "degraded-state".to_string(),
                help_link: None,
                error_code: Some("DEGRADED_STATE".to_string()),
                severity: Severity::Critical,
            },
            AppError::MultiError(errors) => {
                let child_metas: Vec<ErrorMetadata> =
                    errors.iter().map(|e| e.get_metadata()).collect();
                let status_code = child_metas
                    .iter()
                    .map(|m| m.status_code)
                    .find(|s| s.is_server_error())
                    .or_else(|| {
                        // Prioritize 400 Bad Request if present, otherwise default to first error
                        child_metas
                            .iter()
                            .map(|m| m.status_code)
                            .find(|s| *s == StatusCode::BAD_REQUEST)
                            .or_else(|| child_metas.first().map(|m| m.status_code))
                    })
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                let severity = if child_metas.iter().any(|m| m.severity == Severity::Critical) {
                    Severity::Critical
                } else {
                    Severity::Error
                };
                ErrorMetadata {
                    status_code,
                    type_slug: "multi-error".to_string(),
                    help_link: None,
                    error_code: Some("MULTI_ERROR".to_string()),
                    severity,
                }
            }
            AppError::SecurityBoundaryViolation(_) => ErrorMetadata {
                status_code: StatusCode::FORBIDDEN,
                type_slug: "security-boundary-violation".to_string(),
                help_link: None,
                error_code: Some("SECURITY_BOUNDARY_VIOLATION".to_string()),
                severity: Severity::Critical,
            },
            AppError::OllamaOffline(_) => ErrorMetadata {
                status_code: StatusCode::SERVICE_UNAVAILABLE, // 503
                type_slug: "ollama-offline".to_string(),
                help_link: None,
                error_code: Some("OLLAMA_OFFLINE".to_string()),
                severity: Severity::Critical,
            },
            AppError::WorkflowStepFailed { .. } => ErrorMetadata {
                status_code: StatusCode::UNPROCESSABLE_ENTITY, // 422
                type_slug: "workflow-step-failed".to_string(),
                help_link: None,
                error_code: Some("WORKFLOW_STEP_FAILED".to_string()),
                severity: Severity::Warning,
            },
        }
    }
}
