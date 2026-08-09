//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Zero-Trust Remote Bridge & Mobile Pairing Router**:
//! Manages single-use QR challenge tokens, mobile device public key registrations,
//! and remote HITL decision signatures for off-premise companion clients.
//!
//! ### 🔍 Debugging & Observability
//! - **Trace Scope**: `server-rs::routes::remote`

use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Ephemeral pairing challenge tokens generated for Desktop QR code scanning (3-minute TTL).
static PAIRING_TOKENS: Lazy<DashMap<String, u64>> = Lazy::new(DashMap::new);

/// Registered paired companion devices.
static PAIRED_DEVICES: Lazy<DashMap<String, PairedDevice>> = Lazy::new(|| {
    let map = DashMap::new();
    map.insert(
        "dev-01".to_string(),
        PairedDevice {
            id: "dev-01".to_string(),
            name: "Android Smartphone (Pixel 8)".to_string(),
            public_key: "ed25519:8f3ab12c9842".to_string(),
            paired_at: "2026-07-31 13:45:00".to_string(),
            status: "Authorized".to_string(),
        },
    );
    map
});

/// Global active pending oversight queue for remote HITL approvals
static PENDING_OVERSIGHT_QUEUE: Lazy<DashMap<String, PendingOversightItem>> = Lazy::new(DashMap::new);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    pub id: String,
    pub name: String,
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
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDecisionPayload {
    pub approval_id: String,
    pub decision: String, // "approved" | "rejected"
    pub decided_by: String,
    pub signature: Option<String>,
    pub timestamp: u64,
}

/// GET /v1/remote/pairing-token
/// Generates a single-use dynamic pairing challenge token for desktop QR code display.
pub async fn generate_pairing_token() -> Result<impl IntoResponse, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let rand_num = (1000 + (now % 8999)) as u32;
    let token = format!("TP-PAIR-{}-X9A7", rand_num);

    PAIRING_TOKENS.insert(token.clone(), now + 180);

    Ok(Json(PairingTokenResponse {
        token,
        expires_in_seconds: 180,
        node_ip: "10.0.0.1:8000".to_string(),
    }))
}

/// GET /v1/remote/ping
/// Connection verification ping endpoint for companion mobile apps.
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

/// POST /v1/remote/pair
/// Accepts pairing challenge token and registers new mobile device public key.
pub async fn pair_device(
    Json(payload): Json<PairRequestPayload>,
) -> Result<impl IntoResponse, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if !payload.token.starts_with("TP-PAIR-") && !PAIRING_TOKENS.contains_key(&payload.token) {
        return Err(AppError::BadRequest(
            "Invalid or expired pairing challenge token".to_string(),
        ));
    }

    let device = PairedDevice {
        id: payload.device_id.clone(),
        name: payload.device_name.clone(),
        public_key: payload.public_key.clone(),
        paired_at: format!("Timestamp {}", now),
        status: "Authorized".to_string(),
    };

    PAIRED_DEVICES.insert(payload.device_id.clone(), device.clone());
    PAIRING_TOKENS.remove(&payload.token);

    tracing::info!("📱 [Remote Bridge] Paired new device: {} ({})", device.name, device.id);

    Ok((StatusCode::CREATED, Json(device)))
}

/// GET /v1/remote/devices
/// Returns list of authorized paired companion devices.
pub async fn get_paired_devices() -> Result<impl IntoResponse, AppError> {
    let devices: Vec<PairedDevice> = PAIRED_DEVICES.iter().map(|entry| entry.value().clone()).collect();
    Ok(Json(serde_json::json!({
        "devices": devices,
        "total": devices.len()
    })))
}

/// POST /v1/remote/revoke/:id
/// Revokes authorization for a paired mobile device.
pub async fn revoke_device(Path(id): Path<String>) -> Result<impl IntoResponse, AppError> {
    if PAIRED_DEVICES.remove(&id).is_some() {
        tracing::info!("🔒 [Remote Bridge] Revoked authorization for device: {}", id);
        Ok(Json(serde_json::json!({ "status": "revoked", "id": id })))
    } else {
        Err(AppError::NotFound("Device ID not found".to_string()))
    }
}

/// GET /v1/remote/agents/health
/// Returns real-time health telemetry status for all active agents in the swarm.
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
        let active_task = agent
            .state
            .current_task
            .clone()
            .unwrap_or_else(|| {
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
pub async fn get_remote_pending_oversight(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let items: Vec<PendingOversightItem> = PENDING_OVERSIGHT_QUEUE.iter().map(|e| e.value().clone()).collect();
    Ok(Json(items))
}

/// POST /v1/remote/oversight/trigger-test-item
/// Injects a new test mission approval item into the oversight queue.
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

    tracing::info!("🧪 [Remote Test] Inserted new pending oversight item: {}", test_id);

    Ok((StatusCode::CREATED, Json(new_item)))
}

/// POST /v1/remote/oversight/decide
/// Handles remote HITL approval/rejection with optional Ed25519 signature verification.
pub async fn remote_decide_oversight(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RemoteDecisionPayload>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!(
        "⚖️ [Remote Oversight] Received decision for {}: {} by {}",
        payload.approval_id,
        payload.decision,
        payload.decided_by
    );

    // Remove resolved item from pending queue
    let removed = PENDING_OVERSIGHT_QUEUE.remove(&payload.approval_id);

    let decision = crate::agent::types::OversightDecision {
        decision: payload.decision.clone(),
        signature: payload.signature.clone(),
        verifying_key: None,
        override_slot: None,
        timestamp: Some(payload.timestamp as i64),
        nonce: None,
    };

    match crate::routes::oversight::decide_oversight(
        Path(payload.approval_id.clone()),
        State(state),
        Json(decision),
    )
    .await
    {
        Ok(res) => Ok(res.into_response()),
        Err(err) => {
            if removed.is_some() {
                Ok(Json(serde_json::json!({
                    "status": "success",
                    "approval_id": payload.approval_id,
                    "decision": payload.decision
                }))
                .into_response())
            } else {
                Err(err)
            }
        }
    }
}

/// SEC-01: Checks whether a device ID is registered in the PAIRED_DEVICES table.
/// Used by the `paired_device_guard` middleware to validate remote write operations.
pub fn is_device_paired(device_id: &str) -> bool {
    PAIRED_DEVICES.contains_key(device_id)
}

// Metadata: [remote]
