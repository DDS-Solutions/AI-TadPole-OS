//! @docs SECURITY:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / audit / hasher
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic canonical hashing with domain separation and length prefixing.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::types::HASH_DOMAIN_SEPARATOR;
use sha2::{Digest, Sha256};

#[inline]
pub(crate) fn write_length_prefixed(hasher: &mut Sha256, s: &str) {
    let len = s.len() as u64;
    hasher.update(len.to_be_bytes());
    hasher.update(s.as_bytes());
}

pub fn compute_canonical_hash(
    prev_hash: &str,
    agent_id: &str,
    mission_id: Option<&str>,
    user_id: Option<&str>,
    action: &str,
    params: &str,
    timestamp_rfc3339: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(HASH_DOMAIN_SEPARATOR);
    write_length_prefixed(&mut hasher, prev_hash);
    write_length_prefixed(&mut hasher, agent_id);

    match mission_id {
        Some(m) => {
            hasher.update([1u8]);
            write_length_prefixed(&mut hasher, m);
        }
        None => hasher.update([0u8]),
    }
    match user_id {
        Some(u) => {
            hasher.update([1u8]);
            write_length_prefixed(&mut hasher, u);
        }
        None => hasher.update([0u8]),
    }

    write_length_prefixed(&mut hasher, action);
    write_length_prefixed(&mut hasher, params);
    write_length_prefixed(&mut hasher, timestamp_rfc3339);

    hex::encode(hasher.finalize())
}
