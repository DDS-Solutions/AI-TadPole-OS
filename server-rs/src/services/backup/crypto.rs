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
use zeroize::Zeroize;

const MAGIC_HEADER_V1: &[u8; 16] = b"TADPOLE_SNAP_V1\0";
const MAGIC_HEADER_V2: &[u8; 16] = b"TADPOLE_SNAP_V2\0";
const KDF_ROUNDS_V1: u32 = 100_000;
const KDF_ROUNDS_V2: u32 = 210_000;
const MIN_PASSPHRASE_LEN: usize = 12;
const SALT_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const MAC_SIZE: usize = 32;
const SHA256_BLOCK_SIZE: usize = 64;
const SHA512_BLOCK_SIZE: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryKeyManifest {
    pub version: String,
    pub kdf: String,
    pub rounds: u32,
    pub salt_hex: String,
    pub canary_ciphertext_hex: String,
    pub created_at: i64,
}

/// Computes RFC 2104 HMAC-SHA256 over arbitrary message segments without length-extension exposure.
fn hmac_sha256_parts(key: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut key_block = [0u8; SHA256_BLOCK_SIZE];
    if key.len() > SHA256_BLOCK_SIZE {
        let hashed = Sha256::digest(key);
        key_block[..32].copy_from_slice(&hashed);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; SHA256_BLOCK_SIZE];
    let mut opad = [0x5cu8; SHA256_BLOCK_SIZE];
    for i in 0..SHA256_BLOCK_SIZE {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    for part in parts {
        inner.update(part);
    }
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    let mac = outer.finalize();

    key_block.zeroize();
    ipad.zeroize();
    opad.zeroize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&mac);
    out
}

/// Computes RFC 2104 HMAC-SHA512 over arbitrary message segments.
fn hmac_sha512_parts(key: &[u8], parts: &[&[u8]]) -> [u8; 64] {
    let mut key_block = [0u8; SHA512_BLOCK_SIZE];
    if key.len() > SHA512_BLOCK_SIZE {
        let hashed = Sha512::digest(key);
        key_block[..64].copy_from_slice(&hashed);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; SHA512_BLOCK_SIZE];
    let mut opad = [0x5cu8; SHA512_BLOCK_SIZE];
    for i in 0..SHA512_BLOCK_SIZE {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    let mut inner = Sha512::new();
    inner.update(ipad);
    for part in parts {
        inner.update(part);
    }
    let inner_digest = inner.finalize();

    let mut outer = Sha512::new();
    outer.update(opad);
    outer.update(inner_digest);
    let mac = outer.finalize();

    key_block.zeroize();
    ipad.zeroize();
    opad.zeroize();

    let mut out = [0u8; 64];
    out.copy_from_slice(&mac);
    out
}

/// RFC 2898 PBKDF2-HMAC-SHA512 producing a single 64-byte block (32-byte enc_key + 32-byte mac_key).
fn pbkdf2_hmac_sha512_64(password: &[u8], salt: &[u8], rounds: u32) -> [u8; 64] {
    let block_index = 1u32.to_be_bytes();
    let mut u = hmac_sha512_parts(password, &[salt, &block_index]);
    let mut dk = u;

    for _ in 1..rounds {
        u = hmac_sha512_parts(password, &[&u]);
        for (d, b) in dk.iter_mut().zip(u.iter()) {
            *d ^= *b;
        }
    }

    u.zeroize();
    dk
}

/// Derives a 32-byte encryption key and a 32-byte MAC key from a passphrase and salt
/// using RFC 2898 PBKDF2-HMAC-SHA512 with the requested iteration count.
fn derive_backup_keys_with_rounds(
    passphrase: &str,
    salt: &[u8],
    rounds: u32,
) -> ([u8; 32], [u8; 32]) {
    let mut dk = pbkdf2_hmac_sha512_64(passphrase.as_bytes(), salt, rounds);
    let mut enc_key = [0u8; 32];
    let mut mac_key = [0u8; 32];
    enc_key.copy_from_slice(&dk[0..32]);
    mac_key.copy_from_slice(&dk[32..64]);
    dk.zeroize();

    (enc_key, mac_key)
}

/// Derives a 32-byte encryption key and a 32-byte MAC key from a passphrase and salt.
/// Uses RFC 2898 PBKDF2-HMAC-SHA512 (210,000 iterations per OWASP guidelines).
pub fn derive_backup_keys(passphrase: &str, salt: &[u8]) -> ([u8; 32], [u8; 32]) {
    derive_backup_keys_with_rounds(passphrase, salt, KDF_ROUNDS_V2)
}

/// Encrypts an arbitrary byte payload using ChaCha20 with RFC 2104 HMAC-SHA256 Encrypt-then-MAC (V2 envelope).
pub fn encrypt_backup_payload(payload: &[u8], passphrase: &str) -> Result<Vec<u8>, AppError> {
    let trimmed = passphrase.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(
            "Passphrase cannot be empty".to_string(),
        ));
    }
    if trimmed.chars().count() < MIN_PASSPHRASE_LEN {
        return Err(AppError::BadRequest(format!(
            "Backup passphrase must be at least {} characters",
            MIN_PASSPHRASE_LEN
        )));
    }

    let mut rng = rand::rng();
    let mut salt = [0u8; SALT_SIZE];
    rng.fill(&mut salt);

    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce);

    let (mut enc_key, mut mac_key) = derive_backup_keys(passphrase, &salt);

    let mut ciphertext = payload.to_vec();
    let mut cipher = ChaCha20::new(&enc_key.into(), &nonce.into());
    cipher.apply_keystream(&mut ciphertext);

    // RFC 2104 HMAC-SHA256 over: Header || Salt || Nonce || Ciphertext
    let mac = hmac_sha256_parts(&mac_key, &[MAGIC_HEADER_V2, &salt, &nonce, &ciphertext]);
    enc_key.zeroize();
    mac_key.zeroize();

    let mut envelope = Vec::with_capacity(
        MAGIC_HEADER_V2.len() + SALT_SIZE + NONCE_SIZE + ciphertext.len() + MAC_SIZE,
    );
    envelope.extend_from_slice(MAGIC_HEADER_V2);
    envelope.extend_from_slice(&salt);
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&ciphertext);
    envelope.extend_from_slice(&mac);

    Ok(envelope)
}

