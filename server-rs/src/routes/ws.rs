//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **WebSocket Real-Time Bridge**: Orchestrates the bi-directional
//! interface for high-speed logging, engine events, and **Binary Pulse
//! Broadcasting**. Features **Subprotocol Authentication**: enforces
//! the `bearer.<token>` format in the `Sec-WebSocket-Protocol` header
//! to bypass browser-based Authorization header limitations. Implements
//! the **Gemini Live Proxy**: securely pipes multimodal audio streams
//! between the client and Google's backend, protecting server-side API
//! keys. AI agents must ensure the `sec-websocket-protocol` is echoed in
//! the response to prevent RFC 6455 handshake failures (WS-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Unexpected connection drops due to missing
//!   subprotocol acknowledgments, 403 Forbidden on Origin mismatches,
//!   or buffer overflows during high-frequency pulse bursts.
//! - **Trace Scope**: `server-rs::routes::ws`

use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

const WS_HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
const WS_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);
const LIVE_MAX_SESSION_BYTES: u64 = 100 * 1024 * 1024;

/// The HTTP upgrade endpoint for WebSockets.
/// Auth is handled by the middleware layer (Bearer header or Sec-WebSocket-Protocol) or post-upgrade.
/// SEC-01: The client sends `Sec-WebSocket-Protocol: bearer.<token>` because browsers
/// cannot set Authorization headers on WebSocket upgrades. We MUST echo the protocol
/// back in the upgrade response, or the browser will immediately close the connection.
#[tracing::instrument(skip(state, headers, ws, pre_auth), name = "system::ws_upgrade")]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    pre_auth: Option<Extension<crate::middleware::auth::PreAuthenticated>>,
) -> Result<impl IntoResponse, AppError> {
    // N-06: Redact credential-bearing headers to prevent secret leakage at debug level.
    // Authorization, Cookie, Sec-WebSocket-Protocol (may contain bearer tokens), and X-Api-Key
    // are replaced with "[REDACTED]" before logging.
    tracing::debug!("📥 WS Handshake Headers: {}", redact_headers(&headers));
    // CSRF protection: verify the Origin header for WS upgrades
    if let Err(status) = verify_origin(&headers) {
        return Ok(status.into_response());
    }

    // Extract the subprotocol the client sent (e.g., "bearer.tadpole-dev-token-2026")
    // SEC-01: Split the comma-separated list and pick the first one (usually the bearer token)
    let protocol = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_default();

    // Check for binary pulse protocol support
    // We check the ORIGINAL header for the pulse flag since it might not be the first one
    let full_protocol_header = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let is_pulse_active = full_protocol_header.contains("tadpole-pulse-v1");

    let is_pre_authenticated = pre_auth.is_some();
    let redacted_protocol = if protocol.starts_with("bearer.") {
        "bearer.***"
    } else {
        &protocol
    };
    tracing::info!(
        "✅ WebSocket handshake upgrade. Selected Protocol: {}, Pre-Authenticated: {}",
        redacted_protocol,
        is_pre_authenticated
    );

    // Echo ONLY the selected subprotocol back. Without this, the browser closes the WS immediately
    // because RFC 6455 requires the server to acknowledge *one* requested subprotocol.
    // The idiomatic Axum way is to use .protocols().
    let ws = if !protocol.is_empty() {
        ws.protocols([protocol])
    } else {
        ws
    };

    // C-04: 1 MiB cap for the main WS handler (log/event/telemetry/pulse channels).
    // 16 MiB is excessive for JSON payloads; live_voice_handler retains the 16 MiB cap for audio frames.
    let ws = ws.max_message_size(1024 * 1024).max_frame_size(1024 * 1024);

    Ok(ws
        .on_upgrade(move |socket| {
            handle_socket(socket, state, is_pulse_active, is_pre_authenticated)
        })
        .into_response())
}

