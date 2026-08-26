//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / ws
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Telemetry]`, `[LiveWS]`
//! - **Witness Tests**: `ws_tests::*`

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
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub const WS_HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
pub const WS_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);
pub const WS_AUTH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
pub const WS_LIVE_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
pub const LIVE_MAX_SESSION_BYTES: u64 = 100 * 1024 * 1024;
pub const LIVE_DEBOUNCE_BYTES: u64 = 256 * 1024;
pub const LIVE_ESTIMATED_COST_PER_SEC: f64 = 0.00005;

pub const WS_CLOSE_AUTH_INVALID: u16 = 4401;
pub const WS_CLOSE_AUTH_TIMEOUT: u16 = 4408;
pub const WS_CLOSE_IDLE_TIMEOUT: u16 = 4409;
pub const WS_CLOSE_SESSION_CAP_EXCEEDED: u16 = 4410;

/// Global connection-count semaphore guarding against slowloris resource exhaustion.
static WS_CONNECTION_SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(250)));

/// Semaphore bounding concurrent async oversight resolution tasks spawned from WebSocket frames.
static WS_OVERSIGHT_SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(10)));

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

    // Extract the subprotocol the client sent (e.g., "bearer.<token>" or "tadpole-pulse-v1")
    let protocol = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_default();

    // Check for binary pulse protocol support via exact token matching
    let full_protocol_header = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let is_pulse_active = full_protocol_header
        .split(',')
        .any(|s| s.trim() == "tadpole-pulse-v1");

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

    let ws = if !protocol.is_empty() {
        ws.protocols([protocol])
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

    if !is_pre_authenticated {
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
                // Send auth_ok
                let ok_msg = serde_json::json!({ "type": "auth_ok" });
                if let Ok(ok_str) = serde_json::to_string(&ok_msg) {
                    let _ = sender.send(Message::Text(ok_str.into())).await;
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

    // Spawn a task that forwards global broadcast events to this client
    let mut send_task = tokio::spawn(async move {
        let mut heartbeat = tokio::time::interval(WS_HEARTBEAT_INTERVAL);
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        heartbeat.tick().await;

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    if sender.send(Message::Ping(axum::body::Bytes::new())).await.is_err() {
                        break;
                    }
                    let hb = serde_json::json!({
                        "type": "heartbeat",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    if let Ok(hb_str) = serde_json::to_string(&hb) {
                        if sender.send(Message::Text(hb_str.into())).await.is_err() {
                            break;
                        }
                    }
                }

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

                result = audio_rx.recv() => {
                    match result {
                        Ok(msg) => {
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

                result = pulse_rx.recv() => {
                    match result {
                        Ok(pulse) if is_pulse_active => {
                            if let Ok(encoded) = rmp_serde::encode::to_vec_named(&*pulse) {
                                let mut bin = Vec::with_capacity(encoded.len() + 1);
                                bin.push(0x02);
                                bin.extend_from_slice(&encoded);
                                if sender.send(Message::Binary(bin.into())).await.is_err() {
                                    break;
                                }
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
                                if val.get("type").and_then(|v| v.as_str()) == Some("oversight:decision") {
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

                                                let decision_payload = crate::agent::types::OversightDecision {
                                                    decision: norm_decision,
                                                    signature: Some(sig.to_string()),
                                                    verifying_key: Some(pk.to_string()),
                                                    override_slot,
                                                    timestamp,
                                                    nonce,
                                                };

                                                let ws_state_clone = ws_state.clone();
                                                let id_clone = id.to_string();

                                                if let Ok(permit) = WS_OVERSIGHT_SEMAPHORE.clone().try_acquire_owned() {
                                                    tokio::spawn(async move {
                                                        let _permit = permit;
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
                                                } else {
                                                    tracing::warn!("⚠️ WS: Oversight decision queue full; shedding task");
                                                }
                                            }
                                        }
                                    }
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
                        tracing::info!("⏱️ WS idle timeout reached ({}s), closing connection", WS_IDLE_TIMEOUT.as_secs());
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
    if pre_auth.is_none() {
        return Err(AppError::Unauthorized("Unauthorized".to_string()));
    }

    let permit = WS_CONNECTION_SEMAPHORE
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            AppError::RateLimit("Concurrent WebSocket connection limit reached".to_string())
        })?;

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
        .on_upgrade(move |socket| handle_live_socket(socket, state, permit))
        .into_response())
}

/// Helper to verify the Origin header for WebSocket upgrades (CSRF protection).
fn verify_origin(headers: &HeaderMap) -> Result<(), StatusCode> {
    let is_production = std::env::var("TADPOLE_ENV")
        .map(|v| v.eq_ignore_ascii_case("production"))
        .unwrap_or(false);

    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let allowed_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
        let allowed: Vec<&str> = allowed_env
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let is_allowed = if is_production {
            if allowed.is_empty() {
                tracing::error!(
                    "🚫 SECURITY: ALLOWED_ORIGINS is empty in production. All WS origins are rejected."
                );
                false
            } else {
                allowed.iter().any(|a| a.eq_ignore_ascii_case(origin))
            }
        } else {
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
            let mut ok = defaults.iter().any(|d| d.eq_ignore_ascii_case(origin));
            if !ok && !allowed.is_empty() {
                ok = allowed.iter().any(|a| a.eq_ignore_ascii_case(origin));
            }
            ok
        };

        if !is_allowed {
            tracing::warn!("🚫 WS upgrade rejected: unexpected Origin '{}'", origin);
            return Err(StatusCode::FORBIDDEN);
        }
    } else if is_production {
        tracing::warn!("🚫 WS upgrade rejected: missing Origin header in production");
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

async fn handle_live_socket(
    mut client_ws: WebSocket,
    state: Arc<AppState>,
    _permit: OwnedSemaphorePermit,
) {
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

    // Budget Guard pre-flight check (entity: system:live_voice)
    let live_entity_id = "system:live_voice";
    match state
        .security
        .budget_guard
        .check_budget(live_entity_id, 0.05)
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            tracing::warn!("🚫 [LiveWS] Insufficient budget for Live Voice streaming");
            let _ = client_ws
                .send(Message::Text(
                    "Error: Insufficient budget quota for Live Voice session.".into(),
                ))
                .await;
            return;
        }
        Err(e) => {
            tracing::error!("❌ [LiveWS] Budget check failed (failing closed): {}", e);
            let _ = client_ws
                .send(Message::Text("Error: budget_verification_failed".into()))
                .await;
            return;
        }
    }

    let gemini_url = format!(
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.MultimodalLive?key={}",
        api_key
    );

    use tokio_tungstenite::connect_async;
    let connect_future = connect_async::<String>(gemini_url);
    let (gemini_ws, _) = match tokio::time::timeout(WS_LIVE_CONNECT_TIMEOUT, connect_future).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(e)) => {
            let sanitized_err = state.security.secret_redactor.redact(&e.to_string());
            tracing::error!("❌ [LiveWS] Failed to connect to Gemini: {}", sanitized_err);
            let _ = client_ws
                .send(Message::Text("Error: upstream_connect_failed".into()))
                .await;
            return;
        }
        Err(_) => {
            tracing::error!("❌ [LiveWS] Gemini connection attempt timed out after 10 seconds");
            let _ = client_ws
                .send(Message::Text("Error: upstream_connect_timeout".into()))
                .await;
            return;
        }
    };

    let (gemini_sender, mut gemini_receiver) = gemini_ws.split();
    let gemini_sender = Arc::new(tokio::sync::Mutex::new(gemini_sender));
    let (client_sender, mut client_receiver) = client_ws.split();
    let client_sender = Arc::new(tokio::sync::Mutex::new(client_sender));

    tracing::info!("🎙️ [LiveWS] Gemini Live Proxy Established with Budget Guard");

    let state_for_c2g = state.clone();
    let client_sender_c2g = client_sender.clone();
    let gemini_sender_c2g = gemini_sender.clone();

    let c2g = async move {
        let mut bytes_sent: u64 = 0;
        let mut last_meter_time = std::time::Instant::now();
        let mut accumulated_bytes: u64 = 0;

        while let Some(msg) = client_receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(t)) => {
                    let chunk_len = t.len() as u64;
                    bytes_sent += chunk_len;
                    accumulated_bytes += chunk_len;
                    if bytes_sent > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Session byte cap exceeded ({} bytes). Closing c2g pipe.",
                            bytes_sent
                        );
                        let mut sender = client_sender_c2g.lock().await;
                        let _ = sender
                            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                                code: WS_CLOSE_SESSION_CAP_EXCEEDED,
                                reason: "Session cap exceeded".into(),
                            })))
                            .await;
                        break;
                    }
                    let mut g_sender = gemini_sender_c2g.lock().await;
                    let _ = g_sender
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            t.as_str().into(),
                        ))
                        .await;
                }
                Ok(axum::extract::ws::Message::Binary(b)) => {
                    let chunk_len = b.len() as u64;
                    bytes_sent += chunk_len;
                    accumulated_bytes += chunk_len;
                    if bytes_sent > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Session byte cap exceeded ({} bytes). Closing c2g pipe.",
                            bytes_sent
                        );
                        let mut sender = client_sender_c2g.lock().await;
                        let _ = sender
                            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                                code: WS_CLOSE_SESSION_CAP_EXCEEDED,
                                reason: "Session cap exceeded".into(),
                            })))
                            .await;
                        break;
                    }
                    let mut g_sender = gemini_sender_c2g.lock().await;
                    let _ = g_sender
                        .send(tokio_tungstenite::tungstenite::Message::Binary(b))
                        .await;
                }
                _ => {}
            }

            if last_meter_time.elapsed() >= std::time::Duration::from_secs(5)
                || accumulated_bytes >= LIVE_DEBOUNCE_BYTES
            {
                let secs = last_meter_time.elapsed().as_secs_f64();
                let cost_usd = (secs * LIVE_ESTIMATED_COST_PER_SEC).max(0.0001);
                if let Err(e) = state_for_c2g
                    .security
                    .budget_guard
                    .record_usage(live_entity_id, cost_usd)
                    .await
                {
                    tracing::error!("🚫 [LiveWS] Failed to record usage (failing closed): {}", e);
                    break;
                }

                match state_for_c2g
                    .security
                    .budget_guard
                    .check_budget(live_entity_id, 0.01)
                    .await
                {
                    Ok(true) => {}
                    Ok(false) => {
                        tracing::warn!("🚫 [LiveWS] Quota exhausted during active stream. Terminating session.");
                        break;
                    }
                    Err(e) => {
                        tracing::error!(
                            "🚫 [LiveWS] Mid-stream budget check failed (failing closed): {}",
                            e
                        );
                        break;
                    }
                }

                last_meter_time = std::time::Instant::now();
                accumulated_bytes = 0;
            }
        }
    };

    let client_sender_g2c = client_sender.clone();
    let g2c = async move {
        let mut downstream_bytes: u64 = 0;
        while let Some(msg) = gemini_receiver.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(t)) => {
                    downstream_bytes += t.len() as u64;
                    if downstream_bytes > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Downstream byte cap exceeded ({} bytes).",
                            downstream_bytes
                        );
                        break;
                    }
                    let mut sender = client_sender_g2c.lock().await;
                    let _ = sender
                        .send(axum::extract::ws::Message::Text(t.to_string().into()))
                        .await;
                }
                Ok(tokio_tungstenite::tungstenite::Message::Binary(b)) => {
                    downstream_bytes += b.len() as u64;
                    if downstream_bytes > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Downstream byte cap exceeded ({} bytes).",
                            downstream_bytes
                        );
                        break;
                    }
                    let mut sender = client_sender_g2c.lock().await;
                    let _ = sender.send(axum::extract::ws::Message::Binary(b)).await;
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
fn redact_headers(headers: &HeaderMap) -> String {
    const REDACTED_HEADERS: &[&str] = &[
        "authorization",
        "proxy-authorization",
        "cookie",
        "set-cookie",
        "sec-websocket-protocol",
        "x-api-key",
        "x-goog-api-key",
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
