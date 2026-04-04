//! Global Error Surface — API response formatting
//!
//! Implements RFC 9457 (Problem Details) compliance for all engine errors, 
//! ensuring consistent, redactable JSON responses for the frontend.
//!
//! @docs ARCHITECTURE:Networking

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
    NotImplemented(String),
    /// Catch-all for internal anyhow errors.
    Anyhow(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail, type_uri) = match self {
            AppError::BadRequest(d) => (
                StatusCode::BAD_REQUEST,
                "Bad Request",
                d,
                "https://tadpole.os/errors/bad-request",
            ),
            AppError::NotFound(d) => (
                StatusCode::NOT_FOUND,
                "Not Found",
                d,
                "https://tadpole.os/errors/not-found",
            ),
            AppError::Unauthorized(d) => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                d,
                "https://tadpole.os/errors/unauthorized",
            ),
            AppError::Forbidden(d) => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                d,
                "https://tadpole.os/errors/forbidden",
            ),
            AppError::ValidationError(d) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Error",
                d,
                "https://tadpole.os/errors/validation",
            ),
            AppError::InternalServerError(d) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                d,
                "https://tadpole.os/errors/internal",
            ),
            AppError::NotImplemented(d) => (
                StatusCode::NOT_IMPLEMENTED,
                "Not Implemented",
                d,
                "https://tadpole.os/errors/not-implemented",
            ),
            AppError::Anyhow(d) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                format!("{}", d),
                "https://tadpole.os/errors/internal",
            ),
        };

        let body = Json(ProblemDetails {
            type_uri: type_uri.to_string(),
            title: title.to_string(),
            status: status.as_u16(),
            detail,
            instance: None,
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
