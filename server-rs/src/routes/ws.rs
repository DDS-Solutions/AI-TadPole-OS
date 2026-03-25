use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

/// The HTTP upgrade endpoint for WebSockets.
/// Auth is handled by the middleware layer (Bearer header or Sec-WebSocket-Protocol).
/// SEC-01: The client sends `Sec-WebSocket-Protocol: bearer.<token>` because browsers
/// cannot set Authorization headers on WebSocket upgrades. We MUST echo the protocol
/// back in the upgrade response, or the browser will immediately close the connection.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    tracing::debug!("📥 WS Handshake Headers: {:?}", headers);
    // CSRF protection: verify the Origin header for WS upgrades
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let allowed = [
            "http://localhost:5173",
            "http://127.0.0.1:5173",
            "http://localhost:8000",
            "http://127.0.0.1:8000",
        ];
        if !allowed.contains(&origin) {
            tracing::warn!("🚫 WS upgrade rejected: unexpected Origin '{}'", origin);
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    // Extract the subprotocol the client sent (e.g., "bearer.tadpole-dev-token-2026")
    let protocol = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    tracing::info!(
        "✅ WebSocket handshake authorized via middleware. Protocol: {}",
        protocol
    );

    // Echo the subprotocol back. Without this, the browser closes the WS immediately
    // because RFC 6455 requires the server to acknowledge the requested subprotocol.
    let mut response = ws
        .on_upgrade(move |socket| handle_socket(socket, state))
        .into_response();

    if !protocol.is_empty() {
        if let Ok(val) = protocol.parse() {
            response.headers_mut().insert("sec-websocket-protocol", val);
        }
    }

    response
}

/// The actual bi-directional WebSocket loop handling messaging.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to Log entries, Engine events, High-Speed Telemetry, and Audio Streams
    let mut log_rx = state.comms.tx.subscribe();
    let mut event_rx = state.comms.event_tx.subscribe();
    let mut telemetry_rx = state.comms.telemetry_tx.subscribe();
    let mut audio_rx = state.comms.audio_stream_tx.subscribe();

    tracing::info!("🔗 High-Performance WebSocket Connected!");

    // Tell the frontend we connected in Rust.
    state.broadcast_sys("Connected to Tadpole OS [Rust Engine v0.1.0]", "success");

    // Spawn a task that constantly reads our global Broadcast channels
    // and instantly forwards to this specific WebSocket connection
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // 1. Handle System Logs (LogEntry)
                result = log_rx.recv() => {
                    if let Ok(msg) = result {
                        if let Ok(json_str) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json_str.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }

                // 2. Handle Engine Events (serde_json::Value)
                result = event_rx.recv() => {
                    if let Ok(msg) = result {
                        if let Ok(json_str) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json_str.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }

                // 3. Handle High-Speed Telemetry (serde_json::Value)
                result = telemetry_rx.recv() => {
                    if let Ok(msg) = result {
                        if let Ok(json_str) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json_str.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }

                // 4. Handle Real-Time Audio Streams (Vec<u8> binary chunks)
                result = audio_rx.recv() => {
                    if let Ok(msg) = result {
                        if sender.send(Message::Binary(msg.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Spawn a task to drain the receiver and detect client disconnects
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => break,
                Ok(Message::Binary(bin)) => {
                    // Logic for incoming audio chunks (STT) could go here
                    tracing::debug!("📥 Received binary message of {} bytes", bin.len());
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });

    // Exit when either task completes (send failure or client disconnect)
    tokio::select! {
        _ = &mut send_task => { recv_task.abort(); }
        _ = &mut recv_task => { send_task.abort(); }
    }

    tracing::info!("🔗 WebSocket Disconnected.");
}
