//! Tamper-Evident Audit Ledger - Non-Repudiative Logging
//!
//! Orchestrates the high-speed recording of agent actions into a cryptographically
//! linked hash-chain (SHA-256) with optional Ed25519 digital signatures.
//!
//! @docs ARCHITECTURE:SecurityAudit
//!
//! ### AI Assist Note
//! **Tamper-Evident Audit Ledger**: Orchestrates the non-repudiative
//! recording of agent actions into a cryptographically linked
//! **SHA-256 Hash Chain**. Features optional **Ed25519 Digital Signatures**
//! for high-assurance mission trails. Any modification to intermediate
//! rows invalidates the `current_hash` and `signature` of all subsequent
//! entries, enabling **Continuous Integrity Verification** (AUDIT-01).
//! Note: If verification fails during boot, the engine should halt
//! (panic) to prevent state corruption.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Hash chain breakage (`prev_hash` mismatch),
//!   invalid digital signatures (key mismatch), or audit logging
//!   bottlenecks under high-throughput tool execution.
//! - **Trace Scope**: `server-rs::security::audit`

use anyhow::Result;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::sync::Arc;

/// Represents a single entry in the tamper-evident audit ledger.
///
/// Audit entries are linked together using a SHA-256 hash chain, where each
/// entry contains the hash of the preceding one. This ensures that any
/// retrospective tampering with the database will break the chain's validity.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AuditEntry {
    /// Unique identifier for the audit record.
    pub id: String,
    /// UTC timestamp of when the action was recorded.
    pub timestamp: DateTime<Utc>,
    /// Identifier of the agent performing the action.
    pub agent_id: String,
    /// Optional mission identifier associated with the action.
    pub mission_id: Option<String>,
    /// Optional user identifier (e.g., if triggered by a human via UI).
    pub user_id: Option<String>,
    /// The action name (e.g., "tool_call", "budget_auth").
    pub action: String,
    /// JSON string of parameters or arguments used in the action.
    pub params: String,
    /// The SHA-256 hash of the previous entry in the chain.
    pub prev_hash: String,
    /// The SHA-256 hash of the current entry (prev_hash + entry data).
    pub current_hash: String,
    /// Optional Ed25519 digital signature of the `current_hash`.
    pub signature: Option<String>,
}

/// The core manager for the tamper-evident audit ledger.
///
/// Uses a combination of SHA-256 hash chaining and ED25519 digital signatures
/// to ensure non-repudiation and data integrity of the agent action log.
pub struct MerkleAuditTrail {
    /// The database connection pool for persisting audit entries.
    pool: SqlitePool,
    /// Optional signing key used for digital signatures.
    signing_key: Option<SigningKey>,
    /// Pre-parsed verifying key for efficient signature checking.
    verifying_key: Option<VerifyingKey>,
    /// In-memory cache of the last hash to avoid redundant DB reads during tool loops.
    last_hash: Arc<RwLock<Option<String>>>,
}

impl MerkleAuditTrail {
    /// Initializes a new audit manager by loading the private key from the environment.
    ///
    /// # Environment Variables
    /// * `AUDIT_PRIVATE_KEY`: Hex-encoded 32-byte Ed25519 secret key.
    pub fn new(pool: SqlitePool) -> Self {
        let signing_key = std::env::var("AUDIT_PRIVATE_KEY").ok().and_then(|key| {
            let bytes = hex::decode(key).ok()?;
            let bytes: [u8; 32] = bytes.try_into().ok()?;
            Some(SigningKey::from_bytes(&bytes))
        });

        let verifying_key = signing_key.as_ref().map(|k| k.verifying_key());

        Self {
            pool,
            signing_key,
            verifying_key,
            last_hash: Arc::new(RwLock::new(None)),
        }
    }

    /// Creates a mock audit trail for testing and development.
    pub fn mock() -> Self {
        let pool = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        Self {
            pool,
            signing_key: None,
            verifying_key: None,
            last_hash: Arc::new(RwLock::new(None)),
        }
    }

