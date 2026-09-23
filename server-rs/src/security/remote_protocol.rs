//! @docs ARCHITECTURE:Security:RemoteProtocol
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security / RemoteProtocol
//! - **Primary Entrypoints**: `verify_paired_request`, `verify_remote_decision_signature`, `validate_public_key_format`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Ed25519 signature validation with timestamp freshness and nonce single-use anti-replay checks.
//! - `[Structural]` Prevents paired device key substitution without explicit prior revocation.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use base64::Engine;
use dashmap::{mapref::entry::Entry, DashMap};
use ed25519_dalek::{Verifier, VerifyingKey};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct PairingChallengeToken {
    pub _created_at: u64,
    pub expires_at: u64,
}

/// Ephemeral pairing challenge tokens generated for Desktop QR code scanning (3-minute TTL).
pub static PAIRING_TOKENS: Lazy<DashMap<String, PairingChallengeToken>> = Lazy::new(DashMap::new);

/// Registered paired companion devices (starts clean with no hardcoded fallback).
pub static PAIRED_DEVICES: Lazy<DashMap<String, PairedDevice>> = Lazy::new(DashMap::new);

/// Recently accepted per-device nonces. Entries are retained for the same window
/// as request timestamps so a captured signed request cannot be replayed.
pub static USED_REQUEST_NONCES: Lazy<DashMap<String, u64>> = Lazy::new(DashMap::new);

pub const REMOTE_REQUEST_MAX_SKEW_SECS: u64 = 300;

/// Global active pending oversight queue for remote HITL approvals
pub static PENDING_OVERSIGHT_QUEUE: Lazy<DashMap<String, PendingOversightItem>> =
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

pub fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn decode_signature(signature: &str) -> Result<ed25519_dalek::Signature, AppError> {
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

pub fn validate_request_freshness(timestamp: u64, nonce: &str) -> Result<(), AppError> {
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

pub fn consume_request_nonce(device_id: &str, nonce: &str, timestamp: u64) -> Result<(), AppError> {
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

pub fn verify_remote_decision_signature(
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

pub fn is_device_paired(device_id: &str) -> bool {
    PAIRED_DEVICES.contains_key(device_id)
}

pub fn register_paired_device_for_test(device: PairedDevice) {
    PAIRED_DEVICES.insert(device.id.clone(), device);
}
