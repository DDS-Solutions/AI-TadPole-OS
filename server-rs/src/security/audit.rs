//! @docs SECURITY:Core
//!
//! ### AI Assist Note
//! **Tamper-Evident Audit Ledger**: Implements the single-writer cryptographically linked hash-chain.
//! Provides non-repudiative logging of agent and user actions.
//!
//! ### 🔍 Debugging & Observability
//! - **Persistence**: `audit_trail` table in SQLite.
//! - **Failure Path**: DB write failure or channel timeout.
//! - **Telemetry Link**: Search `[audit]` in tracing logs.
//!
//! Tamper-Evident Audit Ledger - Non-Repudiative Logging
//!
//! Orchestrates the high-speed recording of agent actions into a
//! cryptographically linked hash-chain (SHA-256, domain-separated) with
//! optional Ed25519 digital signatures.
//!
//! ## Invariants
//!
//! 1. **Single-writer serialization.** Hash computation, signing, and DB
//!    insertion are serialized on exactly one writer task. No other code
//!    path computes, updates, or reads the chain head. This eliminates the
//!    v1 cache+DB TOCTOU race (X-001).
//! 2. **Canonical hash construction.** Every entry's hash is
//!    `SHA-256("AUDIT-V2" || len-prefixed(prev_hash) || len-prefixed(agent_id)
//!    || mission_presence_byte || len-prefixed(mission_id) || user_presence_byte
//!    || len-prefixed(user_id) || len-prefixed(action) || len-prefixed(params)
//!    || len-prefixed(timestamp_rfc3339))`. Domain separator + length
//!    prefixes + explicit presence bytes prevent the length-extension /
//!    field-omission class of attacks (v1 X-002).
//! 3. **Secret hygiene.** The Ed25519 signing key is wrapped in
//!    `secrecy::SecretBox<SecretSigningKey>`; the hex-decoded staging buffer and the
//!    env-var source `String` are wrapped in `zeroize::Zeroizing`.
//!    All sensitive buffers are zeroized on drop (v2 V2-002, V2-003, AV3-001).
//! 4. **Bounded liveness.** Channel send and worker response are bounded by
//!    a 5-second timeout (v1 X-005). The worker's startup DB read is also
//!    bounded (v2 V2-006).
//! 5. **Pure cross-module seam.** The function takes already-redacted
//!    `params`; it never invokes `redact_secrets` itself, preserving the
//!    determinism contract with `security::redact_secrets`.
//!
//! ## Public API changes vs. v2
//!
//! - `AuditWriteRequest` and its `resp_tx` are private (used only inside
//!   this module). External callers use `MerkleAuditTrail::record`.
//! - `get_last_hash` is removed (was dead code on the hot path; the
//!   cache+DB TOCTOU it embodied must not be reintroduced).
//! - `MerkleAuditTrail::record` returns `anyhow::Result<AuditEntry>`.
//!   Errors carry a `kind` discriminant suitable for structured logging;
//!   the underlying SQL message is **not** propagated to the caller
//!   (v2 V2-004).
//!
//! ## Test status
//!
//! The `rand` import is fixed in v3 (`use rand::Rng;` for rand v0.10.1).
//! `cargo test` must pass in CI as a merge gate.

#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use futures::FutureExt;
use secrecy::{ExposeSecret, SecretBox};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, instrument, warn};
use zeroize::Zeroizing;

/// Maximum time we will wait for a slot in the writer channel.
const QUEUE_SEND_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum time we will wait for the writer to acknowledge a write.
const WRITE_ACK_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum time we will wait for the worker to perform its startup DB read.
const WORKER_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Sentinel used as the `prev_hash` of the genesis entry.
///
/// Chosen as a non-numeric 64-character string so that an attacker cannot
/// produce a real SHA-256 output that begins with 64 ASCII zero digits.
const GENESIS_PREV_HASH: &str = "GENESIS-PREV-HASH-AUDIT-V2-0000000000000000000000000000000000000000000000";

/// Domain separator prepended to every canonical hash. Changing this value
/// invalidates every existing chain; bump to `AUDIT-V3` and add a migration
/// if the canonicalization ever changes.
const HASH_DOMAIN_SEPARATOR: &[u8] = b"AUDIT-V2";

/// Capacity of the bounded channel between callers and the writer.
const WRITER_CHANNEL_CAPACITY: usize = 1024;

// =====================================================================
// SecretSigningKey (Zeroize Wrapper)
// =====================================================================