    /// Records a new action in the audit trail.
    ///
    /// This method is high-performance and uses an in-memory lock for the hash chain
    /// to ensure linearity under high concurrency.
    #[tracing::instrument(skip(self, params), fields(agent_id = %agent_id, action = %action), name = "security::audit_record")]
    pub async fn record(
        &self,
        agent_id: &str,
        mission_id: Option<&str>,
        user_id: Option<&str>,
        action: &str,
        params: &str,
    ) -> Result<AuditEntry> {
        let last_hash = self.get_last_hash().await?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let redacted_params = crate::utils::security::redact_secrets(params);

        let mut hasher = Sha256::new();
        hasher.update(&last_hash);
        hasher.update(agent_id);
        if let Some(m) = mission_id {
            hasher.update(m);
        }
        if let Some(u) = user_id {
            hasher.update(u);
        }
        hasher.update(action);
        hasher.update(&redacted_params);
        hasher.update(now.to_rfc3339());
        let current_hash = hex::encode(hasher.finalize());

        let signature = self.signing_key.as_ref().map(|key| {
            let signature = key.sign(current_hash.as_bytes());
            hex::encode(signature.to_bytes())
        });

        sqlx::query(
            "INSERT INTO audit_trail (id, agent_id, mission_id, user_id, action, params, prev_hash, current_hash, timestamp, signature) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        )
        .bind(&id)
        .bind(agent_id)
        .bind(mission_id)
        .bind(user_id)
        .bind(action)
        .bind(&redacted_params)
        .bind(&last_hash)
        .bind(&current_hash)
        .bind(now)
        .bind(&signature)
        .execute(&self.pool)
        .await?;

        // Update In-Memory Chain Head
        *self.last_hash.write() = Some(current_hash.clone());

        Ok(AuditEntry {
            id,
            timestamp: now,
            agent_id: agent_id.to_string(),
            mission_id: mission_id.map(|m| m.to_string()),
            user_id: user_id.map(|u| u.to_string()),
            action: action.to_string(),
            params: redacted_params,
            prev_hash: last_hash,
            current_hash,
            signature,
        })
    }

    /// Fetches the head of the hash chain.
    async fn get_last_hash(&self) -> Result<String> {
        // PERF: Read-only lock for cache access
        if let Some(h) = self.last_hash.read().as_ref() {
            return Ok(h.clone());
        }

        let row: Option<(String,)> =
            sqlx::query_as("SELECT current_hash FROM audit_trail ORDER BY timestamp DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;

        let hash = row.map(|r| r.0).unwrap_or_else(|| "0".repeat(64));
        *self.last_hash.write() = Some(hash.clone());
        Ok(hash)
    }

    /// Verifies the entire audit trail chain from genesis to head.
    /// This is an exhaustive O(N) operation. For high-speed checks, use `verify_last_n`.
    #[allow(dead_code)]
    #[tracing::instrument(skip(self), name = "security::audit_verify_all")]
    pub async fn verify_chain(&self, public_key_hex: Option<&str>) -> Result<bool> {
        let entries: Vec<AuditEntry> =
            sqlx::query_as("SELECT * FROM audit_trail ORDER BY timestamp ASC")
                .fetch_all(&self.pool)
                .await?;

        let (verified, total) = self.verify_segments(&entries, public_key_hex).await?;
        Ok(verified == total)
    }

    /// Verifies only the last `n` entries (Partial Scan).
    /// Highly useful for high-speed periodic health checks.
    #[tracing::instrument(skip(self), fields(n = n), name = "security::audit_verify_n")]
    pub async fn verify_last_n(&self, n: usize, public_key_hex: Option<&str>) -> Result<(usize, usize)> {
        let entries: Vec<AuditEntry> =
            sqlx::query_as("SELECT * FROM audit_trail ORDER BY timestamp DESC LIMIT ?")
                .bind(n as i64)
                .fetch_all(&self.pool)
                .await?;

        // Reverse to ascending for sequential re-hashing
        let mut entries = entries;
        entries.reverse();

        self.verify_segments(&entries, public_key_hex).await
    }

    /// Verifies the integrity of a single audit entry (hash and signature).
    #[allow(dead_code)]
    pub fn verify_record(&self, entry: &AuditEntry) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&entry.prev_hash);
        hasher.update(&entry.agent_id);
        if let Some(ref m) = entry.mission_id {
            hasher.update(m);
        }
        if let Some(ref u) = entry.user_id {
            hasher.update(u);
        }
        hasher.update(&entry.action);
        hasher.update(&entry.params);
        hasher.update(entry.timestamp.to_rfc3339());
        let computed = hex::encode(hasher.finalize());

        if entry.current_hash != computed {
            return false;
        }

        // Verify digital signature if applicable
        if let (Some(pk), Some(sig_hex)) = (self.verifying_key, &entry.signature) {
            if let Ok(sig_bytes) = hex::decode(sig_hex) {
                if let Ok(sig_arr) = sig_bytes.try_into() {
                    let signature = Signature::from_bytes(&sig_arr);
                    return pk.verify(entry.current_hash.as_bytes(), &signature).is_ok();
                }
            }
            return false;
        }

        true
    }

