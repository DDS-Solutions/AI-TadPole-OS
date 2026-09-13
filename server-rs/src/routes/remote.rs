//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / remote
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::Engine;
use dashmap::{mapref::entry::Entry, DashMap};
use ed25519_dalek::{Verifier, VerifyingKey};
use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct PairingChallengeToken {
    pub _created_at: u64,
    pub expires_at: u64,
}

/// Ephemeral pairing challenge tokens generated for Desktop QR code scanning (3-minute TTL).
static PAIRING_TOKENS: Lazy<DashMap<String, PairingChallengeToken>> = Lazy::new(DashMap::new);

/// Registered paired companion devices (starts clean with no hardcoded fallback).
static PAIRED_DEVICES: Lazy<DashMap<String, PairedDevice>> = Lazy::new(DashMap::new);

/// Recently accepted per-device nonces. Entries are retained for the same window
/// as request timestamps so a captured signed request cannot be replayed.
static USED_REQUEST_NONCES: Lazy<DashMap<String, u64>> = Lazy::new(DashMap::new);

const REMOTE_REQUEST_MAX_SKEW_SECS: u64 = 300;

/// Global active pending oversight queue for remote HITL approvals
static PENDING_OVERSIGHT_QUEUE: Lazy<DashMap<String, PendingOversightItem>> =
    Lazy::new(DashMap::new);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub user_name: String,
    pub public_key: String,
    pub paired_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingOversightItem {
    pub id: String,
    pub agent_name: String,
    pub tool_name: String,
    pub target_resource: String,
    pub rationale: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingTokenResponse {
    pub token: String,
    pub expires_in_seconds: u64,
    pub node_ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRequestPayload {
    pub token: String,
    pub device_id: String,
    pub device_name: String,
    #[serde(default)]
    pub user_name: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceUpdatePayload {
    pub device_name: String,
    #[serde(default)]
    pub user_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDecisionPayload {
    pub approval_id: String,
    pub decision: String, // "approved" | "rejected"
    pub decided_by: String,
    pub signature: Option<String>,
    pub timestamp: u64,
    pub nonce: String,
}

fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn decode_signature(signature: &str) -> Result<ed25519_dalek::Signature, AppError> {
    let sig_bytes = hex::decode(signature)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(signature))
        .map_err(|_| AppError::BadRequest("Signature must be hex or base64 encoded".to_string()))?;

    if sig_bytes.len() != 64 {
        return Err(AppError::BadRequest(format!(
            "Ed25519 signature must be 64 bytes (got {})",
            sig_bytes.len()
        )));
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);
    Ok(ed25519_dalek::Signature::from_bytes(&sig_array))
}

fn validate_request_freshness(timestamp: u64, nonce: &str) -> Result<(), AppError> {
    if nonce.trim().len() < 16 || nonce.len() > 128 {
        return Err(AppError::BadRequest(
            "Remote request nonce must contain 16-128 characters".to_string(),
        ));
    }

    let now = unix_timestamp_secs();
    if now.abs_diff(timestamp) > REMOTE_REQUEST_MAX_SKEW_SECS {
        return Err(AppError::Unauthorized(
            "Remote request timestamp is outside the accepted five-minute window".to_string(),
        ));
    }
    Ok(())
}

fn consume_request_nonce(device_id: &str, nonce: &str, timestamp: u64) -> Result<(), AppError> {
    let cutoff = unix_timestamp_secs().saturating_sub(REMOTE_REQUEST_MAX_SKEW_SECS);
    USED_REQUEST_NONCES.retain(|_, accepted_at| *accepted_at >= cutoff);

    let nonce_key = format!("{}:{}", device_id, nonce);
    match USED_REQUEST_NONCES.entry(nonce_key) {
        Entry::Vacant(entry) => {
            entry.insert(timestamp);
            Ok(())
        }
        Entry::Occupied(_) => Err(AppError::Unauthorized(
            "Remote request nonce has already been used".to_string(),
        )),
    }
}

/// Verifies proof-of-possession headers for remote endpoints without request bodies.
/// Canonical format: `METHOD:/v1/path:timestamp:nonce`.
pub fn verify_paired_request(
    device_id: &str,
    method: &str,
    path: &str,
    timestamp: u64,
    nonce: &str,
    signature: &str,
) -> Result<(), AppError> {
    validate_request_freshness(timestamp, nonce)?;
    let paired_device = PAIRED_DEVICES.get(device_id).ok_or_else(|| {
        AppError::Forbidden("Device is not paired or has been revoked".to_string())
    })?;
    let verifying_key = validate_public_key_format(&paired_device.public_key)?;
    let canonical = format!("{}:{}:{}:{}", method.to_uppercase(), path, timestamp, nonce);
    verifying_key
        .verify(canonical.as_bytes(), &decode_signature(signature)?)
        .map_err(|_| AppError::Unauthorized("Remote request signature is invalid".to_string()))?;
    drop(paired_device);
    consume_request_nonce(device_id, nonce, timestamp)
}

fn verify_remote_decision_signature(
    device_id: &str,
    payload: &RemoteDecisionPayload,
) -> Result<Option<String>, AppError> {
    if device_id != payload.decided_by {
        return Err(AppError::Unauthorized(
            "Decision signer does not match X-Device-Id".to_string(),
        ));
    }
    validate_request_freshness(payload.timestamp, &payload.nonce)?;

    let paired_device = PAIRED_DEVICES.get(device_id).ok_or_else(|| {
        AppError::Unauthorized(
            "Decision rejected: device is not paired or has been revoked".to_string(),
        )
    })?;
    let verifying_key = validate_public_key_format(&paired_device.public_key)?;
    let signature = payload.signature.as_deref().ok_or_else(|| {
        AppError::BadRequest("Signature is required for paired devices".to_string())
    })?;
    let canonical = format!(
        "{}:{}:{}:{}",
        payload.approval_id, payload.decision, payload.timestamp, payload.nonce
    );
    verifying_key
        .verify(canonical.as_bytes(), &decode_signature(signature)?)
        .map_err(|_| {
            AppError::Unauthorized("Ed25519 decision signature verification failed".to_string())
        })?;
    let verifying_key_hex = hex::encode(verifying_key.to_bytes());
    drop(paired_device);
    consume_request_nonce(device_id, &payload.nonce, payload.timestamp)?;
    Ok(Some(verifying_key_hex))
}

/// Sweeps expired pairing tokens from the global DashMap store.
pub fn cleanup_expired_tokens() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    PAIRING_TOKENS.retain(|_, token| token.expires_at > now);
}

/// Validates and parses a companion device public key into an Ed25519 VerifyingKey.
pub fn validate_public_key_format(key: &str) -> Result<VerifyingKey, AppError> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(
            "Public key cannot be empty".to_string(),
        ));
    }
    let raw_key = trimmed.strip_prefix("ed25519:").unwrap_or(trimmed);
    let key_bytes = hex::decode(raw_key)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(raw_key))
        .map_err(|_| {
            AppError::BadRequest("Public key must be valid hex or base64 encoded bytes".to_string())
        })?;

    if key_bytes.len() != 32 {
        return Err(AppError::BadRequest(format!(
            "Ed25519 public key must be exactly 32 bytes (got {})",
            key_bytes.len()
        )));
    }

    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&key_bytes);
    VerifyingKey::from_bytes(&bytes)
        .map_err(|e| AppError::BadRequest(format!("Invalid Ed25519 public key point: {}", e)))
}

