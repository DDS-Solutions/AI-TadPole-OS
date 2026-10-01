//! @docs ARCHITECTURE:Security:SSRFGuard
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security / SSRFGuard
//! - **Primary Entrypoints**: `validate_public_http_url`, `ValidatedUrl`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Validates outbound URLs before fetching; rejects private/local/reserved/CGNAT/loopback IP spaces.
//! - `[Structural]` Prevents DNS rebinding by resolving and retaining target IP.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use std::net::IpAddr;

pub(crate) fn is_blocked_public_fetch_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    host == "localhost"
        || host.ends_with(".localhost")
        || host == "host.docker.internal"
        || host.ends_with(".local")
}

pub(crate) fn is_public_routable_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_broadcast()
                || ip.octets()[0] == 0
                || ip.octets()[0] >= 224
                || is_ipv4_cgnat(ip))
        }
        IpAddr::V6(ip) => {
            // 1. Check IPv4-mapped (::ffff:x.x.x.x) or IPv4-compatible (::x.x.x.x) IPv6
            if let Some(v4) = ip.to_ipv4_mapped() {
                return is_public_routable_ip(IpAddr::V4(v4));
            }
            if let Some(v4) = ip.to_ipv4() {
                return is_public_routable_ip(IpAddr::V4(v4));
            }

            let segs = ip.segments();
            // 2. Check NAT64 (64:ff9b::/96)
            if segs[0] == 0x0064
                && segs[1] == 0xff9b
                && segs[2] == 0
                && segs[3] == 0
                && segs[4] == 0
                && segs[5] == 0
            {
                let v4 = std::net::Ipv4Addr::new(
                    (segs[6] >> 8) as u8,
                    (segs[6] & 0xff) as u8,
                    (segs[7] >> 8) as u8,
                    (segs[7] & 0xff) as u8,
                );
                return is_public_routable_ip(IpAddr::V4(v4));
            }

            // 3. Check 6to4 (2002::/16)
            if segs[0] == 0x2002 {
                let v4 = std::net::Ipv4Addr::new(
                    (segs[1] >> 8) as u8,
                    (segs[1] & 0xff) as u8,
                    (segs[2] >> 8) as u8,
                    (segs[2] & 0xff) as u8,
                );
                return is_public_routable_ip(IpAddr::V4(v4));
            }

            !(ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || is_ipv6_unique_local(ip)
                || is_ipv6_unicast_link_local(ip))
        }
    }
}

pub(crate) fn is_ipv4_cgnat(ip: std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (64..=127).contains(&octets[1])
}

