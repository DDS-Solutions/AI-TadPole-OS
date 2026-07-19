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

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Helper function to safely truncate a string to a maximum number of bytes
/// without breaking UTF-8 char boundaries.
fn safe_truncate_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() > max_bytes {
        let mut end = max_bytes.saturating_sub(3);
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        let mut truncated = s[..end].to_string();
        truncated.push_str("...");
        truncated
    } else {
        s.to_string()
    }
}

/// Helper function to sanitize HTML characters to prevent XSS payloads.
fn sanitize_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Helper function to validate if a help_link is public and safe.
fn is_public_and_safe_link(url: &str) -> bool {
    url.starts_with("https://tadpole.os/")
        || url.starts_with("https://docs.tadpole.os/")
        || url.starts_with("https://console.cloud.google.com/")
}

/// Helper function to format a list of errors.
fn format_errors(errors: &[AppError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

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
    pub severity: String, // NEW: CRITICAL, ERROR, WARNING
}

impl ProblemDetails {
    /// Centralized builder for ProblemDetails that enforces truncation, HTML sanitization,
    /// secret redaction, and internal error masking.
    pub fn build(
        status: StatusCode,
        slug: &str,
        detail: String,
        help_link: Option<String>,
        severity: &str,
        error_code: Option<String>,
        title_override: Option<String>,
    ) -> Self {
        let is_server_err = status.is_server_error();
        let detail_to_process = if is_server_err {
            "An internal server error occurred. Please check system logs.".to_string()
        } else {
            detail
        };

        let truncated = safe_truncate_bytes(&detail_to_process, 2048);
        let sanitized = sanitize_html(&truncated);
        let scrubbed = crate::utils::security::redact_secrets(&sanitized);

        let encoded_slug = urlencoding::encode(slug).into_owned();
        let title = title_override.unwrap_or_else(|| slug.replace(['-', ':'], " ").to_uppercase());

        let safe_help_link = help_link.filter(|link| is_public_and_safe_link(link));

        Self {
            type_uri: format!("https://tadpole.os/errors/{}", encoded_slug),
            title,
            status: status.as_u16(),
            detail: scrubbed,
            instance: None,
            error_code: error_code.or_else(|| Some(slug.to_uppercase())),
            help_link: safe_help_link,
            severity: severity.to_string(),
        }
    }

    /// Creates a new ProblemDetails response compatible with axum.
    pub fn new(status: StatusCode, title: &str, detail: String) -> (StatusCode, Json<Self>) {
        let slug = title.to_lowercase().replace(' ', "-");
        let severity = if status.is_server_error()
            || status == StatusCode::FORBIDDEN
            || status == StatusCode::UNAUTHORIZED
        {
            "CRITICAL"
        } else {
            "ERROR"
        };

        let pd = Self::build(
            status,
            &slug,
            detail,
            None,
            severity,
            Some(slug.to_uppercase()),
            Some(title.to_string()),
        );

        (status, Json(pd))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DomainCode {
    InsufficientQuota,
    InvalidConfiguration,
    AgentDeactivated,
    InvalidStateTransition,
    OversightViolation,
    TaskConflict,
    SystemError,
}

impl std::fmt::Display for DomainCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DomainCode::InsufficientQuota => "insufficient-quota",
            DomainCode::InvalidConfiguration => "invalid-configuration",
            DomainCode::AgentDeactivated => "agent-deactivated",
            DomainCode::InvalidStateTransition => "invalid-state-transition",
            DomainCode::OversightViolation => "oversight-violation",
            DomainCode::TaskConflict => "task-conflict",
            DomainCode::SystemError => "system-error",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderId {
    Anthropic,
    OpenAi,
    Gemini,
    Groq,
    Mcp,
    Audio,
    Runner,
    System,
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProviderId::Anthropic => "anthropic",
            ProviderId::OpenAi => "openai",
            ProviderId::Gemini => "gemini",
            ProviderId::Groq => "groq",
            ProviderId::Mcp => "mcp",
            ProviderId::Audio => "audio",
            ProviderId::Runner => "runner",
            ProviderId::System => "system",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InfrastructureErrorKind {
    Timeout,
    NetworkError,
    RateLimit,
    ApiError,
    Other,
}

/// Trait to extract error metadata for specific domain and infrastructure errors.
pub trait HasErrorMetadata {
    fn get_metadata(&self) -> ErrorMetadata;
}

/// ### 🧬 Protocol: AppError
/// Unified application error enumeration for the Sovereign Engine.
/// Variants are mapped to RFC 9457 types via the IntoResponse implementation.
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Runner(#[from] crate::agent::runner::error::RunnerError),

    #[error(transparent)]
    Skill(#[from] crate::agent::skill_error::SkillError),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Domain Error ({code}): {detail}")]
    DomainError {
        code: DomainCode,
        detail: String,
        help_link: Option<String>,
    },

    #[error("Infrastructure Failure ({provider_id} - {kind:?}): {detail}")]
    InfrastructureError {
        provider_id: ProviderId,
        kind: InfrastructureErrorKind,
        detail: String,
        help_link: Option<String>,
    },

    #[error("Quantization Fallback ({model_id}): {detail}")]
    QuantizationFallback {
        model_id: String,
        suggested_quant: String,
        detail: String,
    },

    #[error("Not Implemented: {0}")]
    NotImplemented(String),

    #[error("Rate Limit Exceeded: {0}")]
    RateLimit(String),

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Degraded State: {0}")]
    DegradedState(String),

    #[error("Ollama service is offline or unreachable: {0}")]
    OllamaOffline(String),

    #[error("Multiple errors occurred: {}", format_errors(.0))]
    MultiError(Vec<AppError>),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),

    #[error("Graph Error: {0}")]
    Graph(#[from] crate::intelligence::graph::GraphError),

    /// A file path was found to lie outside the workspace root boundary.
    /// Used by the intelligence pipeline to reject symlink traversal attempts
    /// without aborting the entire build — callers should skip the file and continue.
    #[error("Security Boundary Violation: {0}")]
    SecurityBoundaryViolation(String),
}

#[derive(Debug, Clone)]
pub struct ErrorMetadata {
    pub status_code: StatusCode,
    pub type_slug: String,
    pub help_link: Option<String>,
    pub error_code: Option<String>,
    pub severity: &'static str,
}

impl HasErrorMetadata for sqlx::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        ErrorMetadata {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            type_slug: "internal-error".to_string(),
            help_link: None,
            error_code: Some("INTERNAL-ERROR".to_string()),
            severity: "CRITICAL",
        }
    }
}

impl HasErrorMetadata for std::io::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        ErrorMetadata {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            type_slug: "internal-error".to_string(),
            help_link: None,
            error_code: Some("INTERNAL-ERROR".to_string()),
            severity: "CRITICAL",
        }
    }
}

impl HasErrorMetadata for reqwest::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        ErrorMetadata {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            type_slug: "internal-error".to_string(),
            help_link: None,
            error_code: Some("INTERNAL-ERROR".to_string()),
            severity: "CRITICAL",
        }
    }
}

