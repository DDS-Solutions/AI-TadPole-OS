//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / crypto
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[crypto]`
//! - **Witness Tests**: none declared

use super::types::{canonical_message, Permission};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

pub(crate) struct Keyring {
    pub(crate) keys: HashMap<String, [u8; 32]>,
    pub(crate) active_key_id: String,
}

pub(crate) fn build_keyring_from_env() -> Keyring {
    let mut keys = HashMap::new();

    let curr = std::env::var("CAPABILITY_KEY_CURR").ok();
    let prev = std::env::var("CAPABILITY_KEY_PREV").ok();

    let active_key_id = if let Some(curr_val) = curr {
        if let Ok(key_bytes) = hex::decode(&curr_val) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                keys.insert("curr".to_string(), key);
                "curr".to_string()
            } else {
                panic!("CAPABILITY_KEY_CURR must be a 32-byte hex string (64 characters)");
            }
        } else {
            panic!("CAPABILITY_KEY_CURR is not a valid hex string");
        }
    } else {
        tracing::warn!("[crypto] No CAPABILITY_KEY_CURR configured; generated ephemeral development key. Tokens will not persist across restarts.");
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        keys.insert("dev".to_string(), key);
        "dev".to_string()
    };

    if let Some(prev_val) = prev {
        match hex::decode(&prev_val) {
            Ok(key_bytes) => {
                if key_bytes.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&key_bytes);
                    keys.insert("prev".to_string(), key);
                } else {
                    tracing::error!("[crypto] CAPABILITY_KEY_PREV is invalid: must be 32 bytes (64 hex characters)");
                }
            }
            Err(e) => {
                tracing::error!(
                    "[crypto] CAPABILITY_KEY_PREV is not a valid hex string: {}",
                    e
                );
            }
        }
    }

    Keyring {
        keys,
        active_key_id,
    }
}

pub(crate) static KEYRING: Lazy<Mutex<Keyring>> =
    Lazy::new(|| Mutex::new(build_keyring_from_env()));

/// Adds or updates a signing key in the keyring.
#[allow(dead_code)]
pub fn set_key(id: &str, key: [u8; 32]) {
    tracing::debug!("[crypto] Updated signing key for id={}", id);
    let mut keyring = KEYRING.lock();
    keyring.keys.insert(id.to_string(), key);
}

/// Sets the active key ID for future capability token mints.
#[allow(dead_code)]
pub fn set_active_key_id(id: &str) -> Result<(), String> {
    let mut keyring = KEYRING.lock();
    if keyring.keys.contains_key(id) {
        keyring.active_key_id = id.to_string();
        Ok(())
    } else {
        Err(format!("Key ID '{}' not found in keyring", id))
    }
}

/// Resets the keyring back to its environment variable configuration.
#[cfg(test)]
#[allow(dead_code)]
pub fn reset_keyring_for_test() {
    let mut keyring = KEYRING.lock();
    *keyring = build_keyring_from_env();
}

/// Computes a standard HMAC-SHA256 tag over `msg` using a 32-byte secret `key`.
pub fn compute_hmac(key: &[u8; 32], msg: &[u8]) -> Vec<u8> {
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..32 {
        ipad[i] ^= key[i];
        opad[i] ^= key[i];
    }

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(ipad);
    inner_hasher.update(msg);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(opad);
    outer_hasher.update(inner_hash);
    let result = outer_hasher.finalize().to_vec();

    // Zeroize intermediate key-derived blocks on stack
    ipad.zeroize();
    opad.zeroize();

    result
}

pub(crate) fn compute_signature(
    id: &str,
    agent_id: &str,
    mission_id: &str,
    permissions: &[Permission],
    expires_at: &chrono::DateTime<chrono::Utc>,
    key: &[u8; 32],
) -> Vec<u8> {
    let msg = canonical_message(
        id,
        agent_id,
        mission_id,
        permissions,
        expires_at.timestamp(),
    );

    compute_hmac(key, &msg)
}

pub fn canonical_a2a_message(
    id: &str,
    mission_id: &str,
    source_agent_id: &str,
    target_agent_id: &str,
    instruction: &str,
    timestamp: i64,
    nonce: &str,
) -> Vec<u8> {
    let instruction_hash = hex::encode(Sha256::digest(instruction.as_bytes()));
    format!(
        "A2A1|{}|{}|{}|{}|{}|{}|{}",
        id, mission_id, source_agent_id, target_agent_id, timestamp, nonce, instruction_hash
    )
    .into_bytes()
}

pub fn sign_a2a_canonical(
    id: &str,
    mission_id: &str,
    source_agent_id: &str,
    target_agent_id: &str,
    instruction: &str,
    timestamp: i64,
    nonce: &str,
) -> String {
    let keyring = KEYRING.lock();
    let key_id = &keyring.active_key_id;
    let key = keyring.keys.get(key_id).expect("Active key must exist");

    let msg = canonical_a2a_message(
        id,
        mission_id,
        source_agent_id,
        target_agent_id,
        instruction,
        timestamp,
        nonce,
    );

    let signature_bytes = compute_hmac(key, &msg);
    format!("{}:{}", key_id, hex::encode(signature_bytes))
}

pub fn verify_a2a_canonical(
    id: &str,
    mission_id: &str,
    source_agent_id: &str,
    target_agent_id: &str,
    instruction: &str,
    timestamp: i64,
    nonce: &str,
    signature_header: &str,
) -> bool {
    let (key_id, sig_hex) = match signature_header.split_once(':') {
        Some((k, s)) => (k, s),
        None => return false,
    };

    let keyring = KEYRING.lock();
    let key = match keyring.keys.get(key_id) {
        Some(k) => k,
        None => return false,
    };

    let decoded_sig = match hex::decode(sig_hex) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let msg = canonical_a2a_message(
        id,
        mission_id,
        source_agent_id,
        target_agent_id,
        instruction,
        timestamp,
        nonce,
    );

    let signature_bytes = compute_hmac(key, &msg);
    bool::from(signature_bytes.ct_eq(&decoded_sig))
}

pub fn sign_a2a_envelope(envelope_data: &str) -> String {
    let keyring = KEYRING.lock();
    let key_id = &keyring.active_key_id;
    let key = keyring.keys.get(key_id).expect("Active key must exist");

    let mut msg = Vec::with_capacity(5 + envelope_data.len());
    msg.extend_from_slice(b"A2A1\0");
    msg.extend_from_slice(envelope_data.as_bytes());

    let signature_bytes = compute_hmac(key, &msg);
    format!("{}:{}", key_id, hex::encode(signature_bytes))
}

pub fn verify_a2a_envelope(envelope_data: &str, signature_header: &str) -> bool {
    let (key_id, sig_hex) = match signature_header.split_once(':') {
        Some((k, s)) => (k, s),
        None => return false,
    };

    let keyring = KEYRING.lock();
    let key = match keyring.keys.get(key_id) {
        Some(k) => k,
        None => return false,
    };

    let decoded_sig = match hex::decode(sig_hex) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut msg = Vec::with_capacity(5 + envelope_data.len());
    msg.extend_from_slice(b"A2A1\0");
    msg.extend_from_slice(envelope_data.as_bytes());

    let signature_bytes = compute_hmac(key, &msg);
    bool::from(signature_bytes.ct_eq(&decoded_sig))
}