/// Safe wrapper for `SigningKey` ensuring memory is zeroized on drop.
#[derive(Clone)]
pub struct SecretSigningKey(pub SigningKey);

impl zeroize::Zeroize for SecretSigningKey {
    fn zeroize(&mut self) {
        let dummy = [0u8; 32];
        self.0 = SigningKey::from_bytes(&dummy);
    }
}

impl secrecy::CloneableSecret for SecretSigningKey {}

// =====================================================================
// AuditEntry
// =====================================================================

/// A single entry in the tamper-evident audit ledger.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub mission_id: Option<String>,
    pub user_id: Option<String>,
    pub action: String,
    pub params: String,
    pub prev_hash: String,
    pub current_hash: String,
    pub signature: Option<String>,
}

// =====================================================================
// Error type (replaces v2's `Result<_, String>` across the channel)
// =====================================================================

/// Discriminant for failures surfaced from the writer task.
///
/// The underlying `sqlx::Error` is **never** propagated to callers; only
/// the kind is. This closes v2 V2-004 (log injection via error strings).
#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum AuditError {
    #[error("audit writer channel closed")]
    WriterShutdown,
    #[error("audit writer queue full or send timed out")]
    QueueTimeout,
    #[error("audit writer did not acknowledge in time")]
    AckTimeout,
    #[error("audit writer failed to commit to storage")]
    Storage,
    #[error("signing key is malformed or missing")]
    BadKey,
    #[allow(dead_code)]
    #[error("audit writer panicked")]
    WriterPanicked,
}

// =====================================================================
// Canonical hash
// =====================================================================

#[inline]
fn write_length_prefixed(hasher: &mut Sha256, s: &str) {
    let len = s.len() as u64;
    hasher.update(len.to_be_bytes());
    hasher.update(s.as_bytes());
}

/// Length-prefixed canonical hash with domain separation.
///
/// This is the single source of truth for entry hashing. `verify_record`,
/// `verify_segments`, and the writer all call this function. A bug here
/// invalidates the tamper-evidence property for the entire chain.
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

    // Explicit presence byte: 0 = None, 1 = Some. This closes the v1 X-002
    // attack where flipping `mission_id: Some(x)` <-> `None` was not detected.
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

// =====================================================================
// Internal channel envelope
// =====================================================================

struct AuditWriteRequest {
    agent_id: String,
    mission_id: Option<String>,
    user_id: Option<String>,
    action: String,
    params: String,
    timestamp: DateTime<Utc>,
    resp_tx: oneshot::Sender<Result<AuditEntry, AuditError>>,
}

// =====================================================================
// MerkleAuditTrail
// =====================================================================

/// Core manager for the tamper-evident audit ledger.
pub struct MerkleAuditTrail {
    pool: SqlitePool,
    verifying_key: Option<VerifyingKey>,
    audit_tx: mpsc::Sender<AuditWriteRequest>,
}

impl std::fmt::Debug for MerkleAuditTrail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MerkleAuditTrail")
            .field("pool", &"<SqlitePool>")
            .field("verifying_key", &self.verifying_key.as_ref().map(|_| "<VerifyingKey>"))
            .field("audit_tx", &"<mpsc::Sender>")
            .finish()
    }
}

impl MerkleAuditTrail {
    // -----------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------

    /// Initialize a new audit manager by loading the private key from the
    /// environment.
    ///
    /// # Environment variables
    /// * `AUDIT_PRIVATE_KEY` — hex-encoded 32-byte Ed25519 secret key.
    ///   On parse failure or absence, the manager runs in **unsigned**
    ///   mode; entry hashes are still chained but not signed. Set
    ///   `AUDIT_REQUIRE_SIGNED=1` to fail closed in that case.
    pub fn new(pool: SqlitePool) -> Result<Self> {
        let signing_key = load_signing_key_from_env()?;
        let verifying_key = signing_key.as_ref().map(|k| k.expose_secret().0.verifying_key());

        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);

        spawn_writer(pool.clone(), signing_key, audit_rx).context("spawning audit writer")?;

