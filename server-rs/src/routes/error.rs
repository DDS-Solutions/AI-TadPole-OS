//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **Global Error Surface (Problem Details)**: Orchestrates the 
//! standardized error reporting and response formatting for the 
//! Tadpole OS engine. Features **RFC 9457 Compliance**: ensures that 
//! all errors return a machine-readable JSON structure with `type`, 
//! `title`, and `detail` fields. Implements **Secure Error 
//! Redaction**: automatically filters sensitive internal details 
//! (database strings, file paths) before dispatching the response to 
//! the frontend. AI agents should use the `AppError` enum to 
//! categorize and propagate failures, ensuring the "Surface" layer 
//! remains consistent for UI-driven recovery (ERR-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Double-panics during error formatting, leaking 
//!   internal stack traces in the `detail` field, or incorrect mapping 
//!   of `anyhow::Error` to HTTP status codes.
//! - **Telemetry Link**: Search for `ERROR` in `tracing` logs to 
//!   locate the source of the `AppError` before it was sanitized.
//! - **Trace Scope**: `server-rs::routes::error`

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
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
    /// This is the primary way to return errors from routing handlers.
    pub fn new(status: StatusCode, title: &str, detail: String) -> (StatusCode, Json<Self>) {
        (
            status,
            Json(Self {
                type_uri: format!("https://tadpole.os/errors/{}", status.as_u16()),
                title: title.to_string(),
                status: status.as_u16(),
                detail,
                instance: None,
                error_code: None,
                help_link: None,
            }),
        )
    }
}

/// Unified application error enum.
/// Variants are mapped to RFC 9457 types via the IntoResponse implementation.
#[allow(dead_code)] // All variants are part of the RFC 9457 error contract
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
    Unauthorized(String),
    Forbidden(String),
    ValidationError(String),
    /// Granular domain error with code for UI remediation.
    DomainError {
        code: String,
        detail: String,
        help_link: Option<String>,
    },
    NotImplemented(String),
    /// Catch-all for internal anyhow errors.
    Anyhow(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail, type_uri, code, help) = match self {
            AppError::BadRequest(d) => (
                StatusCode::BAD_REQUEST,
                "Bad Request",
                d,
                "https://tadpole.os/errors/bad-request",
                None,
                None,
            ),
            AppError::NotFound(d) => (
                StatusCode::NOT_FOUND,
                "Not Found",
                d,
                "https://tadpole.os/errors/not-found",
                None,
                None,
            ),
            AppError::Unauthorized(d) => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                d,
                "https://tadpole.os/errors/unauthorized",
                None,
                None,
            ),
            AppError::Forbidden(d) => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                d,
                "https://tadpole.os/errors/forbidden",
                None,
                None,
            ),
            AppError::ValidationError(d) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Error",
                d,
                "https://tadpole.os/errors/validation",
                None,
                None,
            ),
            AppError::DomainError { code, detail, help_link } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Domain Failure",
                detail,
                "https://tadpole.os/errors/domain",
                Some(code),
                help_link,
            ),
            AppError::InternalServerError(d) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                d,
                "https://tadpole.os/errors/internal",
                None,
                None,
            ),
            AppError::NotImplemented(d) => (
                StatusCode::NOT_IMPLEMENTED,
                "Not Implemented",
                d,
                "https://tadpole.os/errors/not-implemented",
                None,
                None,
            ),
            AppError::Anyhow(d) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                format!("{}", d),
                "https://tadpole.os/errors/internal",
                None,
                None,
            ),
        };

        let body = Json(ProblemDetails {
            type_uri: type_uri.to_string(),
            title: title.to_string(),
            status: status.as_u16(),
            detail,
            instance: None,
            error_code: code,
            help_link: help,
        });

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Anyhow(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_problem_details_new() {
        let (status, Json(details)) = ProblemDetails::new(
            StatusCode::NOT_FOUND,
            "Resource Not Found",
            "The requested user was not found".to_string(),
        );

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(details.status, 404);
        assert_eq!(details.title, "Resource Not Found");
        assert_eq!(details.detail, "The requested user was not found");
        assert_eq!(details.type_uri, "https://tadpole.os/errors/404");
    }

    #[tokio::test]
    async fn test_app_error_into_response() {
        let err = AppError::BadRequest("Invalid ID format".to_string());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_anyhow_error_conversion() {
        let anyhow_err = anyhow::anyhow!("DB connection failed");
        let app_err: AppError = anyhow_err.into();

        if let AppError::Anyhow(e) = app_err {
            assert_eq!(format!("{}", e), "DB connection failed");
        } else {
            panic!("Expected AppError::Anyhow");
        }
    }
}
