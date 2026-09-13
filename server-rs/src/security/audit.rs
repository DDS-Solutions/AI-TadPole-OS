//! @docs SECURITY:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / audit
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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
pub const GENESIS_PREV_HASH: &str =
    "GENESIS-PREV-HASH-AUDIT-V2-0000000000000000000000000000000000000000000000";

/// Domain separator prepended to every canonical hash.
const HASH_DOMAIN_SEPARATOR: &[u8] = b"AUDIT-V2";

/// Capacity of the bounded channel between callers and the writer.
const WRITER_CHANNEL_CAPACITY: usize = 1024;

/// Maximum allowable bytes for string fields to prevent unbounded memory/hash DoS.
pub const MAX_AGENT_ID_BYTES: usize = 256;
pub const MAX_ACTION_BYTES: usize = 256;
pub const MAX_PARAMS_BYTES: usize = 1024 * 1024; // 1 MB
pub const MAX_OPTIONAL_ID_BYTES: usize = 256;

// =====================================================================
// SecretSigningKey (Zeroize Wrapper)
// =====================================================================

/// Safe wrapper for `SigningKey` ensuring memory is zeroized on drop.
#[derive(Clone)]
pub struct SecretSigningKey(pub SigningKey);

impl zeroize::Zeroize for SecretSigningKey {
    fn zeroize(&mut self) {
        self.0 = SigningKey::from_bytes(&[0u8; 32]);
    }
}

impl secrecy::CloneableSecret for SecretSigningKey {}

impl std::ops::Deref for SecretSigningKey {
    type Target = SigningKey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// =====================================================================
// AuditEntry
// =====================================================================

/// A single entry in the tamper-evident audit ledger.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AuditEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<i64>,
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
// Error type
// =====================================================================

/// Discriminant for failures surfaced from the writer task.
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
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
    #[error("audit input exceeds maximum allowed size bounds")]
    InputTooLarge,
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

// =====================================================================
// Internal channel envelope
// =====================================================================

struct AuditWriteRequest {
    agent_id: String,
    mission_id: Option<String>,
    user_id: Option<String>,
    action: String,
    params: String,
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
            .field(
                "verifying_key",
                &self.verifying_key.as_ref().map(|_| "<VerifyingKey>"),
            )
            .field("audit_tx", &"<mpsc::Sender>")
            .finish()
    }
}

impl MerkleAuditTrail {
    // -----------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------

    /// Initialize a new audit manager by loading configuration and key
    /// from the environment.
    ///
    /// # Environment variables
    /// * `AUDIT_PRIVATE_KEY` — hex-encoded 32-byte Ed25519 secret key.
    /// * `AUDIT_REQUIRE_SIGNED` — set to `1` or `true` to fail closed if
    ///   the private key is missing or invalid.
    pub fn new(pool: SqlitePool) -> Result<Self> {
        let signing_key = load_signing_key_from_env()?;
        let verifying_key = signing_key
            .as_ref()
            .map(|k| k.expose_secret().0.verifying_key());

        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);

        spawn_writer(pool.clone(), signing_key, audit_rx).context("spawning audit writer")?;

