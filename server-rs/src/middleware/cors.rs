//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **CORS Policy**: Orchestrates the supported cross-origin request
//! configurations for the Tadpole OS engine. Features **Dynamic Origin
//! Support** via the `ALLOWED_ORIGINS` environment variable (comma-separated).
//! Automatically handles **Deep-Link Protocols** (tauri://localhost) and
//! local development environments (ports 5173, 8000). Enforces **Credential
//! Misalignment Protection**: credentials are automatically disabled if
//! wildcard (`*`) mode is active (CORS-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: `Origin mismatch` preflight errors on custom
//!   Bunker deployments, missing `Authorization` or `x-request-id`
//!   headers in the allowed list, or credential failures on wildcard origins.
//! - **Trace Scope**: `server-rs::middleware::cors`

use tower_http::cors::CorsLayer;

use axum::http::HeaderValue;

/// Configures the CORS policy for the engine.
/// Handles dynamic origins from the ALLOWED_ORIGINS environment variable.
pub fn create_cors_layer() -> CorsLayer {
    let mut origins = vec![
        HeaderValue::from_static("http://localhost:5173"),
        HeaderValue::from_static("http://127.0.0.1:5173"),
        HeaderValue::from_static("http://localhost:8000"),
        HeaderValue::from_static("http://127.0.0.1:8000"),
        HeaderValue::from_static("tauri://localhost"),
        HeaderValue::from_static("http://tauri.localhost"),
    ];

    let mut cors = CorsLayer::new();

    // SEC-03: Dynamic CORS Origins (e.g. for Bunker/Remote deployments)
    let allow_credentials = if let Ok(allowed) = std::env::var("ALLOWED_ORIGINS") {
        if allowed == "*" {
            // RELAXED MODE: Allow all for troubleshooting legacy hardware
            tracing::warn!("⚠️ CORS RELAXED: Allowing all origins (*)");
            cors = cors.allow_origin(tower_http::cors::Any);
            false // Cannot use credentials with wildcard origin
        } else {
            for origin in allowed.split(',') {
                if let Ok(val) = origin.trim().parse::<HeaderValue>() {
                    if !origins.contains(&val) {
                        origins.push(val);
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
        axum::http::Method::GET,
        axum::http::Method::POST,
        axum::http::Method::PUT,
        axum::http::Method::DELETE,
        axum::http::Method::OPTIONS,
    ])
    .allow_headers([
        axum::http::header::CONTENT_TYPE,
        axum::http::header::AUTHORIZATION,
        axum::http::HeaderName::from_static("x-request-id"),
        axum::http::HeaderName::from_static("traceparent"),
    ])
    .allow_credentials(allow_credentials)
}
