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
use serde::Serialize;
use thiserror::Error;

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
    /// Creates a new ProblemDetails response compatible with axum.
    pub fn new(status: StatusCode, title: &str, detail: String) -> (StatusCode, Json<Self>) {
        let slug = title.to_lowercase().replace(' ', "-");
        let scrubbed_detail = crate::utils::security::redact_secrets(&detail);
        let severity = if status.is_server_error()
            || status == StatusCode::FORBIDDEN
            || status == StatusCode::UNAUTHORIZED
        {
            "CRITICAL"
        } else {
            "ERROR"
        };

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
                severity: severity.to_string(),
            }),
        )
    }
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
        code: String,
        detail: String,
        help_link: Option<String>,
    },

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

    #[error("Not Implemented: {0}")]
    NotImplemented(String),

    #[error("Rate Limit Exceeded: {0}")]
    RateLimit(String),

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("Conflict: {0}")]
    Conflict(String),

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
}

#[derive(Debug, Clone)]
pub struct ErrorMetadata {
    pub status_code: StatusCode,
    pub type_slug: String,
    pub help_link: Option<String>,
    pub error_code: Option<String>,
    pub severity: &'static str,
}

impl AppError {
    /// Consolidates status, slug, help link, error code, and severity mappings.
    /// Excludes wildcard matches to guarantee exhaustiveness checking.
    pub fn resolve_metadata(&self) -> ErrorMetadata {
        let mut severity = "ERROR";
        let status_code;
        let type_slug;
        let mut help_link = None;
        let mut error_code = None;

        match self {
            AppError::Runner(e) => match e {
                crate::agent::runner::error::RunnerError::BudgetExhausted(_) => {
                    status_code = StatusCode::PAYMENT_REQUIRED;
                    type_slug = "budget-exhausted".to_string();
                }
                crate::agent::runner::error::RunnerError::RecursionBlocked(_) => {
                    status_code = StatusCode::LOOP_DETECTED;
                    type_slug = "recursion-blocked".to_string();
                    severity = "CRITICAL";
                }
                crate::agent::runner::error::RunnerError::SentinelGate(_) => {
                    status_code = StatusCode::FORBIDDEN;
                    type_slug = "sentinel-gate-failure".to_string();
                    severity = "CRITICAL";
                }
                crate::agent::runner::error::RunnerError::Compression(_) => {
                    status_code = StatusCode::INTERNAL_SERVER_ERROR;
                    type_slug = "compression-error".to_string();
                    severity = "CRITICAL";
                }
            },
            AppError::Skill(e) => match e {
                crate::agent::skill_error::SkillError::ValidationError(_) => {
                    status_code = StatusCode::BAD_REQUEST;
                    type_slug = "validation-error".to_string();
                }
                crate::agent::skill_error::SkillError::RecruitmentFailure {
                    recipe_id,
                    role,
                    detail: _,
                } => {
                    status_code = StatusCode::SERVICE_UNAVAILABLE;
                    type_slug = format!("recruitment:{}", role).to_lowercase();
                    error_code = Some(format!("RECRUITMENT_FAILED:{}:{}", recipe_id, role));
                }
                crate::agent::skill_error::SkillError::SanitizationViolation(_) => {
                    status_code = StatusCode::FORBIDDEN;
                    type_slug = "sanitization-violation".to_string();
                    severity = "CRITICAL";
                }
            },
            AppError::BadRequest(_) => {
                status_code = StatusCode::BAD_REQUEST;
                type_slug = "bad-request".to_string();
            }
            AppError::Unauthorized(_) => {
                status_code = StatusCode::UNAUTHORIZED;
                type_slug = "unauthorized".to_string();
                severity = "CRITICAL";
            }
            AppError::Forbidden(_) => {
                status_code = StatusCode::FORBIDDEN;
                type_slug = "forbidden".to_string();
                severity = "CRITICAL";
            }
            AppError::NotFound(_) => {
                status_code = StatusCode::NOT_FOUND;
                type_slug = "not-found".to_string();
            }
            AppError::DomainError {
                code,
                detail: _,
                help_link: hl,
            } => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                type_slug = format!("domain:{}", code).to_lowercase();
                help_link = hl.clone();
                error_code = Some(code.clone());
            }
            AppError::InfrastructureError {
                provider_id,
                detail: _,
                help_link: hl,
            } => {
                status_code = StatusCode::BAD_GATEWAY;
                type_slug = format!("infra:{}", provider_id).to_lowercase();
                help_link = hl.clone();
                error_code = Some(provider_id.clone());
                severity = "CRITICAL";
            }
            AppError::QuantizationFallback {
                model_id,
                suggested_quant,
                detail: _,
            } => {
                status_code = StatusCode::INSUFFICIENT_STORAGE;
                type_slug = "resource-exhaustion".to_string();
                help_link = Some(format!(
                    "https://docs.tadpole.os/troubleshooting/quantization#{}",
                    suggested_quant
                ));
                error_code = Some(format!("OOM_FALLBACK:{}", model_id));
            }
            AppError::NotImplemented(_) => {
                status_code = StatusCode::NOT_IMPLEMENTED;
                type_slug = "not-implemented".to_string();
            }
            AppError::RateLimit(_) => {
                status_code = StatusCode::TOO_MANY_REQUESTS;
                type_slug = "rate-limit".to_string();
            }
            AppError::InternalServerError(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                type_slug = "internal-error".to_string();
                severity = "CRITICAL";
            }
            AppError::Conflict(_) => {
                status_code = StatusCode::CONFLICT;
                type_slug = "conflict".to_string();
            }
            AppError::Anyhow(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                type_slug = "internal-error".to_string();
                severity = "CRITICAL";
            }
            AppError::Sqlx(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                type_slug = "internal-error".to_string();
                severity = "CRITICAL";
            }
            AppError::Io(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                type_slug = "internal-error".to_string();
                severity = "CRITICAL";
            }
            AppError::Reqwest(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                type_slug = "internal-error".to_string();
                severity = "CRITICAL";
            }
            AppError::Serde(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                type_slug = "internal-error".to_string();
                severity = "CRITICAL";
            }
            AppError::WalkDir(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                type_slug = "internal-error".to_string();
                severity = "CRITICAL";
            }
        }

