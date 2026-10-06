//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / auth
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Structural]` Security: role-based distinction (`AuthenticatedRole::Admin` vs `AuthenticatedRole::Deploy`) enforced via request extensions with strict 403 Forbidden separation.
//! - `[Structural]` Constant-time credential comparison avoiding timing side-channel leakage.
//! - `[Structural]` WebSocket upgrades require a `bearer.<token>` subprotocol. Post-connect frames are not a substitute for the upgrade check.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN
//! - **Telemetry Targets**: none declared

use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
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
/// `a.len() != b.len()` early return intentionally leaks token length to avoid
/// comparing mismatched buffer sizes. This is a standard RFC/cryptographic tradeoff.
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
/// Compares both admin and deploy credentials in constant-time to avoid timing side-channels.
/// Returns `None` if the token is empty, neither token is configured, or no match is found.
pub fn resolve_token_role(token: &str, state: &AppState) -> Option<AuthenticatedRole> {
    if token.is_empty() {
        return None;
    }
    let t = token.as_bytes();
    let admin_ok = !state.security.admin_token.is_empty()
        && constant_time_eq(t, state.security.admin_token.as_bytes());
    let deploy_ok = !state.security.deploy_token.is_empty()
        && constant_time_eq(t, state.security.deploy_token.as_bytes());

    match (admin_ok, deploy_ok) {
        (true, _) => Some(AuthenticatedRole::Admin),
        (false, true) => Some(AuthenticatedRole::Deploy),
        (false, false) => None,
    }
}

/// Extracts the token value from an `Authorization` header value per RFC 9110 / 7235.
/// Matches the `Bearer` scheme case-insensitively with single space separator.
///
/// Uses byte-level inspection to prevent panics on non-ASCII/multi-byte inputs (B1).
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    let trimmed = auth_header.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 7 && bytes[..6].eq_ignore_ascii_case(b"bearer") && bytes[6] == b' ' {
        // Safe: bytes[0..7] is ASCII "bearer ", so index 7 is guaranteed a UTF-8 character boundary.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerifiedAuth {
    pub role: AuthenticatedRole,
}

impl VerifiedAuth {
    pub fn new(role: AuthenticatedRole) -> Self {
        Self { role }
    }
}

/// Response-extension marker indicating an authentication failure occurred in token validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthFailure;

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
                response.extensions_mut().insert(VerifiedAuth { role });
                return Ok(response);
            } else {
                tracing::warn!("🚫 Invalid token provided in Authorization header");
                let mut res = StatusCode::UNAUTHORIZED.into_response();
                res.extensions_mut().insert(AuthFailure);
                return Ok(res);
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
                response.extensions_mut().insert(VerifiedAuth { role });
                return Ok(response);
            } else {
                tracing::warn!(
                    "🚫 Unauthorized WebSocket upgrade: invalid bearer subprotocol token"
                );
                let mut res = StatusCode::UNAUTHORIZED.into_response();
                res.extensions_mut().insert(AuthFailure);
                return Ok(res);
            }
        } else {
            let path = req.uri().path();
            tracing::warn!(
                "🚫 Unauthorized WebSocket upgrade: path '{}' requires bearer subprotocol auth",
                path
            );
            let mut res = StatusCode::UNAUTHORIZED.into_response();
            res.extensions_mut().insert(AuthFailure);
            return Ok(res);
        }
    } else {
        tracing::warn!("🚫 Missing or malformed Authorization header");
    }

    let mut res = StatusCode::UNAUTHORIZED.into_response();
    res.extensions_mut().insert(AuthFailure);
    Ok(res)
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
                // SEC: Role mismatch is 403 Forbidden, distinct from 401 Unauthorized.
                // Prevents brute-force limiters from counting role probes as credential strikes.
                return Err(crate::error::AppError::Forbidden(
                    "Administrative privileges required".to_string(),
                ));
            }
        }

        // Fallback: direct header extraction (e.g., when extractor is tested directly)
        use axum::extract::FromRef;
        let app_state = Arc::<AppState>::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok());

        if let Some(token) = auth_header.and_then(extract_bearer_token) {
            match resolve_token_role(token, &app_state) {
                Some(AuthenticatedRole::Admin) => return Ok(RequireAdmin),
                Some(AuthenticatedRole::Deploy) => {
                    return Err(crate::error::AppError::Forbidden(
                        "Administrative privileges required".to_string(),
                    ));
                }
                None => {}
            }
        }

        Err(crate::error::AppError::Unauthorized(
            "Authentication required".to_string(),
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

        for scheme in ["bearer", "BEARER", "Bearer", "bEaReR"] {
            let req = Request::builder()
                .uri("/")
                .header(header::AUTHORIZATION, format!("{} test-token", scheme))
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
    async fn test_auth_websocket_protocol_pulse_only_rejected() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/v1/engine/ws", get(dummy_handler))
            .route("/engine/ws", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        // Pulse-only upgrade is no longer sufficient. The bearer subprotocol is required.
        for path in ["/v1/engine/ws", "/engine/ws"] {
            let req = Request::builder()
                .uri(path)
                .header(header::UPGRADE, "websocket")
                .header("sec-websocket-protocol", "tadpole-pulse-v1")
                .body(Body::empty())
                .unwrap();

            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(
                res.status(),
                StatusCode::UNAUTHORIZED,
                "Path {} must reject a pulse-only upgrade",
                path
            );
        }
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
    async fn test_auth_websocket_authorized_bearer_subprotocol() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let valid_token = state.security.deploy_token.clone();
        let app = Router::new()
            .route("/v1/engine/ws", get(dummy_handler))
            .layer(from_fn_with_state(state, validate_token));

        // Combined bearer subprotocol with pulse subprotocol -> accepted with 200 OK
        let req = Request::builder()
            .uri("/v1/engine/ws")
            .header(header::UPGRADE, "websocket")
            .header(
                "sec-websocket-protocol",
                format!("bearer.{}, tadpole-pulse-v1", valid_token),
            )
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Valid bearer subprotocol must be accepted"
        );

        // Invalid bearer token -> rejected with 401 Unauthorized
        let req_invalid = Request::builder()
            .uri("/v1/engine/ws")
            .header(header::UPGRADE, "websocket")
            .header(
                "sec-websocket-protocol",
                "bearer.definitely_invalid_token_12345, tadpole-pulse-v1",
            )
            .body(Body::empty())
            .unwrap();

        let res_invalid = app.oneshot(req_invalid).await.unwrap();
        assert_eq!(
            res_invalid.status(),
            StatusCode::UNAUTHORIZED,
            "Invalid bearer subprotocol must be rejected"
        );
    }

    #[tokio::test]
    async fn test_require_admin_role_separation() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/admin", get(admin_handler))
            .layer(from_fn_with_state(state.clone(), validate_token))
            .with_state(state);

        // 1. Deploy token should be rejected with 403 FORBIDDEN for admin route (B3 fix)
        let req_deploy = Request::builder()
            .uri("/admin")
            .header(header::AUTHORIZATION, "Bearer test-token") // default test-token is deploy_token in mock
            .body(Body::empty())
            .unwrap();
        let res_deploy = app.clone().oneshot(req_deploy).await.unwrap();
        assert_eq!(
            res_deploy.status(),
            StatusCode::FORBIDDEN,
            "Deploy token attempting admin route must return 403 Forbidden, not 401"
        );

        // 2. Admin token should succeed with 200 OK
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

    #[test]
    fn test_extract_bearer_never_panics_on_non_ascii() {
        // B1 regression test: ensure multi-byte UTF-8 sequences never panic byte-slicing
        let cases = [
            "abcde€",
            "€€€€€€€",
            "\u{2003}Bearer x",
            "Bearer€x",
            "\u{FFFF}\u{FFFF}\u{FFFF}x",
            "Bearer 🦀",
            "🦀Bearer token",
            "bearer\u{00A0}token",
        ];
        for case in cases {
            let res = extract_bearer_token(case);
            // Must execute cleanly without unhandled panics
            let _ = res;
        }
        assert_eq!(extract_bearer_token("abcde€"), None);
        assert_eq!(extract_bearer_token("Bearer 🦀"), Some("🦀"));
    }
}
