//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! - Session internals preserve authentication, bounded transport and permit ownership.
//!
//! ### 🔍 Debugging & Observability
//! - Session failures use the existing sanitized tracing targets.

use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::HeaderMap,
    response::IntoResponse,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::OwnedSemaphorePermit;

use super::shared::{
    redact_headers, verify_origin, WS_CONNECTION_SEMAPHORE, WS_OVERSIGHT_SEMAPHORE,
};
use super::{
    WS_AUTH_TIMEOUT, WS_CLOSE_AUTH_INVALID, WS_CLOSE_AUTH_TIMEOUT, WS_CLOSE_IDLE_TIMEOUT,
    WS_HEARTBEAT_INTERVAL, WS_IDLE_TIMEOUT,
};

/// The HTTP upgrade endpoint for WebSockets.
/// Supports both post-upgrade JSON auth frame (primary for web clients)
/// and Sec-WebSocket-Protocol Bearer token header (RFC 6455).
#[tracing::instrument(skip(state, headers, ws, pre_auth), name = "system::ws_upgrade")]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    pre_auth: Option<Extension<crate::middleware::auth::PreAuthenticated>>,
) -> Result<impl IntoResponse, AppError> {
    // Acquire connection permit before processing upgrade
    let permit = WS_CONNECTION_SEMAPHORE
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            AppError::RateLimit("Concurrent WebSocket connection limit reached".to_string())
        })?;

    tracing::debug!("📥 WS Handshake Headers: {}", redact_headers(&headers));

    // CSRF protection: verify the Origin header for WS upgrades
    if let Err(status) = verify_origin(&headers) {
        return Ok(status.into_response());
    }

    // Extract all client-requested subprotocols and detect capabilities
    let full_protocol_header = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let requested_protocols: Vec<String> = full_protocol_header
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let is_pulse_active = requested_protocols.iter().any(|s| s == "tadpole-pulse-v1");

    let is_pre_authenticated = pre_auth.is_some();
    let redacted_protocols: Vec<&str> = requested_protocols
        .iter()
        .map(|p| {
            if p.starts_with("bearer.") {
                "bearer.***"
            } else {
                p.as_str()
            }
        })
        .collect();
    tracing::info!(
        "✅ WebSocket handshake upgrade. Requested Protocols: [{}], Pre-Authenticated: {}",
        redacted_protocols.join(", "),
        is_pre_authenticated
    );

    // Echo back all requested protocols so the client knows which capabilities are active
    let ws = if !requested_protocols.is_empty() {
        ws.protocols(requested_protocols)
    } else {
        ws
    };

    let ws = ws.max_message_size(1024 * 1024).max_frame_size(1024 * 1024);

    Ok(ws
        .on_upgrade(move |socket| {
            handle_socket(socket, state, is_pulse_active, is_pre_authenticated, permit)
        })
        .into_response())
}

