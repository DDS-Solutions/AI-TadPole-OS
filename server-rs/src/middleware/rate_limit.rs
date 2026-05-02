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
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};
use crate::middleware::extract_client_ip;

/// Static fallback values to prevent repeated allocations
static FALLBACK_LIMIT: HeaderValue = HeaderValue::from_static("0");

/// Tracks rate limit buckets by IP.
/// Key: IP address (as string)
/// Value: (tokens, last_refill_timestamp)
/// Utilizing `moka` for high-performance concurrent access and automated eviction.
static RATE_BUCKETS: Lazy<moka::future::Cache<String, (f64, Instant)>> = Lazy::new(|| {
    moka::future::Cache::builder()
        .max_capacity(20000)
        .time_to_idle(Duration::from_secs(600)) // 10 minute idle eviction
        .build()
});

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
    let client_ip = extract_client_ip(&req);

    // 0. Skip rate limiting for local loopback (desktop app safety)
    if client_ip == "127.0.0.1" || client_ip == "::1" {
        return Ok(next.run(req).await);
    }

    let now = Instant::now();
    let (mut tokens, mut last_refill) = RATE_BUCKETS
        .get(&client_ip)
        .await
        .unwrap_or((*MAX_TOKENS, now));

    // 1. Refill based on elapsed time (Token Bucket Algorithm)
    let elapsed = now.duration_since(last_refill).as_secs_f64();
    tokens = (tokens + elapsed * *REFILL_RATE_PER_SEC).min(*MAX_TOKENS);
    last_refill = now;

    if tokens < 1.0 {
        tracing::warn!(
            "🚫 [Security] Rate limit exceeded for IP: {}. Blocked.",
            client_ip
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // 2. Consume 1 token and update cache
    tokens -= 1.0;
    let current_tokens = tokens;
    RATE_BUCKETS.insert(client_ip.clone(), (tokens, last_refill)).await;

    // 3. Calculate reset time (seconds until full refill)
    let reset_secs = if tokens < *MAX_TOKENS {
        ((*MAX_TOKENS - tokens) / *REFILL_RATE_PER_SEC).ceil() as u64
    } else {
        0
    };

    // 4. Proceed with request
    let mut response = next.run(req).await;

    // 5. Inject headers into response
    let headers = response.headers_mut();
    
    headers.insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&(*MAX_TOKENS as u32).to_string())
            .unwrap_or_else(|_| FALLBACK_LIMIT.clone()),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        HeaderValue::from_str(&(current_tokens as u32).to_string())
            .unwrap_or_else(|_| FALLBACK_LIMIT.clone()),
    );
    headers.insert(
        "X-RateLimit-Reset",
        HeaderValue::from_str(&reset_secs.to_string())
            .unwrap_or_else(|_| FALLBACK_LIMIT.clone()),
    );

    Ok(response)
}

/// No longer needed with `moka`'s automated eviction, but kept as a no-op 
/// to maintain compatibility with the background task structure.
pub fn evict_stale_buckets(_max_age: std::time::Duration) {
    // moka handles this automatically via `time_to_idle`
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
    async fn test_rate_limiting_full_flow() {
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(axum::middleware::from_fn(inject_rate_limit_headers));

        // 1. Initial request from unknown IP
        let req = Request::builder()
            .uri("/")
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        
        // 2. Verify headers
        let headers = res.headers();
        assert!(headers.contains_key("X-RateLimit-Limit"));
        assert!(headers.contains_key("X-RateLimit-Remaining"));
        assert!(headers.contains_key("X-RateLimit-Reset"));

        // 3. Localhost should skip rate limiting (no headers)
        let req_local = Request::builder()
            .uri("/")
            .header("X-Forwarded-For", "127.0.0.1")
            .body(Body::empty())
            .unwrap();
        
        let res_local = app.clone().oneshot(req_local).await.unwrap();
        assert_eq!(res_local.status(), StatusCode::OK);
        assert!(!res_local.headers().contains_key("X-RateLimit-Limit"));

        // Cleanup
        RATE_BUCKETS.invalidate_all();
    }
}

// Metadata: [rate_limit]

// Metadata: [rate_limit]