impl HasErrorMetadata for serde_json::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        ErrorMetadata {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            type_slug: "internal-error".to_string(),
            help_link: None,
            error_code: Some("INTERNAL-ERROR".to_string()),
            severity: "CRITICAL",
        }
    }
}

impl HasErrorMetadata for walkdir::Error {
    fn get_metadata(&self) -> ErrorMetadata {
        ErrorMetadata {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            type_slug: "internal-error".to_string(),
            help_link: None,
            error_code: Some("INTERNAL-ERROR".to_string()),
            severity: "CRITICAL",
        }
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

        ErrorMetadata {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            type_slug: "internal-error".to_string(),
            help_link: None,
            error_code: Some("INTERNAL-ERROR".to_string()),
            severity: "CRITICAL",
        }
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
                error_code: Some("GRAPH-ERROR".to_string()),
                severity: "ERROR",
            },
            AppError::Anyhow(e) => e.get_metadata(),

            AppError::BadRequest(_) => ErrorMetadata {
                status_code: StatusCode::BAD_REQUEST,
                type_slug: "bad-request".to_string(),
                help_link: None,
                error_code: Some("BAD_REQUEST".to_string()),
                severity: "ERROR",
            },
            AppError::Unauthorized(_) => ErrorMetadata {
                status_code: StatusCode::UNAUTHORIZED,
                type_slug: "unauthorized".to_string(),
                help_link: None,
                error_code: Some("UNAUTHORIZED".to_string()),
                severity: "CRITICAL",
            },
            AppError::Forbidden(_) => ErrorMetadata {
                status_code: StatusCode::FORBIDDEN,
                type_slug: "forbidden".to_string(),
                help_link: None,
                error_code: Some("FORBIDDEN".to_string()),
                severity: "CRITICAL",
            },
            AppError::NotFound(_) => ErrorMetadata {
                status_code: StatusCode::NOT_FOUND,
                type_slug: "not-found".to_string(),
                help_link: None,
                error_code: Some("NOT_FOUND".to_string()),
                severity: "ERROR",
            },
            AppError::DomainError {
                code,
                detail: _,
                help_link: hl,
            } => {
                let type_slug = format!("domain:{}", code).to_lowercase();
                let error_code = code.to_string().to_uppercase();
                let status_code = match code {
                    DomainCode::InsufficientQuota => StatusCode::PAYMENT_REQUIRED,
                    DomainCode::InvalidConfiguration => StatusCode::BAD_REQUEST,
                    DomainCode::AgentDeactivated => StatusCode::FORBIDDEN,
                    DomainCode::InvalidStateTransition => StatusCode::BAD_REQUEST,
                    DomainCode::OversightViolation => StatusCode::FORBIDDEN,
                    DomainCode::TaskConflict => StatusCode::CONFLICT,
                    DomainCode::SystemError => StatusCode::INTERNAL_SERVER_ERROR,
                };
                ErrorMetadata {
                    status_code,
                    type_slug,
                    help_link: hl.clone(),
                    error_code: Some(error_code),
                    severity: "ERROR",
                }
            }
            AppError::InfrastructureError {
                provider_id,
                kind: _,
                detail: _,
                help_link: hl,
            } => {
                let type_slug = format!("infra:{}", provider_id).to_lowercase();
                let error_code = provider_id.to_string().to_uppercase();
                ErrorMetadata {
                    status_code: StatusCode::BAD_GATEWAY,
                    type_slug,
                    help_link: hl.clone(),
                    error_code: Some(error_code),
                    severity: "CRITICAL",
                }
            }
            AppError::QuantizationFallback {
                model_id,
                suggested_quant,
                detail: _,
            } => ErrorMetadata {
                status_code: StatusCode::INSUFFICIENT_STORAGE,
                type_slug: "resource-exhaustion".to_string(),
                help_link: Some(format!(
                    "https://docs.tadpole.os/troubleshooting/quantization#{}",
                    suggested_quant
                )),
                error_code: Some(format!("OOM_FALLBACK:{}", model_id)),
                severity: "ERROR",
            },
            AppError::NotImplemented(_) => ErrorMetadata {
                status_code: StatusCode::NOT_IMPLEMENTED,
                type_slug: "not-implemented".to_string(),
                help_link: None,
                error_code: Some("NOT_IMPLEMENTED".to_string()),
                severity: "ERROR",
            },
            AppError::RateLimit(_) => ErrorMetadata {
                status_code: StatusCode::TOO_MANY_REQUESTS,
                type_slug: "rate-limit".to_string(),
                help_link: None,
                error_code: Some("RATE_LIMIT".to_string()),
                severity: "ERROR",
            },
            AppError::InternalServerError(_) => ErrorMetadata {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                type_slug: "internal-error".to_string(),
                help_link: None,
                error_code: Some("INTERNAL-ERROR".to_string()),
                severity: "CRITICAL",
            },
            AppError::Conflict(_) => ErrorMetadata {
                status_code: StatusCode::CONFLICT,
                type_slug: "conflict".to_string(),
                help_link: None,
                error_code: Some("CONFLICT".to_string()),
                severity: "ERROR",
            },
            AppError::DegradedState(_) => ErrorMetadata {
                status_code: StatusCode::SERVICE_UNAVAILABLE,
                type_slug: "degraded-state".to_string(),
                help_link: None,
                error_code: Some("DEGRADED-STATE".to_string()),
                severity: "CRITICAL",
            },
            AppError::MultiError(errors) => {
                let status_code = errors
                    .first()
                    .map(|e| e.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                let severity = if errors
                    .iter()
                    .any(|e| e.resolve_metadata().severity == "CRITICAL")
                {
                    "CRITICAL"
                } else {
                    "ERROR"
                };
                ErrorMetadata {
                    status_code,
                    type_slug: "multi-error".to_string(),
                    help_link: None,
                    error_code: Some("MULTI-ERROR".to_string()),
                    severity,
                }
            }
            AppError::SecurityBoundaryViolation(_) => ErrorMetadata {
                status_code: StatusCode::FORBIDDEN,
                type_slug: "security-boundary-violation".to_string(),
                help_link: None,
                error_code: Some("SECURITY-BOUNDARY-VIOLATION".to_string()),
                severity: "CRITICAL",
            },
            AppError::OllamaOffline(_) => ErrorMetadata {
                status_code: StatusCode::SERVICE_UNAVAILABLE, // 503
                type_slug: "ollama-offline".to_string(),
                help_link: None,
                error_code: Some("OLLAMA_OFFLINE".to_string()),
                severity: "CRITICAL",
            },
        }
    }
}

