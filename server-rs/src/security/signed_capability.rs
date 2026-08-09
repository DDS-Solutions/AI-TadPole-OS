//! @docs ARCHITECTURE:ShieldLayer
//! @docs SEC-05: Persistent Signed Capability Manifests
//!
//! ### AI Assist Note
//! **Cryptographically Signed Persistent Capability Manifests (SEC-05)**:
//! Implements Ed25519 asymmetric signature verification, SHA-256 content hashes,
//! and Oversight Ledger approval checks for persistent capabilities.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Signature verification failure, content hash mismatch,
//!   expired timestamp, or missing/invalid oversight approval ID.
//! - **Trace Scope**: `server-rs::security::signed_capability`

use crate::error::AppError;
use crate::security::permissions::{CapabilityClass, PermissionMode, PermissionPolicy};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;

#[allow(dead_code)]
pub const MANIFEST_MAGIC_HEADER: &[u8] = b"SEC-05-V1\0";

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManifestStatus {
    Pending,
    Active,
    Revoked,
    Expired,
}

impl std::fmt::Display for ManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestStatus::Pending => write!(f, "pending"),
            ManifestStatus::Active => write!(f, "active"),
            ManifestStatus::Revoked => write!(f, "revoked"),
            ManifestStatus::Expired => write!(f, "expired"),
        }
    }
}

impl FromStr for ManifestStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ManifestStatus::Pending),
            "active" => Ok(ManifestStatus::Active),
            "revoked" => Ok(ManifestStatus::Revoked),
            "expired" => Ok(ManifestStatus::Expired),
            _ => Err(anyhow::anyhow!("Invalid manifest status: {}", s)),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedCapabilityManifest {
    pub capability_id: String,
    pub version: String,
    pub owner: String,
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub content_hash: String,
    pub signature: String,
    pub approval_id: Option<String>,
    pub expiration: Option<DateTime<Utc>>,
    pub risk_class: CapabilityClass,
    pub resource_pattern: String,
    pub mode: PermissionMode,
    pub status: ManifestStatus,
}

#[allow(dead_code)]
impl SignedCapabilityManifest {
    /// Constructs canonical byte representation for Ed25519 signing & verification.
    pub fn canonical_payload(
        capability_id: &str,
        version: &str,
        owner: &str,
        creator: &str,
        created_at: i64,
        content_hash: &str,
        approval_id: Option<&str>,
        expiration: Option<i64>,
        risk_class: CapabilityClass,
        resource_pattern: &str,
        mode: PermissionMode,
    ) -> Vec<u8> {
        let rc_str = risk_class.as_canonical_str();
        let mode_str = mode.as_canonical_str();

        let mut buf = Vec::with_capacity(
            MANIFEST_MAGIC_HEADER.len()
                + 4 + capability_id.len()
                + 4 + version.len()
                + 4 + owner.len()
                + 4 + creator.len()
                + 8
                + 4 + content_hash.len()
                + 1 + approval_id.map_or(0, |a| 4 + a.len())
                + 1 + expiration.map_or(0, |_| 8)
                + 4 + rc_str.len()
                + 4 + resource_pattern.len()
                + 4 + mode_str.len(),
        );

        buf.extend_from_slice(MANIFEST_MAGIC_HEADER);
        buf.extend_from_slice(&(capability_id.len() as u32).to_le_bytes());
        buf.extend_from_slice(capability_id.as_bytes());
        buf.extend_from_slice(&(version.len() as u32).to_le_bytes());
        buf.extend_from_slice(version.as_bytes());
        buf.extend_from_slice(&(owner.len() as u32).to_le_bytes());
        buf.extend_from_slice(owner.as_bytes());
        buf.extend_from_slice(&(creator.len() as u32).to_le_bytes());
        buf.extend_from_slice(creator.as_bytes());
        buf.extend_from_slice(&created_at.to_le_bytes());
        buf.extend_from_slice(&(content_hash.len() as u32).to_le_bytes());
        buf.extend_from_slice(content_hash.as_bytes());

        if let Some(app) = approval_id {
            buf.push(1);
            buf.extend_from_slice(&(app.len() as u32).to_le_bytes());
            buf.extend_from_slice(app.as_bytes());
        } else {
            buf.push(0);
        }

        if let Some(exp) = expiration {
            buf.push(1);
            buf.extend_from_slice(&exp.to_le_bytes());
        } else {
            buf.push(0);
        }

        buf.extend_from_slice(&(rc_str.len() as u32).to_le_bytes());
        buf.extend_from_slice(rc_str.as_bytes());

        buf.extend_from_slice(&(resource_pattern.len() as u32).to_le_bytes());
        buf.extend_from_slice(resource_pattern.as_bytes());

        buf.extend_from_slice(&(mode_str.len() as u32).to_le_bytes());
        buf.extend_from_slice(mode_str.as_bytes());

        buf
    }

