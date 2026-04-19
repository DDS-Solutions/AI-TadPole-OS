//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **Unified Error Engine (AppError)**: Orchestrates the failure logic 
//! across the swarm runner, database, and HTTP layers. Features 
//! **RFC 9457 (Problem Details)** compliance via `IntoResponse`.
//! This is the single source of truth for error reporting in the 
//! Tadpole OS engine. Use the `?` operator to propagate errors 
//! from any layer to the HTTP surface (ERR-03).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Incorrect HTTP status mapping for domain errors.
//! - **Trace Scope**: `server-rs::error`

use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use thiserror::Error;
use serde::Serialize;

/// RFC 9457 (Problem Details for HTTP APIs) compliant error structure.
#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub type_uri: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: Option<String>,
    pub error_code: Option<String>,
    pub help_link: Option<String>,
}

impl ProblemDetails {
    /// Creates a new ProblemDetails response compatible with axum.
    pub fn new(status: StatusCode, title: &str, detail: String) -> (StatusCode, Json<Self>) {
        let slug = title.to_lowercase().replace(' ', "-");
        let scrubbed_detail = crate::utils::security::redact_secrets(&detail);
        (
            status,
            Json(Self {
                type_uri: format!("https://tadpole.os/errors/{}", slug),
                title: title.to_string(),
                status: status.as_u16(),
                detail: scrubbed_detail,
                instance: None,
                error_code: Some(slug.to_uppercase()),
                help_link: None,
            }),
        )
    }
}

/// ### 🧬 Protocol: AppError
/// Unified application error enumeration for the Sovereign Engine. 
/// Variants are mapped to RFC 9457 types via the IntoResponse implementation.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Validation Error: {0}")]
    ValidationError(String),

    #[error("Domain Error ({code}): {detail}")]
    DomainError {
        code: String,
        detail: String,
        help_link: Option<String>,
    },

    #[error("Budget Exhausted: {0}")]
    BudgetExhausted(String),

    #[error("Infrastructure Failure ({provider_id}): {detail}")]
    InfrastructureError {
        provider_id: String,
        detail: String,
        help_link: Option<String>,
    },

    #[error("Quantization Fallback ({model_id}): {detail}")]
    QuantizationFallback {
        model_id: String,
        suggested_quant: String,
        detail: String,
    },

    #[error("Recruitment Failure ({role}): {detail}")]
    RecruitmentFailure {
        recipe_id: String,
        role: String,
        detail: String,
    },

    #[error("Sanitization Violation: {0}")]
    SanitizationViolation(String),

    #[error("Recursion Blocked: {0}")]
    RecursionBlocked(String),

    #[error("Not Implemented: {0}")]
    NotImplemented(String),

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl AppError {
    /// Maps the error variant to a standard HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) | AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) | AppError::SanitizationViolation(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BudgetExhausted(_) => StatusCode::PAYMENT_REQUIRED,
            AppError::RecruitmentFailure { .. } => StatusCode::SERVICE_UNAVAILABLE,
            AppError::InfrastructureError { .. } => StatusCode::BAD_GATEWAY,
            AppError::QuantizationFallback { .. } => StatusCode::INSUFFICIENT_STORAGE,
            AppError::RecursionBlocked(_) => StatusCode::LOOP_DETECTED,
            AppError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns a machine-readable slug for the error type.
    pub fn type_slug(&self) -> String {
        match self {
            AppError::BadRequest(_) => "bad-request".to_string(),
            AppError::Unauthorized(_) => "unauthorized".to_string(),
            AppError::Forbidden(_) => "forbidden".to_string(),
            AppError::NotFound(_) => "not-found".to_string(),
            AppError::ValidationError(_) => "validation-error".to_string(),
            AppError::DomainError { code, .. } => format!("domain:{}", code).to_lowercase(),
            AppError::BudgetExhausted(_) => "budget-exhausted".to_string(),
            AppError::InfrastructureError { provider_id, .. } => format!("infra:{}", provider_id).to_lowercase(),
            AppError::QuantizationFallback { .. } => "resource-exhaustion".to_string(),
            AppError::RecruitmentFailure { role, .. } => format!("recruitment:{}", role).to_lowercase(),
            AppError::SanitizationViolation(_) => "sanitization-violation".to_string(),
            AppError::RecursionBlocked(_) => "recursion-blocked".to_string(),
            AppError::NotImplemented(_) => "not-implemented".to_string(),
            AppError::InternalServerError(_) | AppError::Anyhow(_) | AppError::Sqlx(_) | AppError::Io(_) => "internal-error".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let slug = self.type_slug();
        let detail = format!("{}", self);
        
        let (help_link, error_code) = match &self {
            AppError::DomainError { help_link, code, .. } => (help_link.clone(), Some(code.clone())),
            AppError::InfrastructureError { help_link, provider_id, .. } => (help_link.clone(), Some(provider_id.clone())),
            AppError::QuantizationFallback { suggested_quant, model_id, .. } => (
                Some(format!("https://docs.tadpole.os/troubleshooting/quantization#{}", suggested_quant)),
                Some(format!("OOM_FALLBACK:{}", model_id))
            ),
            AppError::RecruitmentFailure { recipe_id, role, .. } => (
                None,
                Some(format!("RECRUITMENT_FAILED:{}:{}", recipe_id, role))
            ),
            _ => (None, Some(slug.to_uppercase())),
        };

        // SEC-03: Redact secrets from the detail
        let safe_detail = crate::utils::security::redact_secrets(&detail);

        let body = Json(ProblemDetails {
            type_uri: format!("https://tadpole.os/errors/{}", slug),
            title: slug.replace(['-', ':'], " ").to_uppercase(),
            status: status.as_u16(),
            detail: safe_detail,
            instance: None,
            error_code,
            help_link,
        });

        (status, body).into_response()
    }
}
