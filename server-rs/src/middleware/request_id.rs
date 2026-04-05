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

/// Middleware that injects an `X-Request-Id` header into every response.
/// If the client sends one, it is echoed back. Otherwise, a new UUID v4 is generated.
/// This provides end-to-end request tracing through the engine.
pub async fn inject_request_id(req: Request<Body>, next: Next) -> Response {
    // Check if the client sent a request ID
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut response = next.run(req).await;

    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", val);
    }

    response
}