    /// Common logic for verifying a contiguous segment of the audit trail.
    #[tracing::instrument(skip(self, entries), fields(count = entries.len()), name = "security::audit_verify_segment")]
    pub async fn verify_segments(
        &self,
        entries: &[AuditEntry],
        public_key_hex: Option<&str>,
    ) -> Result<(usize, usize)> {
        let public_key = match public_key_hex {
            Some(h) => {
                let bytes = hex::decode(h)?;
                let bytes: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
                Some(VerifyingKey::from_bytes(&bytes)?)
            }
            None => self.verifying_key,
        };

        let total = entries.len();
        let mut verified = 0;
        let mut expected_prev_entry: Option<&AuditEntry> = None;

        for entry in entries {
            // If we have a sequence, verify the linkage
            if let Some(prev) = expected_prev_entry {
                if entry.prev_hash != prev.current_hash {
                    return Ok((verified, total)); // Broken chain
                }
            }

            // Verify intrinsic hash of the entry
            let mut hasher = Sha256::new();
            hasher.update(&entry.prev_hash);
            hasher.update(&entry.agent_id);
            if let Some(ref m) = entry.mission_id {
                hasher.update(m);
            }
            if let Some(ref u) = entry.user_id {
                hasher.update(u);
            }
            hasher.update(&entry.action);
            hasher.update(&entry.params);
            hasher.update(entry.timestamp.to_rfc3339());
            let computed = hex::encode(hasher.finalize());

            if entry.current_hash != computed {
                return Ok((verified, total)); // Tampered entry
            }

            // Verify digital signature if applicable
            if let (Some(pk), Some(sig_hex)) = (public_key, &entry.signature) {
                let sig_bytes = hex::decode(sig_hex)?;
                let signature = Signature::from_bytes(
                    &sig_bytes
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Invalid signature"))?,
                );
                if pk
                    .verify(entry.current_hash.as_bytes(), &signature)
                    .is_err()
                {
                    return Ok((verified, total)); // Tampered signature
                }
            }

            expected_prev_entry = Some(entry);
            verified += 1;
        }

        Ok((verified, total))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE audit_trail (
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
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn test_audit_trail_chaining() {
        let pool = setup_db().await;
        let audit = MerkleAuditTrail {
            pool,
            signing_key: None,
            verifying_key: None,
            last_hash: Arc::new(RwLock::new(None)),
        };

        // Record 1
        let entry1 = audit
            .record(
                "agent-1",
                Some("mission-1"),
                Some("user-1"),
                "tool_call",
                "{\"tool\":\"ls\"}",
            )
            .await
            .unwrap();
        assert_eq!(entry1.prev_hash, "0".repeat(64));

        // Record 2
        let entry2 = audit
            .record(
                "agent-1",
                Some("mission-1"),
                Some("user-1"),
                "tool_call",
                "{\"tool\":\"cat\"}",
            )
            .await
            .unwrap();
        assert_eq!(entry2.prev_hash, entry1.current_hash);

        // Verify chain
        let is_valid = audit.verify_chain(None).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_audit_tamper_detection() {
        let pool = setup_db().await;
        let audit = MerkleAuditTrail {
            pool,
            signing_key: None,
            verifying_key: None,
            last_hash: Arc::new(RwLock::new(None)),
        };

        audit
            .record("agent-1", None, None, "action-1", "{}")
            .await
            .unwrap();
        let entry2 = audit
            .record("agent-1", None, None, "action-2", "{}")
            .await
            .unwrap();

        assert!(audit.verify_chain(None).await.unwrap());

        // Tamper
        sqlx::query("UPDATE audit_trail SET action = 'tampered' WHERE id = ?")
            .bind(&entry2.id)
            .execute(&audit.pool)
            .await
            .unwrap();

        assert!(!audit.verify_chain(None).await.unwrap());
    }

    #[tokio::test]
    async fn test_signature_verification() {
        let pool = setup_db().await;
        let mut rng = rand::rng();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        let audit = MerkleAuditTrail {
            pool,
            signing_key: Some(signing_key),
            verifying_key: Some(verifying_key),
            last_hash: Arc::new(RwLock::new(None)),
        };

        let entry = audit
            .record("agent-1", None, None, "action", "{}")
            .await
            .unwrap();
        assert!(entry.signature.is_some());

        // Verify via verify_chain
        assert!(audit.verify_chain(None).await.unwrap());

        // Verify via manual key
        let pk_hex = hex::encode(verifying_key.to_bytes());
        assert!(audit.verify_chain(Some(&pk_hex)).await.unwrap());

        // Verify single record
        assert!(audit.verify_record(&entry));
    }
}

// Metadata: [audit]

// Metadata: [audit]
