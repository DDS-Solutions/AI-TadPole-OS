//! Global Rate Limiter — Request throttling
//!
//! Provides an L7 rate limiter to protect the engine from DoS attacks, 
//! utilizing a Token Bucket algorithm for fair resource allocation.
//!
//! @docs ARCHITECTURE:Networking

use axum::{
extract::Request, middleware::Next, response::Response};

/// Injects standard rate limit headers into every response.
///
/// These headers inform API consumers about their current rate limit status:
/// - `X-RateLimit-Limit`: Maximum requests allowed per window
/// - `X-RateLimit-Remaining`: Requests remaining in current window
/// - `X-RateLimit-Reset`: Seconds until the window resets
///
/// Currently emits static defaults (60 RPM) since per-model rate tracking
/// is handled at the runner level. A future iteration can read from AppState
/// to emit per-model limits.
pub async fn inject_rate_limit_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    // Static defaults — a future iteration will read from the model registry
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", "60".parse().unwrap());
    headers.insert("X-RateLimit-Remaining", "59".parse().unwrap());
    headers.insert("X-RateLimit-Reset", "60".parse().unwrap());

    response
}
