//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **A2A Routes**: Exposes incoming agent-to-agent HTTP endpoints and handles signature/keyring verification.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing or invalid HMAC-SHA256 signature headers.
//!

use crate::agent::runner::a2a_mailbox::{A2AMailbox, MailboxEnvelope};
use crate::agent::runner::tools::capability::verify_a2a_envelope;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Handles incoming A2A envelopes sent from remote agents.
/// Authenticates the sender cryptographically using X-A2A-Signature and enforces abuse controls.
pub async fn receive_envelope(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<MailboxEnvelope>,
) -> Result<impl IntoResponse, AppError> {
    // 0. Enforce envelope field sanity and payload size limits
    if payload.id.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Envelope ID cannot be empty".to_string(),
        ));
    }
    if payload.source_agent_id.trim().is_empty() || payload.target_agent_id.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Source and target agent IDs are required".to_string(),
        ));
    }
    if payload.instruction.len() > 128 * 1024 {
        return Err(AppError::BadRequest(
            "A2A envelope instruction exceeds maximum size of 128KB".to_string(),
        ));
    }

    // 1. Verify A2A cryptographic signature
    let sig_header = headers
        .get("X-A2A-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Forbidden("Missing X-A2A-Signature header".to_string()))?;

    // Serialize payload to verify signature over JSON string
    let payload_str = serde_json::to_string(&payload)
        .map_err(|e| AppError::BadRequest(format!("Failed to serialize payload: {}", e)))?;

    if !verify_a2a_envelope(&payload_str, sig_header) {
        return Err(AppError::Forbidden("Invalid A2A signature".to_string()));
    }

    tracing::info!(
        "📬 [A2A Mailbox] Validated signature for envelope {} from {} to {}",
        payload.id,
        payload.source_agent_id,
        payload.target_agent_id
    );

    // 2. Insert validated envelope into A2A mailbox database
    let mailbox = A2AMailbox::new(state.resources.pool.clone());
    mailbox.send_envelope(&payload).await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "status": "queued", "envelope_id": payload.id })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::a2a_mailbox::MailboxEnvelope;
    use crate::agent::runner::tools::capability::{
        reset_keyring_for_test, set_key, sign_a2a_envelope,
    };
    use crate::state::AppState;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_receive_envelope_signature_validation() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        sqlx::migrate!("./migrations")
            .run(&state.resources.pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO agents (id, name, role, department, description, status, metadata) \
             VALUES ('agent-bob', 'Bob', 'assistant', 'engineering', 'bob desc', 'active', '{}')",
        )
        .execute(&state.resources.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO mission_history (id, agent_id, title, status) \
             VALUES ('mission-abc', 'agent-bob', 'test mission', 'active')",
        )
        .execute(&state.resources.pool)
        .await
        .unwrap();

        // Setup signature key in keyring
        let test_key = [7u8; 32];
        reset_keyring_for_test();
        set_key("test_a2a_key", test_key);
        set_key("curr", test_key); // Set it as current active key to allow signing

        let app = Router::new()
            .route("/engine/a2a/mailbox/send", post(receive_envelope))
            .with_state(state);

        let envelope = MailboxEnvelope {
            id: "msg-123".to_string(),
            mission_id: "mission-abc".to_string(),
            source_agent_id: "agent-alice".to_string(),
            target_agent_id: "agent-bob".to_string(),
            instruction: "test message".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: None,
        };

        let payload_str = serde_json::to_string(&envelope).unwrap();
        let valid_signature = sign_a2a_envelope(&payload_str);

        // 1. Success case: Valid signature
        let req = Request::builder()
            .method("POST")
            .uri("/engine/a2a/mailbox/send")
            .header("X-A2A-Signature", &valid_signature)
            .header("Content-Type", "application/json")
            .body(Body::from(payload_str.clone()))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        // 2. Failure case: Invalid signature
        let req = Request::builder()
            .method("POST")
            .uri("/engine/a2a/mailbox/send")
            .header(
                "X-A2A-Signature",
                "test_a2a_key:00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            )
            .header("Content-Type", "application/json")
            .body(Body::from(payload_str.clone()))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // 3. Failure case: Missing signature
        let req = Request::builder()
            .method("POST")
            .uri("/engine/a2a/mailbox/send")
            .header("Content-Type", "application/json")
            .body(Body::from(payload_str))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
