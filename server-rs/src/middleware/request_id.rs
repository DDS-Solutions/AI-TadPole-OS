//! @docs ARCHITECTURE:Observability
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / request_id
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Security: client-supplied request IDs are bounded to max 128 characters to prevent log/span bloat.
//!   - enforced_by: `test_oversized_request_id_regenerated`
//! - `[Behavioral]` Standard: conforms strictly to W3C `traceparent` specifications (TRAC-01); malformed headers are regenerated.
//!   - enforced_by: `test_malformed_traceparent_regenerated`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_inject_request_id_middleware`, `test_oversized_request_id_regenerated`, `test_malformed_traceparent_regenerated`

use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Max allowed length for client-supplied request IDs.
const MAX_REQUEST_ID_LEN: usize = 128;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RequestId(pub String);

/// Validates whether a client-provided `x-request-id` is acceptable (bounded length, non-empty, visible ASCII).
fn is_valid_request_id(id: &str) -> bool {
    let trimmed = id.trim();
    !trimmed.is_empty()
        && trimmed.len() <= MAX_REQUEST_ID_LEN
        && trimmed.chars().all(|c| c.is_ascii_graphic())
}

/// Validates a W3C `traceparent` header per RFC / W3C Trace Context spec.
/// Format: `00-{32 hex trace_id}-{16 hex span_id}-{2 hex flags}`
fn is_valid_w3c_traceparent(tp: &str) -> bool {
    let parts: Vec<&str> = tp.split('-').collect();
    if parts.len() != 4 {
        return false;
    }
    let (version, trace_id, span_id, flags) = (parts[0], parts[1], parts[2], parts[3]);

    version == "00"
        && trace_id.len() == 32
        && trace_id.chars().all(|c| c.is_ascii_hexdigit())
        && trace_id != "00000000000000000000000000000000"
        && span_id.len() == 16
        && span_id.chars().all(|c| c.is_ascii_hexdigit())
        && span_id != "0000000000000000"
        && flags.len() == 2
        && flags.chars().all(|c| c.is_ascii_hexdigit())
}

/// Generates a fresh standard W3C `traceparent` header.
fn generate_w3c_traceparent() -> String {
    let trace_id = Uuid::new_v4().simple();
    let span_id = Uuid::new_v4().simple();
    format!("00-{}-{}-01", trace_id, &span_id.to_string()[..16])
}

/// Middleware that injects an `X-Request-Id` and `traceparent` header.
///
/// ### 🛰️ Trace Propagation (TRAC-01)
/// Ensures that backend internal `tracing` spans are synchronized with
/// client request IDs and W3C distributed trace context.
pub async fn inject_request_id(mut req: Request<Body>, next: Next) -> Response {
    // 1. Get or Generate Request-ID
    let request_id_str = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .filter(|id| is_valid_request_id(id))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 2. Get or Generate Traceparent (W3C Standard)
    let trace_parent_str = req
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|tp| is_valid_w3c_traceparent(tp))
        .map(|s| s.to_string())
        .unwrap_or_else(generate_w3c_traceparent);

    // 3. Extract trace ID for span recording
    let trace_id_str = trace_parent_str.split('-').nth(1).unwrap_or("").to_string();

    // 4. Synchronize with internal tracing span
    let span = tracing::Span::current();
    span.record("request_id", &request_id_str);
    if !trace_id_str.is_empty() {
        span.record("trace_id", &trace_id_str);
    }

    // TRAC-02: Store RequestId in request extensions for downstream retrieval
    req.extensions_mut()
        .insert(RequestId(request_id_str.clone()));

    let mut response = next.run(req).await;

    // Record HTTP status code into the current span
    let http_status = response.status().as_u16();
    tracing::Span::current().record("http_status", http_status);

    // 5. Inject headers into response
    if let Ok(val) = HeaderValue::from_str(&request_id_str) {
        response.headers_mut().insert("x-request-id", val);
    }
    if let Ok(val) = HeaderValue::from_str(&trace_parent_str) {
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
            .header("x-request-id", "test-id-123")
            .header(
                "traceparent",
                "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            )
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("x-request-id").unwrap(), "test-id-123");
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

        let rid = res.headers().get("x-request-id").unwrap().to_str().unwrap();
        assert!(
            Uuid::parse_str(rid).is_ok(),
            "Generated request ID should be a valid UUID"
        );

        let tp = res.headers().get("traceparent").unwrap().to_str().unwrap();
        assert!(
            is_valid_w3c_traceparent(tp),
            "Generated traceparent should be valid W3C"
        );
    }

    #[tokio::test]
    async fn test_oversized_request_id_regenerated() {
        let app = Router::new()
            .route("/", get(|| async { StatusCode::OK }))
            .layer(from_fn(inject_request_id));

        let huge_id = "a".repeat(200);
        let req = Request::builder()
            .uri("/")
            .header("x-request-id", huge_id)
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        let returned_id = res.headers().get("x-request-id").unwrap().to_str().unwrap();
        assert_ne!(
            returned_id.len(),
            200,
            "Oversized ID must not be echoed back"
        );
        assert!(Uuid::parse_str(returned_id).is_ok());
    }

    #[tokio::test]
    async fn test_malformed_traceparent_regenerated() {
        let app = Router::new()
            .route("/", get(|| async { StatusCode::OK }))
            .layer(from_fn(inject_request_id));

        let req = Request::builder()
            .uri("/")
            .header("traceparent", "malformed-traceparent-value")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        let returned_tp = res.headers().get("traceparent").unwrap().to_str().unwrap();
        assert_ne!(returned_tp, "malformed-traceparent-value");
        assert!(is_valid_w3c_traceparent(returned_tp));
    }
}
