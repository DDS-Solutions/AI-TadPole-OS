//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **Auth Middleware**: Orchestrates the validation of **Bearer Tokens**
//! for the Tadpole OS engine. Implements **Constant-Time Comparison**
//! (`constant_time_eq`) to mask timing-based side-channel attacks on
//! the `NEURAL_TOKEN`. Supports **Dual-Mechanism Validation**: standard
//! `Authorization` headers for REST, and `Sec-WebSocket-Protocol`
//! (bearer.<token>) for browser-based WebSocket upgrades (AUTH-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing `Authorization` header, invalid token
//!   checksum, or malformed WebSocket subprotocol strings. Triggers
//!   `401 Unauthorized`.
//! - **Trace Scope**: `server-rs::middleware::auth`

use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use subtle::ConstantTimeEq;

/// Constant-time string comparison to prevent timing-based side-channel attacks.
///
/// ### 🔒 Security: Constant-Time Comparison (AUTH-01)
/// Standard string equality checks return `false` as soon as they find the 
/// first differing byte. An attacker can use this timing information to 
/// guess a token one character at a time. 
/// 
/// This implementation uses the `subtle` crate to ensure that the execution 
/// time is deterministic relative to the input length, preventing 
/// optimizer-induced early returns.
pub(crate) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Middleware to validate the Bearer token.
/// Supports two mechanisms:
/// 1. Standard `Authorization: Bearer <token>` header (REST endpoints)
/// 2. `Sec-WebSocket-Protocol: bearer.<token>` header (browser WebSocket upgrades)
pub async fn validate_token(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for standard Authorization header first
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok());

    if let Some(auth_str) = auth_header {
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            if constant_time_eq(token.as_bytes(), state.security.deploy_token.as_bytes()) {
                return Ok(next.run(req).await);
            } else {
                tracing::warn!("🚫 Invalid token provided in Authorization header");
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // Fallback: check Sec-WebSocket-Protocol for browser WS connections
    // Browsers cannot set Authorization headers on WebSocket upgrade requests,
    // so the frontend sends the token as a subprotocol: "bearer.<token>"
    let is_ws_upgrade = req
        .headers()
        .get(axum::http::header::UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);

    if is_ws_upgrade {
        let proto_header = req
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // SEC-01: Split the comma-separated list of protocols
        // Browsers often combine multiple subprotocols in one header
        for protocol in proto_header.split(',') {
            let protocol = protocol.trim();
            if let Some(token) = protocol.strip_prefix("bearer.") {
                if constant_time_eq(token.as_bytes(), state.security.deploy_token.as_bytes()) {
                    return Ok(next.run(req).await);
                }
            }
        }
        tracing::warn!(header = ?proto_header, "🚫 Unauthorized WebSocket upgrade: invalid or missing protocol token");
    } else {
        tracing::warn!("🚫 Missing or malformed Authorization header");
    }

    Err(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
        middleware::from_fn_with_state,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn dummy_handler() -> StatusCode {
        StatusCode::OK
    }

    #[tokio::test]
    async fn test_auth_bearer_success() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer test-token")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_bearer_invalid() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer wrong-token")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_websocket_protocol_success() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        let req = Request::builder()
            .uri("/")
            .header(header::UPGRADE, "websocket")
            .header("sec-websocket-protocol", "bearer.test-token")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_websocket_protocol_malformed() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        // Missing "bearer." prefix
        let req = Request::builder()
            .uri("/")
            .header(header::UPGRADE, "websocket")
            .header("sec-websocket-protocol", "test-token")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_missing_header() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}

// Metadata: [auth]

// Metadata: [auth]