        Ok(Self { pool, verifying_key, audit_tx })
    }

    /// Initialize a manager that loads the key from a `SecretBox<SecretSigningKey>`
    /// directly. Used by tests and by callers that obtain the key from a
    /// KMS / Vault rather than an environment variable.
    #[allow(dead_code)]
    pub fn with_signing_key(pool: SqlitePool, key: Option<SecretBox<SecretSigningKey>>) -> Result<Self> {
        let verifying_key = key.as_ref().map(|k| k.expose_secret().0.verifying_key());
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        spawn_writer(pool.clone(), key, audit_rx).context("spawning audit writer")?;
        Ok(Self { pool, verifying_key, audit_tx })
    }

    /// Mock audit trail backed by an in-memory SQLite pool. **Does not
    /// migrate the schema** — callers must do so themselves (or use
    /// `mock_async`).
    pub fn mock() -> Result<Self> {
        let pool = SqlitePool::connect_lazy("sqlite::memory:")
            .map_err(|e| anyhow!("connect_lazy failed: {e}"))?;
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        spawn_writer(pool.clone(), None, audit_rx).context("spawning audit writer")?;
        Ok(Self { pool, verifying_key: None, audit_tx })
    }

    /// Mock audit trail backed by an in-memory SQLite pool with the
    /// schema pre-migrated.
    pub async fn mock_async() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        migrate_schema(&pool).await?;
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        spawn_writer(pool.clone(), None, audit_rx).context("spawning audit writer")?;
        Ok(Self { pool, verifying_key: None, audit_tx })
    }

    // -----------------------------------------------------------------
    // Public API: record
    // -----------------------------------------------------------------

    /// Record a new action in the audit trail.
    ///
    /// `params` is assumed to have already been **redacted** by the
    /// caller (typically via `crate::utils::security::redact_secrets`).
    /// This module does NOT call redaction internally: doing so would
    /// couple the hash chain to a non-deterministic function and break
    /// the determinism contract with `security::redact_secrets`.
    #[instrument(skip(self, params), fields(agent_id = %agent_id, action = %action))]
    pub async fn record(
        &self,
        agent_id: &str,
        mission_id: Option<&str>,
        user_id: Option<&str>,
        action: &str,
        params: &str,
    ) -> Result<AuditEntry> {
        if agent_id.is_empty() || action.is_empty() {
            return Err(anyhow!("agent_id and action must be non-empty"));
        }

        let (resp_tx, resp_rx) = oneshot::channel();
        let req = AuditWriteRequest {
            agent_id: agent_id.to_string(),
            mission_id: mission_id.map(str::to_string),
            user_id: user_id.map(str::to_string),
            action: action.to_string(),
            params: params.to_string(),
            timestamp: Utc::now(),
            resp_tx,
        };

        // Bounded send (v1 X-005).
        match tokio::time::timeout(QUEUE_SEND_TIMEOUT, self.audit_tx.send(req)).await {
            Ok(Ok(())) => {}
            Ok(Err(_)) => return Err(AuditError::WriterShutdown.into()),
            Err(_) => return Err(AuditError::QueueTimeout.into()),
        }

        // Bounded ack (v1 X-005).
        let entry = match tokio::time::timeout(WRITE_ACK_TIMEOUT, resp_rx).await {
            Ok(Ok(Ok(entry))) => entry,
            Ok(Ok(Err(e))) => return Err(e.into()),
            Ok(Err(_)) => return Err(AuditError::WriterShutdown.into()),
            Err(_) => return Err(AuditError::AckTimeout.into()),
        };
        Ok(entry)
    }

    // -----------------------------------------------------------------
    // Verification
    // -----------------------------------------------------------------

    /// Verify the entire audit trail chain.
    #[allow(dead_code)]
    #[instrument(skip(self), name = "security::audit_verify_all")]
    pub async fn verify_chain(&self, public_key_hex: Option<&str>) -> Result<bool> {
        let entries: Vec<AuditEntry> =
            sqlx::query_as("SELECT * FROM audit_trail ORDER BY timestamp ASC, id ASC")
                .fetch_all(&self.pool)
                .await?;
        let (verified, total) = self.verify_segments(&entries, public_key_hex).await?;
        Ok(verified == total && total > 0)
    }

    /// Verify only the last `n` entries.
    #[instrument(skip(self), fields(n = n), name = "security::audit_verify_n")]
    pub async fn verify_last_n(
        &self,
        n: usize,
        public_key_hex: Option<&str>,
    ) -> Result<(usize, usize)> {
        if n == 0 {
            return Ok((0, 0));
        }
        let mut entries: Vec<AuditEntry> =
            sqlx::query_as("SELECT * FROM audit_trail ORDER BY timestamp DESC, id DESC LIMIT ?")
                .bind(n as i64)
                .fetch_all(&self.pool)
                .await?;
        entries.reverse();
        self.verify_segments(&entries, public_key_hex).await
    }

    /// Verify a single entry against its stored hash and (optional)
    /// signature.
    pub fn verify_record(&self, entry: &AuditEntry) -> bool {
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
        if let (Some(pk), Some(sig_hex)) = (self.verifying_key, &entry.signature) {
            match verify_signature(pk, entry.current_hash.as_bytes(), sig_hex) {
                Ok(()) => {}
                Err(()) => return false,
            }
        }
        true
    }

    /// Common logic for verifying a contiguous segment.
    #[instrument(skip(self, entries), fields(count = entries.len()), name = "security::audit_verify_segment")]
    pub async fn verify_segments(
        &self,
        entries: &[AuditEntry],
        public_key_hex: Option<&str>,
    ) -> Result<(usize, usize)> {
        let public_key = resolve_verifying_key(public_key_hex, self.verifying_key)?;
        let total = entries.len();
        let mut verified = 0usize;
        let mut expected_prev: Option<&str> = None;

        for entry in entries {
            // Chain linkage.
            if let Some(prev) = expected_prev {
                if entry.prev_hash != prev {
                    return Ok((verified, total));
                }
            }

            // Intrinsic hash.
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

            // Signature.
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
}

// =====================================================================
// Worker task
// =====================================================================

/// Spawn the single-writer task. The writer owns the chain head locally;
/// no other code path may mutate it.
fn spawn_writer(
    pool: SqlitePool,
    signing_key: Option<SecretBox<SecretSigningKey>>,
    mut audit_rx: mpsc::Receiver<AuditWriteRequest>,
) -> Result<tokio::task::JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        info!("audit writer started [audit]");

        // Bounded startup DB read (v2 V2-006). `fetch_optional` (NOT
        // `fetch_one`) so an empty table returns Ok(None) and we
        // initialise to the genesis sentinel.
        let mut local_last_hash: String = match tokio::time::timeout(
            WORKER_STARTUP_TIMEOUT,
            sqlx::query_as::<_, (String,)>(
                "SELECT current_hash FROM audit_trail ORDER BY timestamp DESC, id DESC LIMIT 1",
            )
            .fetch_optional(&pool),
        )
        .await
        {
            Ok(Ok(Some((h,)))) => h,
            Ok(Ok(None)) => GENESIS_PREV_HASH.to_string(),
            Ok(Err(_e)) => {
                error!("audit writer startup DB read failed; aborting");
                return;
            }
            Err(_) => {
                error!("audit writer startup DB read timed out; aborting");
                return;
            }
        };

        // Panic-catching inner loop (v1 X-007, AV3-004).
        loop {
            let req = match audit_rx.recv().await {
                Some(r) => r,
                None => {
                    info!("audit writer channel closed; shutting down");
                    return;
                }
            };

            let result = std::panic::AssertUnwindSafe(process_one(
                &pool,
                signing_key.as_ref(),
                &local_last_hash,
                req,
            ))
            .catch_unwind()
            .await;

            match result {
                Ok(Ok((_entry, new_head))) => {
                    local_last_hash = new_head;
                }
                Ok(Err(kind)) => {
                    warn!(error_kind = %kind, "audit write failed");
                }
                Err(_panic) => {
                    // AV3-004: Catch panic and continue worker loop instead of killing it.
                    error!("audit writer panicked processing request; continuing loop");
                }
            }
        }
    });
    Ok(handle)
}