/// Decrypts and authenticates a V2 (210k rounds) or legacy V1 (100k rounds) backup envelope.
pub fn decrypt_backup_payload(envelope: &[u8], passphrase: &str) -> Result<Vec<u8>, AppError> {
    let min_len = MAGIC_HEADER_V2.len() + SALT_SIZE + NONCE_SIZE + MAC_SIZE;
    if envelope.len() < min_len {
        return Err(AppError::BadRequest(
            "Corrupt or invalid backup envelope: header too short".to_string(),
        ));
    }

    let (header, kdf_rounds) = if envelope.starts_with(MAGIC_HEADER_V2) {
        (MAGIC_HEADER_V2, KDF_ROUNDS_V2)
    } else if envelope.starts_with(MAGIC_HEADER_V1) {
        (MAGIC_HEADER_V1, KDF_ROUNDS_V1)
    } else {
        return Err(AppError::BadRequest(
            "Invalid snapshot format: header signature mismatch".to_string(),
        ));
    };

    let mut offset = header.len();
    let salt = &envelope[offset..offset + SALT_SIZE];
    offset += SALT_SIZE;

    let nonce = &envelope[offset..offset + NONCE_SIZE];
    offset += NONCE_SIZE;

    let ciphertext_len = envelope.len() - offset - MAC_SIZE;
    let ciphertext = &envelope[offset..offset + ciphertext_len];
    offset += ciphertext_len;

    let expected_mac = &envelope[offset..offset + MAC_SIZE];

    let (mut enc_key, mut mac_key) = derive_backup_keys_with_rounds(passphrase, salt, kdf_rounds);

    // Verify RFC 2104 HMAC-SHA256 in constant time
    let computed_mac = hmac_sha256_parts(&mac_key, &[header, salt, nonce, ciphertext]);
    mac_key.zeroize();

    if computed_mac.ct_eq(expected_mac).unwrap_u8() != 1 {
        enc_key.zeroize();
        return Err(AppError::Unauthorized(
            "Invalid passphrase or tampered backup payload".to_string(),
        ));
    }

    let mut plaintext = ciphertext.to_vec();
    let mut nonce_arr = [0u8; NONCE_SIZE];
    nonce_arr.copy_from_slice(nonce);
    let mut cipher = ChaCha20::new(&enc_key.into(), &nonce_arr.into());
    cipher.apply_keystream(&mut plaintext);
    enc_key.zeroize();

    Ok(plaintext)
}

