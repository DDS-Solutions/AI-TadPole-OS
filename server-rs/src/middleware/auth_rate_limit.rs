//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / auth_rate_limit
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Advisory: UNVERIFIED]` Security: loopback bypass gated on *socket* address (`ConnectInfo`), never on derived client IP.
//! - `[Advisory: UNVERIFIED]` Security: failure counter reset only on verified authentication (response extension), not header heuristics.
//! - `[Advisory: UNVERIFIED]` Correctness: failure windows reset after `BLOCK_DURATION`, preventing permanent soft-lock behind NAT.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Security]`
//! - **Witness Tests**: none declared

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use crate::middleware::{extract_client_ip, extract_client_ip_addr};

const MAX_FAILURES: u32 = 5;
const BLOCK_DURATION: Duration = Duration::from_secs(600); // 10 minutes

/// Typed failure record replacing the raw `(u32, Instant)` tuple.
/// Tracks both the failure count and the window start time for
/// deterministic expiry and window-based reset semantics.
#[derive(Clone, Debug)]
struct FailureRecord {
    count: u32,
    window_start: Instant,
}

impl FailureRecord {
    fn new() -> Self {
        Self {
            count: 1,
            window_start: Instant::now(),
        }
    }

    /// Returns `true` if the failure window has expired.
    fn is_window_expired(&self) -> bool {
        self.window_start.elapsed() > BLOCK_DURATION
    }

    /// Returns `true` if this IP is currently blocked (enough failures within a live window).
    fn is_blocked(&self) -> bool {
        self.count >= MAX_FAILURES && !self.is_window_expired()
    }
}

/// Tracks failed login attempts by IP.
/// Key: IP address (as string)
/// Value: [`FailureRecord`] with count and window start
///
/// Uses `moka` for automated TTL eviction and thread-safe atomicity.
/// Capacity note: under spoofable IPs, eviction pressure could let attackers
/// evict their own entries. The loopback-hardening fix (ConnectInfo-based skip)
/// mitigates the primary spoofing vector.
///
/// Collateral damage: all clients behind a shared NAT/corporate proxy share
/// one bucket. 5 shared failures locks out everyone behind that IP. This is
/// an accepted tradeoff for simplicity; a `(IP, target_account)` composite key
/// would reduce collateral but adds complexity.
static AUTH_FAILURE_LOG: LazyLock<moka::future::Cache<String, FailureRecord>> =
    LazyLock::new(|| {
        moka::future::Cache::builder()
            .max_capacity(20000)
            .time_to_live(BLOCK_DURATION)
            .build()
    });

