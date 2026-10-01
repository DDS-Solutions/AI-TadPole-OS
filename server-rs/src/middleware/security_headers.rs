//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / security_headers
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Structural]` Security: HSTS is strictly conditional (injected only on verified HTTPS or trusted-proxy TLS).
//! - `[Structural]` Security: defense-in-depth CSP restricts object embedding, base URI manipulation, and framing.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use axum::{
    extract::ConnectInfo,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

/// Middleware to inject security headers into all responses (SEC-01).
///
/// Implements:
/// - Content-Security-Policy (CSP)
/// - Strict-Transport-Security (HSTS - conditional on verified HTTPS / trusted-proxy TLS)
/// - X-Content-Type-Options (nosniff)
/// - X-Frame-Options (DENY)
/// - Referrer-Policy
/// - Permissions-Policy
pub async fn inject_security_headers(req: axum::extract::Request, next: Next) -> Response {
    let socket_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip());

    let proxy_is_trusted = socket_ip
        .map(|ip| crate::middleware::is_ip_trusted(&ip))
        .unwrap_or(false);

    let is_https = req.uri().scheme() == Some(&axum::http::uri::Scheme::HTTPS)
        || (proxy_is_trusted
            && req
                .headers()
                .get("x-forwarded-proto")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().eq_ignore_ascii_case("https"))
                .unwrap_or(false));

    let mut response: Response = next.run(req).await;
    tracing::trace!("[SecurityHeaders] Injecting security headers into response");
    let headers = response.headers_mut();

    // 1. Content-Security-Policy (preserve handler-set stricter policies)
    if !headers.contains_key(header::CONTENT_SECURITY_POLICY) {
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' ws: wss:; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none';",
            ),
        );
    }

    // 2. Strict-Transport-Security (HSTS)
    // Only injected when served over verified HTTPS / trusted TLS proxy.
    if is_https && !headers.contains_key(header::STRICT_TRANSPORT_SECURITY) {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }

    // 3. X-Content-Type-Options
    if !headers.contains_key(header::X_CONTENT_TYPE_OPTIONS) {
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
    }

    // 4. X-Frame-Options (redundancy alongside frame-ancestors 'none' for legacy clients)
    if !headers.contains_key(header::X_FRAME_OPTIONS) {
        headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    }

    // 5. Referrer-Policy
    if !headers.contains_key(header::REFERRER_POLICY) {
        headers.insert(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );
    }

    // 6. Permissions-Policy
    if !headers.contains_key("permissions-policy") {
        headers.insert(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        );
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

    async fn dummy_handler() -> StatusCode {
        StatusCode::OK
    }

    #[tokio::test]
    async fn test_security_headers_injected() {
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn(inject_security_headers));

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let headers = res.headers();
        assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
        assert!(headers.contains_key(header::X_CONTENT_TYPE_OPTIONS));
        assert!(headers.contains_key(header::X_FRAME_OPTIONS));
        assert!(headers.contains_key(header::REFERRER_POLICY));
        assert!(headers.contains_key("permissions-policy"));

        // Plain HTTP -> No HSTS
        assert!(!headers.contains_key(header::STRICT_TRANSPORT_SECURITY));
    }

    #[tokio::test]
    async fn test_hsts_conditional_on_trusted_proxy_https() {
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn(inject_security_headers));

        // 1. Trusted proxy (loopback 127.0.0.1) forwarding https -> HSTS injected
        let req = Request::builder()
            .uri("/")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))))
            .header("x-forwarded-proto", "https")
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res
            .headers()
            .contains_key(header::STRICT_TRANSPORT_SECURITY));
        let hsts = res
            .headers()
            .get(header::STRICT_TRANSPORT_SECURITY)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(hsts.contains("max-age=31536000"));
        assert!(
            !hsts.contains("preload"),
            "HSTS should omit preload by default"
        );

        // 2. Untrusted proxy (10.0.0.1) claiming https -> HSTS NOT injected (B5 fix)
        let untrusted_req = Request::builder()
            .uri("/")
            .extension(ConnectInfo(SocketAddr::from(([10, 0, 0, 99], 1234))))
            .header("x-forwarded-proto", "https")
            .body(Body::empty())
            .unwrap();

        let untrusted_res = app.oneshot(untrusted_req).await.unwrap();
        assert_eq!(untrusted_res.status(), StatusCode::OK);
        assert!(
            !untrusted_res
                .headers()
                .contains_key(header::STRICT_TRANSPORT_SECURITY),
            "Untrusted proxy must NOT be permitted to inject HSTS"
        );
    }
}
