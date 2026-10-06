//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / cors
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Structural]` Security: prohibits wildcard origin (`*`) in production mode unless explicitly opted in via ALLOW_UNSAFE_CORS.
//! - `[Structural]` Performance: caches preflight OPTIONS responses for 1 hour (`max_age`).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use axum::http::{HeaderName, HeaderValue, Method};
use std::time::Duration;
use tower_http::cors::CorsLayer;

pub const CORS_PREFLIGHT_MAX_AGE_SECS: u64 = 3600;

/// Validates that a string is a valid origin (scheme + host + optional port), rejecting "null" and wildcards.
pub fn is_valid_cors_origin(origin: &str) -> bool {
    let trimmed = origin.trim();
    if trimmed.is_empty() || trimmed == "*" || trimmed.eq_ignore_ascii_case("null") {
        return false;
    }
    if !(trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("tauri://"))
    {
        return false;
    }
    if let Ok(uri) = trimmed.parse::<axum::http::Uri>() {
        let p = uri.path();
        if p != "/" && !p.is_empty() {
            return false;
        }
        if uri.query().is_some() {
            return false;
        }
    }
    true
}

/// Configures the CORS policy for the engine.
/// Handles dynamic origins from the `ALLOWED_ORIGINS` environment variable.
pub fn create_cors_layer() -> CorsLayer {
    // Default allowed origins covering Vite local dev, Tauri desktop origins, and web engine
    let mut origins = vec![
        HeaderValue::from_static("http://localhost:5173"),
        HeaderValue::from_static("http://127.0.0.1:5173"),
        HeaderValue::from_static("http://localhost:5174"),
        HeaderValue::from_static("http://127.0.0.1:5174"),
        HeaderValue::from_static("http://localhost:8000"),
        HeaderValue::from_static("http://127.0.0.1:8000"),
        // Tauri v1 (macOS) & Tauri v2 custom protocol schemes
        HeaderValue::from_static("tauri://localhost"),
        HeaderValue::from_static("http://tauri.localhost"),
    ];

    let mut cors = CorsLayer::new();

    // SEC-03: Dynamic CORS Origins (e.g. for Bunker/Remote deployments)
    let allow_credentials = if let Ok(allowed) = std::env::var("ALLOWED_ORIGINS") {
        let trimmed_allowed = allowed.trim();
        if trimmed_allowed == "*" {
            let is_prod = std::env::var("APP_ENV")
                .map(|v| v.trim().eq_ignore_ascii_case("production"))
                .unwrap_or(false);
            let allow_unsafe = std::env::var("ALLOW_UNSAFE_CORS")
                .map(|v| {
                    let t = v.trim();
                    t == "true" || t == "1"
                })
                .unwrap_or(false);

            if !is_prod && allow_unsafe {
                tracing::warn!("⚠️ [CORS] RELAXED: Allowing all origins (*)");
                cors = cors.allow_origin(tower_http::cors::Any);
                false // Cannot use credentials with wildcard origin
            } else {
                tracing::error!("🚨 [CORS] ERROR: Wildcard origin (*) is prohibited in production or without explicit ALLOW_UNSAFE_CORS. Falling back to default local origins.");
                cors = cors.allow_origin(origins);
                true
            }
        } else {
            for raw_origin in trimmed_allowed.split(',') {
                let origin = raw_origin.trim();
                if origin.is_empty() {
                    continue;
                }
                if !is_valid_cors_origin(origin) {
                    tracing::warn!(
                        "⚠️ [CORS] Invalid origin '{}' in ALLOWED_ORIGINS: must be valid scheme://host. Skipping.",
                        origin
                    );
                    continue;
                }
                match origin.parse::<HeaderValue>() {
                    Ok(val) => {
                        if !origins.contains(&val) {
                            origins.push(val);
                        }
                    }
                    Err(err) => {
                        tracing::warn!(
                            "⚠️ [CORS] Failed to parse origin '{}' in ALLOWED_ORIGINS: {}. Skipping.",
                            origin,
                            err
                        );
                    }
                }
            }
            cors = cors.allow_origin(origins);
            true
        }
    } else {
        cors = cors.allow_origin(origins);
        true
    };

    cors.allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
        Method::PATCH,
    ])
    .allow_headers([
        axum::http::header::CONTENT_TYPE,
        axum::http::header::AUTHORIZATION,
        HeaderName::from_static("x-request-id"),
        HeaderName::from_static("traceparent"),
    ])
    .expose_headers([
        HeaderName::from_static("x-request-id"),
        HeaderName::from_static("traceparent"),
        HeaderName::from_static("sunset"),
        HeaderName::from_static("deprecation"),
        HeaderName::from_static("link"),
        HeaderName::from_static("x-ratelimit-limit"),
        HeaderName::from_static("x-ratelimit-remaining"),
        HeaderName::from_static("x-ratelimit-reset"),
        HeaderName::from_static("retry-after"),
    ])
    .max_age(Duration::from_secs(CORS_PREFLIGHT_MAX_AGE_SECS))
    .allow_credentials(allow_credentials)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[test]
    fn test_is_valid_cors_origin() {
        assert!(is_valid_cors_origin("http://localhost:3000"));
        assert!(is_valid_cors_origin("https://app.tadpole.io"));
        assert!(is_valid_cors_origin("tauri://localhost"));

        // Invalid origins
        assert!(!is_valid_cors_origin("null"));
        assert!(!is_valid_cors_origin("NULL"));
        assert!(!is_valid_cors_origin("*"));
        assert!(!is_valid_cors_origin(""));
        assert!(!is_valid_cors_origin("example.com")); // missing scheme
        assert!(!is_valid_cors_origin("ftp://example.com")); // invalid scheme
        assert!(!is_valid_cors_origin("https://app.tadpole.io/api")); // has path
        assert!(!is_valid_cors_origin("https://app.tadpole.io?param=1")); // has query
    }

    #[test]
    fn test_create_cors_layer_default() {
        let _layer = create_cors_layer();
    }

    #[tokio::test]
    async fn test_cors_preflight_headers_and_max_age() {
        let app = Router::new()
            .route("/test", get(|| async { StatusCode::OK }))
            .layer(create_cors_layer());

        // Preflight OPTIONS request from an allowed origin
        let req = Request::builder()
            .method(Method::OPTIONS)
            .uri("/test")
            .header(header::ORIGIN, "http://localhost:5173")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
            .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let headers = res.headers();
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "http://localhost:5173"
        );
        assert_eq!(headers.get(header::ACCESS_CONTROL_MAX_AGE).unwrap(), "3600");
        assert_eq!(
            headers
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
    }
}
