//! @docs ARCHITECTURE:Observability
//!
//! ### AI Assist Note
//! **Request Tracer**: Orchestrates the end-to-end correlation of API
//! transactions. Injects a unique **X-Request-ID** (UUID v4) into every
//! request/response cycle. If the client provides a pre-existing
//! identifier, it is preserved and echoed to maintain the **Sovereign
//! Trace Chain**. Ensures that every log entry and telemetry span
//! can be reconciled across the distributed engine (TRAC-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Header duplication (middleware double-nesting) or
//!   missing ID in the final response if the middleware execution is
//!   bypassed by a high-level error handler.
//! - **Trace Scope**: `server-rs::middleware::request_id`

use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Middleware that injects an `X-Request-Id` and `traceparent` header into every response.
/// If the client sends ones, they are echoed back or extended.
pub async fn inject_request_id(req: Request<Body>, next: Next) -> Response {
    // 1. Get or Generate Request-ID
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 2. Get or Generate Traceparent (W3C Standard)
    // Format: 00-{trace-id}-{span-id}-{flags}
    let trace_parent = req
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let trace_id = Uuid::new_v4().simple();
            let span_id = Uuid::new_v4().simple();
            format!("00-{}-{}-01", trace_id, &span_id.to_string()[..16])
        });

    let mut response = next.run(req).await;

    // 3. Inject headers into response
    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", val);
    }
    if let Ok(val) = HeaderValue::from_str(&trace_parent) {
        response.headers_mut().insert("traceparent", val);
    }

    response
}