        Ok(Self {
            pool,
            verifying_key,
            audit_tx,
        })
    }

    /// Initialize a manager with an explicit key. Used by tests and KMS integrations.
    pub fn with_signing_key(
        pool: SqlitePool,
        key: Option<SecretBox<SecretSigningKey>>,
    ) -> Result<Self> {
        let verifying_key = key.as_ref().map(|k| k.expose_secret().0.verifying_key());
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        spawn_writer(pool.clone(), key, audit_rx).context("spawning audit writer")?;
        Ok(Self {
            pool,
            verifying_key,
            audit_tx,
        })
    }

    /// Mock audit trail backed by an in-memory SQLite pool.
    pub fn mock() -> Result<Self> {
        let pool = SqlitePool::connect_lazy("sqlite::memory:")
            .map_err(|e| anyhow!("connect_lazy failed: {e}"))?;
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        spawn_writer(pool.clone(), None, audit_rx).context("spawning audit writer")?;
        Ok(Self {
            pool,
            verifying_key: None,
            audit_tx,
        })
    }

    /// Returns `true` if the writer channel is active and receptive.
    pub fn is_alive(&self) -> bool {
        !self.audit_tx.is_closed()
    }

    /// Mock audit trail backed by an in-memory SQLite pool with schema initialized.
    pub async fn mock_async() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        migrate_schema(&pool).await?;
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        spawn_writer(pool.clone(), None, audit_rx).context("spawning audit writer")?;
        Ok(Self {
            pool,
            verifying_key: None,
            audit_tx,
        })
    }

    // -----------------------------------------------------------------
    // Public API: record
    // -----------------------------------------------------------------

    /// Record a new action in the audit trail.
    ///
    /// `params` is assumed to have already been redacted by the caller.
    /// Timestamp and sequence assignment are serialized on the writer task.
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
        if agent_id.len() > MAX_AGENT_ID_BYTES
            || action.len() > MAX_ACTION_BYTES
            || params.len() > MAX_PARAMS_BYTES
        {
            return Err(AuditError::InputTooLarge.into());
        }
        if let Some(m) = mission_id {
            if m.len() > MAX_OPTIONAL_ID_BYTES {
                return Err(AuditError::InputTooLarge.into());
            }
        }
        if let Some(u) = user_id {
            if u.len() > MAX_OPTIONAL_ID_BYTES {
                return Err(AuditError::InputTooLarge.into());
            }
        }

        let (resp_tx, resp_rx) = oneshot::channel();
        let req = AuditWriteRequest {
            agent_id: agent_id.to_string(),
            mission_id: mission_id.map(str::to_string),
            user_id: user_id.map(str::to_string),
            action: action.to_string(),
            params: params.to_string(),
            resp_tx,
        };

        // Bounded send.
        match tokio::time::timeout(QUEUE_SEND_TIMEOUT, self.audit_tx.send(req)).await {
            Ok(Ok(())) => {}
            Ok(Err(_)) => return Err(AuditError::WriterShutdown.into()),
            Err(_) => return Err(AuditError::QueueTimeout.into()),
        }

        // Bounded ack.
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

    /// Verify the entire audit trail chain from genesis.
    #[instrument(skip(self), name = "security::audit_verify_all")]
    pub async fn verify_chain(&self, public_key_hex: Option<&str>) -> Result<bool> {
        let entries: Vec<AuditEntry> = sqlx::query_as("SELECT * FROM audit_trail ORDER BY seq ASC")
            .fetch_all(&self.pool)
            .await?;
        if entries.is_empty() {
            return Ok(true);
        }

        // Enforce Genesis Anchor on first entry.
        if entries[0].prev_hash != GENESIS_PREV_HASH {
            return Ok(false);
        }

        let (verified, total) = self
            .verify_segments(&entries, public_key_hex, Some(GENESIS_PREV_HASH))
            .await?;
        Ok(verified == total && total > 0)
    }

    /// Verify the most recent `n` entries.
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
            sqlx::query_as("SELECT * FROM audit_trail ORDER BY seq DESC LIMIT ?")
                .bind(n as i64)
                .fetch_all(&self.pool)
                .await?;
        entries.reverse();

        let initial_expected_prev = if entries.first().and_then(|e| e.seq) == Some(1) {
            Some(GENESIS_PREV_HASH)
        } else {
            None
        };

        self.verify_segments(&entries, public_key_hex, initial_expected_prev)
            .await
    }

    /// Verify a single entry against its stored hash and signature.
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
            if verify_signature(pk, entry.current_hash.as_bytes(), sig_hex).is_err() {
                return false;
            }
        }
        true
    }

    /// Common logic for verifying a contiguous segment with optional initial prev anchor.
    #[instrument(skip(self, entries), fields(count = entries.len()), name = "security::audit_verify_segment")]
    pub async fn verify_segments(
        &self,
        entries: &[AuditEntry],
        public_key_hex: Option<&str>,
        initial_expected_prev: Option<&str>,
    ) -> Result<(usize, usize)> {
        let public_key = resolve_verifying_key(public_key_hex, self.verifying_key)?;
        let total = entries.len();
        let mut verified = 0usize;
        let mut expected_prev: Option<&str> = initial_expected_prev;

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

/// Spawn the single-writer task owning the chain head and sequence authority.
fn spawn_writer(
    pool: SqlitePool,
    signing_key: Option<SecretBox<SecretSigningKey>>,
    mut audit_rx: mpsc::Receiver<AuditWriteRequest>,
) -> Result<tokio::task::JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        info!("audit writer started [audit]");

        // Ensure schema exists on in-memory / unmigrated pools to prevent immediate abort in mock environments
        let _ = migrate_schema(&pool).await;

        // Bounded startup DB read with 3 retries.
        let mut local_last_hash: String = GENESIS_PREV_HASH.to_string();
        let mut startup_ok = false;

        for attempt in 1..=3 {
            match tokio::time::timeout(
                WORKER_STARTUP_TIMEOUT,
                sqlx::query_as::<_, (String,)>(
                    "SELECT current_hash FROM audit_trail ORDER BY seq DESC LIMIT 1",
                )
                .fetch_optional(&pool),
            )
            .await
            {
                Ok(Ok(Some((h,)))) => {
                    local_last_hash = h;
                    startup_ok = true;
                    break;
                }
                Ok(Ok(None)) => {
                    local_last_hash = GENESIS_PREV_HASH.to_string();
                    startup_ok = true;
                    break;
                }
                Ok(Err(e)) => {
                    warn!(
                        attempt = attempt,
                        error = %e,
                        "audit writer startup DB read failed; retrying"
                    );
                }
                Err(_) => {
                    warn!(
                        attempt = attempt,
                        "audit writer startup DB read timed out; retrying"
                    );
                }
            }
            tokio::time::sleep(Duration::from_millis(50 * attempt as u64)).await;
        }

        if !startup_ok {
            error!("audit writer startup DB read failed permanently after 3 attempts; aborting");
            return;
        }

        // Panic-catching inner loop.
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
                    error!("audit writer panicked processing request; recovering head from DB");
                    if let Ok(Ok(Some((h,)))) = tokio::time::timeout(
                        Duration::from_secs(3),
                        sqlx::query_as::<_, (String,)>(
                            "SELECT current_hash FROM audit_trail ORDER BY seq DESC LIMIT 1",
                        )
                        .fetch_optional(&pool),
                    )
                    .await
                    {
                        local_last_hash = h;
                    }
                }
            }
        }
    });
    Ok(handle)
}