/// GET /v1/remote/pairing-token
/// Generates a single-use dynamic pairing challenge token using CSPRNG entropy for desktop QR display.
#[tracing::instrument(name = "remote::generate_pairing_token")]
pub async fn generate_pairing_token() -> Result<Json<PairingTokenResponse>, AppError> {
    cleanup_expired_tokens();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut random_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut random_bytes);
    let token = format!("TP-PAIR-{}", hex::encode(random_bytes));

    let challenge = PairingChallengeToken {
        _created_at: now,
        expires_at: now + 180,
    };

    PAIRING_TOKENS.insert(token.clone(), challenge);

    let node_ip = std::env::var("SOVEREIGN_NODE_IP")
        .or_else(|_| std::env::var("LAN_IP"))
        .unwrap_or_else(|_| "127.0.0.1:8000".to_string());

    Ok(Json(PairingTokenResponse {
        token,
        expires_in_seconds: 180,
        node_ip,
    }))
}

/// GET /v1/remote/ping
/// Connection verification ping endpoint for companion mobile apps.
#[tracing::instrument(name = "remote::ping")]
pub async fn ping_remote_node() -> Result<impl IntoResponse, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(Json(serde_json::json!({
        "status": "online",
        "node": "Tadpole OS Sovereign Engine",
        "timestamp": now
    })))
}

