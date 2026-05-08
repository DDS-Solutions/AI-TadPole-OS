//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **Security Hardening**: Middleware layer for injecting defense-in-depth headers.
//! Enforces CSP, HSTS, and Frame-Options to protect the Sovereign Dash from 
//! clickjacking and XSS.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Strict CSP blocking legitimate websocket connections or asset loading on specific edge browsers.
//! - **Telemetry Link**: Search `[SecurityHeaders]` in server traces.
//!

use axum::{
    http::{header, HeaderValue},
    middleware::Next,
    response::IntoResponse,
};

use tracing::debug;

/// Middleware to inject security headers into all responses (SEC-01).
///
/// Implements:
/// - Content-Security-Policy (CSP)
/// - Strict-Transport-Security (HSTS)
/// - X-Content-Type-Options (nosniff)
/// - X-Frame-Options (DENY)
/// - Referrer-Policy
pub async fn inject_security_headers(
    req: axum::extract::Request,
    next: Next,
) -> impl IntoResponse {
    let mut response: axum::response::Response = next.run(req).await;
    debug!("[SecurityHeaders] Injecting security headers into response");
    let headers = response.headers_mut();

    // 1. Content-Security-Policy
    // Conservative policy allowing only essential dashboard resources.
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' http://localhost:8000 http://127.0.0.1:8000 ws: wss:; frame-ancestors 'none';"),
    );

    // 2. Strict-Transport-Security (HSTS)
    // Only applied if non-localhost, but safe to include for production readiness.
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // 3. X-Content-Type-Options
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // 4. X-Frame-Options
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // 5. Referrer-Policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}

// Metadata: [security_headers]

// Metadata: [security_headers]
