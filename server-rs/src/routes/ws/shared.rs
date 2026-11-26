//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! - Session internals preserve authentication, bounded transport and permit ownership.
//!
//! ### 🔍 Debugging & Observability
//! - Session failures use the existing sanitized tracing targets.

use axum::http::{HeaderMap, StatusCode};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Global connection-count semaphore guarding against slowloris resource exhaustion.
pub(super) static WS_CONNECTION_SEMAPHORE: Lazy<Arc<Semaphore>> =
    Lazy::new(|| Arc::new(Semaphore::new(250)));

/// Semaphore bounding concurrent async oversight resolution tasks spawned from WebSocket frames.
pub(super) static WS_OVERSIGHT_SEMAPHORE: Lazy<Arc<Semaphore>> =
    Lazy::new(|| Arc::new(Semaphore::new(10)));

/// Helper to verify the Origin header for WebSocket upgrades (CSRF protection).
pub(super) fn verify_origin(headers: &HeaderMap) -> Result<(), StatusCode> {
    let is_production = std::env::var("TADPOLE_ENV")
        .map(|v| v.eq_ignore_ascii_case("production"))
        .unwrap_or(false);

    let allowed_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
    verify_origin_policy(headers, is_production, &allowed_env)
}

fn verify_origin_policy(
    headers: &HeaderMap,
    is_production: bool,
    allowed_env: &str,
) -> Result<(), StatusCode> {
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let allowed: Vec<&str> = allowed_env
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let is_allowed = if is_production {
            if allowed.is_empty() {
                tracing::error!(
                    "🚫 SECURITY: ALLOWED_ORIGINS is empty in production. All WS origins are rejected."
                );
                false
            } else {
                allowed.iter().any(|a| a.eq_ignore_ascii_case(origin))
            }
        } else {
            let defaults = [
                "http://localhost:5173",
                "http://127.0.0.1:5173",
                "http://localhost:5174",
                "http://127.0.0.1:5174",
                "http://localhost:8000",
                "http://127.0.0.1:8000",
                "tauri://localhost",
                "http://tauri.localhost",
            ];
            let mut ok = defaults.iter().any(|d| d.eq_ignore_ascii_case(origin));
            if !ok && !allowed.is_empty() {
                ok = allowed.iter().any(|a| a.eq_ignore_ascii_case(origin));
            }
            ok
        };

        if !is_allowed {
            tracing::warn!("🚫 WS upgrade rejected: unexpected Origin '{}'", origin);
            return Err(StatusCode::FORBIDDEN);
        }
    } else if is_production {
        tracing::warn!("🚫 WS upgrade rejected: missing Origin header in production");
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

/// N-06: Header redaction utility for debug logging.
pub(super) fn redact_headers(headers: &HeaderMap) -> String {
    const REDACTED_HEADERS: &[&str] = &[
        "authorization",
        "proxy-authorization",
        "cookie",
        "set-cookie",
        "sec-websocket-protocol",
        "x-api-key",
        "x-goog-api-key",
    ];

    let mut parts: Vec<String> = Vec::new();
    for (name, value) in headers.iter() {
        let name_lower = name.as_str().to_ascii_lowercase();
        if REDACTED_HEADERS.contains(&name_lower.as_str()) {
            parts.push(format!("{}: [REDACTED]", name));
        } else if let Ok(v) = value.to_str() {
            parts.push(format!("{}: {}", name, v));
        } else {
            parts.push(format!("{}: [non-UTF8]", name));
        }
    }
    format!("{{{}}}", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_origin_requires_an_exact_configured_origin() {
        let mut headers = HeaderMap::new();
        assert_eq!(
            verify_origin_policy(&headers, true, "https://app.example"),
            Err(StatusCode::FORBIDDEN)
        );
        headers.insert("origin", "https://app.example.evil.test".parse().unwrap());
        assert_eq!(
            verify_origin_policy(&headers, true, "https://app.example"),
            Err(StatusCode::FORBIDDEN)
        );
        headers.insert("origin", "https://app.example".parse().unwrap());
        assert!(verify_origin_policy(
            &headers,
            true,
            " https://app.example , https://other.example "
        )
        .is_ok());
        assert_eq!(
            verify_origin_policy(&headers, true, ""),
            Err(StatusCode::FORBIDDEN)
        );
    }

    #[test]
    fn diagnostic_headers_redact_credentials_and_subprotocol_tokens() {
        let mut headers = HeaderMap::new();
        for key in [
            "authorization",
            "cookie",
            "sec-websocket-protocol",
            "x-api-key",
            "x-goog-api-key",
        ] {
            headers.insert(key, "secret-marker".parse().unwrap());
        }
        headers.insert("origin", "https://app.example".parse().unwrap());
        let diagnostic = redact_headers(&headers);
        assert!(!diagnostic.contains("secret-marker"));
        assert!(diagnostic.contains("https://app.example"));
    }
}
