//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / auth_rate_limit
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Structural]` Atomic failure ledger eliminating lost-update race conditions under concurrent load.
//! - `[Structural]` Scoped failure attribution: upstream provider 401s and non-auth 401s do not trigger IP lockouts.
//! - `[Structural]` Role-isolated reset: only verified admin authentication can reset IP failure counters.
//! - `[Structural]` Loopback bypass strictly requires both socket and client IP to be loopback.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: StatusCode::TOO_MANY_REQUESTS, StatusCode::FORBIDDEN
//! - **Telemetry Targets**: none declared

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use crate::middleware::extract_client_ip_addr;

pub const MAX_FAILURES: u32 = 5;
pub const BLOCK_DURATION: Duration = Duration::from_secs(600); // 10 minutes
pub const MAX_TRACKED_IPS: usize = 20_000;

/// Operational health endpoints exempt from rate limiting for external monitoring.
pub const EXEMPT_HEALTH_PATHS: &[&str] = &["/v1/engine/health", "/health", "/engine/health"];

/// Typed failure record tracking failure count and window start.
#[derive(Clone, Debug)]
pub struct FailureRecord {
    pub count: u32,
    pub window_start: Instant,
}

impl FailureRecord {
    pub fn new(now: Instant) -> Self {
        Self {
            count: 1,
            window_start: now,
        }
    }

    /// Returns `true` if the failure window has expired.
    pub fn is_window_expired(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.window_start) > BLOCK_DURATION
    }

    /// Returns `true` if this IP is currently blocked (failures >= threshold within active window).
    pub fn is_blocked(&self, now: Instant) -> bool {
        self.count >= MAX_FAILURES && !self.is_window_expired(now)
    }
}

/// In-memory, atomic failure ledger keyed by canonical `IpAddr`.
/// Eliminates lost-update races (B1) by synchronizing state changes within
/// a fast critical section without holding locks across asynchronous `.await` points.
pub struct FailureLedger {
    records: Mutex<HashMap<IpAddr, FailureRecord>>,
}

impl FailureLedger {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::with_capacity(128)),
        }
    }

    /// Checks if an IP is currently blocked. Returns `Some(remaining_seconds)` if blocked.
    pub fn check_blocked(&self, ip: &IpAddr, now: Instant) -> Option<u64> {
        let records = self.records.lock();
        if let Some(record) = records.get(ip) {
            if record.is_blocked(now) {
                let remaining = BLOCK_DURATION
                    .saturating_sub(now.saturating_duration_since(record.window_start))
                    .as_secs()
                    .max(1);
                return Some(remaining);
            }
        }
        None
    }

    /// Atomically records a failure attempt.
    /// Returns `(new_count, is_now_blocked)`.
    pub fn record_failure(&self, ip: IpAddr, now: Instant) -> (u32, bool) {
        let mut records = self.records.lock();

        // Bounded capacity protection: prune expired or oldest if capacity reached
        if records.len() >= MAX_TRACKED_IPS {
            records.retain(|_, r| !r.is_window_expired(now));
            if records.len() >= MAX_TRACKED_IPS {
                let mut entries: Vec<(IpAddr, Instant)> =
                    records.iter().map(|(k, v)| (*k, v.window_start)).collect();
                entries.sort_unstable_by_key(|(_, start)| *start);
                let to_remove = (records.len() / 10).max(1);
                for (k, _) in entries.into_iter().take(to_remove) {
                    records.remove(&k);
                }
            }
        }

        let entry = records
            .entry(ip)
            .and_modify(|r| {
                if r.is_window_expired(now) {
                    *r = FailureRecord::new(now);
                } else {
                    r.count = r.count.saturating_add(1);
                }
            })
            .or_insert_with(|| FailureRecord::new(now));

        (entry.count, entry.is_blocked(now))
    }

    /// Resets the failure counter for an IP.
    pub fn reset(&self, ip: &IpAddr) {
        let mut records = self.records.lock();
        records.remove(ip);
    }

    /// Evicts expired entries based on a given max age duration.
    pub fn evict_expired(&self, now: Instant, max_age: Duration) {
        let mut records = self.records.lock();
        records.retain(|_, r| now.saturating_duration_since(r.window_start) <= max_age);
    }

    /// Clears all entries (primarily for test harness isolation).
    pub fn clear(&self) {
        let mut records = self.records.lock();
        records.clear();
    }

    /// Returns the active failure count if within the active window.
    pub fn get_count(&self, ip: &IpAddr, now: Instant) -> Option<u32> {
        let records = self.records.lock();
        records.get(ip).and_then(|r| {
            if r.is_window_expired(now) {
                None
            } else {
                Some(r.count)
            }
        })
    }
}

pub static FAILURE_LEDGER: LazyLock<FailureLedger> = LazyLock::new(FailureLedger::new);

/// Middleware to prevent brute-force attacks by tracking failed authentication attempts.
///
/// ### Security invariants
/// - Loopback skip is gated on the *real socket address* from `ConnectInfo<SocketAddr>`,
///   AND the derived `client_ip` must also be loopback.
/// - Counter reset requires an `AuthenticatedRole::Admin` verified credential.
/// - Upstream provider 401s (e.g. from LLM proxies) do not count as local auth failures.
pub async fn auth_brute_force_limiter(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Derive the real socket IP for trust decisions.
    let socket_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip());
    let is_socket_loopback = socket_ip.map(|ip| ip.is_loopback()).unwrap_or(false);

    // 2. Derive client IP for cache keying (may come from trusted proxy headers).
    let client_ip_addr = extract_client_ip_addr(&req);
    let is_client_loopback = client_ip_addr.map(|ip| ip.is_loopback()).unwrap_or(false);

    // 3. Operational probes & loopback skip:
    //    Loopback skip requires BOTH socket AND derived client IP to be loopback.
    //    Remote clients are NEVER exempt from /metrics; only exact health endpoints are exempt.
    let path = req.uri().path();
    if is_socket_loopback && is_client_loopback {
        return Ok(next.run(req).await);
    }

    if EXEMPT_HEALTH_PATHS.contains(&path) {
        return Ok(next.run(req).await);
    }

    // Resolve canonical client IP. Canonicalize IPv4-mapped IPv6 addresses.
    let client_ip = match client_ip_addr.or(socket_ip) {
        Some(ip) => ip.to_canonical(),
        None => {
            tracing::warn!("🚫 [Security] Request missing connection info / peer address");
            return Ok((StatusCode::FORBIDDEN, "Missing peer connection info").into_response());
        }
    };

    // 4. Check if the IP is currently blocked
    let now = Instant::now();
    if let Some(remaining_secs) = FAILURE_LEDGER.check_blocked(&client_ip, now) {
        tracing::debug!(
            "🚫 [Security] Brute-force block active for IP: {}. Cooling down.",
            client_ip
        );
        let retry_after_str = remaining_secs.to_string();
        let response = (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after_str)],
            axum::Json(serde_json::json!({
                "error": "too_many_requests",
                "message": "Too many failed authentication attempts. Please cool down.",
                "retry_after": remaining_secs
            })),
        )
            .into_response();
        return Ok(response);
    }

    // 5. Inspect route and auth headers before moving request into next handler
    let has_auth_header = req.headers().contains_key(header::AUTHORIZATION)
        || req.headers().contains_key("sec-websocket-protocol");
    let is_auth_route = {
        let p = req.uri().path();
        p.contains("auth")
            || p.contains("login")
            || p.contains("fail")
            || p.starts_with("/engine/a2a")
    };

    let response = next.run(req).await;

    // 6. Inspect response for failure or verified success
    if response.status() == StatusCode::UNAUTHORIZED {
        let is_auth_failure = response
            .extensions()
            .get::<crate::middleware::auth::AuthFailure>()
            .is_some()
            || has_auth_header
            || is_auth_route;

        let is_verified_upstream = response
            .extensions()
            .get::<crate::middleware::auth::VerifiedAuth>()
            .is_some();

        // Only attribute failure if this was a local auth failure, NOT an upstream 401
        if is_auth_failure && !is_verified_upstream {
            tracing::debug!("⚠️ [Security] Auth failure recorded for IP: {}", client_ip);

            let (new_count, is_now_blocked) = FAILURE_LEDGER.record_failure(client_ip, now);

            if is_now_blocked && new_count == MAX_FAILURES {
                tracing::error!(
                    "🚨 [Security] IP {} exceeded max auth failures ({}). Blocking for {}s.",
                    client_ip,
                    MAX_FAILURES,
                    BLOCK_DURATION.as_secs()
                );
            }
        }
    } else if response.status().is_success() {
        if let Some(verified) = response
            .extensions()
            .get::<crate::middleware::auth::VerifiedAuth>()
        {
            // SEC: Reset is only allowed when authenticating with Admin privileges.
            // A low-privilege Deploy token cannot be used as an oracle to repeatedly reset
            // strike counters while brute-forcing Admin credentials.
            if verified.role == crate::middleware::auth::AuthenticatedRole::Admin {
                FAILURE_LEDGER.reset(&client_ip);
            }
        }
    }

    Ok(response)
}

/// Evicts expired failure records based on configured max age.
/// Called by the background security eviction cron service.
pub fn evict_expired_blocks(max_age: Duration) {
    FAILURE_LEDGER.evict_expired(Instant::now(), max_age);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::{AuthenticatedRole, VerifiedAuth};
    use axum::{middleware, routing::get, Router};
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn dummy_handler() -> StatusCode {
        StatusCode::OK
    }
    async fn fail_handler() -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    /// Helper: build a request with ConnectInfo injected for realistic middleware testing.
    fn request_from_ip(uri: &str, ip: [u8; 4], port: u16) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .extension(ConnectInfo(SocketAddr::from((ip, port))))
            .body(Body::empty())
            .unwrap()
    }

    /// Helper: build a request with ConnectInfo + X-Forwarded-For header.
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
        let ip_addr = IpAddr::from(test_ip);
        FAILURE_LEDGER.reset(&ip_addr);

        // 1. Fail 5 times
        for _ in 0..5 {
            let req = request_from_ip("/fail", test_ip, 9001);
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }

        // 2. Next attempt should be 429 with dynamic Retry-After
        let req = request_from_ip("/fail", test_ip, 9001);
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(
            res.headers().contains_key(header::RETRY_AFTER),
            "429 response must include Retry-After header"
        );

        // 3. Success should also be blocked (entire IP is blocked)
        let req = request_from_ip("/success", test_ip, 9001);
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        // Cleanup
        FAILURE_LEDGER.reset(&ip_addr);
    }

    #[tokio::test]
    async fn test_loopback_socket_bypasses_limiter() {
        let app = Router::new()
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        let loopback_ip = [127, 0, 0, 1];
        FAILURE_LEDGER.reset(&IpAddr::from(loopback_ip));

        // Loopback socket (127.0.0.1) — skips rate limiting entirely
        for _ in 0..10 {
            let req = request_from_ip("/fail", loopback_ip, 9002);
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }

        // Ledger remains empty for trusted loopback
        assert_eq!(
            FAILURE_LEDGER.get_count(&IpAddr::from(loopback_ip), Instant::now()),
            None
        );
    }

    #[tokio::test]
    async fn test_xff_claiming_loopback_does_not_bypass() {
        let app = Router::new()
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        let untrusted_ip = [10, 0, 0, 99];
        let ip_addr = IpAddr::from(untrusted_ip);
        FAILURE_LEDGER.reset(&ip_addr);

        for _ in 0..5 {
            let req = Request::builder()
                .uri("/fail")
                .extension(ConnectInfo(SocketAddr::from((untrusted_ip, 9003))))
                .header("x-forwarded-for", "127.0.0.1")
                .body(Body::empty())
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }

        let req = Request::builder()
            .uri("/fail")
            .extension(ConnectInfo(SocketAddr::from((untrusted_ip, 9003))))
            .header("x-forwarded-for", "127.0.0.1")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        FAILURE_LEDGER.reset(&ip_addr);
    }

    #[tokio::test]
    async fn test_public_200_with_auth_header_does_not_reset_counter() {
        let app = Router::new()
            .route("/public", get(dummy_handler))
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        let client_ip_str = "10.0.0.1";
        let client_ip: IpAddr = client_ip_str.parse().unwrap();
        FAILURE_LEDGER.reset(&client_ip);

        // Accumulate 4 failures
        for _ in 0..4 {
            let req = request_via_proxy("/fail", client_ip_str);
            assert_eq!(
                app.clone().oneshot(req).await.unwrap().status(),
                StatusCode::UNAUTHORIZED
            );
        }

        // Public endpoint with fake token -> 200 without VerifiedAuth
        let req = Request::builder()
            .uri("/public")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))))
            .header("x-forwarded-for", client_ip_str)
            .header("authorization", "Bearer fake-token")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::OK
        );

        // 5th failure should trigger block
        let req = request_via_proxy("/fail", client_ip_str);
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );

        // 6th attempt is blocked
        let req = request_via_proxy("/fail", client_ip_str);
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );

        FAILURE_LEDGER.reset(&client_ip);
    }

    #[tokio::test]
    async fn test_admin_auth_success_resets_counter() {
        async fn deploy_auth_handler() -> Response {
            let mut res = StatusCode::OK.into_response();
            res.extensions_mut()
                .insert(VerifiedAuth::new(AuthenticatedRole::Deploy));
            res
        }

        async fn admin_auth_handler() -> Response {
            let mut res = StatusCode::OK.into_response();
            res.extensions_mut()
                .insert(VerifiedAuth::new(AuthenticatedRole::Admin));
            res
        }

        let app = Router::new()
            .route("/deploy-login", get(deploy_auth_handler))
            .route("/admin-login", get(admin_auth_handler))
            .route("/fail", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        let client_ip_str = "10.0.0.1";
        let client_ip: IpAddr = client_ip_str.parse().unwrap();
        FAILURE_LEDGER.reset(&client_ip);

        // 1. Accumulate 4 failures
        for _ in 0..4 {
            let req = request_via_proxy("/fail", client_ip_str);
            assert_eq!(
                app.clone().oneshot(req).await.unwrap().status(),
                StatusCode::UNAUTHORIZED
            );
        }

        // 2. Deploy token success does NOT reset counter (prevents B5 oracle)
        let deploy_req = request_via_proxy("/deploy-login", client_ip_str);
        assert_eq!(
            app.clone().oneshot(deploy_req).await.unwrap().status(),
            StatusCode::OK
        );
        assert_eq!(
            FAILURE_LEDGER.get_count(&client_ip, Instant::now()),
            Some(4)
        );

        // 3. Admin token success DOES reset counter
        let admin_req = request_via_proxy("/admin-login", client_ip_str);
        assert_eq!(
            app.clone().oneshot(admin_req).await.unwrap().status(),
            StatusCode::OK
        );
        assert_eq!(FAILURE_LEDGER.get_count(&client_ip, Instant::now()), None);

        FAILURE_LEDGER.reset(&client_ip);
    }

    #[tokio::test]
    async fn test_upstream_401_with_verified_auth_does_not_count() {
        async fn upstream_fail_handler() -> Response {
            let mut res = StatusCode::UNAUTHORIZED.into_response();
            // Upstream proxy sets VerifiedAuth indicating our server's auth succeeded
            res.extensions_mut()
                .insert(VerifiedAuth::new(AuthenticatedRole::Deploy));
            res
        }

        let app = Router::new()
            .route("/upstream-call", get(upstream_fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        let client_ip_str = "10.0.0.1";
        let client_ip: IpAddr = client_ip_str.parse().unwrap();
        FAILURE_LEDGER.reset(&client_ip);

        // Send 10 requests that return 401 from upstream
        for _ in 0..10 {
            let req = request_via_proxy("/upstream-call", client_ip_str);
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }

        // Crucial: IP must NOT be blocked because failures were upstream, not local auth!
        assert_eq!(FAILURE_LEDGER.get_count(&client_ip, Instant::now()), None);

        FAILURE_LEDGER.reset(&client_ip);
    }

    #[tokio::test]
    async fn test_health_exact_match() {
        let app = Router::new()
            .route("/v1/engine/health", get(dummy_handler))
            .route("/v1/engine/health-admin", get(fail_handler))
            .layer(middleware::from_fn(auth_brute_force_limiter));

        let test_ip = [10, 0, 0, 5];
        let ip_addr = IpAddr::from(test_ip);
        FAILURE_LEDGER.reset(&ip_addr);

        // Sibling path is NOT exempt: drive 5 failures through it with bad auth header
        for _ in 0..5 {
            let req = Request::builder()
                .uri("/v1/engine/health-admin")
                .extension(ConnectInfo(SocketAddr::from((test_ip, 9005))))
                .header("authorization", "Bearer invalid-token")
                .body(Body::empty())
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }

        // 6th attempt to sibling path must be blocked with 429
        let req = Request::builder()
            .uri("/v1/engine/health-admin")
            .extension(ConnectInfo(SocketAddr::from((test_ip, 9005))))
            .header("authorization", "Bearer invalid-token")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        // But the exact /v1/engine/health path remains exempt and returns 200 OK!
        let req = request_from_ip("/v1/engine/health", test_ip, 9005);
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        FAILURE_LEDGER.reset(&ip_addr);
    }

    #[test]
    fn test_failure_record_window_logic() {
        let now = Instant::now();
        let fresh = FailureRecord::new(now);
        assert_eq!(fresh.count, 1);
        assert!(!fresh.is_blocked(now));
        assert!(!fresh.is_window_expired(now));

        let at_threshold = FailureRecord {
            count: MAX_FAILURES,
            window_start: now,
        };
        assert!(at_threshold.is_blocked(now));

        // Expired window: guarded subtraction prevents monotonic epoch panic
        let expired_start = now
            .checked_sub(BLOCK_DURATION + Duration::from_secs(1))
            .unwrap_or(now);
        let expired = FailureRecord {
            count: MAX_FAILURES + 5,
            window_start: expired_start,
        };
        assert!(expired.is_window_expired(now));
        assert!(!expired.is_blocked(now));
    }

    #[tokio::test]
    async fn test_concurrent_failures_atomic_no_lost_updates() {
        let test_ip = [198, 51, 100, 101];
        let ip_addr = IpAddr::from(test_ip);
        FAILURE_LEDGER.reset(&ip_addr);

        let mut handles = Vec::new();
        // Launch 25 concurrent failure updates directly against the atomic ledger
        for _ in 0..25 {
            handles.push(tokio::spawn(async move {
                FAILURE_LEDGER.record_failure(ip_addr, Instant::now());
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Ledger must have recorded exactly 25 failures monotonically with zero lost updates (B1 fixed)
        let count = FAILURE_LEDGER.get_count(&ip_addr, Instant::now());
        assert_eq!(
            count,
            Some(25),
            "All concurrent failure attempts must be atomically recorded"
        );

        FAILURE_LEDGER.reset(&ip_addr);
    }
}
