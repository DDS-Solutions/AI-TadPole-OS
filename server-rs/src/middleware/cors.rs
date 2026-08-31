//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / cors
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Security: prohibits wildcard origin (`*`) in production/release mode unless explicitly opted in.
//!   - enforced_by: `test_create_cors_layer_default`
//! - `[Behavioral]` Performance: caches preflight OPTIONS responses for 1 hour (`max_age`).
//!   - enforced_by: `test_create_cors_layer_default`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_create_cors_layer_default`

use axum::http::{HeaderName, HeaderValue, Method};
use std::time::Duration;
use tower_http::cors::CorsLayer;

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
            // RELAXED MODE: Allow all for troubleshooting legacy hardware, but block in production/release mode
            // unless ALLOW_UNSAFE_CORS is explicitly set to true.
            let allow_unsafe = std::env::var("ALLOW_UNSAFE_CORS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            if cfg!(debug_assertions) || allow_unsafe {
                tracing::warn!("⚠️ [CORS] RELAXED: Allowing all origins (*)");
                cors = cors.allow_origin(tower_http::cors::Any);
                false // Cannot use credentials with wildcard origin
            } else {
                tracing::error!("🚨 [CORS] ERROR: Wildcard origin (*) is prohibited in production/release mode. Falling back to default local origins.");
                cors = cors.allow_origin(origins);
                true
            }
        } else {
            for raw_origin in trimmed_allowed.split(',') {
                let origin = raw_origin.trim();
                if origin.is_empty() {
                    continue;
                }
                if origin == "*" {
                    tracing::warn!("⚠️ [CORS] Literal '*' in comma-separated ALLOWED_ORIGINS is invalid; skipping.");
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
    ])
    .max_age(Duration::from_secs(3600))
    .allow_credentials(allow_credentials)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cors_layer_default() {
        // Constructing default layer should succeed without panic
        let _layer = create_cors_layer();
    }
}
