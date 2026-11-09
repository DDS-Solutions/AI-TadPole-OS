//! @docs SECURITY:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / audit / verifier
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Cryptographic signature and Merkle chain verification.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::hasher::compute_canonical_hash;
use super::types::{AuditEntry, AuditError, GENESIS_PREV_HASH};
use anyhow::Result;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sqlx::SqlitePool;

pub(crate) fn resolve_verifying_key(
    hex_str: Option<&str>,
    fallback: Option<VerifyingKey>,
) -> Result<Option<VerifyingKey>> {
    match hex_str {
        Some(h) => {
            let bytes = hex::decode(h).map_err(|_| AuditError::BadKey)?;
            let arr: [u8; 32] = bytes
                .as_slice()
                .try_into()
                .map_err(|_| AuditError::BadKey)?;
            Ok(Some(
                VerifyingKey::from_bytes(&arr).map_err(|_| AuditError::BadKey)?,
            ))
        }
        None => Ok(fallback),
    }
}

pub(crate) fn verify_signature(
    pk: VerifyingKey,
    message: &[u8],
    sig_hex: &str,
) -> Result<(), AuditError> {
    let bytes = hex::decode(sig_hex).map_err(|_| AuditError::BadKey)?;
    let arr: [u8; 64] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| AuditError::BadKey)?;
    let sig = Signature::from_bytes(&arr);
    pk.verify(message, &sig).map_err(|_| AuditError::BadKey)
}

pub async fn verify_chain(
    pool: &SqlitePool,
    verifying_key: Option<VerifyingKey>,
    public_key_hex: Option<&str>,
) -> Result<bool> {
    let entries: Vec<AuditEntry> = sqlx::query_as("SELECT * FROM audit_trail ORDER BY seq ASC")
        .fetch_all(pool)
        .await?;
    if entries.is_empty() {
        return Ok(true);
    }

    if entries[0].prev_hash != GENESIS_PREV_HASH {
        return Ok(false);
    }

    let (verified, total) = verify_segments(
        &entries,
        verifying_key,
        public_key_hex,
        Some(GENESIS_PREV_HASH),
    )
    .await?;
    Ok(verified == total && total > 0)
}

pub async fn verify_last_n(
    pool: &SqlitePool,
    verifying_key: Option<VerifyingKey>,
    n: usize,
    public_key_hex: Option<&str>,
) -> Result<(usize, usize)> {
    if n == 0 {
        return Ok((0, 0));
    }
    let mut entries: Vec<AuditEntry> =
        sqlx::query_as("SELECT * FROM audit_trail ORDER BY seq DESC LIMIT ?")
            .bind(n as i64)
            .fetch_all(pool)
            .await?;
    entries.reverse();

    let initial_expected_prev = if entries.first().and_then(|e| e.seq) == Some(1) {
        Some(GENESIS_PREV_HASH)
    } else {
        None
    };

    verify_segments(
        &entries,
        verifying_key,
        public_key_hex,
        initial_expected_prev,
    )
    .await
}

pub fn verify_record(verifying_key: Option<VerifyingKey>, entry: &AuditEntry) -> bool {
    let computed = compute_canonical_hash(
        &entry.prev_hash,
        &entry.agent_id,
        entry.mission_id.as_deref(),
        entry.user_id.as_deref(),
        &entry.action,
        &entry.params,
        &entry.timestamp.to_rfc3339(),
    );
    if entry.current_hash != computed {
        return false;
    }
    if let (Some(pk), Some(sig_hex)) = (verifying_key, &entry.signature) {
        if verify_signature(pk, entry.current_hash.as_bytes(), sig_hex).is_err() {
            return false;
        }
    }
    true
}

pub async fn verify_segments(
    entries: &[AuditEntry],
    verifying_key: Option<VerifyingKey>,
    public_key_hex: Option<&str>,
    initial_expected_prev: Option<&str>,
) -> Result<(usize, usize)> {
    let public_key = resolve_verifying_key(public_key_hex, verifying_key)?;
    let total = entries.len();
    let mut verified = 0usize;
    let mut expected_prev: Option<&str> = initial_expected_prev;

    for entry in entries {
        if let Some(prev) = expected_prev {
            if entry.prev_hash != prev {
                return Ok((verified, total));
            }
        }

        let computed = compute_canonical_hash(
            &entry.prev_hash,
            &entry.agent_id,
            entry.mission_id.as_deref(),
            entry.user_id.as_deref(),
            &entry.action,
            &entry.params,
            &entry.timestamp.to_rfc3339(),
        );
        if entry.current_hash != computed {
            return Ok((verified, total));
        }

        if let (Some(pk), Some(sig_hex)) = (public_key, &entry.signature) {
            if verify_signature(pk, entry.current_hash.as_bytes(), sig_hex).is_err() {
                return Ok((verified, total));
            }
        }

        expected_prev = Some(entry.current_hash.as_str());
        verified += 1;
    }
    Ok((verified, total))
}
