//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / a2a
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`, `AppError::Sqlx`
//! - **Telemetry Targets**: `[A2A Mailbox]`
//! - **Witness Tests**: `a2a::tests::test_receive_envelope_signature_validation`, `a2a::tests::test_receive_envelope_canonical_and_replay`

use crate::agent::runner::a2a_mailbox::{A2AMailbox, MailboxEnvelope};
use crate::agent::runner::tools::capability::verify_a2a_canonical;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Handles incoming A2A envelopes sent from remote agents or peers.
/// Authenticates the sender cryptographically using `X-A2A-Signature` (proving authorized swarm membership)
/// and enforces local-target recipient existence with replay prevention.
pub async fn receive_envelope(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<MailboxEnvelope>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Verify A2A cryptographic signature header exists (fail-closed immediately on missing auth)
    let sig_header = headers
        .get("X-A2A-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Forbidden("Missing X-A2A-Signature header".to_string()))?;

    // 2. Enforce local recipient target on inbound route (disallow open-proxy external relaying)
    if payload.target_agent_id.starts_with("http://")
        || payload.target_agent_id.starts_with("https://")
    {
        return Err(AppError::BadRequest(
            "Inbound A2A delivery only accepts local recipient agent IDs, not remote URLs"
                .to_string(),
        ));
    }

    // 3. Validate source agent identity bounds
    if payload.source_agent_id.trim().is_empty() || payload.source_agent_id.len() > 128 {
        return Err(AppError::BadRequest(
            "source_agent_id must be non-empty and at most 128 characters".to_string(),
        ));
    }

    // 4. Enforce mandatory timestamp requirement and windowing (±5 minutes)
    let ts = payload.timestamp.ok_or_else(|| {
        AppError::BadRequest("Missing required 'timestamp' in A2A envelope".to_string())
    })?;
    let now_ms = chrono::Utc::now().timestamp_millis().max(0) as u64;
    let ts_u = ts.max(0) as u64;
    let diff = now_ms.abs_diff(ts_u);
    if diff > 300_000 {
        return Err(AppError::Forbidden(
            "A2A envelope timestamp is expired or too far in the future".to_string(),
        ));
    }

    let nonce = payload.nonce.as_deref().ok_or_else(|| {
        AppError::BadRequest("Missing required 'nonce' in A2A envelope".to_string())
    })?;
    if nonce.trim().is_empty() || nonce.len() > 128 {
        return Err(AppError::BadRequest(
            "nonce must be non-empty and at most 128 characters".to_string(),
        ));
    }

    // 5. Verify A2A cryptographic signature over canonical message (before recording nonce)
    let is_valid = verify_a2a_canonical(
        &payload.id,
        &payload.mission_id,
        &payload.source_agent_id,
        &payload.target_agent_id,
        &payload.instruction,
        ts,
        nonce,
        sig_header,
    );

    if !is_valid {
        return Err(AppError::Forbidden("Invalid A2A signature".to_string()));
    }

    // 6. Mandatory anti-replay defense via used_nonces table (with TTL pruning)
    let prune_threshold = (now_ms.saturating_sub(600_000)) as i64; // Prune nonces older than 10 minutes
    let _ = sqlx::query("DELETE FROM used_nonces WHERE timestamp < ?")
        .bind(prune_threshold)
        .execute(&state.resources.pool)
        .await;

    let res = sqlx::query("INSERT INTO used_nonces (nonce, timestamp) VALUES (?, ?)")
        .bind(nonce)
        .bind(ts)
        .execute(&state.resources.pool)
        .await;

    if let Err(e) = res {
        let is_unique_violation = if let Some(db_err) = e.as_database_error() {
            db_err
                .code()
                .map(|c| c == "2067" || c == "1555" || c == "23000")
                .unwrap_or(false)
        } else {
            false
        } || e.to_string().contains("UNIQUE constraint failed");

        if is_unique_violation {
            return Err(AppError::Forbidden(
                "Replay attack detected: nonce already used".to_string(),
            ));
        }
        return Err(AppError::Sqlx(e));
    }

    // 7. Verify recipient agent exists in local registry or DB
    let agent_exists = state.registry.agents.contains_key(&payload.target_agent_id)
        || sqlx::query_scalar::<_, i64>("SELECT 1 FROM agents WHERE id = ?1")
            .bind(&payload.target_agent_id)
            .fetch_optional(&state.resources.pool)
            .await
            .map_err(AppError::Sqlx)?
            .is_some();

    if !agent_exists {
        return Err(AppError::NotFound(format!(
            "Target recipient agent '{}' does not exist on this node",
            payload.target_agent_id
        )));
    }

    tracing::info!(
        "📬 [A2A Mailbox] Validated signature for envelope {} from {} to {}",
        payload.id,
        payload.source_agent_id,
        payload.target_agent_id
    );

    // 7. Insert validated envelope into A2A mailbox database via chokepoint
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
        reset_keyring_for_test, set_key, sign_a2a_canonical,
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
        set_key("curr", test_key);

        let app = Router::new()
            .route("/engine/a2a/mailbox/send", post(receive_envelope))
            .with_state(state);

        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "test-nonce-101";

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
            timestamp: Some(now),
            nonce: Some(nonce.to_string()),
        };

        let valid_signature = sign_a2a_canonical(
            &envelope.id,
            &envelope.mission_id,
            &envelope.source_agent_id,
            &envelope.target_agent_id,
            &envelope.instruction,
            now,
            nonce,
        );

        let payload_str = serde_json::to_string(&envelope).unwrap();

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
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // 4. Failure case: Missing timestamp or nonce returns BAD_REQUEST
        let no_ts_env = MailboxEnvelope {
            id: "msg-no-ts".to_string(),
            mission_id: "mission-abc".to_string(),
            source_agent_id: "agent-alice".to_string(),
            target_agent_id: "agent-bob".to_string(),
            instruction: "test message".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: None,
            nonce: Some("some-nonce".to_string()),
        };
        let no_ts_payload = serde_json::to_string(&no_ts_env).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri("/engine/a2a/mailbox/send")
            .header("X-A2A-Signature", &valid_signature)
            .header("Content-Type", "application/json")
            .body(Body::from(no_ts_payload))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // 5. Failure case: Non-existent recipient target agent
        let missing_env = MailboxEnvelope {
            id: "msg-missing".to_string(),
            mission_id: "mission-abc".to_string(),
            source_agent_id: "agent-alice".to_string(),
            target_agent_id: "agent-nonexistent".to_string(),
            instruction: "test message".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: Some(now),
            nonce: Some("nonce-missing-agent".to_string()),
        };
        let missing_payload_str = serde_json::to_string(&missing_env).unwrap();
        let sig = sign_a2a_canonical(
            &missing_env.id,
            &missing_env.mission_id,
            &missing_env.source_agent_id,
            &missing_env.target_agent_id,
            &missing_env.instruction,
            now,
            "nonce-missing-agent",
        );
        let req = Request::builder()
            .method("POST")
            .uri("/engine/a2a/mailbox/send")
            .header("X-A2A-Signature", &sig)
            .header("Content-Type", "application/json")
            .body(Body::from(missing_payload_str))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // 6. Failure case: Inbound remote URL rejected
        let remote_env = MailboxEnvelope {
            id: "msg-remote".to_string(),
            mission_id: "mission-abc".to_string(),
            source_agent_id: "agent-alice".to_string(),
            target_agent_id: "http://remote-cluster/send".to_string(),
            instruction: "test message".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: Some(now),
            nonce: Some("nonce-remote".to_string()),
        };
        let remote_payload_str = serde_json::to_string(&remote_env).unwrap();
        let sig = sign_a2a_canonical(
            &remote_env.id,
            &remote_env.mission_id,
            &remote_env.source_agent_id,
            &remote_env.target_agent_id,
            &remote_env.instruction,
            now,
            "nonce-remote",
        );
        let req = Request::builder()
            .method("POST")
            .uri("/engine/a2a/mailbox/send")
            .header("X-A2A-Signature", &sig)
            .header("Content-Type", "application/json")
            .body(Body::from(remote_payload_str))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_receive_envelope_canonical_and_replay() {
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
             VALUES ('mission-canon', 'agent-bob', 'test mission', 'active')",
        )
        .execute(&state.resources.pool)
        .await
        .unwrap();

        let test_key = [9u8; 32];
        reset_keyring_for_test();
        set_key("curr", test_key);

        let app = Router::new()
            .route("/engine/a2a/mailbox/send", post(receive_envelope))
            .with_state(state);

        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "unique-nonce-101";

        let envelope = MailboxEnvelope {
            id: "msg-canon-1".to_string(),
            mission_id: "mission-canon".to_string(),
            source_agent_id: "agent-alice".to_string(),
            target_agent_id: "agent-bob".to_string(),
            instruction: "canonical message payload".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: Some(now),
            nonce: Some(nonce.to_string()),
        };

        let sig = sign_a2a_canonical(
            &envelope.id,
            &envelope.mission_id,
            &envelope.source_agent_id,
            &envelope.target_agent_id,
            &envelope.instruction,
            now,
            nonce,
        );

        let payload_str = serde_json::to_string(&envelope).unwrap();

        // First delivery: succeeds
        let req = Request::builder()
            .method("POST")
            .uri("/engine/a2a/mailbox/send")
            .header("X-A2A-Signature", &sig)
            .header("Content-Type", "application/json")
            .body(Body::from(payload_str.clone()))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        // Second delivery with same nonce: Replay attack rejected
        let req_replay = Request::builder()
            .method("POST")
            .uri("/engine/a2a/mailbox/send")
            .header("X-A2A-Signature", &sig)
            .header("Content-Type", "application/json")
            .body(Body::from(payload_str))
            .unwrap();
        let response_replay = app.oneshot(req_replay).await.unwrap();
        assert_eq!(response_replay.status(), StatusCode::FORBIDDEN);
    }
}