/// The actual bi-directional WebSocket loop handling messaging.
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    is_pulse_active: bool,
    is_pre_authenticated: bool,
    _permit: OwnedSemaphorePermit,
) {
    let (mut sender, mut receiver) = socket.split();

    if is_pre_authenticated {
        // Send auth_ok for pre-authenticated connections (e.g. Sec-WebSocket-Protocol Bearer)
        let ok_msg = serde_json::json!({ "type": "auth_ok" });
        if let Ok(ok_str) = serde_json::to_string(&ok_msg) {
            if sender.send(Message::Text(ok_str.into())).await.is_err() {
                tracing::warn!("⚠️ WS: Failed to deliver auth_ok — aborting session");
                return;
            }
        }
    } else {
        // Post-upgrade auth flow: wait for the JSON auth frame with a 5-second timeout
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

        match tokio::time::timeout(WS_AUTH_TIMEOUT, &mut auth_future).await {
            Ok(Some(true)) => {
                // Send auth_ok — bail if delivery fails to prevent ghost authenticated sessions
                let ok_msg = serde_json::json!({ "type": "auth_ok" });
                if let Ok(ok_str) = serde_json::to_string(&ok_msg) {
                    if sender.send(Message::Text(ok_str.into())).await.is_err() {
                        tracing::warn!("⚠️ WS: Failed to deliver auth_ok — aborting session");
                        return;
                    }
                }
            }
            Ok(Some(false)) => {
                let err_msg =
                    serde_json::json!({ "type": "auth_error", "message": "Invalid credentials" });
                if let Ok(err_str) = serde_json::to_string(&err_msg) {
                    let _ = sender.send(Message::Text(err_str.into())).await;
                }
                let _ = sender
                    .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                        code: WS_CLOSE_AUTH_INVALID,
                        reason: "Unauthorized".into(),
                    })))
                    .await;
                return;
            }
            _ => {
                let _ = sender
                    .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                        code: WS_CLOSE_AUTH_TIMEOUT,
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

    // Send connection welcome message with build version
    let welcome_msg = serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "source": "System",
        "text": format!("Connected to Tadpole OS [Rust Engine v{}]", env!("CARGO_PKG_VERSION")),
        "severity": "success",
        "category": "system",
        "tags": ["ws"]
    });
    if let Ok(welcome_str) = serde_json::to_string(&welcome_msg) {
        let _ = sender.send(Message::Text(welcome_str.into())).await;
    }

    // mpsc channel: recv_task signals send_task for pong echo, auth_ok, and idle close
    enum RecvSignal {
        Pong,
        AuthOk,
        IdleClose,
    }
    let (signal_tx, mut signal_rx) = tokio::sync::mpsc::channel::<RecvSignal>(4);

    // Spawn a task that forwards global broadcast events to this client
    let mut send_task = tokio::spawn(async move {
        let mut heartbeat = tokio::time::interval(WS_HEARTBEAT_INTERVAL);
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        heartbeat.tick().await;
        let mut consecutive_send_failures: u8 = 0;

        // Macro for resilient send: retries once on transient failure before breaking.
        macro_rules! resilient_send {
            ($sender:expr, $msg:expr, $label:expr, $fail_count:expr) => {
                if $sender.send($msg).await.is_err() {
                    $fail_count += 1;
                    if $fail_count >= 3 {
                        tracing::warn!(
                            "⚠️ [WS] {} send failed {} consecutive times, closing",
                            $label,
                            $fail_count
                        );
                        break;
                    }
                    tracing::debug!(
                        "⚠️ [WS] {} send failed (attempt {}), continuing",
                        $label,
                        $fail_count
                    );
                } else {
                    $fail_count = 0;
                }
            };
        }

        loop {
            tokio::select! {
                // Handle signals from recv_task: pong echo, auth_ok, and idle close
                Some(signal) = signal_rx.recv() => {
                    match signal {
                        RecvSignal::Pong => {
                            let pong = serde_json::json!({ "type": "pong", "timestamp": chrono::Utc::now().to_rfc3339() });
                            if let Ok(pong_str) = serde_json::to_string(&pong) {
                                resilient_send!(sender, Message::Text(pong_str.into()), "pong", consecutive_send_failures);
                            }
                        }
                        RecvSignal::AuthOk => {
                            let ok = serde_json::json!({ "type": "auth_ok" });
                            if let Ok(ok_str) = serde_json::to_string(&ok) {
                                resilient_send!(sender, Message::Text(ok_str.into()), "auth_ok", consecutive_send_failures);
                            }
                        }
                        RecvSignal::IdleClose => {
                            let _ = sender.send(Message::Close(Some(axum::extract::ws::CloseFrame {
                                code: WS_CLOSE_IDLE_TIMEOUT,
                                reason: "Idle timeout".into(),
                            }))).await;
                            break;
                        }
                    }
                }

                _ = heartbeat.tick() => {
                    resilient_send!(sender, Message::Ping(axum::body::Bytes::new()), "ping", consecutive_send_failures);
                    let hb = serde_json::json!({
                        "type": "heartbeat",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    if let Ok(hb_str) = serde_json::to_string(&hb) {
                        resilient_send!(sender, Message::Text(hb_str.into()), "heartbeat", consecutive_send_failures);
                    }
                }

                result = log_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if let Ok(json_str) = serde_json::to_string(&msg) {
                                resilient_send!(sender, Message::Text(json_str.into()), "log", consecutive_send_failures);
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Log subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }

                result = event_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if let Ok(json_str) = serde_json::to_string(&msg) {
                                resilient_send!(sender, Message::Text(json_str.into()), "event", consecutive_send_failures);
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Event subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }

                result = telemetry_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if let Ok(json_str) = serde_json::to_string(&msg) {
                                resilient_send!(sender, Message::Text(json_str.into()), "telemetry", consecutive_send_failures);
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Telemetry subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }

                result = audio_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            let mut bin = Vec::with_capacity(msg.len() + 1);
                            bin.push(0x01);
                            bin.extend_from_slice(&msg);
                            resilient_send!(sender, Message::Binary(bin.into()), "audio", consecutive_send_failures);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Audio subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }

                result = pulse_rx.recv() => {
                    match result {
                        Ok(pulse) if is_pulse_active => {
                            if let Ok(encoded) = rmp_serde::encode::to_vec_named(&*pulse) {
                                let mut bin = Vec::with_capacity(encoded.len() + 1);
                                bin.push(0x02);
                                bin.extend_from_slice(&encoded);
                                resilient_send!(sender, Message::Binary(bin.into()), "pulse", consecutive_send_failures);
                            }
                        }
                        Ok(_) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Pulse subscription lagged by {} frames", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        }
    });

    // Spawn a task to drain the receiver and detect client disconnects
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
                                match val.get("type").and_then(|v| v.as_str()) {
                                    // App-level ping: signal send_task to echo pong
                                    Some("ping") => {
                                        let _ = signal_tx.try_send(RecvSignal::Pong);
                                    }
                                    Some("auth") => {
                                        tracing::debug!("ℹ️ WS: Received post-upgrade auth frame on already authenticated session");
                                        let _ = signal_tx.try_send(RecvSignal::AuthOk);
                                    }
                                    Some("oversight:decision") => {
                                        if let Some(id) = val.get("id").and_then(|v| v.as_str()) {
                                            if let (Some(decision), Some(sig), Some(pk)) = (
                                                val.get("decision").and_then(|v| v.as_str()),
                                                val.get("signature").and_then(|v| v.as_str()),
                                                val.get("verifying_key").and_then(|v| v.as_str()),
                                            ) {
                                                let norm_decision = decision.trim().to_lowercase();
                                                if norm_decision == "approved" || norm_decision == "rejected" {
                                                    let override_slot = val.get("override_slot").and_then(|v| v.as_str()).map(|s| s.to_string());
                                                    let timestamp = val.get("timestamp").and_then(|v| v.as_i64());
                                                    let nonce = val.get("nonce").and_then(|v| v.as_str()).map(|s| s.to_string());
                                                    let user_answer = val.get("user_answer").and_then(|v| v.as_str()).map(|s| s.to_string());

                                                    let decision_payload = crate::agent::types::OversightDecision {
                                                        decision: norm_decision,
                                                        signature: Some(sig.to_string()),
                                                        verifying_key: Some(pk.to_string()),
                                                        override_slot,
                                                        timestamp,
                                                        nonce,
                                                        user_answer,
                                                    };

                                                    let ws_state_clone = ws_state.clone();
                                                    let id_clone = id.to_string();

                                                    if let Ok(permit) = WS_OVERSIGHT_SEMAPHORE.clone().try_acquire_owned() {
                                                        tokio::spawn(async move {
                                                            let _permit = permit;
                                                            if let Err(e) = crate::routes::oversight::resolve_oversight_decision(
                                                                &ws_state_clone,
                                                                &id_clone,
                                                                &decision_payload,
                                                            )
                                                            .await
                                                            {
                                                                match e {
                                                                    crate::error::AppError::NotFound(_) | crate::error::AppError::Conflict(_) => {
                                                                        tracing::debug!("ℹ️ WS: Oversight decision for {} already resolved: {:?}", id_clone, e);
                                                                    }
                                                                    _ => {
                                                                        tracing::error!("🚫 WS: Failed to resolve oversight decision for {}: {:?}", id_clone, e);
                                                                    }
                                                                }
                                                            }
                                                        });
                                                    } else {
                                                        tracing::warn!("⚠️ WS: Oversight decision queue full; shedding task");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some(Ok(_)) => {
                            last_activity = tokio::time::Instant::now();
                        }
                        Some(Err(e)) => {
                            tracing::warn!("⚠️ WS receive error: {:?}", e);
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(WS_HEARTBEAT_INTERVAL) => {
                    if last_activity.elapsed() > WS_IDLE_TIMEOUT {
                        tracing::info!("⏱️ WS idle timeout reached ({}s), sending 4409 close frame", WS_IDLE_TIMEOUT.as_secs());
                        let _ = signal_tx.try_send(RecvSignal::IdleClose);
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => { recv_task.abort(); }
        _ = &mut recv_task => { send_task.abort(); }
    }

    tracing::info!("🔗 WebSocket Disconnected.");
}
