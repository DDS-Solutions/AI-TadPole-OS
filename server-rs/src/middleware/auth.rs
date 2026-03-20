use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Constant-time string comparison to prevent timing-based side-channel attacks.
/// 
/// Unlike a standard equality check (which may return early on the first mismatch), 
/// this implementation iterates through the entire slice, accumulating mismatches 
/// via XOR. This ensures that the execution time is deterministic relative to 
/// the input length, masking whether a mismatch occurred early or late.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
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
        let proto = req
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if let Some(token) = proto.strip_prefix("bearer.") {
            if constant_time_eq(token.as_bytes(), state.security.deploy_token.as_bytes()) {
                return Ok(next.run(req).await);
            }
        }
        tracing::warn!("🚫 Unauthorized WebSocket upgrade: invalid or missing protocol token");
    } else {
        tracing::warn!("🚫 Missing or malformed Authorization header");
    }

    Err(StatusCode::UNAUTHORIZED)
}
