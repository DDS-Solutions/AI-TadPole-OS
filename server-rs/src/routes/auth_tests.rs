//! Auth Route Tests — Session and token verification
//!
//! @docs ARCHITECTURE:Networking

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
    let app_state = Arc::new(AppState::new().await.expect("Failed to initialize state for auth tests"));

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
