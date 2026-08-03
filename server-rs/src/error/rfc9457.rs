//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **RFC 9457 HTTP Response Formatter**: Implements `IntoResponse` for `AppError`,
//! converting resolved error metadata into RFC 9457 Problem Details JSON responses
//! with the correct `application/problem+json` content type header.
//! Extracted from the unified error engine to isolate HTTP response concerns.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Incorrect content-type header or missing severity field.
//! - **Trace Scope**: `server-rs::error::rfc9457`

use axum::response::{IntoResponse, Response};
use axum::Json;

use super::{AppError, ProblemDetails};

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

// Metadata: [error::rfc9457]
