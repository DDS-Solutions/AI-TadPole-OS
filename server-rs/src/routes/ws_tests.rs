//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / ws_tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `ws_tests::*`

use axum::http::StatusCode;
use axum::{routing::get, Router};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::{routes::ws::ws_handler, state::AppState};

async fn spawn_app() -> (String, Arc<AppState>) {
    let app_state = Arc::new(AppState::new_mock().await);

    let app = Router::new()
        .route("/engine/ws", get(ws_handler))
        .route(
            "/engine/live-voice",
            get(crate::routes::ws::live_voice_handler),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            crate::middleware::auth::validate_token,
        ))
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

    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);

    // We can gracefully close
    let mut ws = ws_stream;
    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn test_ws_post_upgrade_json_auth_success() {
    let (base_url, state) = spawn_app().await;
    let url = format!("{}/engine/ws", base_url);

    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert("Origin", "http://localhost:5173".parse().unwrap());

    let (mut ws_stream, response) = connect_async(request).await.expect("Failed to connect");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);

    // Send post-upgrade auth frame
    let auth_frame = serde_json::json!({
        "type": "auth",
        "token": state.security.deploy_token
    });
    ws_stream
        .send(Message::Text(auth_frame.to_string().into()))
        .await
        .expect("Failed to send auth frame");

    // Expect auth_ok
    if let Some(Ok(Message::Text(reply))) = ws_stream.next().await {
        let parsed: serde_json::Value = serde_json::from_str(&reply).unwrap();
        assert_eq!(parsed.get("type").and_then(|v| v.as_str()), Some("auth_ok"));
    } else {
        panic!("Expected auth_ok frame");
    }

    let _ = ws_stream.close(None).await;
}

#[tokio::test]
async fn test_ws_post_upgrade_json_auth_invalid_token() {
    let (base_url, _state) = spawn_app().await;
    let url = format!("{}/engine/ws", base_url);

    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert("Origin", "http://localhost:5173".parse().unwrap());

    let (mut ws_stream, response) = connect_async(request).await.expect("Failed to connect");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);

    // Send invalid auth token
    let auth_frame = serde_json::json!({
        "type": "auth",
        "token": "completely-invalid-secret-token"
    });
    ws_stream
        .send(Message::Text(auth_frame.to_string().into()))
        .await
        .expect("Failed to send auth frame");

    // Expect auth_error or close frame
    if let Some(Ok(Message::Text(reply))) = ws_stream.next().await {
        let parsed: serde_json::Value = serde_json::from_str(&reply).unwrap();
        assert_eq!(
            parsed.get("type").and_then(|v| v.as_str()),
            Some("auth_error")
        );
    }
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

    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
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
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    } else {
        panic!("Expected HTTP error");
    }
}

#[tokio::test]
async fn test_live_voice_unauthorized_blocked() {
    let (base_url, _state) = spawn_app().await;
    let url = format!("{}/engine/live-voice", base_url);

    let mut request = url.into_client_request().unwrap();
    // Do NOT send the subprotocol/auth token
    request
        .headers_mut()
        .insert("Origin", "http://localhost:5173".parse().unwrap());

    let result = connect_async(request).await;

    assert!(
        result.is_err(),
        "Expected connection to fail due to 401 Unauthorized"
    );
    if let Err(tokio_tungstenite::tungstenite::Error::Http(response)) = result {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    } else {
        panic!("Expected HTTP error");
    }
}