/// Middleware to prevent brute-force attacks by tracking failed authentication attempts.
///
/// ### Security invariants
/// - Loopback skip is gated on the *real socket address* from `ConnectInfo<SocketAddr>`,
///   NOT the derived `client_ip` (which can be spoofed via trusted-proxy headers).
/// - Counter reset requires a `VerifiedAuth` response extension (set by `validate_token`
///   on genuine credential success), not just "Authorization header was present + 200."
pub async fn auth_brute_force_limiter(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();

    // 1. Derive the real socket IP for trust decisions.
    //    Only the actual peer address can qualify for the loopback skip —
    //    a header-declared 127.0.0.1 from an external client must NOT bypass the limiter.
    let socket_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip());
    let is_socket_loopback = socket_ip.map(|ip| ip.is_loopback()).unwrap_or(false);

    // 2. Derive client IP for cache keying (may come from trusted proxy headers).
    let client_ip = extract_client_ip(&req);

    // 3. Skip rate limiting only for genuine local clients and operational probes.
    //    Loopback skip requires BOTH the socket AND derived client IP to be loopback.
    //    A local reverse proxy (socket=127.0.0.1) forwarding external traffic
    //    (client_ip=external via XFF) must still rate-limit the external client.
    //    Health check uses exact match to prevent sibling paths from being exempted.
    let client_ip_addr = extract_client_ip_addr(&req);
    let is_client_loopback = client_ip_addr.map(|ip| ip.is_loopback()).unwrap_or(false);
    if (is_socket_loopback && is_client_loopback)
        || path == "/v1/engine/health"
        || path == "/metrics"
    {
        return Ok(next.run(req).await);
    }

    // 4. Check if the IP is currently blocked
    if let Some(record) = AUTH_FAILURE_LOG.get(&client_ip).await {
        if record.is_blocked() {
            // Downgraded to debug to prevent log spam from blocked IPs hammering the endpoint.
            // The initial block event is logged at error level (see step 6 below).
            tracing::debug!(
                "🚫 [Security] Brute-force block active for IP: {}. Cooling down.",
                client_ip
            );
            let response = (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, "600")],
            )
                .into_response();
            return Ok(response);
        }
    }

    // 5. Proceed with the request
    let response = next.run(req).await;

    // 6. Inspect response for failure or verified success
    if response.status() == StatusCode::UNAUTHORIZED {
        tracing::debug!("⚠️ [Security] Auth failure recorded for IP: {}", client_ip);

        // Atomic-ish update: get-then-insert. Under concurrent failures near the threshold,
        // two requests can both read count=4 and both write count=5. This errs slightly
        // in the attacker's favor (they might get 6 guesses instead of 5) — acceptable
        // for a rate limiter. The alternative (moka entry API) requires Clone on the closure
        // return which adds complexity for marginal benefit.
        let record = if let Some(existing) = AUTH_FAILURE_LOG.get(&client_ip).await {
            if existing.is_window_expired() {
                // Window expired: reset to fresh window with count=1.
                // This prevents the "1-failure-re-blocks-forever" emergent behavior
                // where an expired entry with count>=5 gets incremented to 6 on
                // the next failure, immediately re-blocking.
                FailureRecord::new()
            } else {
                FailureRecord {
                    count: existing.count + 1,
                    window_start: existing.window_start,
                }
            }
        } else {
            FailureRecord::new()
        };

        let should_block = record.count >= MAX_FAILURES;
        AUTH_FAILURE_LOG.insert(client_ip.clone(), record).await;

        if should_block {
            tracing::error!(
                "🚨 [Security] IP {} exceeded max auth failures ({}). Blocking for {}s.",
                client_ip,
                MAX_FAILURES,
                BLOCK_DURATION.as_secs()
            );
        }
    } else if response.status().is_success() {
        // Only reset the counter when authentication *actually succeeded* —
        // verified by the presence of a VerifiedAuth response extension
        // (set by validate_token middleware on genuine credential validation).
        //
        // This prevents the "reset oracle" attack where an attacker sends
        // `Authorization: garbage` to a public endpoint returning 200,
        // which previously wiped the failure counter.
        if response
            .extensions()
            .get::<crate::middleware::auth::VerifiedAuth>()
            .is_some()
        {
            AUTH_FAILURE_LOG.invalidate(&client_ip).await;
        }
    }

    Ok(response)
}