/// Hydrates paired companion devices from SQLite into memory on engine startup.
pub async fn load_paired_devices(pool: &sqlx::SqlitePool) -> Result<(), AppError> {
    let rows = sqlx::query(
        "SELECT id, name, user_name, public_key, paired_at, status FROM paired_devices WHERE status != 'revoked'"
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        use sqlx::Row;
        let device = PairedDevice {
            id: row.get("id"),
            name: row.get("name"),
            user_name: row.get("user_name"),
            public_key: row.get("public_key"),
            paired_at: row.get("paired_at"),
            status: row.get("status"),
        };
        PAIRED_DEVICES.insert(device.id.clone(), device);
    }
    tracing::info!(
        "📱 [Remote Bridge] Hydrated {} paired companion device(s) from SQLite",
        PAIRED_DEVICES.len()
    );
    Ok(())
}

/// POST /v1/remote/pair
/// Accepts pairing challenge token and registers new mobile device public key.
#[tracing::instrument(skip(state, payload), fields(device_id = %payload.device_id), name = "remote::pair_device")]
pub async fn pair_device(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PairRequestPayload>,
) -> Result<impl IntoResponse, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if !payload.token.starts_with("TP-PAIR-") {
        return Err(AppError::BadRequest(
            "Invalid pairing challenge token format".to_string(),
        ));
    }

    let sanitized_device_id = crate::utils::security::sanitize_id(&payload.device_id);
    if sanitized_device_id.is_empty() {
        return Err(AppError::BadRequest(
            "Device ID cannot be empty or invalid".to_string(),
        ));
    }

    validate_public_key_format(&payload.public_key)?;
    let device_name = payload.device_name.trim();
    if device_name.is_empty() || device_name.len() > 128 {
        return Err(AppError::BadRequest(
            "Device name must contain 1-128 characters".to_string(),
        ));
    }
    if payload.user_name.len() > 128 {
        return Err(AppError::BadRequest(
            "User name cannot exceed 128 characters".to_string(),
        ));
    }

    // Re-pairing Hijack Defense: If device already exists, reject if public key doesn't match
    if let Some(existing) = PAIRED_DEVICES.get(&sanitized_device_id) {
        if existing.public_key.trim() != payload.public_key.trim() {
            return Err(AppError::Forbidden(
                "Security violation: Re-pairing an existing device with a different public key is rejected without prior revocation.".to_string(),
            ));
        }
    }

    // Consume the single-use challenge only after the full device payload has passed validation
    let token_entry = match PAIRING_TOKENS.remove(&payload.token) {
        Some((_, entry)) => entry,
        None => {
            return Err(AppError::Unauthorized(
                "Invalid, expired, or previously consumed pairing token".to_string(),
            ));
        }
    };

    if now > token_entry.expires_at {
        return Err(AppError::Unauthorized(
            "Pairing challenge token has expired".to_string(),
        ));
    }

    let is_update = PAIRED_DEVICES.contains_key(&sanitized_device_id);

    let device = PairedDevice {
        id: sanitized_device_id.clone(),
        name: device_name.to_string(),
        user_name: payload.user_name.trim().to_string(),
        public_key: payload.public_key.trim().to_string(),
        paired_at: format!("{}", now),
        status: "Authorized".to_string(),
    };

    // Persist to SQLite database for restart survivability first (RED-06)
    sqlx::query(
        "INSERT OR REPLACE INTO paired_devices (id, name, user_name, public_key, paired_at, status) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&sanitized_device_id)
    .bind(&device.name)
    .bind(&device.user_name)
    .bind(&device.public_key)
    .bind(&device.paired_at)
    .bind(&device.status)
    .execute(&state.resources.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to persist paired device to database: {}", e);
        AppError::InternalServerError(format!("Database persistence error: {}", e))
    })?;

    PAIRED_DEVICES.insert(sanitized_device_id.clone(), device.clone());

    if is_update {
        tracing::info!(
            "📱 [Remote Bridge] Re-paired/updated companion device: {} ({})",
            device.name,
            device.id
        );
    } else {
        tracing::info!(
            "📱 [Remote Bridge] Paired new companion device: {} ({})",
            device.name,
            device.id
        );
    }

    Ok((StatusCode::CREATED, Json(device)))
}

