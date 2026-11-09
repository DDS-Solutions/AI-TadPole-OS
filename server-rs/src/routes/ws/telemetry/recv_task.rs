//! @docs ARCHITECTURE:Networking
//! ### AI Assist Note
//! Inbound WebSocket message dispatcher for ping/pong, post-upgrade re-auth, and oversight decisions.
//! ### 🔍 Debugging & Observability
//! Dispatches oversight tasks through bounded semaphores and tracks client idle disconnects.

use axum::extract::ws::{Message, WebSocket};
use futures::stream::{SplitStream, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::super::shared::WS_OVERSIGHT_SEMAPHORE;
use super::super::{WS_HEARTBEAT_INTERVAL, WS_IDLE_TIMEOUT};
use super::types::RecvSignal;
use crate::agent::types::OversightDecision;
use crate::state::AppState;

fn parse_oversight_decision(val: &Value) -> Option<(String, OversightDecision)> {
    let id = val.get("id").and_then(|v| v.as_str())?.to_string();
    let decision = val.get("decision").and_then(|v| v.as_str())?;
    let sig = val.get("signature").and_then(|v| v.as_str())?.to_string();
    let pk = val
        .get("verifying_key")
        .and_then(|v| v.as_str())?
        .to_string();

    let norm_decision = decision.trim().to_lowercase();
    if norm_decision != "approved" && norm_decision != "rejected" {
        return None;
    }

    let override_slot = val
        .get("override_slot")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let timestamp = val.get("timestamp").and_then(|v| v.as_i64());
    let nonce = val
        .get("nonce")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let user_answer = val
        .get("user_answer")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let payload = OversightDecision {
        decision: norm_decision,
        signature: Some(sig),
        verifying_key: Some(pk),
        override_slot,
        timestamp,
        nonce,
        user_answer,
    };
    Some((id, payload))
}

fn dispatch_oversight_task(state: Arc<AppState>, id: String, payload: OversightDecision) {
    let Ok(permit) = WS_OVERSIGHT_SEMAPHORE.clone().try_acquire_owned() else {
        tracing::warn!("⚠️ WS: Oversight decision queue full; shedding task");
        return;
    };
    tokio::spawn(async move {
        let _permit = permit;
        if let Err(e) =
            crate::routes::oversight::resolve_oversight_decision(&state, &id, &payload).await
        {
            match e {
                crate::error::AppError::NotFound(_) | crate::error::AppError::Conflict(_) => {
                    tracing::debug!("ℹ️ WS: Oversight decision for {id} already resolved: {e:?}");
                }
                _ => {
                    tracing::error!("🚫 WS: Failed to resolve oversight decision for {id}: {e:?}");
                }
            }
        }
    });
}

fn handle_text_frame(text: &str, state: &Arc<AppState>, signal_tx: &mpsc::Sender<RecvSignal>) {
    let Ok(val) = serde_json::from_str::<Value>(text) else {
        return;
    };
    match val.get("type").and_then(|v| v.as_str()) {
        Some("ping") => {
            let _ = signal_tx.try_send(RecvSignal::Pong);
        }
        Some("auth") => {
            tracing::debug!(
                "ℹ️ WS: Received post-upgrade auth frame on already authenticated session"
            );
            let _ = signal_tx.try_send(RecvSignal::AuthOk);
        }
        Some("oversight:decision") => {
            if let Some((id, payload)) = parse_oversight_decision(&val) {
                dispatch_oversight_task(state.clone(), id, payload);
            }
        }
        _ => {}
    }
}

fn process_incoming_msg(
    msg_res: Result<Message, axum::Error>,
    state: &Arc<AppState>,
    signal_tx: &mpsc::Sender<RecvSignal>,
) -> bool {
    match msg_res {
        Ok(Message::Close(_)) => false,
        Ok(Message::Pong(_)) => true,
        Ok(Message::Binary(bin)) => {
            tracing::debug!("📥 Received binary message of {} bytes", bin.len());
            true
        }
        Ok(Message::Text(text)) => {
            handle_text_frame(&text, state, signal_tx);
            true
        }
        Ok(_) => true,
        Err(e) => {
            tracing::warn!("⚠️ WS receive error: {:?}", e);
            false
        }
    }
}

fn check_idle_timeout(
    last_activity: tokio::time::Instant,
    signal_tx: &mpsc::Sender<RecvSignal>,
) -> bool {
    if last_activity.elapsed() <= WS_IDLE_TIMEOUT {
        return false;
    }
    tracing::info!(
        "⏱️ WS idle timeout reached ({}s), sending 4409 close frame",
        WS_IDLE_TIMEOUT.as_secs()
    );
    let _ = signal_tx.try_send(RecvSignal::IdleClose);
    true
}

fn handle_recv_step(
    msg: Option<Result<Message, axum::Error>>,
    state: &Arc<AppState>,
    signal_tx: &mpsc::Sender<RecvSignal>,
    last_activity: &mut tokio::time::Instant,
) -> bool {
    let Some(res) = msg else {
        return false;
    };
    if !process_incoming_msg(res, state, signal_tx) {
        return false;
    }
    *last_activity = tokio::time::Instant::now();
    true
}

async fn run_recv_loop(
    mut receiver: SplitStream<WebSocket>,
    state: Arc<AppState>,
    signal_tx: mpsc::Sender<RecvSignal>,
) {
    let mut last_activity = tokio::time::Instant::now();
    loop {
        tokio::select! {
            msg = receiver.next() => {
                if !handle_recv_step(msg, &state, &signal_tx, &mut last_activity) { break; }
            }
            _ = tokio::time::sleep(WS_HEARTBEAT_INTERVAL) => {
                if check_idle_timeout(last_activity, &signal_tx) { break; }
            }
        }
    }
}

pub fn spawn_recv_task(
    receiver: SplitStream<WebSocket>,
    state: Arc<AppState>,
    signal_tx: mpsc::Sender<RecvSignal>,
) -> JoinHandle<()> {
    tokio::spawn(run_recv_loop(receiver, state, signal_tx))
}