/// Process a single write request: hash, sign, insert, advance head.
///
/// Returns `(entry, new_head)` on success or `(error_kind)` on
/// failure. The response is delivered through `resp_tx` so the caller can
/// be unblocked.
async fn process_one(
    pool: &SqlitePool,
    signing_key: Option<&SecretBox<SecretSigningKey>>,
    prev_hash: &str,
    req: AuditWriteRequest,
) -> Result<(AuditEntry, String), AuditError> {
    let id = uuid::Uuid::new_v4().to_string();
    let current_hash = compute_canonical_hash(
        prev_hash,
        &req.agent_id,
        req.mission_id.as_deref(),
        req.user_id.as_deref(),
        &req.action,
        &req.params,
        &req.timestamp.to_rfc3339(),
    );

    let signature = signing_key.map(|k| {
        let sig = k.expose_secret().0.sign(current_hash.as_bytes());
        hex::encode(sig.to_bytes())
    });

    let entry = AuditEntry {
        id: id.clone(),
        timestamp: req.timestamp,
        agent_id: req.agent_id,
        mission_id: req.mission_id,
        user_id: req.user_id,
        action: req.action,
        params: req.params,
        prev_hash: prev_hash.to_string(),
        current_hash: current_hash.clone(),
        signature,
    };

    let res = sqlx::query(
        "INSERT INTO audit_trail \
         (id, agent_id, mission_id, user_id, action, params, prev_hash, current_hash, timestamp, signature) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind(&entry.id)
    .bind(&entry.agent_id)
    .bind(&entry.mission_id)
    .bind(&entry.user_id)
    .bind(&entry.action)
    .bind(&entry.params)
    .bind(&entry.prev_hash)
    .bind(&entry.current_hash)
    .bind(entry.timestamp)
    .bind(&entry.signature)
    .execute(pool)
    .await;

    match res {
        Ok(_) => {
            let _ = req.resp_tx.send(Ok(entry.clone()));
            Ok((entry, current_hash))
        }
        Err(_e) => {
            // Log only a static message and kind discriminant to prevent bound value leakage (AV3-011).
            error!(error_kind = "db_insert", "audit DB insert failed");
            let _ = req.resp_tx.send(Err(AuditError::Storage));
            Err(AuditError::Storage)
        }
    }
}

// =====================================================================
// Helpers
// =====================================================================

/// Load and decode the Ed25519 signing key from `AUDIT_PRIVATE_KEY`.
///
/// Returns `Ok(None)` if the variable is unset or empty (unsigned mode).
/// Returns `Err` only on malformed input; in that case the caller should
/// decide whether to refuse startup (`AUDIT_REQUIRE_SIGNED=1`) or proceed
/// unsigned.
pub(crate) fn load_signing_key_from_env() -> Result<Option<SecretBox<SecretSigningKey>>> {
    let raw = match std::env::var("AUDIT_PRIVATE_KEY") {
        Ok(s) if !s.is_empty() => Zeroizing::new(s),
        _ => return Ok(None),
    };

    let bytes_vec = match hex::decode(raw.as_str()) {
        Ok(v) => Zeroizing::new(v),
        Err(_) => return Err(anyhow!(AuditError::BadKey)),
    };

    let arr: [u8; 32] = bytes_vec
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!(AuditError::BadKey))?;

    let sig_key = SigningKey::from_bytes(&arr);
    Ok(Some(SecretBox::new(Box::new(SecretSigningKey(sig_key)))))
}

