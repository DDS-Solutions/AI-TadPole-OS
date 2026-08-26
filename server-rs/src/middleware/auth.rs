//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / auth
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Security: role-based distinction (`AuthenticatedRole::Admin` vs `AuthenticatedRole::Deploy`)
//!   enforced via request extensions.
//!   - enforced_by: `test_require_admin_role_separation`
//! - `[Advisory: UNVERIFIED]` Security: unauthenticated WebSocket upgrade pass-through restricted to explicit post-connect allowlist.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_auth_bearer_success`, `test_require_admin_role_separation`

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
///
/// NOTE on residual timing characteristics:
/// 1. `a.len() != b.len()` early return intentionally leaks token length to avoid
///    comparing mismatched buffer sizes. This is a standard RFC/cryptographic tradeoff.
/// 2. Sequential comparison in [`resolve_token_role`] evaluates admin token before deploy
///    token, introducing negligible sub-microsecond variation on which role matched.
pub(crate) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Strongly typed authentication role resolved by the [`validate_token`] middleware.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthenticatedRole {
    /// Token matched `deploy_token` — standard operational access.
    Deploy,
    /// Token matched `admin_token` — full administrative privileges.
    Admin,
}

/// Resolves the authenticated role for a provided token string.
/// Returns `None` if the token is empty, neither token is configured, or no match is found.
pub fn resolve_token_role(token: &str, state: &AppState) -> Option<AuthenticatedRole> {
    if token.is_empty() {
        return None;
    }
    if !state.security.admin_token.is_empty()
        && constant_time_eq(token.as_bytes(), state.security.admin_token.as_bytes())
    {
        Some(AuthenticatedRole::Admin)
    } else if !state.security.deploy_token.is_empty()
        && constant_time_eq(token.as_bytes(), state.security.deploy_token.as_bytes())
    {
        Some(AuthenticatedRole::Deploy)
    } else {
        None
    }
}

/// Backward compatibility check for any valid access token.
#[allow(dead_code)]
fn is_valid_access_token(token: &str, state: &AppState) -> bool {
    resolve_token_role(token, state).is_some()
}

/// Extracts the token value from an `Authorization` header value per RFC 9110 / 7235.
/// Matches the `Bearer` scheme case-insensitively with single space separator.
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    let trimmed = auth_header.trim();
    if trimmed.len() >= 7
        && trimmed[..6].eq_ignore_ascii_case("bearer")
        && trimmed.as_bytes()[6] == b' '
    {
        let token = trimmed[7..].trim();
        if !token.is_empty() {
            return Some(token);
        }
    }
    None
}

/// Marker struct to indicate that the request has been authenticated via header or subprotocol before upgrade.
#[derive(Clone, Copy, Debug)]
pub struct PreAuthenticated;

/// Response-extension marker indicating the request was verified against a valid credential.
/// Consumed by the brute-force limiter to safely reset failure counters only on genuine
/// auth successes — not merely on "some header was present + 200 returned."
#[derive(Clone, Copy, Debug)]
pub struct VerifiedAuth;

