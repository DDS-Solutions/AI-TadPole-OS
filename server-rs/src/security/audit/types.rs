//! @docs SECURITY:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / audit / types
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use std::time::Duration;
use tokio::sync::oneshot;

pub const QUEUE_SEND_TIMEOUT: Duration = Duration::from_secs(5);
pub const WRITE_ACK_TIMEOUT: Duration = Duration::from_secs(5);
pub const WORKER_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);
pub const GENESIS_PREV_HASH: &str =
    "GENESIS-PREV-HASH-AUDIT-V2-0000000000000000000000000000000000000000000000";
pub const HASH_DOMAIN_SEPARATOR: &[u8] = b"AUDIT-V2";
pub const WRITER_CHANNEL_CAPACITY: usize = 1024;
pub const MAX_AGENT_ID_BYTES: usize = 256;
pub const MAX_ACTION_BYTES: usize = 256;
pub const MAX_PARAMS_BYTES: usize = 1024 * 1024; // 1 MB
pub const MAX_OPTIONAL_ID_BYTES: usize = 256;

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

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::FromRow, specta::Type,
)]
pub struct AuditEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(type = Option<f64>)]
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

pub(crate) struct AuditWriteRequest {
    pub(crate) agent_id: String,
    pub(crate) mission_id: Option<String>,
    pub(crate) user_id: Option<String>,
    pub(crate) action: String,
    pub(crate) params: String,
    pub(crate) resp_tx: oneshot::Sender<Result<AuditEntry, AuditError>>,
}
