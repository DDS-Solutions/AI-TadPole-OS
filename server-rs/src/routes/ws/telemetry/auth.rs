//! @docs ARCHITECTURE:Networking
//! ### AI Assist Note
//! Post-upgrade and pre-authenticated WebSocket authentication handshake.
//! ### 🔍 Debugging & Observability
//! Authentication failures emit sanitized warn traces and terminate early.

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures::sink::SinkExt;
use futures::stream::{SplitSink, SplitStream, StreamExt};
use serde_json::Value;

use super::super::{WS_AUTH_TIMEOUT, WS_CLOSE_AUTH_INVALID, WS_CLOSE_AUTH_TIMEOUT};
use crate::state::AppState;

fn verify_token_match(token: &str, expected: &str) -> bool {
    !token.is_empty()
        && !expected.is_empty()
        && crate::middleware::auth::constant_time_eq(token.as_bytes(), expected.as_bytes())
}

fn extract_auth_token(val: &Value) -> Option<&str> {
    if val.get("type").and_then(|v| v.as_str()) != Some("auth") {
        return None;
    }
    val.get("token").and_then(|v| v.as_str())
}

fn parse_auth_text(text: &str, expected_token: &str) -> Option<bool> {
    let val = serde_json::from_str::<Value>(text).ok()?;
    if val.get("type").and_then(|v| v.as_str()) != Some("auth") {
        return None;
    }
    let Some(token) = extract_auth_token(&val) else {
        return Some(false);
    };
    Some(verify_token_match(token, expected_token))
}

async fn check_auth_frame(
    receiver: &mut SplitStream<WebSocket>,
    expected_token: &str,
) -> Option<bool> {
    while let Some(msg_res) = receiver.next().await {
        let msg = match msg_res {
            Ok(Message::Close(_)) => return None,
            Ok(Message::Text(text)) => text,
            _ => continue,
        };
        if let Some(valid) = parse_auth_text(&msg, expected_token) {
            return Some(valid);
        }
    }
    None
}

async fn send_auth_ok(sender: &mut SplitSink<WebSocket, Message>) -> bool {
    let ok_msg = serde_json::json!({ "type": "auth_ok" });
    let Ok(ok_str) = serde_json::to_string(&ok_msg) else {
        return false;
    };
    if sender.send(Message::Text(ok_str.into())).await.is_err() {
        tracing::warn!("⚠️ WS: Failed to deliver auth_ok — aborting session");
        return false;
    }
    true
}

async fn handle_auth_rejection(
    sender: &mut SplitSink<WebSocket, Message>,
    code: u16,
    reason: &'static str,
) {
    if code == WS_CLOSE_AUTH_INVALID {
        let err_msg = serde_json::json!({ "type": "auth_error", "message": "Invalid credentials" });
        if let Ok(err_str) = serde_json::to_string(&err_msg) {
            let _ = sender.send(Message::Text(err_str.into())).await;
        }
    }
    let _ = sender
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })))
        .await;
}

pub async fn authenticate_socket(
    receiver: &mut SplitStream<WebSocket>,
    sender: &mut SplitSink<WebSocket, Message>,
    state: &AppState,
    is_pre_authenticated: bool,
) -> bool {
    if is_pre_authenticated {
        return send_auth_ok(sender).await;
    }

    let auth_check = check_auth_frame(receiver, &state.security.deploy_token);
    let outcome = tokio::time::timeout(WS_AUTH_TIMEOUT, auth_check).await;

    match outcome {
        Ok(Some(true)) => send_auth_ok(sender).await,
        Ok(Some(false)) => {
            handle_auth_rejection(sender, WS_CLOSE_AUTH_INVALID, "Unauthorized").await;
            false
        }
        _ => {
            handle_auth_rejection(sender, WS_CLOSE_AUTH_TIMEOUT, "Authentication Timeout").await;
            false
        }
    }
}
