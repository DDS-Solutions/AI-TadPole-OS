//! Real-Time Event & Telemetry Stream — WebSockets
//!
//! Provides the primary bi-directional bridge for high-speed logging, engine 
//! events, and audio streaming. Includes subprotocol-based authentication.
//!
//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Subprotocol Authentication**: Browser clients must send tokens via the 
//! `sec-websocket-protocol` header in `bearer.<token>` format. The engine 
//! MUST echo this protocol back during handshake or the connection will close.

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
            "tauri://localhost",
            "http://tauri.localhost",
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

    // Check for binary pulse protocol support
    let is_pulse_active = protocol.contains("tadpole-pulse-v1");

    tracing::info!(
        "✅ WebSocket handshake authorized via middleware. Protocol: {}",
        protocol
    );

    // Echo the subprotocol back. Without this, the browser closes the WS immediately
    // because RFC 6455 requires the server to acknowledge the requested subprotocol.
    let mut response = ws
        .on_upgrade(move |socket| handle_socket(socket, state, is_pulse_active))
        .into_response();

    if !protocol.is_empty() {
        if let Ok(val) = protocol.parse() {
            response.headers_mut().insert("sec-websocket-protocol", val);
        }
    }

    response
}

/// The actual bi-directional WebSocket loop handling messaging.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, is_pulse_active: bool) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to Log entries, Engine events, High-Speed Telemetry, and Audio Streams
    let mut log_rx = state.comms.tx.subscribe();
    let mut event_rx = state.comms.event_tx.subscribe();
    let mut telemetry_rx = state.comms.telemetry_tx.subscribe();
    let mut audio_rx = state.comms.audio_stream_tx.subscribe();
    let mut pulse_rx = state.comms.pulse_tx.subscribe();

    tracing::info!("🔗 High-Performance WebSocket Connected!");

    // Tell the frontend we connected in Rust.
    state.broadcast_sys(
        "Connected to Tadpole OS [Rust Engine v0.1.0]",
        "success",
        None,
    );

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
                        // Prepend header 0x01 (Audio)
                        let mut bin = Vec::with_capacity(msg.len() + 1);
                        bin.push(0x01);
                        bin.extend_from_slice(&msg);
                        if sender.send(Message::Binary(bin.into())).await.is_err() {
                            break;
                        }
                    }
                }

                // 5. Handle High-Speed Binary Pulses (MessagePack encoded)
                result = pulse_rx.recv() => {
                    if is_pulse_active {
                        if let Ok(pulse) = result {
                            // MessagePack binary encoding
                            if let Ok(encoded) = rmp_serde::to_vec(&*pulse) {
                                // Prepend header 0x02 (Swarm Pulse)
                                let mut bin = Vec::with_capacity(encoded.len() + 1);
                                bin.push(0x02);
                                bin.extend_from_slice(&encoded);
                                if sender.send(Message::Binary(bin.into())).await.is_err() {
                                    break;
                                }
                            }
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

/// Specialized WebSocket handler for Gemini Live Multimodal API.
/// Proxies client audio/setup to Google's backend to protect API keys.
pub async fn live_voice_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let protocol = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let mut response = ws
        .on_upgrade(move |socket| handle_live_socket(socket, state))
        .into_response();

    if !protocol.is_empty() {
        if let Ok(val) = protocol.parse() {
            response.headers_mut().insert("sec-websocket-protocol", val);
        }
    }
    response
}

async fn handle_live_socket(mut client_ws: WebSocket, _state: Arc<AppState>) {
    let api_key = match std::env::var("GOOGLE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            let _ = client_ws
                .send(Message::Text(
                    "Error: GOOGLE_API_KEY not found on server".into(),
                ))
                .await;
            return;
        }
    };

    let gemini_url = format!(
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.MultimodalLive?key={}",
        api_key
    );

    // Connect to Gemini
    use tokio_tungstenite::connect_async;
    let (gemini_ws, _) = match connect_async::<String>(gemini_url).await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("❌ [LiveWS] Failed to connect to Gemini: {}", e);
            let _ = client_ws
                .send(Message::Text(
                    format!("Error: Failed to connect to Gemini: {}", e).into(),
                ))
                .await;
            return;
        }
    };

    let (mut gemini_sender, mut gemini_receiver) = gemini_ws.split();
    let (mut client_sender, mut client_receiver) = client_ws.split();

    tracing::info!("🎙️ [LiveWS] Gemini Live Proxy Established");

    // Pipe Client -> Gemini
    let c2g = async move {
        while let Some(msg) = client_receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(t)) => {
                    let _ = gemini_sender
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            t.as_str().into(),
                        ))
                        .await;
                }
                Ok(axum::extract::ws::Message::Binary(b)) => {
                    let _ = gemini_sender
                        .send(tokio_tungstenite::tungstenite::Message::Binary(b.into()))
                        .await;
                }
                _ => {}
            }
        }
    };

    // Pipe Gemini -> Client
    let g2c = async move {
        while let Some(msg) = gemini_receiver.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(t)) => {
                    let _ = client_sender
                        .send(axum::extract::ws::Message::Text(t.into()))
                        .await;
                }
                Ok(tokio_tungstenite::tungstenite::Message::Binary(b)) => {
                    let _ = client_sender
                        .send(axum::extract::ws::Message::Binary(b.into()))
                        .await;
                }
                _ => {}
            }
        }
    };

    tokio::select! {
        _ = c2g => { tracing::info!("🎙️ [LiveWS] Client closed connection"); }
        _ = g2c => { tracing::info!("🎙️ [LiveWS] Gemini closed connection"); }
    }
}