/// Generates a standalone JSON recovery key manifest containing the KDF parameters,
/// the exact canary envelope salt, and an authenticated canary ciphertext.
pub fn generate_recovery_key_manifest(passphrase: &str) -> Result<RecoveryKeyManifest, AppError> {
    let canary = b"TADPOLE_CANARY_VALID";
    let encrypted_canary = encrypt_backup_payload(canary, passphrase)?;
    let canary_salt = &encrypted_canary[MAGIC_HEADER_V2.len()..MAGIC_HEADER_V2.len() + SALT_SIZE];

    Ok(RecoveryKeyManifest {
        version: "2.0".to_string(),
        kdf: "PBKDF2-HMAC-SHA512".to_string(),
        rounds: KDF_ROUNDS_V2,
        salt_hex: hex::encode(canary_salt),
        canary_ciphertext_hex: hex::encode(encrypted_canary),
        created_at: chrono::Utc::now().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc4231_hmac_sha256_and_sha512_test_vectors() {
        // RFC 4231 Test Case 1: Key = 0x0b * 20, Data = "Hi There"
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let mac256 = hmac_sha256_parts(&key, &[data]);
        assert_eq!(
            hex::encode(mac256),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        let mac512 = hmac_sha512_parts(&key, &[data]);
        assert_eq!(
            hex::encode(mac512),
            "87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cdedaa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854"
        );

        // RFC 4231 Test Case 2: Key = "Jefe", Data = "what do ya want for nothing?"
        let mac256_2 = hmac_sha256_parts(b"Jefe", &[b"what do ya want for nothing?"]);
        assert_eq!(
            hex::encode(mac256_2),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn test_pbkdf2_hmac_sha512_rfc_vector() {
        // Known PBKDF2-HMAC-SHA512 vector: password="password", salt="salt", c=1, dkLen=64
        let dk = pbkdf2_hmac_sha512_64(b"password", b"salt", 1);
        assert_eq!(
            hex::encode(dk),
            "867f70cf1ade02cff3752599a3a53dc4af34c7a669815ae5d513554e1c8cf252c02d470a285a0501bad999bfe943c08f050235d7d68b1da55e63f73b60a57fce"
        );
    }

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
        let encrypted = encrypt_backup_payload(payload, "RightPassphrase!2026").unwrap();

        let err = decrypt_backup_payload(&encrypted, "WrongPassphrase!2026");
        assert!(err.is_err());
    }

    #[test]
    fn test_backup_rejects_short_passphrase() {
        let err = encrypt_backup_payload(b"Secret data", "short");
        assert!(matches!(err, Err(AppError::BadRequest(_))));
    }

    #[test]
    fn test_backup_tamper_detection() {
        let payload = b"Tamper test data";
        let mut encrypted = encrypt_backup_payload(payload, "TestPassphrase!2026").unwrap();

        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;

        let err = decrypt_backup_payload(&encrypted, "TestPassphrase!2026");
        assert!(err.is_err());
    }

    #[test]
    fn test_recovery_manifest_salt_matches_canary_envelope() {
        let manifest = generate_recovery_key_manifest("ManifestPass!2026").unwrap();
        assert_eq!(manifest.kdf, "PBKDF2-HMAC-SHA512");
        assert_eq!(manifest.rounds, KDF_ROUNDS_V2);
        let canary_bytes = hex::decode(&manifest.canary_ciphertext_hex).unwrap();
        let envelope_salt =
            hex::encode(&canary_bytes[MAGIC_HEADER_V2.len()..MAGIC_HEADER_V2.len() + SALT_SIZE]);
        assert_eq!(manifest.salt_hex, envelope_salt);
        let decoded = decrypt_backup_payload(&canary_bytes, "ManifestPass!2026").unwrap();
        assert_eq!(decoded, b"TADPOLE_CANARY_VALID");
    }

    #[test]
    fn test_legacy_v1_envelope_decrypt_backward_compatibility() {
        let payload = b"Legacy V1 snapshot payload";
        let pass = "LegacyPassphrase!2026";
        let salt = [0x42u8; SALT_SIZE];
        let nonce = [0x24u8; NONCE_SIZE];
        let (enc_key, mac_key) = derive_backup_keys_with_rounds(pass, &salt, KDF_ROUNDS_V1);

        let mut ciphertext = payload.to_vec();
        let mut cipher = ChaCha20::new(&enc_key.into(), &nonce.into());
        cipher.apply_keystream(&mut ciphertext);
        let mac = hmac_sha256_parts(&mac_key, &[MAGIC_HEADER_V1, &salt, &nonce, &ciphertext]);

        let mut v1_envelope = Vec::new();
        v1_envelope.extend_from_slice(MAGIC_HEADER_V1);
        v1_envelope.extend_from_slice(&salt);
        v1_envelope.extend_from_slice(&nonce);
        v1_envelope.extend_from_slice(&ciphertext);
        v1_envelope.extend_from_slice(&mac);

        let decrypted = decrypt_backup_payload(&v1_envelope, pass).expect("V1 must still decrypt");
        assert_eq!(decrypted, payload);
    }
}
