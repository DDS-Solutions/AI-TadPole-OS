//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **Auth Rate Limiter**: Orchestrates brute-force protection for the
//! `NEURAL_TOKEN`. Tracks failed authentication attempts by client IP
//! using an in-memory **DashMap**. Enforces a **Cool-Down Policy**: 5
//! consecutive failures result in a 10-minute block (`BLOCK_DURATION`).
//! Coordinates with `ConnectInfo` to resolve client identifiers (BRUTE-01).
//! Note: Success automatically resets the failure counter for that IP.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: False positive blocks on shared NAT gateways,
//!   memory bloat from high IP churn (ephemeral attackers), or missing
//!   `ConnectInfo` in reverse proxy setups.
//! - **Trace Scope**: `server-rs::middleware::auth_rate_limit`

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

/// Tracks failed login attempts by IP.
/// Key: IP address (as string)
/// Value: (failure_count, last_failure_timestamp)
static AUTH_FAILURE_LOG: Lazy<DashMap<String, (u32, Instant)>> = Lazy::new(DashMap::new);

const MAX_FAILURES: u32 = 5;
const BLOCK_DURATION: Duration = Duration::from_secs(600); // 10 minutes

/// Middleware to prevent brute-force attacks by tracking failed authentication attempts.
pub async fn auth_brute_force_limiter(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|addr| addr.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // 1. Skip rate limiting for local loopback (desktop app safety)
    if client_ip == "127.0.0.1" || client_ip == "::1" {
        return Ok(next.run(req).await);
    }

    // 2. Check if the IP is currently blocked
    if let Some(entry) = AUTH_FAILURE_LOG.get(&client_ip) {
        let (count, last_attempt) = *entry;
        if count >= MAX_FAILURES && last_attempt.elapsed() < BLOCK_DURATION {
            tracing::warn!(
                "🚫 [Security] Brute-force block active for IP: {}. Cooling down.",
                client_ip
            );
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        // Auto-reset if the block duration has passed
        if last_attempt.elapsed() >= BLOCK_DURATION {
            drop(entry); // release read lock before write
            AUTH_FAILURE_LOG.remove(&client_ip);
        }
    }

    // 2. Proceed with the request
    let response = next.run(req).await;

    // 3. Inspect response for UNAUTHORIZED status
    if response.status() == StatusCode::UNAUTHORIZED {
        tracing::debug!("⚠️ [Security] Auth failure recorded for IP: {}", client_ip);

        let mut entry = AUTH_FAILURE_LOG
            .entry(client_ip)
            .or_insert((0, Instant::now()));
        entry.0 += 1;
        entry.1 = Instant::now();

        if entry.0 >= MAX_FAILURES {
            tracing::error!(
                "🚨 [Security] IP {} exceeded max auth failures. Blocking for 10m.",
                entry.key()
            );
        }
    } else if response.status().is_success() {
        // Reset failures on success
        AUTH_FAILURE_LOG.remove(&client_ip);
    }

    Ok(response)
}

/// Evicts auth failure records that have exceeded the block duration.
/// Called periodically by the background eviction task to prevent unbounded
/// memory growth from ephemeral attacker IPs.
pub fn evict_expired_blocks(max_age: std::time::Duration) {
    let before = AUTH_FAILURE_LOG.len();
    AUTH_FAILURE_LOG.retain(|_, (_, last_attempt)| last_attempt.elapsed() < max_age);
    let evicted = before - AUTH_FAILURE_LOG.len();
    if evicted > 0 {
        tracing::debug!(
            "🧹 [AuthRateLimit] Evicted {} expired block(s) ({} remaining)",
            evicted,
            AUTH_FAILURE_LOG.len()
        );
    }
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
        AUTH_FAILURE_LOG.clear();
    }
}