/// GET /v1/remote/devices
/// Returns list of authorized paired companion devices.
#[tracing::instrument(name = "remote::get_devices")]
pub async fn get_paired_devices() -> Result<impl IntoResponse, AppError> {
    let devices: Vec<PairedDevice> = PAIRED_DEVICES
        .iter()
        .map(|entry| entry.value().clone())
        .collect();
    Ok(Json(serde_json::json!({
        "devices": devices,
        "total": devices.len()
    })))
}

/// PUT /v1/remote/devices/:id
/// Updates administrator-controlled display metadata without altering key material.
#[tracing::instrument(skip(state, payload), fields(device_id = %id), name = "remote::update_device")]
pub async fn update_paired_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<DeviceUpdatePayload>,
) -> Result<impl IntoResponse, AppError> {
    let device_name = payload.device_name.trim();
    if device_name.is_empty() || device_name.len() > 128 {
        return Err(AppError::BadRequest(
            "Device name must contain 1-128 characters".to_string(),
        ));
    }
    if payload.user_name.len() > 128 {
        return Err(AppError::BadRequest(
            "User name cannot exceed 128 characters".to_string(),
        ));
    }

    if !PAIRED_DEVICES.contains_key(&id) {
        return Err(AppError::NotFound("Device ID not found".to_string()));
    }

    sqlx::query("UPDATE paired_devices SET name = ?, user_name = ? WHERE id = ?")
        .bind(device_name)
        .bind(payload.user_name.trim())
        .bind(&id)
        .execute(&state.resources.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update paired device in database: {}", e);
            AppError::InternalServerError(format!("Database update error: {}", e))
        })?;

    let mut device = PAIRED_DEVICES
        .get_mut(&id)
        .ok_or_else(|| AppError::NotFound("Device ID not found".to_string()))?;
    device.name = device_name.to_string();
    device.user_name = payload.user_name.trim().to_string();
    let updated = device.clone();
    drop(device);

    Ok(Json(updated))
}

/// POST /v1/remote/revoke/:id
/// Revokes authorization for a paired mobile device.
#[tracing::instrument(skip(state), fields(device_id = %id), name = "remote::revoke_device")]
pub async fn revoke_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    if !PAIRED_DEVICES.contains_key(&id) {
        return Err(AppError::NotFound("Device ID not found".to_string()));
    }

    sqlx::query("UPDATE paired_devices SET status = 'revoked' WHERE id = ?")
        .bind(&id)
        .execute(&state.resources.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to persist device revocation to database: {}", e);
            AppError::InternalServerError(format!("Database revocation error: {}", e))
        })?;

    PAIRED_DEVICES.remove(&id);

    tracing::info!(
        "🔒 [Remote Bridge] Revoked authorization for device: {}",
        id
    );
    Ok(Json(serde_json::json!({ "status": "revoked", "id": id })))
}

/// GET /v1/remote/agents/health
/// Returns real-time health telemetry status for all active agents in the swarm.
#[tracing::instrument(skip(state), name = "remote::get_agents_health")]
pub async fn get_remote_agents_health(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut list = Vec::new();
    for entry in state.registry.agents.iter() {
        let agent = entry.value();
        let raw_status = agent.health.status.to_uppercase();
        let status = match raw_status.as_str() {
            "RUNNING" | "ACTIVE" | "THINKING" | "CODING" | "SPEAKING" => "RUNNING",
            "HALTED" | "STOPPED" | "FROZEN" => "HALTED",
            "ERROR" | "FAILED" => "ERROR",
            _ => "IDLE",
        };
        let active_task = agent.state.current_task.clone().unwrap_or_else(|| {
            if agent.identity.description.is_empty() {
                format!("Role: {}", agent.identity.role)
            } else {
                agent.identity.description.clone()
            }
        });
        list.push(serde_json::json!({
            "id": agent.identity.id,
            "name": agent.identity.name,
            "status": status,
            "stepCount": agent.state.current_reasoning_turn,
            "activeTask": active_task
        }));
    }
    Ok(Json(serde_json::json!(list)))
}

