//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / ledger
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`, `AppError::Sqlx`
//! - **Telemetry Targets**: none declared
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

use crate::agent::types::{OversightDecision, OversightEntry};
use crate::error::AppError;
use crate::routes::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use super::{
    is_production_env, DB_QUERY_TIMEOUT, MAX_AGENTS_CEILING, MAX_BUDGET_USD, MAX_CLUSTERS_CEILING,
    MAX_SWARM_DEPTH_CEILING, MAX_TASK_LENGTH_CEILING, MIN_AGENTS_FLOOR, MIN_BUDGET_USD,
    MIN_CLUSTERS_FLOOR, MIN_SWARM_DEPTH_FLOOR, MIN_TASK_LENGTH_FLOOR, OVS_SIG_WINDOW_MS,
};

mod decisions;
mod listing;
mod settings;
mod signatures;

pub use decisions::decide_oversight;
pub(crate) use decisions::{
    resolve_oversight_decision, resolve_verified_remote_oversight_decision,
};
pub use listing::{get_ledger, get_pending};
pub use settings::{get_settings, update_settings, OversightSettingsPayload};
pub(crate) use signatures::{verify_oversight_signature, verify_oversight_signature_canonical};

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use ed25519_dalek::{Signer, SigningKey};

    async fn fetch_settings_payload(state: &Arc<AppState>) -> OversightSettingsPayload {
        let response = get_settings(State(state.clone()))
            .await
            .expect("get_settings failed");
        let (parts, body) = response.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::OK);

        let bytes = axum::body::to_bytes(body, 64 * 1024)
            .await
            .expect("failed to read body");
        serde_json::from_slice(&bytes).expect("failed to deserialize")
    }

    #[tokio::test]
    async fn test_get_settings_reflects_state() {
        let state = Arc::new(AppState::new_minimal_mock().await);

        // Pre-condition assertion: initial cluster policies should be empty
        let initial = fetch_settings_payload(&state).await;
        let initial_policies = initial.cluster_privacy_policies.unwrap_or_default();
        assert!(
            initial_policies.is_empty(),
            "Expected empty initial policies"
        );

        // Mutate state
        state
            .governance
            .cluster_privacy_policies
            .insert("cl-command".to_string(), false);
        state
            .governance
            .cluster_privacy_policies
            .insert("cl-secure".to_string(), true);

        // Post-mutation assertion
        let updated = fetch_settings_payload(&state).await;
        let policies = updated
            .cluster_privacy_policies
            .expect("missing cluster policies");
        assert_eq!(policies.get("cl-command"), Some(&false));
        assert_eq!(policies.get("cl-secure"), Some(&true));
    }

    #[test]
    fn test_verify_oversight_signature_valid() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let entry_id = "act-123";
        let decision = "approved";
        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "nonce-abc-789";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, now, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(now),
            Some(nonce),
            Some(&pubkey_hex),
            true,
        );

        assert!(result.is_ok(), "Valid signature should pass: {:?}", result);
    }

    #[test]
    fn test_verify_oversight_signature_expired_timestamp() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let entry_id = "act-123";
        let decision = "approved";
        // 10 minutes in the past (> 5 min window)
        let expired_ts = chrono::Utc::now().timestamp_millis() - 600_000;
        let nonce = "nonce-abc-789";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, expired_ts, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(expired_ts),
            Some(nonce),
            Some(&pubkey_hex),
            true,
        );

        assert!(result.is_err(), "Expired timestamp should fail");
        assert!(result.unwrap_err().contains("expired"));
    }

    #[test]
    fn test_verify_oversight_signature_pinned_key_case_insensitivity() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());
        let pinned_hex_uppercase = pubkey_hex.to_uppercase();

        let entry_id = "act-123";
        let decision = "approved";
        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "nonce-case-test";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, now, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(now),
            Some(nonce),
            Some(&pinned_hex_uppercase),
            true,
        );

        assert!(
            result.is_ok(),
            "Decoded byte comparison should handle uppercase hex: {:?}",
            result
        );
    }

    #[test]
    fn test_verify_oversight_signature_pinned_mismatch() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let other_key = SigningKey::generate(&mut rand::rng());
        let other_pubkey_hex = hex::encode(other_key.verifying_key().as_bytes());

        let entry_id = "act-123";
        let decision = "approved";
        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "nonce-mismatch-test";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, now, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(now),
            Some(nonce),
            Some(&other_pubkey_hex),
            true,
        );

        assert!(result.is_err(), "Key mismatch against pinned key must fail");
        assert!(result.unwrap_err().contains("does not match"));
    }

    #[test]
    fn test_verify_oversight_signature_require_pinned_key_fail_closed() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let entry_id = "act-123";
        let decision = "approved";
        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "nonce-failclosed-test";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, now, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(now),
            Some(nonce),
            None,
            true, // require_pinned_key
        );

        assert!(
            result.is_err(),
            "Missing pinned key when required must fail closed"
        );
    }

    #[tokio::test]
    async fn test_resolve_oversight_decision_invalid_decision_string() {
        let state = AppState::new_minimal_mock().await;
        let payload = OversightDecision {
            decision: "Approve".to_string(), // Invalid: capitalized
            signature: None,
            verifying_key: None,
            timestamp: None,
            nonce: None,
            override_slot: None,
            user_answer: None,
        };

        let result = resolve_oversight_decision(&state, "act-test-id", &payload).await;
        match result {
            Err(AppError::BadRequest(msg)) => {
                assert!(msg.contains("Invalid oversight decision"));
            }
            other => panic!(
                "Expected BadRequest for invalid decision string, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn test_update_settings_bounds_validation() {
        let state = Arc::new(AppState::new_minimal_mock().await);

        // Test max_agents bounds
        let payload = OversightSettingsPayload {
            dispatch_paused: None,
            max_agents: Some(0),
            auto_approve_safe_skills: None,
            privacy_mode: None,
            cluster_privacy_policies: None,
            max_clusters: None,
            max_swarm_depth: None,
            max_task_length: None,
            default_budget_usd: None,
            default_model: None,
            default_provider: None,
        };
        assert!(update_settings(
            State(state.clone()),
            crate::middleware::auth::RequireAdmin,
            Json(payload)
        )
        .await
        .is_err());

        // Test max_clusters bounds
        let payload = OversightSettingsPayload {
            dispatch_paused: None,
            max_agents: None,
            auto_approve_safe_skills: None,
            privacy_mode: None,
            cluster_privacy_policies: None,
            max_clusters: Some(51),
            max_swarm_depth: None,
            max_task_length: None,
            default_budget_usd: None,
            default_model: None,
            default_provider: None,
        };
        assert!(update_settings(
            State(state.clone()),
            crate::middleware::auth::RequireAdmin,
            Json(payload)
        )
        .await
        .is_err());

        // Test default_budget_usd bounds
        let payload = OversightSettingsPayload {
            dispatch_paused: None,
            max_agents: None,
            auto_approve_safe_skills: None,
            privacy_mode: None,
            cluster_privacy_policies: None,
            max_clusters: None,
            max_swarm_depth: None,
            max_task_length: None,
            default_budget_usd: Some(-1.0),
            default_model: None,
            default_provider: None,
        };
        assert!(update_settings(
            State(state.clone()),
            crate::middleware::auth::RequireAdmin,
            Json(payload)
        )
        .await
        .is_err());

        // Test invalid model string
        let payload = OversightSettingsPayload {
            dispatch_paused: None,
            max_agents: None,
            auto_approve_safe_skills: None,
            privacy_mode: None,
            cluster_privacy_policies: None,
            max_clusters: None,
            max_swarm_depth: None,
            max_task_length: None,
            default_budget_usd: None,
            default_model: Some("model; rm -rf /".to_string()),
            default_provider: None,
        };
        assert!(update_settings(
            State(state.clone()),
            crate::middleware::auth::RequireAdmin,
            Json(payload)
        )
        .await
        .is_err());
    }
}
