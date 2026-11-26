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
use crate::middleware::auth::{resolve_token_role, AuthenticatedRole};
use crate::state::AppState;

fn extract_auth_token(val: &Value) -> Option<&str> {
    if val.get("type").and_then(|v| v.as_str()) != Some("auth") {
        return None;
    }
    val.get("token").and_then(|v| v.as_str())
}

fn parse_auth_text(text: &str, state: &AppState) -> Option<Option<AuthenticatedRole>> {
    let val = serde_json::from_str::<Value>(text).ok()?;
    if val.get("type").and_then(|v| v.as_str()) != Some("auth") {
        return None;
    }
    let Some(token) = extract_auth_token(&val) else {
        return Some(None);
    };
    Some(resolve_token_role(token, state))
}

async fn check_auth_frame(
    receiver: &mut SplitStream<WebSocket>,
    state: &AppState,
) -> Option<Option<AuthenticatedRole>> {
    while let Some(msg_res) = receiver.next().await {
        let msg = match msg_res {
            Ok(Message::Close(_)) => return None,
            Ok(Message::Text(text)) => text,
            _ => continue,
        };
        if let Some(resolved) = parse_auth_text(&msg, state) {
            return Some(resolved);
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
    pre_authenticated_role: Option<AuthenticatedRole>,
) -> Option<AuthenticatedRole> {
    if let Some(role) = pre_authenticated_role {
        return send_auth_ok(sender).await.then_some(role);
    }

    let auth_check = check_auth_frame(receiver, state);
    let outcome = tokio::time::timeout(WS_AUTH_TIMEOUT, auth_check).await;

    match outcome {
        Ok(Some(Some(role))) => send_auth_ok(sender).await.then_some(role),
        Ok(Some(None)) => {
            handle_auth_rejection(sender, WS_CLOSE_AUTH_INVALID, "Unauthorized").await;
            None
        }
        _ => {
            handle_auth_rejection(sender, WS_CLOSE_AUTH_TIMEOUT, "Authentication Timeout").await;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_auth_text_resolves_both_admin_and_deploy_roles() {
        let state = AppState::new_minimal_mock().await;

        let admin_frame = serde_json::json!({
            "type": "auth",
            "token": state.security.admin_token
        })
        .to_string();
        assert_eq!(
            parse_auth_text(&admin_frame, &state),
            Some(Some(AuthenticatedRole::Admin))
        );

        let deploy_frame = serde_json::json!({
            "type": "auth",
            "token": state.security.deploy_token
        })
        .to_string();
        assert_eq!(
            parse_auth_text(&deploy_frame, &state),
            Some(Some(AuthenticatedRole::Deploy))
        );

        let bad_frame = serde_json::json!({
            "type": "auth",
            "token": "invalid-token"
        })
        .to_string();
        assert_eq!(parse_auth_text(&bad_frame, &state), Some(None));
    }
}
