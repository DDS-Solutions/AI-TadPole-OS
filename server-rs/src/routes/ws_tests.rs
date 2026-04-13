//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **WebSocket Integration Tests (Real-Time Verification)**: 
//! Orchestrates the verification of real-time event broadcasting and 
//! state synchronization for the Tadpole OS engine. Features 
//! **Subprotocol Auth Validation**: ensures that the `bearer.<token>` 
//! handshake correctly authenticates WebSocket clients. Implements 
//! **CSRF & Origin Protection Tests**: validates that the `ws_handler` 
//! correctly enforces origin-based security policies (CORS) during 
//! the upgrade handshake. AI agents should run these tests after 
//! modifying the `ws` module or updating the `AppState`'s security 
//! configurations to prevent unauthorized streaming of system 
//! telemetry (NET-05).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Unexpected 403 Forbidden for valid origins due 
//!   to incorrect `ALLOWED_ORIGINS` regex, 101 Switching Protocols 
//!   failures due to missing `Sec-WebSocket-Key`, or port 
//!   conflicts during parallel test execution.
//! - **Trace Scope**: `server-rs::routes::ws_tests`

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::{routes::ws::ws_handler, state::AppState};

async fn spawn_app() -> (String, Arc<AppState>) {
    let app_state = Arc::new(AppState::new_mock().await);

    let app = Router::new()
        .route("/engine/ws", get(ws_handler))
        .with_state(app_state.clone());

    // Bind to a random local port with fallback for CI environments
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(_) => match TcpListener::bind("0.0.0.0:0").await {
            Ok(l) => l,
            Err(_) => TcpListener::bind("[::1]:0")
                .await
                .expect("Failed to bind to any local address in test environment"),
        },
    };

    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let host = match addr.ip() {
        std::net::IpAddr::V4(v4) if v4.is_unspecified() => "127.0.0.1".to_string(),
        std::net::IpAddr::V6(v6) if v6.is_unspecified() => "[::1]".to_string(),
        std::net::IpAddr::V4(v4) => v4.to_string(),
        std::net::IpAddr::V6(v6) => format!("[{}]", v6),
    };

    (format!("ws://{}:{}", host, port), app_state)
}

#[tokio::test]
async fn test_ws_valid_connection_and_auth() {
    let (base_url, state) = spawn_app().await;
    let url = format!("{}/engine/ws", base_url);

    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        format!("bearer.{}", state.security.deploy_token)
            .parse()
            .unwrap(),
    );
    // CSRF bypass valid origin
    request
        .headers_mut()
        .insert("Origin", "http://localhost:5173".parse().unwrap());

    let (ws_stream, response) = connect_async(request).await.expect("Failed to connect");

    assert_eq!(response.status(), 101);

    // We can gracefully close
    let mut ws = ws_stream;
    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn test_ws_missing_origin_allowed() {
    let (base_url, state) = spawn_app().await;
    let url = format!("{}/engine/ws", base_url);

    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        format!("bearer.{}", state.security.deploy_token)
            .parse()
            .unwrap(),
    );

    let (_, response) = connect_async(request).await.expect("Failed to connect");

    assert_eq!(response.status(), 101);
}

#[tokio::test]
async fn test_ws_invalid_origin_blocked() {
    let (base_url, state) = spawn_app().await;
    let url = format!("{}/engine/ws", base_url);

    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        format!("bearer.{}", state.security.deploy_token)
            .parse()
            .unwrap(),
    );
    request
        .headers_mut()
        .insert("Origin", "http://evil-cors-site.com".parse().unwrap());

    let result = connect_async(request).await;

    assert!(
        result.is_err(),
        "Expected connection to fail due to 403 Forbidden"
    );
    if let Err(tokio_tungstenite::tungstenite::Error::Http(response)) = result {
        assert_eq!(response.status(), 403);
    } else {
        panic!("Expected HTTP error");
    }
}
