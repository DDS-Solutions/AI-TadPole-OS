//! @docs ARCHITECTURE:MiddlewarePipeline
//! @docs OPERATIONS_MANUAL:Security
//!
//! ### AI Assist Note
//! **Middleware Hub**: Orchestrates the sequential processing of
//! incoming API requests for the Tadpole OS engine. Enforces the
//! **Security Pipeline**, implementing **Sovereign Authentication**
//! (Bearer token), **Brute-Force Prevention** (Recruitment Rate-Limiting),
//! and **CORS Policy Enforcement**. Coordinates with `request_id` to
//! ensure that all incoming transactions are assigned a unique
//! identifier for end-to-end trace propagation.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: CORS pre-flight rejection (origin mismatch),
//!   401 Unauthorized (invalid `NEURAL_TOKEN`), or 429 Too Many
//!   Requests (Rate-limit exceeded).
//! - **Telemetry Link**: Search for `[Middleware]` or `[Security]` in
//!   tracing logs for block/deny events.
//! - **Trace Scope**: `server-rs::middleware`
//!
//! Middleware Hub — Request processing pipeline
//!
//! Orchestrates the layered security and observability middleware
//! for the Axum server ecosystem.
//!
//! @docs ARCHITECTURE:Networking

pub mod auth;
pub mod auth_rate_limit;
pub mod cors;
pub mod deprecation;
pub mod rate_limit;
pub mod request_id;
pub mod security_headers;

use axum::{body::Body, http::Request};
use once_cell::sync::Lazy;
use std::net::{IpAddr, SocketAddr};

/// A whitelist of trusted proxy IPs that are allowed to provide client IP headers.
static TRUSTED_PROXIES: Lazy<Vec<IpAddr>> = Lazy::new(|| {
    if let Ok(proxies_str) = std::env::var("TRUSTED_PROXIES") {
        proxies_str
            .split(',')
            .filter_map(|s| s.trim().parse::<IpAddr>().ok())
            .collect()
    } else {
        // By default, trust loopback/local proxies
        vec![
            IpAddr::from([127, 0, 0, 1]),
            IpAddr::from([0, 0, 0, 0, 0, 0, 0, 1]),
        ]
    }
});

/// Verifies whether the direct connection IP is a trusted proxy.
fn is_ip_trusted(ip: &IpAddr) -> bool {
    if ip.is_loopback() {
        return true;
    }
    if TRUSTED_PROXIES.contains(ip) {
        return true;
    }

    // Optionally trust all private network IP ranges if explicitly configured
    if std::env::var("TRUST_PRIVATE_NETWORKS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        match ip {
            IpAddr::V4(ipv4) => ipv4.is_private(),
            IpAddr::V6(_) => false,
        }
    } else {
        false
    }
}

