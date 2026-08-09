//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **CBS Cryptography**: Implements keyring storage, hot key rotation, HMAC-SHA256 signature
//! generation, and Agent-to-Agent (A2A) envelope authentication.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use subtle::ConstantTimeEq;
use super::types::{canonical_message, Permission};

pub(crate) struct Keyring {
    pub(crate) keys: HashMap<String, [u8; 32]>,
    pub(crate) active_key_id: String,
}

pub(crate) static KEYRING: Lazy<Mutex<Keyring>> = Lazy::new(|| {
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
        // Fallback to random key for development/testing
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        keys.insert("dev".to_string(), key);
        "dev".to_string()
    };

    if let Some(prev_val) = prev {
        if let Ok(key_bytes) = hex::decode(&prev_val) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                keys.insert("prev".to_string(), key);
            }
        }
    }

    Mutex::new(Keyring {
        keys,
        active_key_id,
    })
});

/// Adds or updates a signing key in the keyring.
#[allow(dead_code)]
pub fn set_key(id: &str, key: [u8; 32]) {
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
    keyring.keys.clear();

    let curr = std::env::var("CAPABILITY_KEY_CURR").ok();
    let prev = std::env::var("CAPABILITY_KEY_PREV").ok();

    keyring.active_key_id = if let Some(curr_val) = curr {
        if let Ok(key_bytes) = hex::decode(&curr_val) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                keyring.keys.insert("curr".to_string(), key);
                "curr".to_string()
            } else {
                "dev".to_string()
            }
        } else {
            "dev".to_string()
        }
    } else {
        let mut key = [0u8; 32];
        let mut rng = rand::rng();
        rng.fill_bytes(&mut key);
        keyring.keys.insert("dev".to_string(), key);
        "dev".to_string()
    };

    if let Some(prev_val) = prev {
        if let Ok(key_bytes) = hex::decode(&prev_val) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                keyring.keys.insert("prev".to_string(), key);
            }
        }
    }
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
    outer_hasher.finalize().to_vec()
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

pub fn sign_a2a_envelope(envelope_data: &str) -> String {
    let keyring = KEYRING.lock();
    let key_id = keyring.active_key_id.clone();
    let key = keyring.keys.get(&key_id).expect("Active key must exist");

    let signature_bytes = compute_hmac(key, envelope_data.as_bytes());
    format!("{}:{}", key_id, hex::encode(signature_bytes))
}

pub fn verify_a2a_envelope(envelope_data: &str, signature_header: &str) -> bool {
    let parts: Vec<&str> = signature_header.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    let key_id = parts[0];
    let sig_hex = parts[1];

    let keyring = KEYRING.lock();
    let key = match keyring.keys.get(key_id) {
        Some(k) => k,
        None => return false,
    };

    let decoded_sig = match hex::decode(sig_hex) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let signature_bytes = compute_hmac(key, envelope_data.as_bytes());
    bool::from(signature_bytes.ct_eq(&decoded_sig))
}
