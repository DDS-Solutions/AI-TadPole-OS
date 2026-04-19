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

/// Middleware that injects an `X-Request-Id` and `traceparent` header.
/// 
/// ### 🛰️ Trace Propagation
/// Ensures that the backend's internal `tracing` spans are perfectly aligned 
/// with the frontend's XHR/WebSocket requests. Bridges the W3C `traceparent` 
/// standard into the engine's cognitive loop (TRAC-01).
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

    // 3. Extract Trace-ID for span record
    let trace_id = if trace_parent.starts_with("00-") && trace_parent.len() >= 35 {
        trace_parent.split('-').nth(1).unwrap_or("").to_string()
    } else {
        "".to_string()
    };

    // 4. Synchronize with internal tracing span
    let span = tracing::Span::current();
    span.record("request_id", &request_id);
    if !trace_id.is_empty() {
        span.record("trace_id", &trace_id);
    }

    let mut response = next.run(req).await;

    // 5. Inject headers into response
    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", val);
    }
    if let Ok(val) = HeaderValue::from_str(&trace_parent) {
        response.headers_mut().insert("traceparent", val);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware::from_fn,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_inject_request_id_middleware() {
        let app = Router::new()
            .route("/", get(|| async { StatusCode::OK }))
            .layer(from_fn(inject_request_id));

        let req = Request::builder()
            .uri("/")
            .header("x-request-id", "test-id")
            .header("traceparent", "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("x-request-id").unwrap(), "test-id");
        assert_eq!(
            res.headers().get("traceparent").unwrap(),
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        );
    }

    #[tokio::test]
    async fn test_generate_request_id_if_missing() {
        let app = Router::new()
            .route("/", get(|| async { StatusCode::OK }))
            .layer(from_fn(inject_request_id));

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();

        assert!(res.headers().contains_key("x-request-id"));
        assert!(res.headers().contains_key("traceparent"));
    }
}