/// GET /v1/remote/oversight/pending
/// Returns pending HITL oversight items for the mobile approval ledger.
#[tracing::instrument(skip(state), name = "remote::get_pending_oversight")]
pub async fn get_remote_pending_oversight(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut items = Vec::new();

    for entry in state.comms.oversight_queue.iter() {
        let oe = entry.value();
        let (tool_name, target, rationale) = if let Some(tc) = &oe.tool_call {
            (
                tc.skill.clone(),
                format!("{}/{}", tc.department, tc.agent_id),
                tc.description.clone(),
            )
        } else if let Some(sp) = &oe.skill_proposal {
            (
                format!("proposal:{:?}", sp.r#type).to_lowercase(),
                sp.name.clone(),
                sp.description.clone(),
            )
        } else {
            (
                "unknown_tool".to_string(),
                "Engine".to_string(),
                "Awaiting HITL verification".to_string(),
            )
        };

        items.push(PendingOversightItem {
            id: oe.id.clone(),
            agent_name: oe
                .mission_id
                .clone()
                .unwrap_or_else(|| "System Agent".to_string()),
            tool_name,
            target_resource: target,
            rationale,
            timestamp: oe.created_at.clone(),
        });
    }

    for entry in PENDING_OVERSIGHT_QUEUE.iter() {
        if !items.iter().any(|i| &i.id == entry.key()) {
            items.push(entry.value().clone());
        }
    }

    Ok(Json(items))
}

/// POST /v1/remote/oversight/trigger-test-item
/// Injects a new test mission approval item into the oversight queue.
#[tracing::instrument(name = "remote::trigger_test_oversight_item")]
pub async fn trigger_test_oversight_item() -> Result<impl IntoResponse, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let test_id = format!("ovr-{}", 1000 + (now % 8999));

    let new_item = PendingOversightItem {
        id: test_id.clone(),
        agent_name: "Security-Audit-Swarm".to_string(),
        tool_name: "deploy_prod_patch".to_string(),
        target_resource: "server-rs (Port 8000)".to_string(),
        rationale: "Deploying zero-trust mobile bridge security update.".to_string(),
        timestamp: "18:50:12".to_string(),
    };

    PENDING_OVERSIGHT_QUEUE.insert(test_id.clone(), new_item.clone());

    tracing::info!(
        "🧪 [Remote Test] Inserted new pending oversight item: {}",
        test_id
    );

    Ok((StatusCode::CREATED, Json(new_item)))
}