pub(crate) fn is_ipv6_unique_local(ip: std::net::Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

pub(crate) fn is_ipv6_unicast_link_local(ip: std::net::Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

/// A validated public outbound HTTP/HTTPS target.
/// Used to prevent DNS rebinding by forcing caller connections to the resolved IP.
#[derive(Debug, Clone)]
pub struct ValidatedUrl {
    pub ip: IpAddr,
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]
    pub url: reqwest::Url,
}

/// Validates an agent/user-controlled outbound URL before the engine fetches it.
/// Blocks local, private, link-local, multicast, and metadata-style targets after DNS resolution.
/// Returns a `ValidatedUrl` struct, which contains the resolved IP, hostname, and port.
/// Callers must use the resolved `ValidatedUrl` elements to perform the network request, rather than the original string, to prevent DNS rebinding.
pub async fn validate_public_http_url(url: &str) -> Result<ValidatedUrl, AppError> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|_| AppError::BadRequest("URL must be absolute and valid".to_string()))?;

    match parsed.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(AppError::Forbidden(
                "Only http and https URLs are allowed for outbound fetches".to_string(),
            ));
        }
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(AppError::Forbidden(
            "URL credentials are not allowed for outbound fetches".to_string(),
        ));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::BadRequest("URL host is required".to_string()))?;

    if is_blocked_public_fetch_host(host) {
        return Err(AppError::Forbidden(
            "Local or internal hostnames cannot be fetched by agents".to_string(),
        ));
    }

    let host_str = host.to_string();

    let port = parsed.port_or_known_default().ok_or_else(|| {
        AppError::BadRequest("URL must use a scheme with a known port".to_string())
    })?;

    if let Ok(ip) = host_str.parse::<IpAddr>() {
        if is_public_routable_ip(ip) {
            return Ok(ValidatedUrl {
                ip,
                host: host_str,
                port,
                url: parsed,
            });
        }
        return Err(AppError::Forbidden(
            "Local, private, or reserved IP addresses cannot be fetched by agents".to_string(),
        ));
    }

    let lookup_future = tokio::net::lookup_host((host_str.clone(), port));
    let mut addrs =
        match tokio::time::timeout(std::time::Duration::from_secs(3), lookup_future).await {
            Ok(Ok(a)) => a,
            Ok(Err(_)) => {
                return Err(AppError::BadRequest(
                    "URL host could not be resolved".to_string(),
                ))
            }
            Err(_) => {
                return Err(AppError::BadRequest(
                    "URL host DNS resolution timed out".to_string(),
                ))
            }
        };

    let mut saw_addr = false;
    let mut resolved_ip = None;
    for addr in addrs.by_ref() {
        if resolved_ip.is_none() {
            resolved_ip = Some(addr.ip());
        }
        saw_addr = true;
        if !is_public_routable_ip(addr.ip()) {
            return Err(AppError::Forbidden(
                "Resolved URL target is local, private, or reserved".to_string(),
            ));
        }
    }

    if !saw_addr {
        return Err(AppError::BadRequest(
            "URL host did not resolve to any addresses".to_string(),
        ));
    }

    let ip = resolved_ip.ok_or_else(|| {
        AppError::BadRequest("URL host did not resolve to any addresses".to_string())
    })?;

    Ok(ValidatedUrl {
        ip,
        host: host_str,
        port,
        url: parsed,
    })
}

/// Validates that a host name or IP address is loopback, local, or private.
pub fn is_host_private_or_loopback(host: &str) -> bool {
    let lower = host.trim_end_matches('.').to_ascii_lowercase();
    if lower == "localhost"
        || lower.ends_with(".localhost")
        || lower == "host.docker.internal"
        || lower.ends_with(".local")
    {
        return true;
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(v4) => {
                v4.is_loopback()
                    || v4.is_private()
                    || v4.is_link_local() // 10.0.0.1/16
                    || v4.octets()[0] == 0 // 0.0.0.0/8
            }
            IpAddr::V6(v6) => {
                v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local()
            }
        }
    } else {
        false
    }
}

/// SEC-01: Validates that a provider endpoint URL is either an exact local/private host or uses HTTPS.
/// Prevents accidental or attacker-manipulated transmission of API keys over unencrypted external channels.
pub fn validate_url_security(url_str: &str) -> Result<(), AppError> {
    let parsed = url::Url::parse(url_str).map_err(|e| {
        AppError::BadRequest(format!("Invalid URL format for provider endpoint: {}", e))
    })?;

    // Block userinfo credentials in URLs
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(AppError::BadRequest(
            "URLs containing userinfo credentials are not permitted.".to_string(),
        ));
    }

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(AppError::BadRequest(format!(
            "Unsupported URL scheme '{}'. Only HTTP and HTTPS are permitted.",
            scheme
        )));
    }

    let host_str = parsed.host_str().unwrap_or_default().to_lowercase();
    let is_loopback_or_private = is_host_private_or_loopback(&host_str);

    let allow_insecure = std::env::var("TADPOLE_ALLOW_LOCAL_HTTP")
        .map(|v| matches!(v.trim().to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);

    if scheme == "http" && !is_loopback_or_private && !allow_insecure {
        return Err(AppError::BadRequest(
            "Insecure transmission blocked: API keys cannot be sent over plain HTTP to external providers. Use HTTPS."
                .to_string(),
        ));
    }

    Ok(())
}