/// Process a single write request: assign timestamp, hash, sign, insert, advance head.
async fn process_one(
    pool: &SqlitePool,
    signing_key: Option<&SecretBox<SecretSigningKey>>,
    prev_hash: &str,
    req: AuditWriteRequest,
) -> Result<(AuditEntry, String), AuditError> {
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = Utc::now();
    let timestamp_str = timestamp.to_rfc3339();

    let current_hash = compute_canonical_hash(
        prev_hash,
        &req.agent_id,
        req.mission_id.as_deref(),
        req.user_id.as_deref(),
        &req.action,
        &req.params,
        &timestamp_str,
    );

    let signature = signing_key.map(|k| {
        let sig_key = &k.expose_secret().0;
        let sig = sig_key.sign(current_hash.as_bytes());
        hex::encode(sig.to_bytes())
    });

    let mut entry = AuditEntry {
        seq: None,
        id: id.clone(),
        timestamp,
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
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
         RETURNING seq",
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
    .fetch_optional(pool)
    .await;

    match res {
        Ok(Some(row)) => {
            use sqlx::Row;
            entry.seq = row.try_get::<i64, _>("seq").ok();
            let _ = req.resp_tx.send(Ok(entry.clone()));
            Ok((entry, current_hash))
        }
        Ok(None) => {
            let _ = req.resp_tx.send(Ok(entry.clone()));
            Ok((entry, current_hash))
        }
        Err(_e) => {
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
/// Respects `AUDIT_REQUIRE_SIGNED=1` to fail closed on missing/malformed keys.
pub(crate) fn load_signing_key_from_env() -> Result<Option<SecretBox<SecretSigningKey>>> {
    let require_signed = match std::env::var("AUDIT_REQUIRE_SIGNED") {
        Ok(v) => v == "1" || v.eq_ignore_ascii_case("true"),
        Err(_) => false,
    };
    let raw = std::env::var("AUDIT_PRIVATE_KEY").ok();
    load_signing_key_inner(raw.as_deref(), require_signed)
}

/// Pure helper for parsing signing keys against fail-closed rules without env mutation races.
pub(crate) fn load_signing_key_inner(
    raw_key: Option<&str>,
    require_signed: bool,
) -> Result<Option<SecretBox<SecretSigningKey>>> {
    let raw = match raw_key {
        Some(s) if !s.trim().is_empty() => Zeroizing::new(s.trim().to_string()),
        _ => {
            if require_signed {
                return Err(anyhow!(AuditError::BadKey)
                    .context("AUDIT_REQUIRE_SIGNED=1 but AUDIT_PRIVATE_KEY is unset or empty"));
            }
            return Ok(None);
        }
    };

    let bytes_vec = match hex::decode(raw.as_str()) {
        Ok(v) => Zeroizing::new(v),
        Err(e) => {
            if require_signed {
                return Err(anyhow!(AuditError::BadKey)
                    .context(format!("Failed to hex-decode AUDIT_PRIVATE_KEY: {e}")));
            }
            warn!("AUDIT_PRIVATE_KEY hex decode failed ({e}); falling back to unsigned audit mode");
            return Ok(None);
        }
    };

    let arr: [u8; 32] = match bytes_vec.as_slice().try_into() {
        Ok(a) => a,
        Err(_) => {
            if require_signed {
                return Err(anyhow!(AuditError::BadKey)
                    .context("AUDIT_PRIVATE_KEY must be exactly 32 bytes (64 hex characters)"));
            }
            warn!("AUDIT_PRIVATE_KEY is not 32 bytes; falling back to unsigned audit mode");
            return Ok(None);
        }
    };

    let sig_key = SigningKey::from_bytes(&arr);
    Ok(Some(SecretBox::new(Box::new(SecretSigningKey(sig_key)))))
}

/// Resolve a verifying key from a hex string, or fall back to the locally loaded key.
fn resolve_verifying_key(
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

fn verify_signature(pk: VerifyingKey, message: &[u8], sig_hex: &str) -> Result<(), AuditError> {
    let bytes = hex::decode(sig_hex).map_err(|_| AuditError::BadKey)?;
    let arr: [u8; 64] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| AuditError::BadKey)?;
    let sig = Signature::from_bytes(&arr);
    pk.verify(message, &sig).map_err(|_| AuditError::BadKey)
}

async fn migrate_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_trail (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            id TEXT NOT NULL UNIQUE,
            agent_id TEXT NOT NULL,
            action TEXT NOT NULL,
            params TEXT NOT NULL,
            prev_hash TEXT NOT NULL,
            current_hash TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            signature TEXT,
            mission_id TEXT,
            user_id TEXT,
            trace_id TEXT,
            http_status INTEGER,
            cost_micros INTEGER NOT NULL DEFAULT 0,
            content TEXT,
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
            .record(
                "agent-1",
                Some("mission-1"),
                Some("user-1"),
                "tool_call",
                "{\"tool\":\"ls\"}",
            )
            .await
            .unwrap();
        assert_eq!(e1.prev_hash, GENESIS_PREV_HASH);
        assert_eq!(e1.seq, Some(1));

        let e2 = audit
            .record(
                "agent-1",
                Some("mission-1"),
                Some("user-1"),
                "tool_call",
                "{\"tool\":\"cat\"}",
            )
            .await
            .unwrap();
        assert_eq!(e2.prev_hash, e1.current_hash);
        assert_eq!(e2.seq, Some(2));

        assert!(audit.verify_chain(None).await.unwrap());
    }

    #[tokio::test]
    async fn test_audit_tamper_detection() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

        audit
            .record("agent-1", None, None, "action-1", "{}")
            .await
            .unwrap();
        let e2 = audit
            .record("agent-1", None, None, "action-2", "{}")
            .await
            .unwrap();
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

        let entry = audit
            .record("agent-1", None, None, "action", "{}")
            .await
            .unwrap();
        assert!(entry.signature.is_some());
        assert!(audit.verify_chain(None).await.unwrap());
        assert!(audit.verify_record(&entry));

        let sig_key = &key.expose_secret().0;
        let pk_hex = hex::encode(sig_key.verifying_key().to_bytes());
        assert!(audit.verify_chain(Some(&pk_hex)).await.unwrap());
    }

    #[tokio::test]
    async fn test_optional_field_presence_detected() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

        let entry = audit
            .record("agent-1", Some("mission-x"), None, "action", "{}")
            .await
            .unwrap();

        // Tamper optional mission_id
        sqlx::query("UPDATE audit_trail SET mission_id = NULL WHERE id = ?")
            .bind(&entry.id)
            .execute(&audit.pool)
            .await
            .unwrap();

        assert!(!audit.verify_chain(None).await.unwrap());
    }

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
        assert!(audit.verify_chain(None).await.unwrap());
    }

    #[tokio::test]
    async fn test_empty_and_oversized_fields_rejected() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();
        assert!(audit.record("", None, None, "action", "{}").await.is_err());
        assert!(audit.record("agent", None, None, "", "{}").await.is_err());

        // Oversized params
        let huge_params = "x".repeat(MAX_PARAMS_BYTES + 1);
        assert!(audit
            .record("agent", None, None, "action", &huge_params)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_prefix_truncation_detected() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

        let e1 = audit
            .record("agent-1", None, None, "action-1", "{}")
            .await
            .unwrap();
        let _e2 = audit
            .record("agent-1", None, None, "action-2", "{}")
            .await
            .unwrap();
        let _e3 = audit
            .record("agent-1", None, None, "action-3", "{}")
            .await
            .unwrap();

        assert!(audit.verify_chain(None).await.unwrap());

        // Delete genesis entry (prefix truncation)
        sqlx::query("DELETE FROM audit_trail WHERE id = ?")
            .bind(&e1.id)
            .execute(&audit.pool)
            .await
            .unwrap();

        // verify_chain must detect missing genesis anchor
        assert!(!audit.verify_chain(None).await.unwrap());
    }

    #[test]
    fn test_canonical_hash_deterministic_across_threads() {
        let h1 = std::thread::spawn(|| {
            compute_canonical_hash(
                GENESIS_PREV_HASH,
                "a",
                Some("m"),
                Some("u"),
                "act",
                "{}",
                "2024-01-01T00:00:00Z",
            )
        });
        let h2 = std::thread::spawn(|| {
            compute_canonical_hash(
                GENESIS_PREV_HASH,
                "a",
                Some("m"),
                Some("u"),
                "act",
                "{}",
                "2024-01-01T00:00:00Z",
            )
        });
        assert_eq!(h1.join().unwrap(), h2.join().unwrap());
    }

    #[tokio::test]
    async fn test_audit_require_signed_flag_matrix() {
        // Scenario 1: Key unset, REQUIRE_SIGNED=0 -> unsigned mode
        let res1 = load_signing_key_inner(None, false).unwrap();
        assert!(res1.is_none());

        // Scenario 2: Key unset, REQUIRE_SIGNED=1 -> fail closed
        let res2 = load_signing_key_inner(None, true);
        assert!(res2.is_err());

        // Scenario 3: Key malformed, REQUIRE_SIGNED=0 -> unsigned mode
        let res3 = load_signing_key_inner(Some("not-valid-hex"), false).unwrap();
        assert!(res3.is_none());

        // Scenario 4: Key malformed, REQUIRE_SIGNED=1 -> fail closed
        let res4 = load_signing_key_inner(Some("not-valid-hex"), true);
        assert!(res4.is_err());

        // Scenario 5: Valid key
        let valid_key_hex = hex::encode([7u8; 32]);
        let res5 = load_signing_key_inner(Some(&valid_key_hex), true).unwrap();
        assert!(res5.is_some());
    }

    #[tokio::test]
    async fn test_is_alive_probe() {
        let pool = fresh_pool().await;
        let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();
        assert!(audit.is_alive());
    }
}
