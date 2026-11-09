//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / oversight ledger
//!
//! ### AI Assist Note
//! - Keep each oversight boundary in its own focused route module.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::*;

/// C-03: Verifies an ed25519 oversight decision signature.
///
/// `pinned_key` is the hex-encoded public key loaded from `OVERSIGHT_PUBLIC_KEY`
/// at startup (via `state.security.oversight_public_key`). When `Some`, only
/// the pinned key is accepted. When `require_pinned_key` is true, missing
/// startup pinning fails closed before any signature is accepted.
pub(crate) fn verify_oversight_signature(
    entry_id: &str,
    decision: &str,
    signature_hex: &str,
    pubkey_hex: &str,
    timestamp: Option<i64>,
    nonce: Option<&str>,
    pinned_key: Option<&str>,
    require_pinned_key: bool,
) -> Result<(), String> {
    verify_oversight_signature_canonical(
        entry_id,
        decision,
        None,
        None,
        signature_hex,
        pubkey_hex,
        timestamp,
        nonce,
        pinned_key,
        require_pinned_key,
    )
}

pub(crate) fn verify_oversight_signature_canonical(
    entry_id: &str,
    decision: &str,
    user_answer: Option<&str>,
    override_slot: Option<&str>,
    signature_hex: &str,
    pubkey_hex: &str,
    timestamp: Option<i64>,
    nonce: Option<&str>,
    pinned_key: Option<&str>,
    require_pinned_key: bool,
) -> Result<(), String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let sig_bytes =
        hex::decode(signature_hex.trim()).map_err(|e| format!("Invalid signature hex: {}", e))?;
    let pubkey_bytes =
        hex::decode(pubkey_hex.trim()).map_err(|e| format!("Invalid public key hex: {}", e))?;

    if require_pinned_key && pinned_key.is_none() {
        return Err(
            "OVERSIGHT_PUBLIC_KEY is required in production for oversight decisions".to_string(),
        );
    }

    let effective_pinned = pinned_key.map(str::to_string).or_else(|| {
        if require_pinned_key {
            None
        } else {
            std::env::var("OVERSIGHT_PUBLIC_KEY").ok()
        }
    });

    if let Some(ref pinned) = effective_pinned {
        let pinned_bytes = hex::decode(pinned.trim())
            .map_err(|e| format!("Invalid pinned public key hex: {}", e))?;
        if pubkey_bytes != pinned_bytes {
            return Err(
                "Provided public key does not match the pinned OVERSIGHT_PUBLIC_KEY".to_string(),
            );
        }
    } else {
        tracing::warn!(
            "⚠️ OVERSIGHT_PUBLIC_KEY is not configured in environment. Oversight signatures are verified but NOT pinned to an authorized operator!"
        );
    }

    let signature = Signature::from_bytes(
        &sig_bytes
            .try_into()
            .map_err(|_| "Signature must be 64 bytes".to_string())?,
    );
    let verifying_key = VerifyingKey::from_bytes(
        &pubkey_bytes
            .try_into()
            .map_err(|_| "Public key must be 32 bytes".to_string())?,
    )
    .map_err(|e| format!("Invalid verifying key: {}", e))?;

    let ts = timestamp.ok_or_else(|| "Missing signature timestamp".to_string())?;
    let n = nonce.ok_or_else(|| "Missing signature nonce".to_string())?;

    // Enforce timestamp window: ±5 minutes
    let now_ms_u: u64 = chrono::Utc::now().timestamp_millis().max(0) as u64;
    let ts_u: u64 = ts.max(0) as u64;
    let diff = now_ms_u.abs_diff(ts_u);
    if diff > OVS_SIG_WINDOW_MS as u64 {
        return Err("Signature timestamp is expired or too far in the future".to_string());
    }

    // Canonical format v2 binds both value and presence for every optional field.
    let answer_marker = match user_answer {
        Some(ans) => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(ans.as_bytes());
            format!("s{}", hex::encode(hasher.finalize()))
        }
        None => "n".to_string(),
    };
    let slot_marker = match override_slot {
        Some(slot) => format!("s{}", hex::encode(slot.as_bytes())),
        None => "n".to_string(),
    };
    let payload_v2 = format!(
        "ovs:v2|{}|{}|{}|{}|{}|{}",
        entry_id, decision, ts, n, answer_marker, slot_marker
    );

    if verifying_key
        .verify(payload_v2.as_bytes(), &signature)
        .is_ok()
    {
        return Ok(());
    }

    // Backward compatibility: If no answer or slot was provided, also accept ovs:v1
    if user_answer.is_none() && override_slot.is_none() {
        let payload_v1 = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, ts, n);
        if verifying_key
            .verify(payload_v1.as_bytes(), &signature)
            .is_ok()
        {
            return Ok(());
        }
    }

    Err("Signature verification failed".to_string())
}
