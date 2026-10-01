//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / rate_limit
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Structural]` Security: exhausted rate limits return 429 responses with standard `Retry-After` and `X-RateLimit-*` headers rather than bare rejections.
//! - `[Structural]` Security: loopback skip requires both socket and client IP to be verified loopbacks.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: StatusCode::TOO_MANY_REQUESTS, StatusCode::FORBIDDEN
//! - **Telemetry Targets**: none declared

use crate::middleware::extract_client_ip_addr;
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use parking_lot::Mutex;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

/// Static fallback values to prevent repeated allocations
static FALLBACK_LIMIT: HeaderValue = HeaderValue::from_static("0");

pub const MIN_RATE_LIMIT: u32 = 1;
pub const MAX_RATE_LIMIT: u32 = 1_000_000;
pub const DEFAULT_RATE_LIMIT: u32 = 2000;

/// Tracks rate limit buckets by canonical IP.
/// Key: IP address (`std::net::IpAddr`)
/// Value: Arc<Mutex<(tokens, last_refill_timestamp)>>
/// Utilizing `moka` for high-performance concurrent access and automated eviction,
/// combined with a Mutex per client IP to ensure atomic read-refill-consume without TOCTOU race conditions.
static RATE_BUCKETS: LazyLock<moka::future::Cache<IpAddr, Arc<Mutex<(f64, Instant)>>>> =
    LazyLock::new(|| {
        moka::future::Cache::builder()
            .max_capacity(20000)
            .time_to_idle(Duration::from_secs(600)) // 10 minute idle eviction
            .build()
    });

static MAX_TOKENS: LazyLock<f64> = LazyLock::new(|| {
    if let Ok(raw) = std::env::var("ENGINE_RATE_LIMIT") {
        match raw.trim().parse::<u32>() {
            Ok(val) if (MIN_RATE_LIMIT..=MAX_RATE_LIMIT).contains(&val) => val as f64,
            _ => {
                tracing::warn!(
                    "⚠️ [RateLimit] Invalid ENGINE_RATE_LIMIT '{}'; defaulting to {}",
                    raw,
                    DEFAULT_RATE_LIMIT
                );
                DEFAULT_RATE_LIMIT as f64
            }
        }
    } else {
        DEFAULT_RATE_LIMIT as f64
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

    let client_ip = match client_ip_addr.or(socket_ip) {
        Some(ip) => ip.to_canonical(),
        None => {
            tracing::warn!("🚫 [RateLimit] Request missing connection info / peer address");
            return Ok((StatusCode::FORBIDDEN, "Missing peer connection info").into_response());
        }
    };

    let now = Instant::now();
    let bucket_arc = RATE_BUCKETS
        .get_with(client_ip, async {
            Arc::new(Mutex::new((*MAX_TOKENS, now)))
        })
        .await;

    let (is_blocked, current_tokens, retry_after_secs, reset_secs) = {
        let mut guard = bucket_arc.lock();
        let (ref mut tokens, ref mut last_refill) = *guard;
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        *tokens = (*tokens + elapsed * *REFILL_RATE_PER_SEC).min(*MAX_TOKENS);
        *last_refill = now;

        if *tokens < 1.0 {
            let retry_after = ((1.0 - *tokens) / *REFILL_RATE_PER_SEC).ceil().max(1.0) as u64;
            let reset = ((*MAX_TOKENS - *tokens) / *REFILL_RATE_PER_SEC)
                .ceil()
                .max(1.0) as u64;
            (true, *tokens, retry_after, reset)
        } else {
            *tokens -= 1.0;
            let remaining = *tokens;
            let reset = if remaining < *MAX_TOKENS {
                ((*MAX_TOKENS - remaining) / *REFILL_RATE_PER_SEC).ceil() as u64
            } else {
                0
            };
            (false, remaining, 0, reset)
        }
    };

    let limit_str = (*MAX_TOKENS as u32).to_string();
    let remaining_str = (current_tokens.floor().max(0.0) as u32).to_string();
    let reset_str = reset_secs.to_string();

    let limit_header = HeaderValue::from_str(&limit_str).unwrap_or(FALLBACK_LIMIT.clone());
    let remaining_header = HeaderValue::from_str(&remaining_str).unwrap_or(FALLBACK_LIMIT.clone());
    let reset_header = HeaderValue::from_str(&reset_str).unwrap_or(FALLBACK_LIMIT.clone());

    if is_blocked {
        tracing::warn!(
            "🚫 [RateLimit] Rate limit exceeded for IP: {}. Retry-After: {}s",
            client_ip,
            retry_after_secs
        );
        let retry_str = retry_after_secs.to_string();
        let retry_header = HeaderValue::from_str(&retry_str).unwrap_or(FALLBACK_LIMIT.clone());

        let response = (
            StatusCode::TOO_MANY_REQUESTS,
            [
                (header::RETRY_AFTER, retry_header),
                (
                    header::HeaderName::from_static("x-ratelimit-limit"),
                    limit_header,
                ),
                (
                    header::HeaderName::from_static("x-ratelimit-remaining"),
                    remaining_header,
                ),
                (
                    header::HeaderName::from_static("x-ratelimit-reset"),
                    reset_header,
                ),
            ],
            "Too Many Requests: Rate limit exceeded.\n",
        )
            .into_response();

        return Ok(response);
    }

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        header::HeaderName::from_static("x-ratelimit-limit"),
        limit_header,
    );
    headers.insert(
        header::HeaderName::from_static("x-ratelimit-remaining"),
        remaining_header,
    );
    headers.insert(
        header::HeaderName::from_static("x-ratelimit-reset"),
        reset_header,
    );

    Ok(response)
}

/// Evicts stale rate limiting buckets older than the specified max age.
/// Automated background maintenance task called by the security cron runner.
#[allow(dead_code)]
pub fn evict_stale_buckets(_max_age: std::time::Duration) {
    // Moka handles idle eviction automatically via time_to_idle.
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

        let test_ip: IpAddr = "10.0.0.1".parse().unwrap();

        // Seed bucket with 0.5 tokens (insufficient for 1.0 consumption)
        RATE_BUCKETS
            .insert(test_ip, Arc::new(Mutex::new((0.5, Instant::now()))))
            .await;

        let req = Request::builder()
            .uri("/")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(res.headers().contains_key(header::RETRY_AFTER));
        assert_eq!(res.headers().get("X-RateLimit-Remaining").unwrap(), "0");
        assert!(res.headers().contains_key("X-RateLimit-Limit"));
        assert!(res.headers().contains_key("X-RateLimit-Reset"));

        RATE_BUCKETS.invalidate(&test_ip).await;
    }

    #[tokio::test]
    async fn test_rate_limiting_concurrent_burst_no_bypass() {
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(axum::middleware::from_fn(inject_rate_limit_headers));

        let test_ip: IpAddr = "10.0.0.1".parse().unwrap();

        // Initialize bucket with exactly 3.0 tokens
        RATE_BUCKETS
            .insert(test_ip, Arc::new(Mutex::new((3.0, Instant::now()))))
            .await;

        let mut handles = Vec::new();
        // Fire 10 concurrent requests
        for _ in 0..10 {
            let app_clone = app.clone();
            handles.push(tokio::spawn(async move {
                let req = Request::builder()
                    .uri("/")
                    .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
                    .header("X-Forwarded-For", "10.0.0.1")
                    .body(Body::empty())
                    .unwrap();
                app_clone.oneshot(req).await.unwrap()
            }));
        }

        let mut ok_count = 0;
        let mut rate_limited_count = 0;
        for handle in handles {
            let res = handle.await.unwrap();
            if res.status() == StatusCode::OK {
                ok_count += 1;
            } else if res.status() == StatusCode::TOO_MANY_REQUESTS {
                rate_limited_count += 1;
            }
        }

        assert_eq!(ok_count, 3);
        assert_eq!(rate_limited_count, 7);

        RATE_BUCKETS.invalidate(&test_ip).await;
    }
}
