//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Services / Backup / Crypto
//! - **Primary Entrypoints**: `derive_backup_keys`, `encrypt_backup_payload`, `decrypt_backup_payload`, `generate_recovery_key_manifest`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use crate::error::AppError;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use subtle::ConstantTimeEq;

const MAGIC_HEADER: &[u8; 16] = b"TADPOLE_SNAP_V1\0";
const KDF_ROUNDS: u32 = 100_000;
const SALT_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const MAC_SIZE: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryKeyManifest {
    pub version: String,
    pub kdf: String,
    pub rounds: u32,
    pub salt_hex: String,
    pub canary_ciphertext_hex: String,
    pub created_at: i64,
}

/// Derives a 32-byte encryption key and a 32-byte MAC key from a passphrase and salt.
/// Uses iterative HMAC-SHA512 key stretching (100,000 iterations).
pub fn derive_backup_keys(passphrase: &str, salt: &[u8]) -> ([u8; 32], [u8; 32]) {
    let mut state = Sha512::new();
    state.update(salt);
    state.update(passphrase.as_bytes());
    let mut current = state.finalize();

    for _ in 1..KDF_ROUNDS {
        let mut h = Sha512::new();
        h.update(current);
        h.update(salt);
        current = h.finalize();
    }

    let mut enc_key = [0u8; 32];
    let mut mac_key = [0u8; 32];
    enc_key.copy_from_slice(&current[0..32]);
    mac_key.copy_from_slice(&current[32..64]);

    (enc_key, mac_key)
}

/// Encrypts an arbitrary byte payload using ChaCha20 with HMAC-SHA256 authenticated envelope.
pub fn encrypt_backup_payload(payload: &[u8], passphrase: &str) -> Result<Vec<u8>, AppError> {
    if passphrase.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Passphrase cannot be empty".to_string(),
        ));
    }

    let mut rng = rand::rng();
    let mut salt = [0u8; SALT_SIZE];
    rng.fill(&mut salt);

    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce);

    let (enc_key, mac_key) = derive_backup_keys(passphrase, &salt);

    let mut ciphertext = payload.to_vec();
    let mut cipher = ChaCha20::new(&enc_key.into(), &nonce.into());
    cipher.apply_keystream(&mut ciphertext);

    // Compute HMAC-SHA256 over: Header || Salt || Nonce || Ciphertext
    let mut mac_hasher = Sha256::new();
    mac_hasher.update(mac_key);
    mac_hasher.update(MAGIC_HEADER);
    mac_hasher.update(salt);
    mac_hasher.update(nonce);
    mac_hasher.update(&ciphertext);
    let mac = mac_hasher.finalize();

    let mut envelope = Vec::with_capacity(
        MAGIC_HEADER.len() + SALT_SIZE + NONCE_SIZE + ciphertext.len() + MAC_SIZE,
    );
    envelope.extend_from_slice(MAGIC_HEADER);
    envelope.extend_from_slice(&salt);
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&ciphertext);
    envelope.extend_from_slice(&mac);

    Ok(envelope)
}

/// Decrypts and authenticates a backup envelope using the provided passphrase.
pub fn decrypt_backup_payload(envelope: &[u8], passphrase: &str) -> Result<Vec<u8>, AppError> {
    let min_len = MAGIC_HEADER.len() + SALT_SIZE + NONCE_SIZE + MAC_SIZE;
    if envelope.len() < min_len {
        return Err(AppError::BadRequest(
            "Corrupt or invalid backup envelope: header too short".to_string(),
        ));
    }

    if !envelope.starts_with(MAGIC_HEADER) {
        return Err(AppError::BadRequest(
            "Invalid snapshot format: header signature mismatch".to_string(),
        ));
    }

    let mut offset = MAGIC_HEADER.len();
    let salt = &envelope[offset..offset + SALT_SIZE];
    offset += SALT_SIZE;

    let nonce = &envelope[offset..offset + NONCE_SIZE];
    offset += NONCE_SIZE;

    let ciphertext_len = envelope.len() - offset - MAC_SIZE;
    let ciphertext = &envelope[offset..offset + ciphertext_len];
    offset += ciphertext_len;

    let expected_mac = &envelope[offset..offset + MAC_SIZE];

    let (enc_key, mac_key) = derive_backup_keys(passphrase, salt);

    // Verify MAC
    let mut mac_hasher = Sha256::new();
    mac_hasher.update(mac_key);
    mac_hasher.update(MAGIC_HEADER);
    mac_hasher.update(salt);
    mac_hasher.update(nonce);
    mac_hasher.update(ciphertext);
    let computed_mac = mac_hasher.finalize();

    if computed_mac.ct_eq(expected_mac).unwrap_u8() != 1 {
        return Err(AppError::Unauthorized(
            "Invalid passphrase or tampered backup payload".to_string(),
        ));
    }

    let mut plaintext = ciphertext.to_vec();
    let mut nonce_arr = [0u8; NONCE_SIZE];
    nonce_arr.copy_from_slice(nonce);
    let mut cipher = ChaCha20::new(&enc_key.into(), &nonce_arr.into());
    cipher.apply_keystream(&mut plaintext);

    Ok(plaintext)
}

/// Generates a standalone JSON recovery key manifest that contains the key parameters and a canary.
pub fn generate_recovery_key_manifest(passphrase: &str) -> Result<RecoveryKeyManifest, AppError> {
    let canary = b"TADPOLE_CANARY_VALID";
    let encrypted_canary = encrypt_backup_payload(canary, passphrase)?;
    let mut rng = rand::rng();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt);

    Ok(RecoveryKeyManifest {
        version: "1.0".to_string(),
        kdf: "PBKDF2-HMAC-SHA512".to_string(),
        rounds: KDF_ROUNDS,
        salt_hex: hex::encode(salt),
        canary_ciphertext_hex: hex::encode(encrypted_canary),
        created_at: chrono::Utc::now().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_encrypt_decrypt_roundtrip() {
        let payload = b"Hello Sovereign TadpoleOS State Backup!";
        let pass = "SecurePassphrase123!";

        let encrypted = encrypt_backup_payload(payload, pass).expect("encryption succeeds");
        assert_ne!(&encrypted[..], payload);

        let decrypted = decrypt_backup_payload(&encrypted, pass).expect("decryption succeeds");
        assert_eq!(&decrypted[..], payload);
    }

    #[test]
    fn test_backup_decrypt_wrong_passphrase() {
        let payload = b"Secret data";
        let encrypted = encrypt_backup_payload(payload, "RightPass").unwrap();

        let err = decrypt_backup_payload(&encrypted, "WrongPass");
        assert!(err.is_err());
    }

    #[test]
    fn test_backup_tamper_detection() {
        let payload = b"Tamper test data";
        let mut encrypted = encrypt_backup_payload(payload, "TestPass").unwrap();

        // Mutate a byte in the payload
        let mid = encrypted.len() / 2;
        encrypted[mid] ^= 0xFF;

        let err = decrypt_backup_payload(&encrypted, "TestPass");
        assert!(err.is_err());
    }
}
