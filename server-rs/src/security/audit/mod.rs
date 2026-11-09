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

#![forbid(unsafe_code)]

pub mod hasher;
pub mod types;
pub mod verifier;
pub mod writer;

#[cfg(test)]
mod tests;

use anyhow::{anyhow, Context, Result};
use ed25519_dalek::VerifyingKey;
use secrecy::{ExposeSecret, SecretBox};
use sqlx::SqlitePool;
use tokio::sync::{mpsc, oneshot};
use tracing::instrument;

pub use hasher::compute_canonical_hash;
pub use types::*;
pub use writer::load_signing_key_from_env;

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
    pub fn new(pool: SqlitePool) -> Result<Self> {
        let signing_key = writer::load_signing_key_from_env()?;
        let verifying_key = signing_key
            .as_ref()
            .map(|k| k.expose_secret().0.verifying_key());
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        writer::spawn_writer(pool.clone(), signing_key, audit_rx)
            .context("spawning audit writer")?;
        Ok(Self {
            pool,
            verifying_key,
            audit_tx,
        })
    }

    pub fn with_signing_key(
        pool: SqlitePool,
        key: Option<SecretBox<SecretSigningKey>>,
    ) -> Result<Self> {
        let verifying_key = key.as_ref().map(|k| k.expose_secret().0.verifying_key());
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        writer::spawn_writer(pool.clone(), key, audit_rx).context("spawning audit writer")?;
        Ok(Self {
            pool,
            verifying_key,
            audit_tx,
        })
    }

    pub fn mock() -> Result<Self> {
        let pool = SqlitePool::connect_lazy("sqlite::memory:")
            .map_err(|e| anyhow!("connect_lazy failed: {e}"))?;
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        writer::spawn_writer(pool.clone(), None, audit_rx).context("spawning audit writer")?;
        Ok(Self {
            pool,
            verifying_key: None,
            audit_tx,
        })
    }

    pub fn is_alive(&self) -> bool {
        !self.audit_tx.is_closed()
    }

    pub async fn mock_async() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        writer::migrate_schema(&pool).await?;
        let (audit_tx, audit_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        writer::spawn_writer(pool.clone(), None, audit_rx).context("spawning audit writer")?;
        Ok(Self {
            pool,
            verifying_key: None,
            audit_tx,
        })
    }

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

        match tokio::time::timeout(QUEUE_SEND_TIMEOUT, self.audit_tx.send(req)).await {
            Ok(Ok(())) => {}
            Ok(Err(_)) => return Err(AuditError::WriterShutdown.into()),
            Err(_) => return Err(AuditError::QueueTimeout.into()),
        }

        let entry = match tokio::time::timeout(WRITE_ACK_TIMEOUT, resp_rx).await {
            Ok(Ok(Ok(entry))) => entry,
            Ok(Ok(Err(e))) => return Err(e.into()),
            Ok(Err(_)) => return Err(AuditError::WriterShutdown.into()),
            Err(_) => return Err(AuditError::AckTimeout.into()),
        };
        Ok(entry)
    }

    #[instrument(skip(self), name = "security::audit_verify_all")]
    pub async fn verify_chain(&self, public_key_hex: Option<&str>) -> Result<bool> {
        verifier::verify_chain(&self.pool, self.verifying_key, public_key_hex).await
    }

    #[instrument(skip(self), fields(n = n), name = "security::audit_verify_n")]
    pub async fn verify_last_n(
        &self,
        n: usize,
        public_key_hex: Option<&str>,
    ) -> Result<(usize, usize)> {
        verifier::verify_last_n(&self.pool, self.verifying_key, n, public_key_hex).await
    }

    pub fn verify_record(&self, entry: &AuditEntry) -> bool {
        verifier::verify_record(self.verifying_key, entry)
    }

    #[instrument(skip(self, entries), fields(count = entries.len()), name = "security::audit_verify_segment")]
    pub async fn verify_segments(
        &self,
        entries: &[AuditEntry],
        public_key_hex: Option<&str>,
        initial_expected_prev: Option<&str>,
    ) -> Result<(usize, usize)> {
        verifier::verify_segments(
            entries,
            self.verifying_key,
            public_key_hex,
            initial_expected_prev,
        )
        .await
    }
}
