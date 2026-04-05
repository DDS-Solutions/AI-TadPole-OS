//! @docs ARCHITECTURE:Security
//! 
//! ### AI Assist Note
//! **Rate Limiter**: Orchestrates the communication of ingestion and 
//! inference capacity status. Injects standard **X-RateLimit** headers 
//! (`Limit`, `Remaining`, `Reset`) into every API response. Currently 
//! employs **Static Defaults** (60 RPM), pending integration with the 
//! dynamic model-registry metrics. Coordinates with the `BudgetGuard` 
//! to ensure transparent quota visibility for nomadic agents (RLMT-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Header injection failure or incorrect interval 
//!   calculations if dynamic override is active. 
//! - **Telemetry Link**: Verify `X-RateLimit-*` headers in the "Network" 
//!   tab of the Tadpole OS dashboard.
//! - **Trace Scope**: `server-rs::middleware::rate_limit`

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
