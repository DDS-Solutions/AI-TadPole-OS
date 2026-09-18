//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / rate_limit
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Security: exhausted rate limits return 429 responses with standard `Retry-After`
//!   and `X-RateLimit-*` headers rather than bare rejections.
//!   - enforced_by: `test_rate_limiting_exhaustion_429`
//! - `[Behavioral]` Security: loopback skip requires both socket and client IP to be verified loopbacks.
//!   - enforced_by: `test_rate_limiting_full_flow`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_rate_limiting_full_flow`, `test_rate_limiting_exhaustion_429`

use crate::middleware::{extract_client_ip, extract_client_ip_addr};
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

/// Static fallback values to prevent repeated allocations
static FALLBACK_LIMIT: HeaderValue = HeaderValue::from_static("0");

/// Tracks rate limit buckets by IP.
/// Key: IP address (as string)
/// Value: (tokens, last_refill_timestamp)
/// Utilizing `moka` for high-performance concurrent access and automated eviction.
static RATE_BUCKETS: LazyLock<moka::future::Cache<String, (f64, Instant)>> = LazyLock::new(|| {
    moka::future::Cache::builder()
        .max_capacity(20000)
        .time_to_idle(Duration::from_secs(600)) // 10 minute idle eviction
        .build()
});

static MAX_TOKENS: LazyLock<f64> = LazyLock::new(|| {
    if let Ok(raw) = std::env::var("ENGINE_RATE_LIMIT") {
        match raw.trim().parse::<f64>() {
            Ok(val) if val.is_finite() && val > 0.0 => val,
            _ => {
                tracing::warn!(
                    "⚠️ [RateLimit] Invalid ENGINE_RATE_LIMIT '{}'; defaulting to 2000.0",
                    raw
                );
                2000.0
            }
        }
    } else {
        2000.0
    }
});

static REFILL_RATE_PER_SEC: LazyLock<f64> = LazyLock::new(|| *MAX_TOKENS / 60.0);

/// Injects standard rate limit headers into every response and enforces limits.
pub async fn inject_rate_limit_headers(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 0. Skip rate limiting only for verified genuine local loopback requests
    let socket_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip());
    let is_socket_loopback = socket_ip.map(|ip| ip.is_loopback()).unwrap_or(false);
    let client_ip_addr = extract_client_ip_addr(&req);
    let is_client_loopback = client_ip_addr.map(|ip| ip.is_loopback()).unwrap_or(false);

    if is_socket_loopback && is_client_loopback {
        return Ok(next.run(req).await);
    }

    let client_ip = extract_client_ip(&req);
    let now = Instant::now();
    let (mut tokens, last_refill) = RATE_BUCKETS
        .get(&client_ip)
        .await
        .unwrap_or((*MAX_TOKENS, now));

    // 1. Refill based on elapsed time (Token Bucket Algorithm)
    let elapsed = now.duration_since(last_refill).as_secs_f64();
    tokens = (tokens + elapsed * *REFILL_RATE_PER_SEC).min(*MAX_TOKENS);
    let updated_refill = now;

    if tokens < 1.0 {
        tracing::warn!(
            "🚫 [Security] Rate limit exceeded for IP: {}. Blocked.",
            client_ip
        );

        let retry_after_secs = ((1.0 - tokens) / *REFILL_RATE_PER_SEC).ceil().max(1.0) as u64;
        let reset_secs = ((*MAX_TOKENS - tokens) / *REFILL_RATE_PER_SEC)
            .ceil()
            .max(1.0) as u64;

        let mut res = (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after_secs.to_string())],
        )
            .into_response();

        let headers = res.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            HeaderValue::from_str(&(*MAX_TOKENS as u32).to_string())
                .unwrap_or_else(|_| FALLBACK_LIMIT.clone()),
        );
        headers.insert("X-RateLimit-Remaining", HeaderValue::from_static("0"));
        headers.insert(
            "X-RateLimit-Reset",
            HeaderValue::from_str(&reset_secs.to_string())
                .unwrap_or_else(|_| FALLBACK_LIMIT.clone()),
        );

        return Ok(res);
    }

    // 2. Consume 1 token and update cache
    tokens -= 1.0;
    let current_tokens = tokens;
    RATE_BUCKETS
        .insert(client_ip.clone(), (tokens, updated_refill))
        .await;

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
        HeaderValue::from_str(&reset_secs.to_string()).unwrap_or_else(|_| FALLBACK_LIMIT.clone()),
    );

    Ok(response)
}

/// No longer needed with `moka`'s automated eviction, but kept as a no-op
/// to maintain compatibility with the background task structure.
#[allow(dead_code)]
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

        // 1. Initial request from unknown IP via trusted proxy
        let req = Request::builder()
            .uri("/")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
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

        // 3. Genuine localhost (socket=127.0.0.1 without external XFF) should skip rate limiting (no headers)
        let req_local = Request::builder()
            .uri("/")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
            .body(Body::empty())
            .unwrap();

        let res_local = app.clone().oneshot(req_local).await.unwrap();
        assert_eq!(res_local.status(), StatusCode::OK);
        assert!(!res_local.headers().contains_key("X-RateLimit-Limit"));

        // Cleanup
        RATE_BUCKETS.invalidate_all();
    }

    #[tokio::test]
    async fn test_rate_limiting_exhaustion_429() {
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(axum::middleware::from_fn(inject_rate_limit_headers));

        let test_ip = "10.0.0.1";

        // Seed bucket with 0.5 tokens (insufficient for 1.0 consumption)
        RATE_BUCKETS
            .insert(test_ip.to_string(), (0.5, Instant::now()))
            .await;

        let req = Request::builder()
            .uri("/")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
            .header("X-Forwarded-For", test_ip)
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(res.headers().contains_key(header::RETRY_AFTER));
        assert_eq!(res.headers().get("X-RateLimit-Remaining").unwrap(), "0");
        assert!(res.headers().contains_key("X-RateLimit-Limit"));
        assert!(res.headers().contains_key("X-RateLimit-Reset"));

        RATE_BUCKETS.invalidate(test_ip).await;
    }
}