impl AppError {
    /// Creates a domain error, storing its raw detail payload and validating its help link immediately.
    pub fn new_domain_error(code: DomainCode, detail: String, help_link: Option<String>) -> Self {
        let safe_help = help_link.filter(|link| is_public_and_safe_link(link));
        AppError::DomainError {
            code,
            detail,
            help_link: safe_help,
        }
    }

    /// Creates an infrastructure error, storing its raw detail payload and validating its help link immediately.
    pub fn new_infrastructure_error(
        provider_id: ProviderId,
        kind: InfrastructureErrorKind,
        detail: String,
        help_link: Option<String>,
    ) -> Self {
        let safe_help = help_link.filter(|link| is_public_and_safe_link(link));
        AppError::InfrastructureError {
            provider_id,
            kind,
            detail,
            help_link: safe_help,
        }
    }

    /// Consolidates status, slug, help link, error code, and severity mappings.
    /// Excludes wildcard matches to guarantee exhaustiveness checking.
    pub fn resolve_metadata(&self) -> ErrorMetadata {
        self.get_metadata()
    }

    /// Maps the error variant to a standard HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        self.resolve_metadata().status_code
    }

    /// Returns a machine-readable slug for the error type.
    pub fn type_slug(&self) -> String {
        self.resolve_metadata().type_slug
    }

    /// Determines if the error is transient and safe to retry.
    pub fn is_retryable(&self) -> bool {
        match self {
            AppError::RateLimit(_) => true,
            AppError::Reqwest(e) => {
                // Retry on timeouts, connection failures, or premature closures (common in local LLM bursts)
                // e.is_request() includes connection resets during payload delivery
                e.is_timeout() || e.is_connect() || e.is_request() || e.is_body()
            }
            _ => false,
        }
    }

    /// Checks if the error is a rate limit error (either explicitly via RateLimit variant,
    /// HTTP 429 status code in Reqwest, or InfrastructureError stating rate limits).
    pub fn is_rate_limit(&self) -> bool {
        match self {
            AppError::RateLimit(_) => true,
            AppError::Reqwest(e) => e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS),
            AppError::InfrastructureError { kind, .. } => {
                *kind == InfrastructureErrorKind::RateLimit
            }
            _ => false,
        }
    }

    /// Checks if the error is a network or timeout error (e.g. Reqwest timeout/connection error
    /// or InfrastructureError mentioning timeout/connect).
    pub fn is_network_timeout(&self) -> bool {
        match self {
            AppError::Reqwest(e) => e.is_timeout() || e.is_connect(),
            AppError::InfrastructureError { kind, .. } => {
                *kind == InfrastructureErrorKind::Timeout
                    || *kind == InfrastructureErrorKind::NetworkError
            }
            AppError::OllamaOffline(_) => true,
            _ => false,
        }
    }

    /// Classifies errors into Transient (retryable) vs Permanent.
    pub fn error_class(&self) -> ErrorClass {
        if self.is_retryable() {
            return ErrorClass::Transient;
        }
        match self {
            AppError::InfrastructureError { kind, .. } => match kind {
                InfrastructureErrorKind::Timeout
                | InfrastructureErrorKind::NetworkError
                | InfrastructureErrorKind::RateLimit => ErrorClass::Transient,
                _ => ErrorClass::Permanent,
            },
            _ => ErrorClass::Permanent,
        }
    }
}