/// Middleware to validate the Bearer token.
/// Supports two mechanisms:
/// 1. Standard `Authorization: Bearer <token>` header (REST endpoints, case-insensitive scheme)
/// 2. `Sec-WebSocket-Protocol: bearer.<token>` header (browser WebSocket upgrades)
pub async fn validate_token(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Check for standard Authorization header first
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok());

    if let Some(auth_str) = auth_header {
        if let Some(token) = extract_bearer_token(auth_str) {
            if let Some(role) = resolve_token_role(token, &state) {
                req.extensions_mut().insert(PreAuthenticated);
                req.extensions_mut().insert(role);
                let mut response = next.run(req).await;
                response.extensions_mut().insert(VerifiedAuth);
                return Ok(response);
            } else {
                tracing::warn!("🚫 Invalid token provided in Authorization header");
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // 2. Fallback: check Sec-WebSocket-Protocol for browser WS connections
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

        let mut has_bearer = false;
        let mut resolved_role = None;

        // SEC-01: Split the comma-separated list of protocols
        // Browsers often combine multiple subprotocols in one header
        for protocol in proto_header.split(',') {
            let protocol = protocol.trim();
            if let Some(token) = protocol.strip_prefix("bearer.") {
                has_bearer = true;
                if let Some(role) = resolve_token_role(token, &state) {
                    resolved_role = Some(role);
                    break;
                }
            }
        }

        if has_bearer {
            if let Some(role) = resolved_role {
                req.extensions_mut().insert(PreAuthenticated);
                req.extensions_mut().insert(role);
                let mut response = next.run(req).await;
                response.extensions_mut().insert(VerifiedAuth);
                return Ok(response);
            } else {
                tracing::warn!(
                    "🚫 Unauthorized WebSocket upgrade: invalid bearer subprotocol token"
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            // Defend WebSocket surface: only explicit post-connect routes may pass without subprotocol auth.
            let path = req.uri().path();
            let is_allowed_post_connect = path == "/v1/engine/ws" || path == "/engine/ws";
            if is_allowed_post_connect {
                // Upgrade request doesn't request bearer.<token> (e.g. it requests 'tadpole-pulse-v1').
                // We allow the upgrade to proceed, but since PreAuthenticated is NOT in extensions,
                // the WebSocket handler will require post-connect authentication.
                return Ok(next.run(req).await);
            } else {
                tracing::warn!(
                    "🚫 Unauthorized WebSocket upgrade: path '{}' requires bearer subprotocol auth",
                    path
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    } else {
        tracing::warn!("🚫 Missing or malformed Authorization header");
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Extractor that requires administrative credentials (admin_token).
/// Verifies the [`AuthenticatedRole::Admin`] extension if present, or falls back
/// to header extraction for direct unit testing.
#[derive(Clone, Copy, Debug)]
pub struct RequireAdmin;

impl<S> axum::extract::FromRequestParts<S> for RequireAdmin
where
    Arc<AppState>: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = crate::error::AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Fast path: verify type-safe extension inserted by validate_token
        if let Some(role) = parts.extensions.get::<AuthenticatedRole>() {
            if *role == AuthenticatedRole::Admin {
                return Ok(RequireAdmin);
            } else {
                return Err(crate::error::AppError::Unauthorized(
                    "Administrative privileges required".to_string(),
                ));
            }
        }

        // Fallback: direct header extraction (e.g., when route is invoked without validate_token layer)
        use axum::extract::FromRef;
        let app_state = Arc::<AppState>::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok());

        if let Some(token) = auth_header.and_then(extract_bearer_token) {
            if !app_state.security.admin_token.is_empty()
                && constant_time_eq(token.as_bytes(), app_state.security.admin_token.as_bytes())
            {
                return Ok(RequireAdmin);
            }
        }

        Err(crate::error::AppError::Unauthorized(
            "Administrative privileges required".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        middleware::from_fn_with_state,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn dummy_handler() -> StatusCode {
        StatusCode::OK
    }

    async fn admin_handler(_admin: RequireAdmin) -> StatusCode {
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
    async fn test_auth_bearer_case_insensitive() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        for scheme in &[
            "bearer test-token",
            "BEARER test-token",
            "Bearer test-token",
        ] {
            let req = Request::builder()
                .uri("/")
                .header(header::AUTHORIZATION, *scheme)
                .body(Body::empty())
                .unwrap();

            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(
                res.status(),
                StatusCode::OK,
                "Failed for scheme: {}",
                scheme
            );
        }
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
            .route("/v1/engine/ws", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        let req = Request::builder()
            .uri("/v1/engine/ws")
            .header(header::UPGRADE, "websocket")
            .header("sec-websocket-protocol", "bearer.test-token")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_websocket_protocol_invalid_bearer() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/v1/engine/ws", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        let req = Request::builder()
            .uri("/v1/engine/ws")
            .header(header::UPGRADE, "websocket")
            .header("sec-websocket-protocol", "bearer.wrong-token")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_websocket_protocol_post_connect_allowed_path() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/v1/engine/ws", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        // Allowed path (/v1/engine/ws) without bearer subprotocol -> allowed to proceed for post-connect auth
        let req = Request::builder()
            .uri("/v1/engine/ws")
            .header(header::UPGRADE, "websocket")
            .header("sec-websocket-protocol", "tadpole-pulse-v1")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_websocket_unauthorized_arbitrary_path_rejected() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/v1/engine/live-voice", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        // Arbitrary WS path without bearer subprotocol -> rejected with 401
        let req = Request::builder()
            .uri("/v1/engine/live-voice")
            .header(header::UPGRADE, "websocket")
            .header("sec-websocket-protocol", "tadpole-pulse-v1")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_require_admin_role_separation() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/admin", get(admin_handler))
            .layer(from_fn_with_state(state.clone(), validate_token))
            .with_state(state);

        // 1. Deploy token should be rejected for admin route
        let req_deploy = Request::builder()
            .uri("/admin")
            .header(header::AUTHORIZATION, "Bearer test-token") // default test-token is deploy_token in mock
            .body(Body::empty())
            .unwrap();
        let res_deploy = app.clone().oneshot(req_deploy).await.unwrap();
        assert_eq!(res_deploy.status(), StatusCode::UNAUTHORIZED);

        // 2. Admin token should succeed
        let req_admin = Request::builder()
            .uri("/admin")
            .header(header::AUTHORIZATION, "Bearer test-admin-token") // admin_token in mock
            .body(Body::empty())
            .unwrap();
        let res_admin = app.oneshot(req_admin).await.unwrap();
        assert_eq!(res_admin.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_missing_header() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_empty_token() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer ")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
