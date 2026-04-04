//! CORS Middleware — Cross-Origin Resource Sharing
//!
//! Configures the security policy for cross-origin requests, supporting 
//! dynamic origins for multi-site deployments (e.g., Bunker/Remote).
//!
//! @docs ARCHITECTURE:Networking

use tower_http::cors::CorsLayer;

use axum::http::HeaderValue;

/// Configures the CORS policy for the engine.
/// Handles dynamic origins from the ALLOWED_ORIGINS environment variable.
pub fn create_cors_layer() -> CorsLayer {
    let mut origins = vec![
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:5173".parse::<HeaderValue>().unwrap(),
        "http://localhost:8000".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:8000".parse::<HeaderValue>().unwrap(),
        "tauri://localhost".parse::<HeaderValue>().unwrap(),
        "http://tauri.localhost".parse::<HeaderValue>().unwrap(),
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
