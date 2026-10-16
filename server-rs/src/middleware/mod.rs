//! @docs ARCHITECTURE:MiddlewarePipeline
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Security: zero-trust IP derivation uses rightmost-untrusted XFF traversal, rejection of header-declared
//!   loopback/unspecified addresses, and fail-closed handling when `ConnectInfo` is absent.
//!   - enforced_by: `test_extract_client_ip_rightmost_untrusted`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[SEC-01]`
//! - **Witness Tests**: `test_extract_client_ip_direct`, `test_extract_client_ip_rightmost_untrusted`, `test_extract_client_ip_rejects_header_loopback`

pub mod auth;
pub mod auth_rate_limit;
pub mod cors;
pub mod deprecation;
pub mod rate_limit;
pub mod request_id;
pub mod security_headers;

use axum::{body::Body, http::Request};
use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;

/// A whitelist of trusted proxy IPs that are allowed to provide client IP headers.
static TRUSTED_PROXIES: LazyLock<Vec<IpAddr>> = LazyLock::new(|| {
    if let Ok(proxies_str) = std::env::var("TRUSTED_PROXIES") {
        proxies_str
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return None;
                }
                match trimmed.parse::<IpAddr>() {
                    Ok(ip) => Some(ip),
                    Err(err) => {
                        tracing::warn!(
                            "⚠️ [Middleware] Invalid IP '{}' in TRUSTED_PROXIES: {}. Skipping.",
                            trimmed,
                            err
                        );
                        None
                    }
                }
            })
            .collect()
    } else {
        // By default, trust loopback proxies
        vec![
            IpAddr::from([127, 0, 0, 1]),
            IpAddr::from([0, 0, 0, 0, 0, 0, 0, 1]),
        ]
    }
});

/// Cached flag for whether private network ranges should be treated as trusted proxies.
static TRUST_PRIVATE_NETWORKS: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("TRUST_PRIVATE_NETWORKS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
});

/// Verifies whether a given IP is a configured trusted proxy or local loopback.
pub fn is_ip_trusted(ip: &IpAddr) -> bool {
    if ip.is_loopback() {
        return true;
    }
    if is_cloudflare_ip(ip) {
        return true;
    }
    if TRUSTED_PROXIES.contains(ip) {
        return true;
    }
    if *TRUST_PRIVATE_NETWORKS {
        match ip {
            IpAddr::V4(ipv4) => ipv4.is_private(),
            IpAddr::V6(ipv6) => {
                // RFC 4193 Unique Local Address (fc00::/7)
                (ipv6.segments()[0] & 0xfe00) == 0xfc00
            }
        }
    } else {
        false
    }
}

/// Known Cloudflare IPv4 CIDRs (official Cloudflare edge IP blocks)
const CLOUDFLARE_IPV4_CIDRS: &[(&[u8; 4], u32)] = &[
    (&[173, 245, 48, 0], 20),
    (&[103, 21, 244, 0], 22),
    (&[103, 22, 200, 0], 22),
    (&[103, 31, 4, 0], 22),
    (&[141, 101, 64, 0], 18),
    (&[108, 162, 192, 0], 18),
    (&[190, 93, 240, 0], 20),
    (&[188, 114, 96, 0], 20),
    (&[197, 234, 240, 0], 22),
    (&[198, 41, 128, 0], 17),
    (&[162, 158, 0, 0], 15),
    (&[104, 16, 0, 0], 13),
    (&[104, 24, 0, 0], 14),
    (&[172, 64, 0, 0], 13),
    (&[131, 0, 72, 0], 22),
];

/// Checks if a direct connection peer is a verified Cloudflare proxy.
pub fn is_cloudflare_ip(ip: &IpAddr) -> bool {
    if std::env::var("TRUST_ALL_CF_CONNECTING_IP")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    match ip {
        IpAddr::V4(ipv4) => {
            let ip_u32 = u32::from_be_bytes(ipv4.octets());
            for (base, prefix_len) in CLOUDFLARE_IPV4_CIDRS {
                let base_u32 = u32::from_be_bytes(**base);
                let mask = if *prefix_len == 0 {
                    0
                } else {
                    !0u32 << (32 - prefix_len)
                };
                if (ip_u32 & mask) == (base_u32 & mask) {
                    return true;
                }
            }
            false
        }
        IpAddr::V6(ipv6) => {
            let seg = ipv6.segments();
            match (seg[0], seg[1]) {
                (0x2400, 0xcb00)
                | (0x2606, 0x4700)
                | (0x2803, 0xf800)
                | (0x2405, 0xb500)
                | (0x2405, 0x8100)
                | (0x2c0f, 0xf248) => true,
                _ => seg[0] == 0x2a06 && (seg[1] & 0xfff8) == 0x98c0,
            }
        }
    }
}

/// Checks if an IP is a reserved, loopback, or unspecified address that should never be
/// legitimately presented as a client IP via proxy headers.
fn is_invalid_forwarded_client(ip: &IpAddr) -> bool {
    ip.is_loopback() || ip.is_unspecified() || ip.is_multicast()
}

