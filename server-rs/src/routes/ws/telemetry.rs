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
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    http::HeaderMap,
    response::IntoResponse,
    Extension,
};
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::OwnedSemaphorePermit;

mod auth;
mod recv_task;
mod send_task;
mod types;

use auth::authenticate_socket;
use recv_task::spawn_recv_task;
use send_task::{send_welcome_message, spawn_send_task};
use types::RecvSignal;

use super::shared::{redact_headers, verify_origin, WS_CONNECTION_SEMAPHORE};

struct SubprotocolInfo {
    protocols: Vec<String>,
    is_pulse_active: bool,
}

fn extract_subprotocols(headers: &HeaderMap, pre_auth: bool) -> SubprotocolInfo {
    let full_protocol_header = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let protocols: Vec<String> = full_protocol_header
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let is_pulse_active = protocols.iter().any(|s| s == "tadpole-pulse-v1");
    let redacted: Vec<&str> = protocols
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
        redacted.join(", "),
        pre_auth
    );
    SubprotocolInfo {
        protocols,
        is_pulse_active,
    }
}

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
    let permit = WS_CONNECTION_SEMAPHORE
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            AppError::RateLimit("Concurrent WebSocket connection limit reached".to_string())
        })?;

    tracing::debug!("📥 WS Handshake Headers: {}", redact_headers(&headers));

    if let Err(status) = verify_origin(&headers) {
        return Ok(status.into_response());
    }

    let is_pre_authenticated = pre_auth.is_some();
    let info = extract_subprotocols(&headers, is_pre_authenticated);

    let mut ws = if !info.protocols.is_empty() {
        ws.protocols(info.protocols)
    } else {
        ws
    };
    ws = ws.max_message_size(1024 * 1024).max_frame_size(1024 * 1024);

    Ok(ws
        .on_upgrade(move |socket| {
            handle_socket(
                socket,
                state,
                info.is_pulse_active,
                is_pre_authenticated,
                permit,
            )
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

    if !authenticate_socket(&mut receiver, &mut sender, &state, is_pre_authenticated).await {
        return;
    }

    tracing::info!("🔗 High-Performance WebSocket Connected!");
    send_welcome_message(&mut sender).await;

    let (signal_tx, signal_rx) = tokio::sync::mpsc::channel::<RecvSignal>(4);
    let mut send_task = spawn_send_task(sender, state.clone(), is_pulse_active, signal_rx);
    let mut recv_task = spawn_recv_task(receiver, state, signal_tx);

    tokio::select! {
        _ = &mut send_task => { recv_task.abort(); }
        _ = &mut recv_task => { send_task.abort(); }
    }

    tracing::info!("🔗 WebSocket Disconnected.");
}