/// No longer manually evicts entries since `moka` handles TTL eviction automatically.
/// Kept as a no-op to maintain signature compatibility with the background cron scheduler
/// in [`crate::startup::services::security`].
#[allow(dead_code)]
pub fn evict_expired_blocks(_max_age: std::time::Duration) {
    // moka handles this automatically via time_to_live.
    // The cron scheduler still calls this function; removing it would require
    // updating the scheduler registration. Harmless to keep.
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

    /// Helper: build a request with ConnectInfo injected for realistic middleware testing.
    /// Uses a non-loopback IP by default so the limiter is active.
    fn request_from_ip(uri: &str, ip: [u8; 4], port: u16) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .extension(ConnectInfo(SocketAddr::from((ip, port))))
            .body(Body::empty())
            .unwrap()
    }

    /// Helper: build a request with ConnectInfo + X-Forwarded-For header.
    /// Socket IP is set to a trusted proxy (127.0.0.1) so XFF is honored.
    fn request_via_proxy(uri: &str, xff_ip: &str) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))))
            .header("x-forwarded-for", xff_ip)
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn test_brute_force_blocking() {
        let app = Router::new()
            .route("/success", get(dummy_handler))
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        let test_ip = [10, 0, 0, 1];

        // 1. Fail 5 times
        for _ in 0..5 {
            let req = request_from_ip("/fail", test_ip, 9001);
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }

        // 2. Next attempt should be 429 with Retry-After
        let req = request_from_ip("/fail", test_ip, 9001);
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            res.headers()
                .get(header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok()),
            Some("600"),
            "429 response must include Retry-After header"
        );

        // 3. Success should also be blocked (entire IP is blocked)
        let req = request_from_ip("/success", test_ip, 9001);
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        // Cleanup
        AUTH_FAILURE_LOG.invalidate("10.0.0.1").await;
    }

    #[tokio::test]
    async fn test_loopback_socket_bypasses_limiter() {
        let app = Router::new()
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        // Loopback socket (127.0.0.1) — should skip rate limiting entirely
        for _ in 0..10 {
            let req = request_from_ip("/fail", [127, 0, 0, 1], 9002);
            let res = app.clone().oneshot(req).await.unwrap();
            // Never gets 429 — loopback is trusted
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn test_xff_claiming_loopback_does_not_bypass() {
        let app = Router::new()
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        // External socket (10.0.0.1) with XFF claiming 127.0.0.1.
        // The limiter must NOT skip — socket IP is what matters for trust.
        for _ in 0..5 {
            let req = Request::builder()
                .uri("/fail")
                .extension(ConnectInfo(SocketAddr::from(([10, 0, 0, 99], 9003))))
                .header("x-forwarded-for", "127.0.0.1")
                .body(Body::empty())
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }

        // Should be blocked now — the loopback claim didn't help
        let req = Request::builder()
            .uri("/fail")
            .extension(ConnectInfo(SocketAddr::from(([10, 0, 0, 99], 9003))))
            .header("x-forwarded-for", "127.0.0.1")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        // Cleanup — the cache key is the *derived* client_ip.
        // Since socket 10.0.0.1 is untrusted, XFF is ignored → key is "10.0.0.1".
        AUTH_FAILURE_LOG.invalidate("10.0.0.1").await;
    }

    #[tokio::test]
    async fn test_public_200_with_auth_header_does_not_reset_counter() {
        // This tests the "reset oracle" fix: sending `Authorization: garbage`
        // to a public endpoint returning 200 must NOT wipe the failure counter.
        let app = Router::new()
            .route("/public", get(dummy_handler))
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        let client_ip = "10.0.0.1";

        // Accumulate 4 failures
        for _ in 0..4 {
            let req = request_via_proxy("/fail", client_ip);
            assert_eq!(
                app.clone().oneshot(req).await.unwrap().status(),
                StatusCode::UNAUTHORIZED
            );
        }

        // Hit a public endpoint with a garbage Authorization header → 200
        // Without the fix, this would reset the counter via `presented_credentials`
        let req = Request::builder()
            .uri("/public")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))))
            .header("x-forwarded-for", client_ip)
            .header("authorization", "Bearer totally-fake-token")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::OK
        );

        // 5th failure should still block — counter was NOT reset by the 200
        let req = request_via_proxy("/fail", client_ip);
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );

        // 6th attempt should be blocked
        let req = request_via_proxy("/fail", client_ip);
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );

        AUTH_FAILURE_LOG.invalidate(client_ip).await;
    }

    #[tokio::test]
    async fn test_public_remote_ping_does_not_reset_failures() {
        let app = Router::new()
            .route("/v1/remote/ping", get(dummy_handler))
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));
        let client_ip = "10.0.0.1";

        for _ in 0..4 {
            let req = request_via_proxy("/fail", client_ip);
            assert_eq!(
                app.clone().oneshot(req).await.unwrap().status(),
                StatusCode::UNAUTHORIZED
            );
        }

        // Public ping — no VerifiedAuth extension → should NOT reset counter
        let ping = request_via_proxy("/v1/remote/ping", client_ip);
        assert_eq!(
            app.clone().oneshot(ping).await.unwrap().status(),
            StatusCode::OK
        );

        let fifth_failure = request_via_proxy("/fail", client_ip);
        assert_eq!(
            app.clone().oneshot(fifth_failure).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );

        let blocked = request_via_proxy("/fail", client_ip);
        assert_eq!(
            app.oneshot(blocked).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );

        AUTH_FAILURE_LOG.invalidate(client_ip).await;
    }

    #[tokio::test]
    async fn test_health_exact_match() {
        let app = Router::new()
            .route("/v1/engine/health", get(dummy_handler))
            .route("/v1/engine/health-admin", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        // Exact health path → exempted (always 200)
        let req = request_from_ip("/v1/engine/health", [10, 0, 0, 5], 9005);
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // Sibling path → NOT exempted (goes through limiter, can return 401)
        let req = request_from_ip("/v1/engine/health-admin", [10, 0, 0, 5], 9005);
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_failure_record_window_logic() {
        // Fresh record: not blocked, window not expired
        let fresh = FailureRecord::new();
        assert_eq!(fresh.count, 1);
        assert!(!fresh.is_blocked());
        assert!(!fresh.is_window_expired());

        // At threshold: blocked
        let at_threshold = FailureRecord {
            count: MAX_FAILURES,
            window_start: Instant::now(),
        };
        assert!(at_threshold.is_blocked());

        // Expired window: not blocked even with high count
        let expired = FailureRecord {
            count: MAX_FAILURES + 5,
            window_start: Instant::now() - BLOCK_DURATION - Duration::from_secs(1),
        };
        assert!(expired.is_window_expired());
        assert!(
            !expired.is_blocked(),
            "expired window should not be blocked"
        );
    }
}