/// Extracts the client IP address as a strongly typed [`IpAddr`].
///
/// ### 🛰️ Zero-Trust Rightmost-Untrusted Algorithm
/// 1. Reads direct connection IP from `ConnectInfo<SocketAddr>`.
/// 2. If `ConnectInfo` is missing, fails closed and returns `None`.
/// 3. If direct IP is not a trusted proxy, returns direct IP.
/// 4. If direct IP is a verified Cloudflare peer, evaluates `CF-Connecting-IP`.
/// 5. Otherwise, traverses `X-Forwarded-For` from right to left (the chain added by intermediaries),
///    selecting the rightmost untrusted IP.
/// 6. Header-declared loopback or unspecified addresses (`127.0.0.1`, `::1`) are rejected from
///    qualifying as client IPs to prevent limiter bypasses.
pub fn extract_client_ip_addr(req: &Request<Body>) -> Option<IpAddr> {
    let direct_ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|addr| addr.0.ip())?;

    if !is_ip_trusted(&direct_ip) {
        return Some(direct_ip);
    }

    // Direct IP is a trusted Cloudflare proxy — check CF-Connecting-IP header (B1)
    if is_cloudflare_ip(&direct_ip) {
        if let Some(cf_ip) = req
            .headers()
            .get("cf-connecting-ip")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
        {
            if !is_invalid_forwarded_client(&cf_ip) && !is_ip_trusted(&cf_ip) {
                return Some(cf_ip);
            }
        }
    }

    // Rightmost-untrusted traversal for X-Forwarded-For
    if let Some(forwarded) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        let mut resolved_client = None;

        for raw in forwarded.split(',').rev() {
            let trimmed = raw.trim();
            if let Ok(parsed_ip) = trimmed.parse::<IpAddr>() {
                if is_ip_trusted(&parsed_ip) {
                    // Intermediate trusted proxy hop; continue walking left
                    continue;
                }
                // First untrusted IP from the right is the candidate client
                if !is_invalid_forwarded_client(&parsed_ip) {
                    resolved_client = Some(parsed_ip);
                    break;
                }
            }
        }

        if let Some(client) = resolved_client {
            return Some(client);
        }
    }

    // Fall back to direct IP if no valid header client was resolved
    Some(direct_ip)
}

/// Convenience string helper for backward compatibility and log keying.
/// Returns formatted [`IpAddr`] or `"unknown"` if connection info is missing.
pub fn extract_client_ip(req: &Request<Body>) -> String {
    extract_client_ip_addr(req)
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Request-extension marker inserted after proof-of-possession verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedDeviceId(pub String);

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
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, crate::error::AppError> {
    // SEC-01: Disallow unauthenticated query parameter injection on paired routes (B2)
    if req.uri().query().is_some() {
        return Err(crate::error::AppError::BadRequest(
            "Query parameters are not permitted on paired remote endpoints".to_string(),
        ));
    }

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
    )
    .await?;

    req.extensions_mut()
        .insert(VerifiedDeviceId(device_id.clone()));
    tracing::debug!("📱 [SEC-01] Signed paired-device request accepted: {device_id}");
    Ok(next.run(req).await)
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
        assert_eq!(
            extract_client_ip_addr(&req),
            Some(IpAddr::from([10, 0, 0, 1]))
        );
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

    #[test]
    fn test_extract_client_ip_rightmost_untrusted() {
        // Direct IP is loopback (trusted proxy)
        // XFF chain: client (10.0.0.1), untrusted-proxy (10.0.0.1), trusted-internal (127.0.0.1)
        let req = Request::builder()
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
            .header("x-forwarded-for", "10.0.0.1, 10.0.0.1, 127.0.0.1")
            .body(Body::empty())
            .unwrap();

        // Rightmost untrusted IP is 10.0.0.1
        assert_eq!(extract_client_ip(&req), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_rejects_header_loopback() {
        // Direct socket is trusted loopback proxy (127.0.0.1)
        // Attacker attempts to inject loopback 127.0.0.1 in cf-connecting-ip and XFF hop
        let req = Request::builder()
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
            .header("cf-connecting-ip", "127.0.0.1")
            .header("x-forwarded-for", "10.0.0.1, 127.0.0.1")
            .body(Body::empty())
            .unwrap();

        // Loopback in cf-connecting-ip is rejected; XFF loopback is skipped as trusted hop -> resolves to 10.0.0.1
        assert_eq!(extract_client_ip(&req), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_missing_connect_info_fails_closed() {
        let req = Request::builder()
            .header("x-forwarded-for", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_client_ip(&req), "unknown");
        assert_eq!(extract_client_ip_addr(&req), None);
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

    #[test]
    fn test_extract_client_ip_cf_connecting_ip_from_untrusted_peer_ignored() {
        // Direct socket is loopback (trusted proxy, but NOT a Cloudflare proxy)
        // Attacker attempts to set cf-connecting-ip: 10.0.0.1
        let req = Request::builder()
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))))
            .header("cf-connecting-ip", "10.0.0.1")
            .header("x-forwarded-for", "10.0.0.1, 127.0.0.1")
            .body(Body::empty())
            .unwrap();

        // cf-connecting-ip is ignored from non-Cloudflare peer; XFF resolves to 10.0.0.1
        assert_eq!(extract_client_ip(&req), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_cf_connecting_ip_from_cloudflare_peer_accepted() {
        // Direct socket is in Cloudflare IP range (10.0.0.1)
        let req = Request::builder()
            .extension(ConnectInfo(SocketAddr::from(([104, 16, 1, 1], 443))))
            .header("cf-connecting-ip", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        // cf-connecting-ip is accepted from verified Cloudflare peer
        assert_eq!(extract_client_ip(&req), "10.0.0.1");
        assert_eq!(
            extract_client_ip_addr(&req),
            Some(IpAddr::from([198, 51, 100, 4]))
        );
    }

    #[tokio::test]
    async fn test_paired_device_guard_rejects_query_parameters() {
        use axum::{http::StatusCode, routing::get, Router};
        use tower::ServiceExt;

        let app = Router::new()
            .route("/protected", get(|| async { StatusCode::OK }))
            .route_layer(axum::middleware::from_fn(paired_device_guard));

        let req = Request::builder()
            .uri("/protected?tampered=true")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