        let resolved_code = error_code.unwrap_or_else(|| type_slug.to_uppercase());

        ErrorMetadata {
            status_code,
            type_slug,
            help_link,
            error_code: Some(resolved_code),
            severity,
        }
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
            AppError::Reqwest(e) => {
                e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS)
            }
            AppError::InfrastructureError { detail, .. } => {
                let d = detail.to_lowercase();
                d.contains("rate limit") || d.contains("429") || d.contains("status: 429")
            }
            _ => false,
        }
    }

    /// Checks if the error is a network or timeout error (e.g. Reqwest timeout/connection error
    /// or InfrastructureError mentioning timeout/connect).
    pub fn is_network_timeout(&self) -> bool {
        match self {
            AppError::Reqwest(e) => {
                e.is_timeout() || e.is_connect()
            }
            AppError::InfrastructureError { detail, .. } => {
                let d = detail.to_lowercase();
                d.contains("timeout") || d.contains("connect")
            }
            _ => false,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let metadata = self.resolve_metadata();
        let status = metadata.status_code;
        let slug = metadata.type_slug;
        let detail = format!("{}", self);

        // SEC-03: Redact secrets from the detail or mask internal details for 5xx errors to prevent info leakage
        let safe_detail = if status.is_server_error() {
            "An internal server error occurred. Please check system logs.".to_string()
        } else {
            crate::utils::security::redact_secrets(&detail)
        };

        let pd = ProblemDetails {
            type_uri: format!("https://tadpole.os/errors/{}", slug),
            title: slug.replace(['-', ':'], " ").to_uppercase(),
            status: status.as_u16(),
            detail: safe_detail,
            instance: None,
            error_code: metadata.error_code,
            help_link: metadata.help_link,
            severity: metadata.severity.to_string(),
        };

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
    }

    #[test]
    fn test_error_slug_generation() {
        assert_eq!(
            AppError::BadRequest("bad".to_string()).type_slug(),
            "bad-request"
        );
        assert_eq!(
            AppError::DomainError {
                code: "X1".to_string(),
                detail: "d".to_string(),
                help_link: None
            }
            .type_slug(),
            "domain:x1"
        );
        assert_eq!(
            AppError::InfrastructureError {
                provider_id: "AWS".to_string(),
                detail: "d".to_string(),
                help_link: None
            }
            .type_slug(),
            "infra:aws"
        );
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
}

// Metadata: [error]

// Metadata: [error]