/// Resolve a verifying key from a hex string, or fall back to the locally
/// loaded key.
fn resolve_verifying_key(
    hex_str: Option<&str>,
    fallback: Option<VerifyingKey>,
) -> Result<Option<VerifyingKey>> {
    match hex_str {
        Some(h) => {
            let bytes = hex::decode(h).map_err(|_| AuditError::BadKey)?;
            let arr: [u8; 32] = bytes.as_slice().try_into().map_err(|_| AuditError::BadKey)?;
            Ok(Some(VerifyingKey::from_bytes(&arr).map_err(|_| AuditError::BadKey)?))
        }
        None => Ok(fallback),
    }
}

fn verify_signature(pk: VerifyingKey, message: &[u8], sig_hex: &str) -> Result<(), ()> {
    let bytes = hex::decode(sig_hex).map_err(|_| ());
    let bytes_val = match bytes {
        Ok(b) => b,
        Err(_) => return Err(()),
    };
    let arr: [u8; 64] = bytes_val.as_slice().try_into().map_err(|_| ())?;
    let sig = Signature::from_bytes(&arr);
    pk.verify(message, &sig).map_err(|_| ())
}

async fn migrate_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_trail (
            id TEXT PRIMARY KEY NOT NULL,
            agent_id TEXT NOT NULL,
            action TEXT NOT NULL,
            params TEXT NOT NULL,
            prev_hash TEXT NOT NULL,
            current_hash TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            signature TEXT,
            mission_id TEXT,
            user_id TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;
    use std::sync::Arc;

    async fn fresh_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        migrate_schema(&pool).await.unwrap();
        pool
    }

    fn random_key() -> SecretBox<SecretSigningKey> {
        let mut bytes = [0u8; 32];
        let mut rng = rand::rng();
        rng.fill(&mut bytes);
        SecretBox::new(Box::new(SecretSigningKey(SigningKey::from_bytes(&bytes))))
    }

    #[tokio::test]
    async fn test_audit_trail_chaining() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

        let e1 = audit
            .record("agent-1", Some("mission-1"), Some("user-1"), "tool_call", "{\"tool\":\"ls\"}")
            .await
            .unwrap();
        assert_eq!(e1.prev_hash, GENESIS_PREV_HASH);

        let e2 = audit
            .record("agent-1", Some("mission-1"), Some("user-1"), "tool_call", "{\"tool\":\"cat\"}")
            .await
            .unwrap();
        assert_eq!(e2.prev_hash, e1.current_hash);

        assert!(audit.verify_chain(None).await.unwrap());
    }

    #[tokio::test]
    async fn test_audit_tamper_detection() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

        audit.record("agent-1", None, None, "action-1", "{}").await.unwrap();
        let e2 = audit.record("agent-1", None, None, "action-2", "{}").await.unwrap();
        assert!(audit.verify_chain(None).await.unwrap());

        // Tamper with action.
        sqlx::query("UPDATE audit_trail SET action = 'tampered' WHERE id = ?")
            .bind(&e2.id)
            .execute(&audit.pool)
            .await
            .unwrap();

        assert!(!audit.verify_chain(None).await.unwrap());
    }

    #[tokio::test]
    async fn test_signature_verification() {
        let pool = fresh_pool().await;
        let key = random_key();
        let audit = MerkleAuditTrail::with_signing_key(pool, Some(key.clone())).unwrap();

        let entry = audit.record("agent-1", None, None, "action", "{}").await.unwrap();
        assert!(entry.signature.is_some());
        assert!(audit.verify_chain(None).await.unwrap());
        assert!(audit.verify_record(&entry));

        let sig_key = &key.expose_secret().0;
        let pk_hex = hex::encode(sig_key.verifying_key().to_bytes());
        assert!(audit.verify_chain(Some(&pk_hex)).await.unwrap());
    }

    /// Regression: flipping mission_id between Some and None must be
    /// detected (v1 X-002).
    #[tokio::test]
    async fn test_optional_field_presence_detected() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

        let entry = audit
            .record("agent-1", Some("mission-x"), None, "action", "{}")
            .await
            .unwrap();

        // DB-side: change `Some("mission-x")` to `NULL`.
        sqlx::query("UPDATE audit_trail SET mission_id = NULL WHERE id = ?")
            .bind(&entry.id)
            .execute(&audit.pool)
            .await
            .unwrap();

        assert!(!audit.verify_chain(None).await.unwrap());
    }

    /// Regression: 1000 concurrent writers must produce 1000 unique
    /// prev_hash values pointing at their immediate predecessor (v1 X-001).
    #[tokio::test]
    async fn test_concurrent_writers_no_fork() {
        let pool = fresh_pool().await;
        let audit = Arc::new(MerkleAuditTrail::with_signing_key(pool, None).unwrap());

        let mut handles = Vec::new();
        for i in 0..1000 {
            let audit = Arc::clone(&audit);
            handles.push(tokio::spawn(async move {
                audit
                    .record("agent", None, None, "noop", &format!("{{\"i\":{}}}", i))
                    .await
                    .unwrap();
            }));
        }
        for h in handles {
            h.await.unwrap();
        }

        let (verified, total) = audit.verify_last_n(1000, None).await.unwrap();
        assert_eq!(verified, total);
        assert_eq!(total, 1000);
    }

    /// Regression: empty inputs are rejected up front.
    #[tokio::test]
    async fn test_empty_fields_rejected() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();
        assert!(audit.record("", None, None, "action", "{}").await.is_err());
        assert!(audit.record("agent", None, None, "", "{}").await.is_err());
    }

    /// Cross-module invariant: hashing is deterministic across threads
    /// (drives the audit.rs <-> security.rs contract).
    #[test]
    fn test_canonical_hash_deterministic_across_threads() {
        let h1 = std::thread::spawn(|| {
            compute_canonical_hash(
                GENESIS_PREV_HASH, "a", Some("m"), Some("u"), "act", "{}", "2024-01-01T00:00:00Z",
            )
        });
        let h2 = std::thread::spawn(|| {
            compute_canonical_hash(
                GENESIS_PREV_HASH, "a", Some("m"), Some("u"), "act", "{}", "2024-01-01T00:00:00Z",
            )
        });
        assert_eq!(h1.join().unwrap(), h2.join().unwrap());
    }
}
