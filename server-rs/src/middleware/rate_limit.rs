//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **Rate Limiter**: Orchestrates the communication of ingestion and
//! inference capacity status. Injects standard **X-RateLimit** headers
//! (`Limit`, `Remaining`, `Reset`) into every API response. Enforces
//! a **Sovereign Token Bucket** policy (100 RPM default). Tracks
//! consumption by client IP to ensure fair resource allocation
//! across the swarm (RLMT-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 429 Too Many Requests status on legitimate
//!   bursts, or memory leaks in the bucket registry during IP churn.
//! - **Telemetry Link**: Verify `X-RateLimit-*` headers in the "Network"
//!   tab of the Tadpole OS dashboard.
//! - **Trace Scope**: `server-rs::middleware::rate_limit`

use axum::{
    body::Body,
    http::{HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::time::Instant;

/// Tracks rate limit buckets by IP.
/// Key: IP address (as string)
/// Value: (tokens, last_refill_timestamp)
static RATE_BUCKETS: Lazy<DashMap<String, (f64, Instant)>> = Lazy::new(DashMap::new);

static MAX_TOKENS: Lazy<f64> = Lazy::new(|| {
    std::env::var("ENGINE_RATE_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2000.0)
});

static REFILL_RATE_PER_SEC: Lazy<f64> = Lazy::new(|| *MAX_TOKENS / 60.0);

/// Injects standard rate limit headers into every response and enforces limits.
pub async fn inject_rate_limit_headers(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|addr| addr.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // 0. Skip rate limiting for local loopback (desktop app safety)
    if client_ip == "127.0.0.1" || client_ip == "::1" {
        return Ok(next.run(req).await);
    }

    let now = Instant::now();
    let current_tokens: f64;
    let reset_secs: u64;

    // 1. Update Bucket (Token Bucket Algorithm)
    {
        let mut entry = RATE_BUCKETS
            .entry(client_ip.clone())
            .or_insert((*MAX_TOKENS, now));
        let (ref mut tokens, ref mut last_refill) = entry.value_mut();

        // Refill based on elapsed time
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        *tokens = (*tokens + elapsed * *REFILL_RATE_PER_SEC).min(*MAX_TOKENS);
        *last_refill = now;

        if *tokens < 1.0 {
            tracing::warn!(
                "🚫 [Security] Rate limit exceeded for IP: {}. Blocked.",
                client_ip
            );
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        // Consume 1 token
        *tokens -= 1.0;
        current_tokens = *tokens;

        // Calculate reset time (seconds until full refill)
        reset_secs = if *tokens < *MAX_TOKENS {
            ((*MAX_TOKENS - *tokens) / *REFILL_RATE_PER_SEC).ceil() as u64
        } else {
            0
        };
    }

    // 2. Proceed with request
    let mut response = next.run(req).await;

    // 3. Inject headers into response
    let headers = response.headers_mut();
    headers.insert(
        "X-RateLimit-Limit",
        (*MAX_TOKENS).to_string().parse().unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        (current_tokens as u32).to_string().parse().unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    headers.insert("X-RateLimit-Reset", reset_secs.to_string().parse().unwrap_or_else(|_| HeaderValue::from_static("0")));

    Ok(response)
}

/// Evicts rate limit buckets that have been idle for longer than `max_age`.
/// Called periodically by the background eviction task to prevent unbounded
/// memory growth from ephemeral client IPs (bot scans, port sweeps).
pub fn evict_stale_buckets(max_age: std::time::Duration) {
    let before = RATE_BUCKETS.len();
    RATE_BUCKETS.retain(|_, (_, last_refill)| last_refill.elapsed() < max_age);
    let evicted = before - RATE_BUCKETS.len();
    if evicted > 0 {
        tracing::debug!(
            "🧹 [RateLimit] Evicted {} stale bucket(s) ({} remaining)",
            evicted,
            RATE_BUCKETS.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    async fn dummy_handler() -> StatusCode {
        StatusCode::OK
    }

    #[tokio::test]
    async fn test_rate_limiting_bucket() {
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(axum::middleware::from_fn(inject_rate_limit_headers));

        // 1. First request should be OK
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().contains_key("X-RateLimit-Remaining"));

        // 2. Clear bucket for testing (force exhaustion)
        let client_ip = "unknown".to_string(); // Default in tests
        RATE_BUCKETS.insert(client_ip.clone(), (0.5, Instant::now()));

        // 3. Next request should fail (429)
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        // Cleanup
        RATE_BUCKETS.clear();
    }
}
