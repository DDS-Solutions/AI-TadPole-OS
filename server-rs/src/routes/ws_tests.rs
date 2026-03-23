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

    let listener = TcpListener::bind("10.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("ws://10.0.0.1:{}", port), app_state)
}

#[tokio::test]
async fn test_ws_valid_connection_and_auth() {
    let (base_url, state) = spawn_app().await;
    let url = format!("{}/engine/ws", base_url);

    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        format!("bearer.{}", state.security.deploy_token).parse().unwrap(),
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
        format!("bearer.{}", state.security.deploy_token).parse().unwrap(),
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
        format!("bearer.{}", state.security.deploy_token).parse().unwrap(),
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