/// Classification of errors for retrying operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ErrorClass {
    Transient,
    Permanent,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let metadata = self.resolve_metadata();
        let status = metadata.status_code;
        let detail = self.to_string();

        let pd = ProblemDetails::build(
            status,
            &metadata.type_slug,
            detail,
            metadata.help_link,
            metadata.severity,
            metadata.error_code,
            None,
        );

        let mut res = (status, Json(pd)).into_response();
        res.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/problem+json"),
        );
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn test_error_status_mapping() {
        assert_eq!(
            AppError::BadRequest("bad".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Unauthorized("auth".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Forbidden("no".to_string()).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::NotFound("lost".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::RateLimit("slow".to_string()).status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::InternalServerError("boom".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        // DomainCode status mapping tests
        assert_eq!(
            AppError::new_domain_error(DomainCode::InsufficientQuota, "limit".to_string(), None)
                .status_code(),
            StatusCode::PAYMENT_REQUIRED
        );
        assert_eq!(
            AppError::new_domain_error(DomainCode::InvalidConfiguration, "cfg".to_string(), None)
                .status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::new_domain_error(DomainCode::AgentDeactivated, "deact".to_string(), None)
                .status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::new_domain_error(
                DomainCode::InvalidStateTransition,
                "transition".to_string(),
                None
            )
            .status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::new_domain_error(
                DomainCode::OversightViolation,
                "oversight".to_string(),
                None
            )
            .status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::new_domain_error(DomainCode::TaskConflict, "conflict".to_string(), None)
                .status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            AppError::new_domain_error(DomainCode::SystemError, "sys".to_string(), None)
                .status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::OllamaOffline("offline".to_string()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn test_error_slug_generation() {
        assert_eq!(
            AppError::BadRequest("bad".to_string()).type_slug(),
            "bad-request"
        );
        assert_eq!(
            AppError::DomainError {
                code: DomainCode::SystemError,
                detail: "d".to_string(),
                help_link: None
            }
            .type_slug(),
            "domain:system-error"
        );
        assert_eq!(
            AppError::InfrastructureError {
                provider_id: ProviderId::Gemini,
                kind: InfrastructureErrorKind::ApiError,
                detail: "d".to_string(),
                help_link: None
            }
            .type_slug(),
            "infra:gemini"
        );
        assert_eq!(
            AppError::OllamaOffline("offline".to_string()).type_slug(),
            "ollama-offline"
        );
        assert_eq!(
            AppError::OllamaOffline("offline".to_string()).resolve_metadata().error_code,
            Some("OLLAMA_OFFLINE".to_string())
        );
    }

    #[test]
    fn test_percent_encoding_slugs() {
        let (status, json_pd) = ProblemDetails::new(
            StatusCode::BAD_REQUEST,
            "Bad Request with space & special characters @#$",
            "detail".to_string(),
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json_pd.0.type_uri.contains("https://tadpole.os/errors/"));
        assert!(json_pd.0.type_uri.contains("%26")); // '&'
        assert!(json_pd.0.type_uri.contains("%40")); // '@'
        assert!(json_pd.0.type_uri.contains("%23")); // '#'
    }

    #[test]
    fn test_max_length_truncation() {
        let long_detail = "a".repeat(3000);
        let (status, json_pd) =
            ProblemDetails::new(StatusCode::BAD_REQUEST, "Bad Request", long_detail.clone());
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json_pd.0.detail.len(), 2048);
        assert!(json_pd.0.detail.ends_with("..."));

        let emoji_detail = "🌟".repeat(1000); // 4000 bytes
        let (_, emoji_pd) =
            ProblemDetails::new(StatusCode::BAD_REQUEST, "Bad Request", emoji_detail);
        assert!(emoji_pd.0.detail.len() <= 2048);
        assert!(emoji_pd.0.detail.ends_with("..."));

        // Boundary tests for short max_bytes to prevent saturating_sub underflow panics
        assert_eq!(safe_truncate_bytes("hello", 0), "...");
        assert_eq!(safe_truncate_bytes("hello", 1), "...");
        assert_eq!(safe_truncate_bytes("hello", 2), "...");
        assert_eq!(safe_truncate_bytes("hello", 3), "...");
        assert_eq!(safe_truncate_bytes("hello", 4), "h...");
        assert_eq!(safe_truncate_bytes("hello", 5), "hello");
    }

    #[tokio::test]
    async fn test_error_redaction_in_response() {
        // Create an error that contains a sensitive API key in the detail
        let error =
            AppError::BadRequest("Failed with key sk-1234567890abcdef1234567890abcdef".to_string());

        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Extract body (this is a bit involved in axum but we can verify the detail redaction)
        // For unit testing purposes, we can just verify that ProblemDetails::new redacts.
        let (status, json_pd) = ProblemDetails::new(
            StatusCode::BAD_REQUEST,
            "Bad Request",
            "key sk-1234567890abcdef1234567890abcdef".to_string(),
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json_pd.0.detail.contains("[REDACTED]"));
        assert!(!json_pd.0.detail.contains("sk-1234567890"));
        assert_eq!(json_pd.0.severity, "ERROR");
    }

    #[test]
    fn test_problem_details_structure() {
        let (status, json_pd) = ProblemDetails::new(
            StatusCode::NOT_FOUND,
            "Not Found",
            "Item not found".to_string(),
        );
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json_pd.0.status, 404);
        assert_eq!(json_pd.0.title, "Not Found");
        assert_eq!(json_pd.0.type_uri, "https://tadpole.os/errors/not-found");
        assert_eq!(json_pd.0.error_code, Some("NOT-FOUND".to_string()));
        assert_eq!(json_pd.0.severity, "ERROR");
    }

    #[tokio::test]
    async fn test_is_retryable() {
        // Happy Path: RateLimit
        let err_rate = AppError::RateLimit("exceeded".to_string());
        assert!(err_rate.is_retryable());

        // Failure Path: Forbidden
        let err_forbidden = AppError::Forbidden("no".to_string());
        assert!(!err_forbidden.is_retryable());

        // Failure Path: BadRequest
        let err_bad = AppError::BadRequest("bad".to_string());
        assert!(!err_bad.is_retryable());

        // Reqwest test: Connection error (retryable)
        let client = reqwest::Client::new();
        let err_conn = client
            .get("http://this-domain-does-not-exist.invalid")
            .send()
            .await
            .unwrap_err();
        let app_err_conn = AppError::Reqwest(err_conn);
        assert!(app_err_conn.is_retryable());

        // Edge Case: Reqwest status 404 (non-retryable)
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let mut request = Vec::new();
                while let Ok(n) = stream.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    request.extend_from_slice(&buf[..n]);
                    if request.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let _ = stream
                    .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                    .await;
                let _ = stream.flush().await;
                // Add a small sleep to allow client to parse the headers before closing stream connection
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let _ = stream.shutdown().await;
            }
        });

        let client = reqwest::Client::new();
        let response_err = client
            .get(format!("http://127.0.0.1:{}", port))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap_err();
        let app_err_status = AppError::Reqwest(response_err);
        assert!(!app_err_status.is_retryable());
    }

    #[test]
    fn test_anyhow_metadata_downcast() {
        let runner_err = crate::agent::runner::error::RunnerError::RecursionBlocked(
            "recursion loop".to_string(),
        );
        let anyhow_err: anyhow::Error = runner_err.into();
        let metadata = anyhow_err.get_metadata();
        assert_eq!(metadata.status_code, StatusCode::LOOP_DETECTED);
        assert_eq!(metadata.type_slug, "recursion-blocked");
        assert_eq!(metadata.severity, "CRITICAL");
    }

    #[test]
    fn test_multi_error_formatting() {
        let errs = vec![
            AppError::BadRequest("bad input".to_string()),
            AppError::NotFound("not found item".to_string()),
        ];
        let multi = AppError::MultiError(errs);
        let msg = multi.to_string();
        assert!(msg.contains("Multiple errors occurred"));
        assert!(msg.contains("Bad Request: bad input"));
        assert!(msg.contains("Not Found: not found item"));

        let metadata = multi.resolve_metadata();
        assert_eq!(metadata.status_code, StatusCode::BAD_REQUEST);
        assert_eq!(metadata.type_slug, "multi-error");
        assert_eq!(metadata.severity, "ERROR");
    }

    #[test]
    fn test_html_sanitization_in_response() {
        let err = AppError::BadRequest("<script>alert(1)</script> & check".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let (_, json_pd) = ProblemDetails::new(
            StatusCode::BAD_REQUEST,
            "Bad Request",
            "<script>alert(1)</script> & check".to_string(),
        );
        assert!(!json_pd.0.detail.contains("<script>"));
        assert!(json_pd.0.detail.contains("&lt;script&gt;"));
        assert!(json_pd.0.detail.contains("&amp;"));
    }

    #[test]
    fn test_help_link_validation() {
        // Safe link
        let err_safe = AppError::new_infrastructure_error(
            ProviderId::Gemini,
            InfrastructureErrorKind::ApiError,
            "error".to_string(),
            Some("https://console.cloud.google.com/billing".to_string()),
        );
        let meta_safe = err_safe.resolve_metadata();
        assert_eq!(
            meta_safe.help_link,
            Some("https://console.cloud.google.com/billing".to_string())
        );

        // Unsafe link (internal, not in whitelist)
        let err_unsafe = AppError::new_infrastructure_error(
            ProviderId::Gemini,
            InfrastructureErrorKind::ApiError,
            "error".to_string(),
            Some("https://internal.tadpole.os/restricted-docs".to_string()),
        );
        let meta_unsafe = err_unsafe.resolve_metadata();
        assert_eq!(meta_unsafe.help_link, None);
    }

    #[test]
    fn test_constructor_sanitization() {
        let err = AppError::new_domain_error(
            DomainCode::SystemError,
            "<script>bad</script>".to_string(),
            None,
        );
        if let AppError::DomainError { detail, .. } = &err {
            assert_eq!(detail, "<script>bad</script>");
        } else {
            panic!("Expected AppError::DomainError");
        }
    }
}

// Metadata: [error]