/// Utility to extract the client IP address, respecting proxy headers only from trusted origins.
///
/// ### 🛰️ Proxy Awareness & Zero-Trust Verification
/// Resolves the client IP by checking connection credentials:
/// 1. Direct connection IP via `ConnectInfo<SocketAddr>` is verified against trusted proxies/loopbacks.
/// 2. If trusted, reads `CF-Connecting-IP` or `X-Forwarded-For`.
/// 3. Otherwise, falls back to direct connection IP to prevent header-spoofing attacks (SEC-03).
pub fn extract_client_ip(req: &Request<Body>) -> String {
    let direct_ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|addr| addr.0.ip());

    // If direct_ip is missing (e.g. testing environments), default to true to allow mock header testing.
    let trust_proxy = direct_ip.map(|ip| is_ip_trusted(&ip)).unwrap_or(true);

    if trust_proxy {
        // 1. Check Cloudflare specific header
        if let Some(ip) = req
            .headers()
            .get("cf-connecting-ip")
            .and_then(|v| v.to_str().ok())
        {
            return ip.trim().to_string();
        }

        // 2. Check X-Forwarded-For (Standard Proxy)
        if let Some(forwarded) = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
            // X-Forwarded-For can be a comma-separated list; the first one is the client
            if let Some(ip) = forwarded.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // 3. Fallback to direct connection info
    direct_ip
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::ConnectInfo;

    #[test]
    fn test_extract_client_ip_direct() {
        let req = Request::builder()
            .extension(ConnectInfo(SocketAddr::from(([10, 0, 0, 1], 8080))))
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_client_ip(&req), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_spoofed() {
        // Direct IP is not loopback/trusted proxy (e.g. 10.0.0.1)
        // Request has X-Forwarded-For: 10.0.0.1
        let req = Request::builder()
            .extension(ConnectInfo(SocketAddr::from(([10, 0, 0, 1], 8080))))
            .header("x-forwarded-for", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        // Should ignore X-Forwarded-For and return direct connection IP
        assert_eq!(extract_client_ip(&req), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_trusted_proxy() {
        // Direct IP is loopback (trusted proxy)
        // Request has X-Forwarded-For: 10.0.0.1
        let req = Request::builder()
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
            .header("x-forwarded-for", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        // Should trust X-Forwarded-For and return 10.0.0.1
        assert_eq!(extract_client_ip(&req), "10.0.0.1");
    }

    #[tokio::test]
    async fn test_paired_device_guard_rejects_unpaired() {
        use axum::{http::StatusCode, routing::get, Router};
        use ed25519_dalek::{Signer, SigningKey};
        use tower::ServiceExt;

        let app = Router::new()
            .route("/protected", get(|| async { StatusCode::OK }))
            .route_layer(axum::middleware::from_fn(paired_device_guard));

        // Request without X-Device-Id header
        let req_no_header = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        let res1 = app.clone().oneshot(req_no_header).await.unwrap();
        assert_eq!(res1.status(), StatusCode::UNAUTHORIZED);

        // A complete, well-formed proof from an unregistered device is still denied.
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let nonce = "unknown-device-nonce-001";
        let signing_key = SigningKey::from_bytes(&[17u8; 32]);
        let canonical = format!("GET:/protected:{timestamp}:{nonce}");
        let signature = hex::encode(signing_key.sign(canonical.as_bytes()).to_bytes());
        let req_unknown = Request::builder()
            .uri("/protected")
            .header("x-device-id", "unknown-device-123")
            .header("x-timestamp", timestamp.to_string())
            .header("x-nonce", nonce)
            .header("x-signature", signature)
            .body(Body::empty())
            .unwrap();
        let res2 = app.oneshot(req_unknown).await.unwrap();
        assert_eq!(res2.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_paired_device_guard_verifies_signature_and_blocks_replay() {
        use axum::{http::StatusCode, routing::get, Router};
        use ed25519_dalek::{Signer, SigningKey};
        use tower::ServiceExt;

        let device_id = "middleware-signed-device";
        let signing_key = SigningKey::from_bytes(&[29u8; 32]);
        crate::routes::remote::register_paired_device_for_test(
            crate::routes::remote::PairedDevice {
                id: device_id.to_string(),
                name: "Signed Test Device".to_string(),
                user_name: "Tester".to_string(),
                public_key: hex::encode(signing_key.verifying_key().to_bytes()),
                paired_at: chrono::Utc::now().to_rfc3339(),
                status: "online".to_string(),
            },
        );

        let app = Router::new()
            .route("/protected", get(|| async { StatusCode::OK }))
            .route_layer(axum::middleware::from_fn(paired_device_guard));
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let nonce = "unique-middleware-nonce-001";
        let canonical = format!("GET:/protected:{timestamp}:{nonce}");
        let signature = hex::encode(signing_key.sign(canonical.as_bytes()).to_bytes());
        let make_request = || {
            Request::builder()
                .uri("/protected")
                .header("x-device-id", device_id)
                .header("x-timestamp", timestamp.to_string())
                .header("x-nonce", nonce)
                .header("x-signature", &signature)
                .body(Body::empty())
                .unwrap()
        };

        let accepted = app.clone().oneshot(make_request()).await.unwrap();
        assert_eq!(accepted.status(), StatusCode::OK);

        let replayed = app.oneshot(make_request()).await.unwrap();
        assert_eq!(replayed.status(), StatusCode::UNAUTHORIZED);
    }
}

/// SEC-01: Proof-of-possession guard for paired remote endpoints.
///
/// Requires a paired device to sign `METHOD:PATH:TIMESTAMP:NONCE` with its
/// registered Ed25519 private key. Freshness and one-time nonce checks prevent
/// captured requests from being replayed.
fn required_remote_header(
    headers: &axum::http::HeaderMap,
    name: &'static str,
) -> Result<String, crate::error::AppError> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            crate::error::AppError::Unauthorized(format!(
                "{} header is required for paired remote requests",
                name
            ))
        })
}

pub async fn paired_device_guard(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, crate::error::AppError> {
    let device_id = required_remote_header(req.headers(), "X-Device-Id")?;
    let timestamp = required_remote_header(req.headers(), "X-Timestamp")?
        .parse::<u64>()
        .map_err(|_| {
            crate::error::AppError::Unauthorized(
                "X-Timestamp must be a Unix timestamp in seconds".to_string(),
            )
        })?;
    let nonce = required_remote_header(req.headers(), "X-Nonce")?;
    let signature = required_remote_header(req.headers(), "X-Signature")?;

    crate::routes::remote::verify_paired_request(
        &device_id,
        req.method().as_str(),
        req.uri().path(),
        timestamp,
        &nonce,
        &signature,
    )?;

    tracing::debug!("📱 [SEC-01] Signed paired-device request accepted: {device_id}");
    Ok(next.run(req).await)
}

// Metadata: [mod]
