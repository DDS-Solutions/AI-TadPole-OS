//! @docs ARCHITECTURE:Networking
//! ### AI Assist Note
//! Outbound WebSocket broadcast forwarder for logs, events, audio, and high-speed telemetry.
//! ### 🔍 Debugging & Observability
//! Dispatches heartbeat pings and records consecutive network transport dropouts.

use axum::extract::ws::{Message, WebSocket};
use futures::sink::SinkExt;
use futures::stream::SplitSink;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::super::{WS_CLOSE_IDLE_TIMEOUT, WS_HEARTBEAT_INTERVAL};
use super::types::RecvSignal;
use crate::state::AppState;

async fn resilient_send(
    sender: &mut SplitSink<WebSocket, Message>,
    msg: Message,
    label: &str,
    fail_count: &mut u8,
) -> bool {
    if sender.send(msg).await.is_err() {
        *fail_count += 1;
        if *fail_count >= 3 {
            tracing::warn!("⚠️ [WS] {label} send failed {fail_count} consecutive times, closing");
            return false;
        }
        tracing::debug!("⚠️ [WS] {label} send failed (attempt {fail_count}), continuing");
    } else {
        *fail_count = 0;
    }
    true
}

fn encode_audio_frame(msg: &[u8]) -> Vec<u8> {
    let mut bin = Vec::with_capacity(msg.len() + 1);
    bin.push(0x01);
    bin.extend_from_slice(msg);
    bin
}

fn encode_pulse_frame(pulse: &crate::telemetry::pulse_types::SwarmPulse) -> Option<Vec<u8>> {
    let encoded = rmp_serde::encode::to_vec_named(pulse).ok()?;
    let mut bin = Vec::with_capacity(encoded.len() + 1);
    bin.push(0x02);
    bin.extend_from_slice(&encoded);
    Some(bin)
}

pub async fn send_welcome_message(sender: &mut SplitSink<WebSocket, Message>) {
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
}

async fn handle_signal(
    signal: RecvSignal,
    sender: &mut SplitSink<WebSocket, Message>,
    failures: &mut u8,
) -> bool {
    match signal {
        RecvSignal::Pong => {
            let pong =
                serde_json::json!({ "type": "pong", "timestamp": chrono::Utc::now().to_rfc3339() });
            let Ok(pong_str) = serde_json::to_string(&pong) else {
                return true;
            };
            resilient_send(sender, Message::Text(pong_str.into()), "pong", failures).await
        }
        RecvSignal::AuthOk => {
            let ok = serde_json::json!({ "type": "auth_ok" });
            let Ok(ok_str) = serde_json::to_string(&ok) else {
                return true;
            };
            resilient_send(sender, Message::Text(ok_str.into()), "auth_ok", failures).await
        }
        RecvSignal::IdleClose => {
            let _ = sender
                .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                    code: WS_CLOSE_IDLE_TIMEOUT,
                    reason: "Idle timeout".into(),
                })))
                .await;
            false
        }
    }
}

async fn send_heartbeat(sender: &mut SplitSink<WebSocket, Message>, failures: &mut u8) -> bool {
    if !resilient_send(
        sender,
        Message::Ping(axum::body::Bytes::new()),
        "ping",
        failures,
    )
    .await
    {
        return false;
    }
    let hb = serde_json::json!({
        "type": "heartbeat",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    let Ok(hb_str) = serde_json::to_string(&hb) else {
        return true;
    };
    resilient_send(sender, Message::Text(hb_str.into()), "heartbeat", failures).await
}

async fn forward_json_msg<T: serde::Serialize>(
    sender: &mut SplitSink<WebSocket, Message>,
    result: Result<T, RecvError>,
    label: &str,
    failures: &mut u8,
) -> bool {
    match result {
        Ok(msg) => {
            let Ok(s) = serde_json::to_string(&msg) else {
                return true;
            };
            resilient_send(sender, Message::Text(s.into()), label, failures).await
        }
        Err(RecvError::Lagged(n)) => {
            tracing::warn!("⚠️ [Telemetry] {label} subscription lagged by {n} frames");
            true
        }
        Err(RecvError::Closed) => false,
    }
}

async fn forward_audio_msg(
    sender: &mut SplitSink<WebSocket, Message>,
    result: Result<Vec<u8>, RecvError>,
    failures: &mut u8,
) -> bool {
    match result {
        Ok(msg) => {
            let bin = encode_audio_frame(&msg);
            resilient_send(sender, Message::Binary(bin.into()), "audio", failures).await
        }
        Err(RecvError::Lagged(n)) => {
            tracing::warn!("⚠️ [Telemetry] Audio subscription lagged by {n} frames");
            true
        }
        Err(RecvError::Closed) => false,
    }
}

async fn forward_pulse_msg(
    sender: &mut SplitSink<WebSocket, Message>,
    result: Result<Arc<crate::telemetry::pulse_types::SwarmPulse>, RecvError>,
    is_pulse_active: bool,
    failures: &mut u8,
) -> bool {
    match result {
        Ok(pulse) if is_pulse_active => {
            let Some(bin) = encode_pulse_frame(&pulse) else {
                return true;
            };
            resilient_send(sender, Message::Binary(bin.into()), "pulse", failures).await
        }
        Ok(_) => true,
        Err(RecvError::Lagged(n)) => {
            tracing::warn!("⚠️ [Telemetry] Pulse subscription lagged by {n} frames");
            true
        }
        Err(RecvError::Closed) => false,
    }
}

async fn run_send_loop(
    mut sender: SplitSink<WebSocket, Message>,
    state: Arc<AppState>,
    is_pulse_active: bool,
    mut signal_rx: mpsc::Receiver<RecvSignal>,
) {
    let mut heartbeat = tokio::time::interval(WS_HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    heartbeat.tick().await;

    let mut log_rx = state.comms.tx.subscribe();
    let mut event_rx = state.comms.event_tx.subscribe();
    let mut telemetry_rx = state.comms.telemetry_tx.subscribe();
    let mut audio_rx = state.comms.audio_stream_tx.subscribe();
    let mut pulse_rx = state.comms.pulse_tx.subscribe();
    let mut failures: u8 = 0;

    loop {
        tokio::select! {
            Some(signal) = signal_rx.recv() => {
                if !handle_signal(signal, &mut sender, &mut failures).await { break; }
            }
            _ = heartbeat.tick() => {
                if !send_heartbeat(&mut sender, &mut failures).await { break; }
            }
            res = log_rx.recv() => if !forward_json_msg(&mut sender, res, "log", &mut failures).await { break; },
            res = event_rx.recv() => if !forward_json_msg(&mut sender, res, "event", &mut failures).await { break; },
            res = telemetry_rx.recv() => if !forward_json_msg(&mut sender, res, "telemetry", &mut failures).await { break; },
            res = audio_rx.recv() => if !forward_audio_msg(&mut sender, res, &mut failures).await { break; },
            res = pulse_rx.recv() => if !forward_pulse_msg(&mut sender, res, is_pulse_active, &mut failures).await { break; },
        }
    }
}

pub fn spawn_send_task(
    sender: SplitSink<WebSocket, Message>,
    state: Arc<AppState>,
    is_pulse_active: bool,
    signal_rx: mpsc::Receiver<RecvSignal>,
) -> JoinHandle<()> {
    tokio::spawn(run_send_loop(sender, state, is_pulse_active, signal_rx))
}
