//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / rfc9457
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use axum::response::{IntoResponse, Response};
use axum::Json;

use super::{AppError, ProblemDetails};

/// Constructs an RFC 9457 compliant Axum HTTP response with `application/problem+json`
/// and hardening headers (`Cache-Control: no-store`).
pub(crate) fn problem_response(status: axum::http::StatusCode, pd: ProblemDetails) -> Response {
    let mut res = (status, Json(pd)).into_response();
    res.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/problem+json"),
    );
    res.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    res
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = axum::http::StatusCode::from_u16(self.status)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        problem_response(status, self)
    }
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
            None,
        );

        problem_response(status, pd)
    }
}