/// POST /v1/remote/oversight/decide
/// Handles remote HITL approval/rejection with mandatory Ed25519 signature verification when paired.
#[tracing::instrument(skip(state, headers, payload), fields(approval_id = %payload.approval_id, decision = %payload.decision), name = "remote::decide_oversight")]
pub async fn remote_decide_oversight(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<RemoteDecisionPayload>,
) -> Result<impl IntoResponse, AppError> {
    let norm_decision = payload.decision.trim().to_lowercase();
    if norm_decision != "approved" && norm_decision != "rejected" {
        return Err(AppError::BadRequest(
            "Decision must be either 'approved' or 'rejected'".to_string(),
        ));
    }

    tracing::info!(
        "⚖️ [Remote Oversight] Received decision for {}: {} by {}",
        payload.approval_id,
        norm_decision,
        payload.decided_by
    );

    let device_id = headers
        .get("x-device-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("X-Device-Id header is required".to_string()))?;

    let verifying_key_hex = verify_remote_decision_signature(device_id, &payload)?;
    tracing::info!(
        "🔒 [Remote Oversight] Verified Ed25519 decision signature from device {}",
        device_id
    );

    let decision = crate::agent::types::OversightDecision {
        decision: norm_decision,
        signature: payload.signature.clone(),
        verifying_key: verifying_key_hex,
        override_slot: None,
        timestamp: Some(payload.timestamp as i64),
        nonce: Some(payload.nonce.clone()),
    };

    let response = crate::routes::oversight::decide_oversight(
        Path(payload.approval_id.clone()),
        State(state),
        Json(decision),
    )
    .await?;

    PENDING_OVERSIGHT_QUEUE.remove(&payload.approval_id);
    Ok(response.into_response())
}

/// Test-only visibility into the paired-device registry.
#[cfg(test)]
pub fn is_device_paired(device_id: &str) -> bool {
    PAIRED_DEVICES.contains_key(device_id)
}

#[cfg(test)]
pub(crate) fn register_paired_device_for_test(device: PairedDevice) {
    PAIRED_DEVICES.insert(device.id.clone(), device);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_pairing_token_entropy() {
        let res1 = generate_pairing_token().await;
        let res2 = generate_pairing_token().await;
        assert!(res1.is_ok());
        assert!(res2.is_ok());

        let t1 = res1.unwrap().0.token;
        let t2 = res2.unwrap().0.token;

        assert_ne!(t1, t2);
        assert!(t1.starts_with("TP-PAIR-"));
        assert_eq!(t1.len(), 8 + 64); // Prefix TP-PAIR- (8) + 32 bytes hex (64)
        assert!(t2.starts_with("TP-PAIR-"));
        assert_eq!(t2.len(), 8 + 64);
    }

    #[tokio::test]
    async fn test_pair_device_rejects_fabricated_token() {
        let state = Arc::new(AppState::new_mock().await);
        let payload = PairRequestPayload {
            token: "TP-PAIR-FABRICATED-TOKEN-123456789".to_string(),
            device_id: "test-device-01".to_string(),
            device_name: "Attacker Phone".to_string(),
            user_name: "Attacker".to_string(),
            public_key: "ed25519:1234567890".to_string(),
        };

        let result = pair_device(State(state), Json(payload)).await;
        assert!(result.is_err());
        assert!(!is_device_paired("test-device-01"));
    }

    #[tokio::test]
    async fn test_pair_device_single_use_and_expiration() {
        let state = Arc::new(AppState::new_mock().await);
        let test_sk = ed25519_dalek::SigningKey::from_bytes(&[0x11; 32]);
        let valid_pubkey = format!(
            "ed25519:{}",
            hex::encode(test_sk.verifying_key().to_bytes())
        );

        // 1. Generate valid challenge token
        let token_str = format!("TP-PAIR-{}", hex::encode([0xab; 32]));
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        PAIRING_TOKENS.insert(
            token_str.clone(),
            PairingChallengeToken {
                _created_at: now,
                expires_at: now + 180,
            },
        );

        let payload = PairRequestPayload {
            token: token_str.clone(),
            device_id: "valid-device-99".to_string(),
            device_name: "Valid Android Phone".to_string(),
            user_name: "Test Operator".to_string(),
            public_key: valid_pubkey,
        };

        // First pairing attempt MUST succeed
        let res1 = pair_device(State(state.clone()), Json(payload.clone())).await;
        assert!(res1.is_ok());
        assert!(is_device_paired("valid-device-99"));

        // Second pairing attempt with same single-use token MUST fail
        let res2 = pair_device(State(state), Json(payload)).await;
        assert!(res2.is_err());
    }

    #[tokio::test]
    async fn test_pair_device_expired_token_rejected() {
        let state = Arc::new(AppState::new_mock().await);
        let test_sk = ed25519_dalek::SigningKey::from_bytes(&[0x22; 32]);
        let valid_pubkey = format!(
            "ed25519:{}",
            hex::encode(test_sk.verifying_key().to_bytes())
        );

        let token_str = format!("TP-PAIR-{}", hex::encode([0xcd; 32]));
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Expired token (10 seconds ago)
        PAIRING_TOKENS.insert(
            token_str.clone(),
            PairingChallengeToken {
                _created_at: now - 190,
                expires_at: now - 10,
            },
        );

        let payload = PairRequestPayload {
            token: token_str,
            device_id: "expired-device-01".to_string(),
            device_name: "Expired Phone".to_string(),
            user_name: "Test Operator".to_string(),
            public_key: valid_pubkey,
        };

        let res = pair_device(State(state), Json(payload)).await;
        assert!(res.is_err());
        assert!(!is_device_paired("expired-device-01"));
    }

    #[tokio::test]
    async fn test_pair_device_invalid_public_key() {
        let state = Arc::new(AppState::new_mock().await);
        let token_str = format!("TP-PAIR-{}", hex::encode([0xef; 32]));
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        PAIRING_TOKENS.insert(
            token_str.clone(),
            PairingChallengeToken {
                _created_at: now,
                expires_at: now + 180,
            },
        );

        let payload = PairRequestPayload {
            token: token_str.clone(),
            device_id: "invalid-key-device".to_string(),
            device_name: "Short Key Phone".to_string(),
            user_name: "Test Operator".to_string(),
            public_key: "short".to_string(), // Invalid key length (< 16 chars)
        };

        let res = pair_device(State(state), Json(payload)).await;
        assert!(res.is_err());
        assert!(PAIRING_TOKENS.contains_key(&token_str));
    }

    #[tokio::test]
    async fn test_cleanup_expired_tokens_sweep() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        PAIRING_TOKENS.insert(
            "TP-PAIR-EXPIRED-SWEEP".to_string(),
            PairingChallengeToken {
                _created_at: now - 200,
                expires_at: now - 20,
            },
        );
        PAIRING_TOKENS.insert(
            "TP-PAIR-ACTIVE-SWEEP".to_string(),
            PairingChallengeToken {
                _created_at: now,
                expires_at: now + 180,
            },
        );

        cleanup_expired_tokens();

        assert!(!PAIRING_TOKENS.contains_key("TP-PAIR-EXPIRED-SWEEP"));
        assert!(PAIRING_TOKENS.contains_key("TP-PAIR-ACTIVE-SWEEP"));
    }

    #[tokio::test]
    async fn test_revoke_device() {
        let state = Arc::new(AppState::new_mock().await);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[0x42; 32]);
        let pubkey_hex = format!(
            "ed25519:{}",
            hex::encode(signing_key.verifying_key().to_bytes())
        );

        let device = PairedDevice {
            id: "revokable-dev-01".to_string(),
            name: "Revokable Device".to_string(),
            user_name: "Test Operator".to_string(),
            public_key: pubkey_hex,
            paired_at: "1000".to_string(),
            status: "Authorized".to_string(),
        };

        PAIRED_DEVICES.insert("revokable-dev-01".to_string(), device);
        assert!(is_device_paired("revokable-dev-01"));

        let res = revoke_device(State(state), Path("revokable-dev-01".to_string())).await;
        assert!(res.is_ok());
        assert!(!is_device_paired("revokable-dev-01"));
    }

    #[tokio::test]
    async fn test_remote_decide_oversight_ed25519_signature_verification() {
        use ed25519_dalek::Signer;

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[0x77; 32]);
        let verifying_key_hex = format!(
            "ed25519:{}",
            hex::encode(signing_key.verifying_key().to_bytes())
        );

        let device_id = "signed-device-123";
        PAIRED_DEVICES.insert(
            device_id.to_string(),
            PairedDevice {
                id: device_id.to_string(),
                name: "Signed Test Companion".to_string(),
                user_name: "Test Operator".to_string(),
                public_key: verifying_key_hex,
                paired_at: "1000".to_string(),
                status: "Authorized".to_string(),
            },
        );

        let approval_id = "ovr-test-sign-999";
        let decision_str = "approved";
        let timestamp = unix_timestamp_secs();
        let nonce = "decision-nonce-valid-0001";

        PENDING_OVERSIGHT_QUEUE.insert(
            approval_id.to_string(),
            PendingOversightItem {
                id: approval_id.to_string(),
                agent_name: "Test-Agent".to_string(),
                tool_name: "test_tool".to_string(),
                target_resource: "test_resource".to_string(),
                rationale: "Testing Ed25519 signature".to_string(),
                timestamp: "12:00:00".to_string(),
            },
        );

        let canonical_msg = format!("{}:{}:{}:{}", approval_id, decision_str, timestamp, nonce);
        let signature = signing_key.sign(canonical_msg.as_bytes());
        let sig_hex = hex::encode(signature.to_bytes());

        let payload_valid = RemoteDecisionPayload {
            approval_id: approval_id.to_string(),
            decision: decision_str.to_string(),
            decided_by: device_id.to_string(),
            signature: Some(sig_hex),
            timestamp,
            nonce: nonce.to_string(),
        };

        // Valid signature payload passes proof-of-possession verification.
        let res_valid = verify_remote_decision_signature(device_id, &payload_valid);
        assert!(res_valid.is_ok());

        // Tampered payload with wrong signature should fail verification
        let payload_invalid = RemoteDecisionPayload {
            approval_id: approval_id.to_string(),
            decision: decision_str.to_string(),
            decided_by: device_id.to_string(),
            signature: Some(hex::encode([0x00; 64])),
            timestamp,
            nonce: "decision-nonce-invalid-0002".to_string(),
        };

        let res_invalid = verify_remote_decision_signature(device_id, &payload_invalid);
        assert!(res_invalid.is_err());
    }
}