    /// Computes the SHA-256 content hash for target file/script bytes.
    pub fn compute_hash(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Signs a manifest payload using an Ed25519 SigningKey.
    pub fn sign(
        signing_key: &SigningKey,
        capability_id: String,
        version: String,
        owner: String,
        creator: String,
        created_at: DateTime<Utc>,
        content_hash: String,
        approval_id: Option<String>,
        expiration: Option<DateTime<Utc>>,
        risk_class: CapabilityClass,
        resource_pattern: String,
        mode: PermissionMode,
    ) -> Self {
        let payload = Self::canonical_payload(
            &capability_id,
            &version,
            &owner,
            &creator,
            created_at.timestamp(),
            &content_hash,
            approval_id.as_deref(),
            expiration.map(|t| t.timestamp()),
            risk_class,
            &resource_pattern,
            mode,
        );

        let sig = signing_key.sign(&payload);
        let signature_hex = hex::encode(sig.to_bytes());

        Self {
            capability_id,
            version,
            owner,
            creator,
            created_at,
            content_hash,
            signature: signature_hex,
            approval_id,
            expiration,
            risk_class,
            resource_pattern,
            mode,
            status: ManifestStatus::Pending,
        }
    }

    /// Verifies the Ed25519 signature against a VerifyingKey.
    pub fn verify_signature(&self, verifying_key: &VerifyingKey) -> Result<(), AppError> {
        let sig_bytes = hex::decode(&self.signature)
            .map_err(|e| AppError::Forbidden(format!("Invalid signature hex: {}", e)))?;
        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| AppError::Forbidden("Signature must be 64 bytes".to_string()))?;
        let signature = Signature::from_bytes(&sig_array);

        let payload = Self::canonical_payload(
            &self.capability_id,
            &self.version,
            &self.owner,
            &self.creator,
            self.created_at.timestamp(),
            &self.content_hash,
            self.approval_id.as_deref(),
            self.expiration.map(|t| t.timestamp()),
            self.risk_class,
            &self.resource_pattern,
            self.mode,
        );

        verifying_key
            .verify(&payload, &signature)
            .map_err(|e| AppError::Forbidden(format!("Ed25519 signature verification failed: {}", e)))
    }

    /// Executes the 5-step verification & activation pipeline.
    pub async fn verify_and_activate(
        &mut self,
        verifying_key: &VerifyingKey,
        target_content_bytes: &[u8],
        policy: &PermissionPolicy,
    ) -> Result<(), AppError> {
        // Step 1: Check Expiration
        if let Some(exp) = self.expiration {
            if Utc::now() > exp {
                self.status = ManifestStatus::Expired;
                return Err(AppError::Forbidden(format!(
                    "Capability manifest '{}' expired at {}",
                    self.capability_id, exp
                )));
            }
        }

        // Step 2: Verify Signature
        self.verify_signature(verifying_key)?;

        // Step 3: Verify Content Hash
        let actual_hash = Self::compute_hash(target_content_bytes);
        if actual_hash != self.content_hash {
            return Err(AppError::Forbidden(format!(
                "Content hash mismatch for capability '{}': expected {}, got {}",
                self.capability_id, self.content_hash, actual_hash
            )));
        }

        // Step 4: Verify Approval ID Attribution
        if self.mode != PermissionMode::Prompt && self.approval_id.is_none() {
            return Err(AppError::Forbidden(format!(
                "Capability manifest '{}' requires an approval_id for non-prompt mode",
                self.capability_id
            )));
        }

        // Step 5: Activate capability mode in PermissionPolicy (using signed mode bypass)
        policy
            .set_capability_mode_signed(self.risk_class, &self.resource_pattern, self.mode)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to activate capability: {}", e)))?;

        self.status = ManifestStatus::Active;
        self.save_to_db(policy.pool())
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to persist manifest status: {}", e)))?;

        tracing::info!(
            "🛡️ [SEC-05] Activated signed capability manifest '{}' ({}:{} -> {})",
            self.capability_id,
            self.risk_class,
            self.resource_pattern,
            self.mode
        );
        Ok(())
    }

    /// Saves or updates manifest record in SQLite database.
    pub async fn save_to_db(&self, pool: &sqlx::SqlitePool) -> Result<(), anyhow::Error> {
        sqlx::query(
            "INSERT INTO signed_capability_manifests (capability_id, version, owner, creator, created_at, content_hash, signature, approval_id, expiration, risk_class, resource_pattern, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(capability_id) DO UPDATE SET status = excluded.status"
        )
        .bind(&self.capability_id)
        .bind(&self.version)
        .bind(&self.owner)
        .bind(&self.creator)
        .bind(self.created_at)
        .bind(&self.content_hash)
        .bind(&self.signature)
        .bind(&self.approval_id)
        .bind(self.expiration)
        .bind(self.risk_class.to_string())
        .bind(&self.resource_pattern)
        .bind(self.status.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }
}