/// The actual bi-directional WebSocket loop handling messaging.
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    is_pulse_active: bool,
    is_pre_authenticated: bool,
) {
    let (mut sender, mut receiver) = socket.split();

    if !is_pre_authenticated {
        // Post-upgrade auth flow: wait for the JSON auth frame with a 5-second timeout
        let auth_timeout = std::time::Duration::from_secs(5);
        let mut auth_future = Box::pin(async {
            while let Some(msg_res) = receiver.next().await {
                match msg_res {
                    Ok(Message::Text(text)) => {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if val.get("type").and_then(|v| v.as_str()) == Some("auth") {
                                if let Some(token) = val.get("token").and_then(|v| v.as_str()) {
                                    if !token.is_empty()
                                        && !state.security.deploy_token.is_empty()
                                        && crate::middleware::auth::constant_time_eq(
                                            token.as_bytes(),
                                            state.security.deploy_token.as_bytes(),
                                        )
                                    {
                                        return Some(true);
                                    }
                                }
                                return Some(false);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => return None,
                    _ => {}
                }
            }
            None
        });

        match tokio::time::timeout(auth_timeout, &mut auth_future).await {
            Ok(Some(true)) => {
                // Send auth_ok
                let ok_msg = serde_json::json!({ "type": "auth_ok" });
                if let Ok(ok_str) = serde_json::to_string(&ok_msg) {
                    let _ = sender.send(Message::Text(ok_str.into())).await;
                }
            }
            Ok(Some(false)) => {
                // Auth error
                let err_msg =
                    serde_json::json!({ "type": "auth_error", "message": "Invalid credentials" });
                if let Ok(err_str) = serde_json::to_string(&err_msg) {
                    let _ = sender.send(Message::Text(err_str.into())).await;
                }
                // Send Close frame with policy violation code 4001
                let _ = sender
                    .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                        code: 4001,
                        reason: "Unauthorized".into(),
                    })))
                    .await;
                return;
            }
            _ => {
                // Timeout or close
                let _ = sender
                    .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                        code: 4001,
                        reason: "Authentication Timeout".into(),
                    })))
                    .await;
                return;
            }
        }
    }

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
        let mut heartbeat = tokio::time::interval(WS_HEARTBEAT_INTERVAL);
        heartbeat.tick().await; // consume the immediate first tick

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    if sender.send(Message::Ping(Vec::new().into())).await.is_err() {
                        break;
                    }
                }

                // 1. Handle System Logs (LogEntry)
                result = log_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if let Ok(json_str) = serde_json::to_string(&msg) {
                                if sender.send(Message::Text(json_str.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Log subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }

                // 2. Handle Engine Events (serde_json::Value)
                result = event_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if let Ok(json_str) = serde_json::to_string(&msg) {
                                if sender.send(Message::Text(json_str.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Event subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }

                // 3. Handle High-Speed Telemetry (serde_json::Value)
                result = telemetry_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if let Ok(json_str) = serde_json::to_string(&msg) {
                                if sender.send(Message::Text(json_str.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Telemetry subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }

                // 4. Handle Real-Time Audio Streams (Vec<u8> binary chunks)
                result = audio_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            // Prepend header 0x01 (Audio)
                            let mut bin = Vec::with_capacity(msg.len() + 1);
                            bin.push(0x01);
                            bin.extend_from_slice(&msg);
                            if sender.send(Message::Binary(bin.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Audio subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }

                // 5. Handle High-Speed Binary Pulses (MessagePack encoded)
                result = pulse_rx.recv() => {
                    if is_pulse_active {
                        match result {
                            Ok(pulse) => {
                                // MessagePack binary encoding
                                if let Ok(encoded) = rmp_serde::encode::to_vec_named(&*pulse) {
                                    // Prepend header 0x02 (Swarm Pulse)
                                    let mut bin = Vec::with_capacity(encoded.len() + 1);
                                    bin.push(0x02);
                                    bin.extend_from_slice(&encoded);
                                    if sender.send(Message::Binary(bin.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                tracing::warn!("⚠️ [Telemetry] Pulse subscription lagged by {} frames", n);
                                continue;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
        }
    });

    // Spawn a task to drain the receiver and detect client disconnects.
    // M-04: send_task emits Ping frames and this task expects Pong or other inbound
    // activity before the idle timeout elapses.
    let ws_state = state.clone();
    let mut recv_task = tokio::spawn(async move {
        let mut last_activity = tokio::time::Instant::now();
        loop {
            tokio::select! {
                msg = receiver.next() => {
                    match msg {
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Ok(Message::Pong(_))) => {
                            last_activity = tokio::time::Instant::now();
                        }
                        Some(Ok(Message::Binary(bin))) => {
                            last_activity = tokio::time::Instant::now();
                            tracing::debug!("📥 Received binary message of {} bytes", bin.len());
                        }
                        Some(Ok(Message::Text(text))) => {
                            last_activity = tokio::time::Instant::now();
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                                if val.get("type").and_then(|v| v.as_str()) == Some("oversight:decision") {
                                    if let Some(id) = val.get("id").and_then(|v| v.as_str()) {
                                        if let (Some(decision), Some(sig), Some(pk)) = (
                                            val.get("decision").and_then(|v| v.as_str()),
                                            val.get("signature").and_then(|v| v.as_str()),
                                            val.get("verifying_key").and_then(|v| v.as_str()),
                                        ) {
                                            let override_slot = val.get("override_slot").and_then(|v| v.as_str()).map(|s| s.to_string());
                                            let timestamp = val.get("timestamp").and_then(|v| v.as_i64());
                                            let nonce = val.get("nonce").and_then(|v| v.as_str()).map(|s| s.to_string());

                                            let decision_payload = crate::agent::types::OversightDecision {
                                                decision: decision.to_string(),
                                                signature: Some(sig.to_string()),
                                                verifying_key: Some(pk.to_string()),
                                                override_slot,
                                                timestamp,
                                                nonce,
                                            };

                                            let ws_state_clone = ws_state.clone();
                                            let id_clone = id.to_string();
                                            tokio::spawn(async move {
                                                if let Err(e) = super::oversight::resolve_oversight_decision(
                                                    &ws_state_clone,
                                                    &id_clone,
                                                    &decision_payload,
                                                )
                                                .await
                                                {
                                                    tracing::error!("🚫 WS: Failed to resolve oversight decision for {}: {:?}", id_clone, e);
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        Some(Ok(_)) => {
                            last_activity = tokio::time::Instant::now();
                        }
                        Some(Err(_)) => break,
                    }
                }
                _ = tokio::time::sleep(WS_HEARTBEAT_INTERVAL) => {
                    if last_activity.elapsed() > WS_IDLE_TIMEOUT {
                        tracing::info!("⏱️ WS idle timeout reached ({}s), closing connection", WS_IDLE_TIMEOUT.as_secs());
                        break;
                    }
                }
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
#[tracing::instrument(
    skip(state, headers, ws, pre_auth),
    name = "system::live_voice_upgrade"
)]
pub async fn live_voice_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    pre_auth: Option<Extension<crate::middleware::auth::PreAuthenticated>>,
) -> Result<impl IntoResponse, AppError> {
    // N-07: Generic error string to avoid leaking capability names to unauthenticated probes.
    if pre_auth.is_none() {
        return Err(AppError::Unauthorized("Unauthorized".to_string()));
    }
    // CSRF protection: verify the Origin header for WS upgrades
    if let Err(status) = verify_origin(&headers) {
        return Ok(status.into_response());
    }
    let protocol = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_default();

    let ws = if !protocol.is_empty() {
        ws.protocols([protocol])
    } else {
        ws
    };

    let ws = ws
        .max_message_size(16 * 1024 * 1024)
        .max_frame_size(16 * 1024 * 1024);

    Ok(ws
        .on_upgrade(move |socket| handle_live_socket(socket, state))
        .into_response())
}

/// Helper to verify the Origin header for WebSocket upgrades (CSRF protection).
///
/// H-03: In production (`TADPOLE_ENV=production`), ONLY explicitly configured
/// `ALLOWED_ORIGINS` are accepted. Dev-default origins (localhost, tauri) are
/// only permitted in non-production environments. When `ALLOWED_ORIGINS` is
/// unset in production, ALL origins are rejected (fail-closed).
fn verify_origin(headers: &HeaderMap) -> Result<(), StatusCode> {
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let allowed_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
        let allowed: Vec<&str> = allowed_env
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let is_production = std::env::var("TADPOLE_ENV")
            .or_else(|_| std::env::var("ENV"))
            .map(|v| v.eq_ignore_ascii_case("production"))
            .unwrap_or(false);

        let is_allowed = if is_production {
            // H-03: Production mode — ONLY explicit ALLOWED_ORIGINS, no dev defaults.
            // If ALLOWED_ORIGINS is empty, ALL origins are rejected (fail-closed).
            if allowed.is_empty() {
                tracing::error!(
                    "🚫 SECURITY: ALLOWED_ORIGINS is empty in production. All WS origins are rejected. \
                     Set ALLOWED_ORIGINS to your production domain(s)."
                );
                false
            } else {
                allowed.contains(&origin)
            }
        } else {
            // Dev/staging: allow internal defaults for dev parity
            let defaults = [
                "http://localhost:5173",
                "http://127.0.0.1:5173",
                "http://localhost:5174",
                "http://127.0.0.1:5174",
                "http://localhost:8000",
                "http://127.0.0.1:8000",
                "tauri://localhost",
                "http://tauri.localhost",
            ];
            let mut ok = defaults.contains(&origin);
            if !ok && !allowed.is_empty() {
                ok = allowed.contains(&origin);
            }
            ok
        };

        if !is_allowed {
            tracing::warn!("🚫 WS upgrade rejected: unexpected Origin '{}'", origin);
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok(())
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

    // Connect to Gemini with a 10-second timeout to prevent resource hangs
    use tokio_tungstenite::connect_async;
    let connect_future = connect_async::<String>(gemini_url);
    let (gemini_ws, _) =
        match tokio::time::timeout(std::time::Duration::from_secs(10), connect_future).await {
            Ok(Ok(conn)) => conn,
            Ok(Err(e)) => {
                tracing::error!("❌ [LiveWS] Failed to connect to Gemini: {}", e);
                let _ = client_ws
                    .send(Message::Text(
                        format!("Error: Failed to connect to Gemini: {}", e).into(),
                    ))
                    .await;
                return;
            }
            Err(_) => {
                tracing::error!("❌ [LiveWS] Gemini connection attempt timed out after 10 seconds");
                let _ = client_ws
                    .send(Message::Text(
                        "Error: Connection to Gemini timed out after 10 seconds.".into(),
                    ))
                    .await;
                return;
            }
        };

    let (mut gemini_sender, mut gemini_receiver) = gemini_ws.split();
    let (mut client_sender, mut client_receiver) = client_ws.split();

    tracing::info!("🎙️ [LiveWS] Gemini Live Proxy Established");

    // M-03: Per-session byte cap on the client-to-Gemini pipe to prevent cost amplification.
    // A single authenticated session could otherwise stream 16 MiB/frame indefinitely.
    // Pipe Client -> Gemini
    let c2g = async move {
        let mut bytes_sent: u64 = 0;
        while let Some(msg) = client_receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(t)) => {
                    bytes_sent += t.len() as u64;
                    if bytes_sent > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Session byte cap exceeded ({} bytes). Closing c2g pipe.",
                            bytes_sent
                        );
                        break;
                    }
                    let _ = gemini_sender
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            t.as_str().into(),
                        ))
                        .await;
                }
                Ok(axum::extract::ws::Message::Binary(b)) => {
                    bytes_sent += b.len() as u64;
                    if bytes_sent > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Session byte cap exceeded ({} bytes). Closing c2g pipe.",
                            bytes_sent
                        );
                        break;
                    }
                    let _ = gemini_sender
                        .send(tokio_tungstenite::tungstenite::Message::Binary(b))
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
                        .send(axum::extract::ws::Message::Text(t.to_string().into()))
                        .await;
                }
                Ok(tokio_tungstenite::tungstenite::Message::Binary(b)) => {
                    let _ = client_sender
                        .send(axum::extract::ws::Message::Binary(b))
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

/// N-06: Header redaction utility for debug logging.
///
/// Strips credential-bearing headers (`authorization`, `cookie`,
/// `sec-websocket-protocol`, `x-api-key`) from the debug output
/// to prevent secret leakage when `RUST_LOG=debug` is enabled.
fn redact_headers(headers: &HeaderMap) -> String {
    const REDACTED_HEADERS: &[&str] = &[
        "authorization",
        "cookie",
        "sec-websocket-protocol",
        "x-api-key",
    ];

    let mut parts: Vec<String> = Vec::new();
    for (name, value) in headers.iter() {
        let name_lower = name.as_str().to_ascii_lowercase();
        if REDACTED_HEADERS.contains(&name_lower.as_str()) {
            parts.push(format!("{}: [REDACTED]", name));
        } else if let Ok(v) = value.to_str() {
            parts.push(format!("{}: {}", name, v));
        } else {
            parts.push(format!("{}: [non-UTF8]", name));
        }
    }
    format!("{{{}}}", parts.join(", "))
}

// Metadata: [ws]
