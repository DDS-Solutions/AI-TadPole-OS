//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **Auth Rate Limiter**: Orchestrates brute-force protection for the
//! `NEURAL_TOKEN`. Tracks failed authentication attempts by client IP
//! using an in-memory **moka** cache. Enforces a **Cool-Down Policy**: 5
//! consecutive failures result in a 10-minute block (`BLOCK_DURATION`).
//! Coordinates with `ConnectInfo` to resolve client identifiers (BRUTE-01).
//! Note: Success automatically resets the failure counter for that IP.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: False positive blocks on shared NAT gateways,
//!   memory bloat from high IP churn (ephemeral attackers), or missing
//!   `ConnectInfo` in reverse proxy setups.
//! - **Telemetry Link**: Search `[Security]` in server traces.
//! - **Trace Scope**: `server-rs::middleware::auth_rate_limit`

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

use crate::middleware::extract_client_ip;

/// Tracks failed login attempts by IP.
/// Key: IP address (as string)
/// Value: (failure_count, last_failure_timestamp)
/// Utilizing `moka` for automated TTL eviction and thread-safe atomicity.
static AUTH_FAILURE_LOG: Lazy<moka::future::Cache<String, (u32, Instant)>> = Lazy::new(|| {
    moka::future::Cache::builder()
        .max_capacity(20000)
        .time_to_live(Duration::from_secs(600)) // 10 minute block/TTL eviction
        .build()
});

const MAX_FAILURES: u32 = 5;
const BLOCK_DURATION: Duration = Duration::from_secs(600); // 10 minutes

/// Middleware to prevent brute-force attacks by tracking failed authentication attempts.
pub async fn auth_brute_force_limiter(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = extract_client_ip(&req);
    let path = req.uri().path();

    // 1. Skip rate limiting for local loopback & all companion remote bridge routes / public endpoints
    if client_ip == "127.0.0.1"
        || client_ip == "::1"
        || path.starts_with("/v1/remote/")
        || path.starts_with("/v1/engine/health")
        || path == "/metrics"
    {
        // Reset failure log when hit via valid remote endpoints
        AUTH_FAILURE_LOG.invalidate(&client_ip).await;
        return Ok(next.run(req).await);
    }

    // 2. Check if the IP is currently blocked
    if let Some((count, last_attempt)) = AUTH_FAILURE_LOG.get(&client_ip).await {
        if count >= MAX_FAILURES && last_attempt.elapsed() < BLOCK_DURATION {
            tracing::warn!(
                "🚫 [Security] Brute-force block active for IP: {}. Cooling down.",
                client_ip
            );
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    // 3. Proceed with the request
    let response = next.run(req).await;

    // 4. Inspect response for UNAUTHORIZED status
    if response.status() == StatusCode::UNAUTHORIZED {
        tracing::debug!("⚠️ [Security] Auth failure recorded for IP: {}", client_ip);

        let (mut count, _) = AUTH_FAILURE_LOG
            .get(&client_ip)
            .await
            .unwrap_or((0, Instant::now()));
        count += 1;
        let now = Instant::now();

        AUTH_FAILURE_LOG
            .insert(client_ip.clone(), (count, now))
            .await;

        if count >= MAX_FAILURES {
            tracing::error!(
                "🚨 [Security] IP {} exceeded max auth failures. Blocking for 10m.",
                client_ip
            );
        }
    } else if response.status().is_success() {
        // Reset failures on success
        AUTH_FAILURE_LOG.invalidate(&client_ip).await;
    }

    Ok(response)
}

/// No longer manually evicts entries since `moka` handles TTL eviction automatically.
/// Kept to maintain signature compatibility with background cron scheduler.
pub fn evict_expired_blocks(_max_age: std::time::Duration) {
    // moka handles this automatically via time_to_live
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{middleware, routing::get, Router};
    use tower::ServiceExt;

    async fn dummy_handler() -> StatusCode {
        StatusCode::OK
    }
    async fn fail_handler() -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    #[tokio::test]
    async fn test_brute_force_blocking() {
        let app = Router::new()
            .route("/success", get(dummy_handler))
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        // Note: In tests, ConnectInfo isn't automatically injected unless setup specifically.
        // The middleware defaults to "unknown" IP if missing.

        // 1. Fail 5 times
        for _ in 0..5 {
            let req = Request::builder().uri("/fail").body(Body::empty()).unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }

        // 2. Next attempt should be 429
        let req = Request::builder().uri("/fail").body(Body::empty()).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        // 3. Success should also be blocked (entire IP is blocked)
        let req = Request::builder()
            .uri("/success")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        // Cleanup the static log for other tests if needed
        AUTH_FAILURE_LOG.invalidate_all();
    }
}

// Metadata: [Security]
