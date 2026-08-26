//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / test_oversight
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::runner::AgentRunner;
use crate::agent::types::ToolCallAudit;
use crate::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_e2e_oversight_approval_loop() {
    // 1. Setup AppState and Database
    let state = Arc::new(
        AppState::new()
            .await
            .expect("Failed to initialize state for oversight tests"),
    );
    let runner = AgentRunner::new(state.clone());

    let test_id = uuid::Uuid::new_v4().to_string();
    let agent_id = format!("test-agent-{}", test_id);
    let mission_id = format!("test-mission-{}", test_id);

    // Seed test data
    sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Oversight Test', 'security', 'Compliance', 'desc', 'idle', '{}')")
        .bind(&agent_id).execute(&state.resources.pool).await.unwrap();
    sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Oversight Verification', 'active')")
        .bind(&mission_id).bind(&agent_id).execute(&state.resources.pool).await.unwrap();

    // 2. Simulate a tool call requiring oversight (like delete_file)
    let tool_call = ToolCallAudit {
        id: "call-123".to_string(),
        agent_id: agent_id.clone(),
        mission_id: Some(mission_id.clone()),
        skill: "delete_file".to_string(),
        params: serde_json::json!({"filename": "critical_file.txt"}),
        department: "Compliance".to_string(),
        description: "Test deletion oversight".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // 3. Start the oversight submission in a task (it blocks until rx)
    let runner_clone = runner.clone();
    let mid_clone = mission_id.clone();
    let handle = tokio::spawn(async move {
        runner_clone
            .submit_oversight(tool_call, Some(mid_clone))
            .await
    });

    // 4. Wait for the resolver to be registered in the queue
    let mut entry_id = String::new();
    let mut resolved = false;
    for _ in 0..10 {
        if let Some(kv) = state.comms.oversight_resolvers.iter().next() {
            entry_id = kv.key().clone();
            resolved = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(resolved, "Oversight resolver should have been registered");

    // 5. Simulate the UI approval event
    if let Some((_, tx)) = state.comms.oversight_resolvers.remove(&entry_id) {
        let _ = tx.send(crate::agent::types::OversightResolution {
            approved: true,
            override_slot: None,
        }); // Approve
    }

    // 6. Verify the result
    let approved = handle.await.unwrap().expect("submit_oversight failed");
    assert!(approved, "Oversight should have been approved");

    // 7. Test Rejection
    let tool_call_rej = ToolCallAudit {
        id: "call-456".to_string(),
        agent_id: agent_id.clone(),
        mission_id: Some(mission_id.clone()),
        skill: "delete_file".to_string(),
        params: serde_json::json!({"filename": "important.txt"}),
        department: "Compliance".to_string(),
        description: "Test rejection oversight".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let runner_clone2 = runner.clone();
    let handle_rej = tokio::spawn(async move {
        runner_clone2
            .submit_oversight(tool_call_rej, Some(mission_id))
            .await
    });

    let mut entry_id_rej = String::new();
    for _ in 0..10 {
        if let Some(kv) = state.comms.oversight_resolvers.iter().next() {
            entry_id_rej = kv.key().clone();
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    if let Some((_, tx)) = state.comms.oversight_resolvers.remove(&entry_id_rej) {
        let _ = tx.send(crate::agent::types::OversightResolution {
            approved: false,
            override_slot: None,
        }); // Reject
    }

    let approved_rej = handle_rej.await.unwrap().expect("submit_oversight failed");
    assert!(!approved_rej, "Oversight should have been rejected");
}

#[tokio::test]
async fn test_model_failover_oversight() {
    // 1. Setup AppState and Database
    let state = Arc::new(
        AppState::new()
            .await
            .expect("Failed to initialize state for oversight tests"),
    );
    let runner = AgentRunner::new(state.clone());

    let test_id = uuid::Uuid::new_v4().to_string();
    let agent_id = format!("test-agent-{}", test_id);
    let mission_id = format!("test-mission-{}", test_id);

    // Seed agent with multiple model slots
    let mut agent = crate::agent::types::EngineAgent::default();
    agent.identity.id = agent_id.clone();
    agent.models.model = crate::agent::types::ModelConfig {
        provider: crate::agent::types::ModelProvider::Ollama,
        model_id: "default-model".to_string(),
        ..Default::default()
    };
    agent.models.planning_slot = Some(crate::agent::types::ModelConfig {
        provider: crate::agent::types::ModelProvider::Gemini,
        model_id: "planning-model".to_string(),
        ..Default::default()
    });
    agent.models.execution_slot = Some(crate::agent::types::ModelConfig {
        provider: crate::agent::types::ModelProvider::Groq,
        model_id: "execution-model".to_string(),
        ..Default::default()
    });
    agent.models.active_model_slot = Some("planning".to_string());
    state.registry.agents.insert(agent_id.clone(), agent);

    // Seed test data in SQLite
    sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Oversight Test', 'security', 'Compliance', 'desc', 'idle', '{}')")
        .bind(&agent_id).execute(&state.resources.pool).await.unwrap();
    sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Oversight Verification', 'active')")
        .bind(&mission_id).bind(&agent_id).execute(&state.resources.pool).await.unwrap();

    // 2. Simulate a failover requiring oversight
    let tool_call = ToolCallAudit {
        id: "failover-123".to_string(),
        agent_id: agent_id.clone(),
        mission_id: Some(mission_id.clone()),
        skill: "model_failover".to_string(),
        params: serde_json::json!({
            "failed_provider": "gemini",
            "failed_model": "planning-model",
            "proposed_fallback_provider": "groq",
            "proposed_fallback_model": "execution-model"
        }),
        department: "Compliance".to_string(),
        description: "Test failover oversight".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let runner_clone = runner.clone();
    let mid_clone = mission_id.clone();
    let handle = tokio::spawn(async move {
        runner_clone
            .submit_oversight_resolution(tool_call, Some(mid_clone))
            .await
    });

    // 3. Wait for the resolver to be registered
    let mut entry_id = String::new();
    let mut resolved = false;
    for _ in 0..10 {
        if let Some(kv) = state.comms.oversight_resolvers.iter().next() {
            entry_id = kv.key().clone();
            resolved = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(resolved, "Oversight resolver should have been registered");

    // 4. Simulate the UI response with slot override to "execution"
    if let Some((_, tx)) = state.comms.oversight_resolvers.remove(&entry_id) {
        let _ = tx.send(crate::agent::types::OversightResolution {
            approved: true,
            override_slot: Some("execution".to_string()),
        });
    }

    // 5. Verify the resolution results
    let resolution = handle
        .await
        .unwrap()
        .expect("submit_oversight_resolution failed");
    assert!(resolution.approved);
    assert_eq!(resolution.override_slot, Some("execution".to_string()));
}

#[tokio::test]
async fn test_e2e_oversight_rest_decision_database_update() {
    use crate::agent::types::OversightDecision;
    use crate::routes::oversight::decide_oversight;
    use axum::extract::{Json, Path, State};
    use ed25519_dalek::{Signer, SigningKey};
    use rand::RngExt;

    // 1. Setup AppState and Database
    let state = Arc::new(
        AppState::new()
            .await
            .expect("Failed to initialize state for oversight tests"),
    );
    let runner = AgentRunner::new(state.clone());

    let test_id = uuid::Uuid::new_v4().to_string();
    let agent_id = format!("test-agent-{}", test_id);
    let mission_id = format!("test-mission-{}", test_id);

    // Seed test data
    sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Oversight Test', 'security', 'Compliance', 'desc', 'idle', '{}')")
        .bind(&agent_id).execute(&state.resources.pool).await.unwrap();
    sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Oversight Verification', 'active')")
        .bind(&mission_id).bind(&agent_id).execute(&state.resources.pool).await.unwrap();

    // 2. Simulate a tool call requiring oversight
    let tool_call = ToolCallAudit {
        id: "call-xyz".to_string(),
        agent_id: agent_id.clone(),
        mission_id: Some(mission_id.clone()),
        skill: "delete_file".to_string(),
        params: serde_json::json!({"filename": "critical_file.txt"}),
        department: "Compliance".to_string(),
        description: "Test deletion oversight".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // 3. Start the oversight submission in a task (it blocks until rx)
    let runner_clone = runner.clone();
    let mid_clone = mission_id.clone();
    let handle = tokio::spawn(async move {
        runner_clone
            .submit_oversight(tool_call, Some(mid_clone))
            .await
    });

    // 4. Wait for the resolver to be registered in the queue
    let mut entry_id = String::new();
    let mut resolved = false;
    for _ in 0..10 {
        if let Some(kv) = state.comms.oversight_resolvers.iter().next() {
            entry_id = kv.key().clone();
            resolved = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(resolved, "Oversight resolver should have been registered");

    // Verify database record has "pending" status initially
    let row_pending: (String,) = sqlx::query_as("SELECT status FROM oversight_log WHERE id = ?")
        .bind(&entry_id)
        .fetch_one(&state.resources.pool)
        .await
        .unwrap();
    assert_eq!(row_pending.0, "pending");

    // 5. Generate ed25519 keys and sign the decision payload
    let mut rng = rand::rng();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes);
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();

    let decision_str = "approved";
    let timestamp = chrono::Utc::now().timestamp_millis();
    let nonce = "unique-test-nonce-12345";
    let payload_str = format!(
        "ovs:v1|{}|{}|{}|{}",
        entry_id, decision_str, timestamp, nonce
    );
    let signature = signing_key.sign(payload_str.as_bytes());

    let signature_hex = hex::encode(signature.to_bytes());
    let pubkey_hex = hex::encode(verifying_key.to_bytes());
    std::env::set_var("OVERSIGHT_PUBLIC_KEY", &pubkey_hex);

    let decision_payload = OversightDecision {
        decision: decision_str.to_string(),
        signature: Some(signature_hex),
        verifying_key: Some(pubkey_hex),
        override_slot: None,
        timestamp: Some(timestamp),
        nonce: Some(nonce.to_string()),
    };

    // 6. Invoke the decide REST endpoint handler directly
    let res = decide_oversight(
        Path(entry_id.clone()),
        State(state.clone()),
        Json(decision_payload),
    )
    .await;
    std::env::remove_var("OVERSIGHT_PUBLIC_KEY");
    res.expect("decide_oversight handler should succeed");

    // 7. Verify the channel was resolved and unblocked the submission task
    let approved = handle.await.unwrap().expect("submit_oversight failed");
    assert!(approved, "Oversight should have been approved");

    // 8. Verify the database now has the updated status and decision
    let row_updated: (String, String, String) =
        sqlx::query_as("SELECT status, decision, decided_by FROM oversight_log WHERE id = ?")
            .bind(&entry_id)
            .fetch_one(&state.resources.pool)
            .await
            .unwrap();
    assert_eq!(row_updated.0, "approved");
    assert_eq!(row_updated.1, "approved");
    assert_eq!(row_updated.2, "user");
}

#[tokio::test]
async fn test_hardened_signature_verification() {
    use crate::agent::types::OversightDecision;
    use crate::routes::oversight::{resolve_oversight_decision, verify_oversight_signature};
    use ed25519_dalek::{Signer, SigningKey};
    use rand::RngExt;

    let state = Arc::new(AppState::new().await.expect("Failed to initialize state"));

    let mut rng = rand::rng();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes);
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();
    let pubkey_hex = hex::encode(verifying_key.to_bytes());

    let entry_id = "test-entry-123";
    let decision = "approved";
    let nonce = "unique-nonce-abc-123";

    // 1. Valid Signature (within ±5 min window)
    let timestamp = chrono::Utc::now().timestamp_millis();
    let payload = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, timestamp, nonce);
    let signature = signing_key.sign(payload.as_bytes());
    let signature_hex = hex::encode(signature.to_bytes());

    let res = verify_oversight_signature(
        entry_id,
        decision,
        &signature_hex,
        &pubkey_hex,
        Some(timestamp),
        Some(nonce),
        None, // C-03: No pinned key in test — falls back to env var
        false,
    );
    assert!(
        res.is_ok(),
        "Valid signature verification failed: {:?}",
        res
    );

    // 2. Expired Timestamp (e.g., -6 minutes / -360_000 ms)
    let expired_timestamp = timestamp - 360_000;
    let expired_payload = format!(
        "ovs:v1|{}|{}|{}|{}",
        entry_id, decision, expired_timestamp, nonce
    );
    let expired_signature = signing_key.sign(expired_payload.as_bytes());
    let expired_signature_hex = hex::encode(expired_signature.to_bytes());

    let res_expired = verify_oversight_signature(
        entry_id,
        decision,
        &expired_signature_hex,
        &pubkey_hex,
        Some(expired_timestamp),
        Some(nonce),
        None,
        false,
    );
    assert!(res_expired.is_err(), "Expired signature should fail");
    assert!(res_expired.unwrap_err().contains("timestamp is expired"));

    // 2b. Extreme/Overflowing Timestamp (i64::MIN)
    let overflow_timestamp = i64::MIN;
    let overflow_payload = format!(
        "ovs:v1|{}|{}|{}|{}",
        entry_id, decision, overflow_timestamp, nonce
    );
    let overflow_signature = signing_key.sign(overflow_payload.as_bytes());
    let overflow_signature_hex = hex::encode(overflow_signature.to_bytes());

    let res_overflow = verify_oversight_signature(
        entry_id,
        decision,
        &overflow_signature_hex,
        &pubkey_hex,
        Some(overflow_timestamp),
        Some(nonce),
        None,
        false,
    );
    assert!(
        res_overflow.is_err(),
        "Overflowing signature should fail safely"
    );
    assert!(res_overflow.unwrap_err().contains("timestamp is expired"));

    // 3. Replay protection (Duplicate Nonces)
    let setup_query = "INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES ('test-agent', 'Test', 'security', 'Compliance', 'desc', 'idle', '{}')";
    sqlx::query(setup_query)
        .execute(&state.resources.pool)
        .await
        .unwrap();

    let entry_insert = "INSERT INTO oversight_log (id, agent_id, entry_type, skill, params, status, created_at, payload) VALUES (?, 'test-agent', 'tool', 'delete_file', '{}', 'pending', datetime('now'), 'desc')";
    sqlx::query(entry_insert)
        .bind(entry_id)
        .execute(&state.resources.pool)
        .await
        .unwrap();

    let oversight_entry = crate::agent::types::OversightEntry {
        id: entry_id.to_string(),
        mission_id: None,
        tool_call: Some(crate::agent::types::ToolCallAudit {
            id: "call-xyz".to_string(),
            agent_id: "test-agent".to_string(),
            mission_id: None,
            skill: "delete_file".to_string(),
            params: serde_json::json!({}),
            department: "Compliance".to_string(),
            description: "Test deletion oversight".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        skill_proposal: None,
        status: "pending".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .comms
        .oversight_queue
        .insert(entry_id.to_string(), oversight_entry);

    let decision_payload = OversightDecision {
        decision: decision.to_string(),
        signature: Some(signature_hex.clone()),
        verifying_key: Some(pubkey_hex.clone()),
        override_slot: None,
        timestamp: Some(timestamp),
        nonce: Some(nonce.to_string()),
    };

    let dec_res = resolve_oversight_decision(&state, entry_id, &decision_payload).await;
    assert!(dec_res.is_ok(), "First decision failed: {:?}", dec_res);

    // Second decision with the same nonce & timestamp: should be rejected as a replay attack
    let entry_id_replay = "test-entry-replay";
    sqlx::query(entry_insert)
        .bind(entry_id_replay)
        .execute(&state.resources.pool)
        .await
        .unwrap();

    let oversight_entry_replay = crate::agent::types::OversightEntry {
        id: entry_id_replay.to_string(),
        mission_id: None,
        tool_call: Some(crate::agent::types::ToolCallAudit {
            id: "call-replay".to_string(),
            agent_id: "test-agent".to_string(),
            mission_id: None,
            skill: "delete_file".to_string(),
            params: serde_json::json!({}),
            department: "Compliance".to_string(),
            description: "Test deletion oversight".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        skill_proposal: None,
        status: "pending".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .comms
        .oversight_queue
        .insert(entry_id_replay.to_string(), oversight_entry_replay);

    let replay_payload_str = format!(
        "ovs:v1|{}|{}|{}|{}",
        entry_id_replay, decision, timestamp, nonce
    );
    let replay_sig = signing_key.sign(replay_payload_str.as_bytes());
    let replay_sig_hex = hex::encode(replay_sig.to_bytes());

    let replay_decision_payload = OversightDecision {
        decision: decision.to_string(),
        signature: Some(replay_sig_hex),
        verifying_key: Some(pubkey_hex),
        override_slot: None,
        timestamp: Some(timestamp),
        nonce: Some(nonce.to_string()),
    };

    let replay_res =
        resolve_oversight_decision(&state, entry_id_replay, &replay_decision_payload).await;
    assert!(
        replay_res.is_err(),
        "Replay attempt with duplicate nonce should fail"
    );
    assert!(replay_res
        .unwrap_err()
        .to_string()
        .contains("nonce already used"));
}

#[tokio::test]
async fn test_admin_role_authorization() {
    use crate::middleware::auth::RequireAdmin;

    let state = Arc::new(AppState::new().await.expect("Failed to initialize state"));

    use axum::extract::FromRequestParts;
    use axum::http::Request;

    // Test with no authorization header -> should fail
    let req = Request::builder().body(()).unwrap();
    let (mut parts, _) = req.into_parts();
    let res = RequireAdmin::from_request_parts(&mut parts, &state).await;
    assert!(res.is_err(), "Should fail with no auth header");

    // Test with incorrect token -> should fail
    let req = Request::builder()
        .header("Authorization", "Bearer invalid-token")
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let res = RequireAdmin::from_request_parts(&mut parts, &state).await;
    assert!(res.is_err(), "Should fail with invalid token");

    // Test with correct token (admin_token) -> should succeed
    let req = Request::builder()
        .header(
            "Authorization",
            format!("Bearer {}", state.security.admin_token),
        )
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let res = RequireAdmin::from_request_parts(&mut parts, &state).await;
    assert!(
        res.is_ok(),
        "Should succeed with valid admin token: {:?}",
        res.err()
    );
}

#[tokio::test]
async fn test_pinned_public_key_verification() {
    use crate::routes::oversight::verify_oversight_signature;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::RngExt;

    let mut rng = rand::rng();
    let mut key_bytes1 = [0u8; 32];
    let mut key_bytes2 = [0u8; 32];
    rng.fill(&mut key_bytes1);
    rng.fill(&mut key_bytes2);

    let signing_key1 = SigningKey::from_bytes(&key_bytes1);
    let pubkey_hex1 = hex::encode(signing_key1.verifying_key().to_bytes());

    let signing_key2 = SigningKey::from_bytes(&key_bytes2);
    let pubkey_hex2 = hex::encode(signing_key2.verifying_key().to_bytes());

    let entry_id = "pinned-test-entry";
    let decision = "approved";
    let nonce = "nonce-xyz-789";
    let timestamp = chrono::Utc::now().timestamp_millis();
    let payload = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, timestamp, nonce);

    let signature1 = signing_key1.sign(payload.as_bytes());
    let signature_hex1 = hex::encode(signature1.to_bytes());

    // 1. Set the env var to pubkey_hex1
    std::env::set_var("OVERSIGHT_PUBLIC_KEY", &pubkey_hex1);

    // 2. Verification using key1 should succeed
    let res1 = verify_oversight_signature(
        entry_id,
        decision,
        &signature_hex1,
        &pubkey_hex1,
        Some(timestamp),
        Some(nonce),
        None, // Env var is set above — fallback will pick it up
        false,
    );
    assert!(
        res1.is_ok(),
        "Verification with pinned key should succeed: {:?}",
        res1
    );

    // 3. Verification using key2 should fail because it doesn't match the pinned key
    let signature2 = signing_key2.sign(payload.as_bytes());
    let signature_hex2 = hex::encode(signature2.to_bytes());
    let res2 = verify_oversight_signature(
        entry_id,
        decision,
        &signature_hex2,
        &pubkey_hex2,
        Some(timestamp),
        Some(nonce),
        None, // Env var is set to key1 — key2 should fail
        false,
    );
    assert!(res2.is_err(), "Verification with unpinned key should fail");
    assert!(res2
        .unwrap_err()
        .contains("does not match the pinned OVERSIGHT_PUBLIC_KEY"));

    // 4. Clean up the environment variable
    std::env::remove_var("OVERSIGHT_PUBLIC_KEY");
}

#[tokio::test]
async fn test_production_requires_pinned_public_key() {
    use crate::routes::oversight::verify_oversight_signature;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::RngExt;

    let mut rng = rand::rng();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes);

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());

    let entry_id = "prod-pin-required-entry";
    let decision = "approved";
    let nonce = "prod-pin-required-nonce";
    let timestamp = chrono::Utc::now().timestamp_millis();
    let payload = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, timestamp, nonce);
    let signature = signing_key.sign(payload.as_bytes());
    let signature_hex = hex::encode(signature.to_bytes());

    let res = verify_oversight_signature(
        entry_id,
        decision,
        &signature_hex,
        &pubkey_hex,
        Some(timestamp),
        Some(nonce),
        None,
        true,
    );

    assert!(
        res.is_err(),
        "production verification must require pinned key"
    );
    assert!(
        res.unwrap_err()
            .contains("OVERSIGHT_PUBLIC_KEY is required in production"),
        "unexpected missing-pin error"
    );
}
