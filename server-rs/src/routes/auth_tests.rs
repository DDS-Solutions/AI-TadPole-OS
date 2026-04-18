//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Auth Route Verification (Session Tests)**: Orchestrates the
//! verification of session and token-based authentication for the
//! Tadpole OS engine. Features **Bearer Token Validation**: ensures
//! that protected routes (e.g., `/v1/protected`) correctly enforce
//! `NEURAL_TOKEN` requirements. Implements **WebSocket Subprotocol
//! Authentication**: validates the `Sec-WebSocket-Protocol` (RFC 6455)
//! auth handshake used by real-time agent dashboards. AI agents
//! should run these tests after modifying `middleware::auth` or
//! updating the engine's security posture to prevent
//! unauthenticated data leaks (SEC-07).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 401 Unauthorized for valid tokens due to casing
//!   mismatches in "Bearer", failure to correctly extract subprotocol
//!   tokens, or database connection errors in the mock `AppState`.
//! - **Trace Scope**: `server-rs::routes::auth_tests`

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower::ServiceExt;

use crate::{middleware::auth::validate_token, state::AppState};

// Helper function to create a test router with auth middleware
async fn test_app() -> (Router, Arc<AppState>) {
    // Relying on .env config or defaults. AppState::new() handles sqlite initialization.
    // In a real environment, you might want to mock this or use an in-memory db.
    let app_state = Arc::new(
        AppState::new()
            .await
            .expect("Failed to initialize state for auth tests"),
    );

    let app = Router::new()
        .route("/protected", get(|| async { "success" }))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            validate_token,
        ));

    (app, app_state)
}

#[tokio::test]
async fn test_auth_valid_bearer_token() {
    let (app, state) = test_app().await;
    let valid_token = format!("Bearer {}", state.security.deploy_token);

    let request = Request::builder()
        .uri("/protected")
        .header("Authorization", valid_token)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_invalid_bearer_token() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/protected")
        .header("Authorization", "Bearer invalid-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_missing_header() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_valid_ws_subprotocol() {
    let (app, state) = test_app().await;
    let subprotocol = format!("bearer.{}", state.security.deploy_token);

    let request = Request::builder()
        .uri("/protected")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Protocol", subprotocol)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_invalid_ws_subprotocol() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/protected")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Protocol", "bearer.invalid-token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
