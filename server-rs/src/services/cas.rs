//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **Lean Workspace Content-Addressable Storage (CAS) Service**
//! Provides transaction-atomic pre-mutation snapshotting, two-tier deduplicated binary BLOB storage (`cas_blobs`),
//! and audit-recorded 1-click file restoration capabilities for workspace files.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: IO read/write errors, SQLite transaction conflicts, hash mismatches, or file size ceiling exceeded.
//! - **Telemetry Link**: Search `[cas_service]` in tracing logs.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::path::Path;

/// Maximum size allowed for individual CAS file versioning (50 MB safety ceiling).
pub const MAX_CAS_FILE_SIZE: usize = 50 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionSummary {
    pub id: i64,
    pub workspace_id: String,
    pub file_path: String,
    pub hash: String,
    pub size_bytes: usize,
    pub version_num: i64,
    pub mission_id: Option<String>,
    pub agent_id: Option<String>,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct FullRevisionRecord {
    pub id: i64,
    pub workspace_id: String,
    pub file_path: String,
    pub hash: String,
    pub content: Vec<u8>,
    pub size_bytes: usize,
    pub version_num: i64,
    pub mission_id: Option<String>,
    pub agent_id: Option<String>,
    pub created_at: String,
}

/// Computes the SHA-256 hexadecimal hash string for arbitrary raw byte slices.
pub fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Normalizes relative workspace file paths to use forward slashes for clean DB indexing.
pub fn normalize_rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Captures the current content state of a workspace file into `file_revisions` and `cas_blobs`
/// before an agent modifies or deletes it. Uses two-tier deduplicated BLOB storage and atomic DB transactions.
pub async fn capture_pre_mutation(
    pool: &SqlitePool,
    workspace_root: &Path,
    target_path: &Path,
    mission_id: Option<&str>,
    agent_id: Option<&str>,
) -> Result<Option<String>, AppError> {
    if !target_path.exists() || !target_path.is_file() {
        return Ok(None);
    }

    let metadata = match tokio::fs::metadata(target_path).await {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(
                "⚠️ [CAS] Failed to read metadata for '{}': {}",
                target_path.display(),
                e
            );
            return Ok(None);
        }
    };

    let file_size = metadata.len() as usize;
    if file_size > MAX_CAS_FILE_SIZE {
        tracing::warn!(
            "⚠️ [CAS] Skipping pre-mutation capture for '{}': file size ({} MB) exceeds safety ceiling (50 MB)",
            target_path.display(),
            file_size / (1024 * 1024)
        );
        return Ok(None);
    }

    let raw_bytes = match tokio::fs::read(target_path).await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(
                "⚠️ [CAS] Failed to read pre-mutation file '{}': {}",
                target_path.display(),
                e
            );
            return Ok(None);
        }
    };

    let hash = compute_sha256(&raw_bytes);
    let rel_path = match target_path.strip_prefix(workspace_root) {
        Ok(p) => normalize_rel_path(p),
        Err(_) => normalize_rel_path(target_path),
    };
    let workspace_id = normalize_rel_path(workspace_root);

    // Atomic transaction for check-and-insert
    let mut tx = pool.begin().await?;

    let latest_opt: Option<(i64, String)> = sqlx::query_as(
        "SELECT version_num, hash FROM file_revisions WHERE workspace_id = ? AND file_path = ? ORDER BY version_num DESC LIMIT 1"
    )
    .bind(&workspace_id)
    .bind(&rel_path)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some((latest_ver, latest_hash)) = latest_opt {
        if latest_hash == hash {
            tx.rollback().await?;
            return Ok(Some(hash));
        }

        let next_ver = latest_ver + 1;
        let created_at = chrono::Utc::now().to_rfc3339();

        // 1. Two-Tier CAS BLOB Deduplication: Store unique binary payload in cas_blobs
        sqlx::query(
            "INSERT OR IGNORE INTO cas_blobs (hash, content, size_bytes, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&hash)
        .bind(&raw_bytes)
        .bind(file_size as i64)
        .bind(&created_at)
        .execute(&mut *tx)
        .await?;

        // 2. Insert revision metadata into file_revisions referencing the content hash
        sqlx::query(
            "INSERT INTO file_revisions (workspace_id, file_path, hash, content, size_bytes, version_num, mission_id, agent_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&workspace_id)
        .bind(&rel_path)
        .bind(&hash)
        .bind(&raw_bytes)
        .bind(file_size as i64)
        .bind(next_ver)
        .bind(mission_id)
        .bind(agent_id)
        .bind(&created_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        tracing::info!(
            "📸 [CAS] Captured file revision v{} for '{}' (hash: {}, size: {} bytes)",
            next_ver,
            rel_path,
            &hash[..8.min(hash.len())],
            file_size
        );
        Ok(Some(hash))
    } else {
        // Initial baseline capture (v0)
        let created_at = chrono::Utc::now().to_rfc3339();

        // 1. Two-Tier CAS BLOB Deduplication: Store unique binary payload in cas_blobs
        sqlx::query(
            "INSERT OR IGNORE INTO cas_blobs (hash, content, size_bytes, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&hash)
        .bind(&raw_bytes)
        .bind(file_size as i64)
        .bind(&created_at)
        .execute(&mut *tx)
        .await?;

        // 2. Insert baseline revision into file_revisions
        sqlx::query(
            "INSERT INTO file_revisions (workspace_id, file_path, hash, content, size_bytes, version_num, mission_id, agent_id, created_at) VALUES (?, ?, ?, ?, ?, 0, ?, ?, ?)"
        )
        .bind(&workspace_id)
        .bind(&rel_path)
        .bind(&hash)
        .bind(&raw_bytes)
        .bind(file_size as i64)
        .bind(mission_id)
        .bind(agent_id)
        .bind(&created_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        tracing::info!(
            "📸 [CAS] Captured baseline revision v0 for '{}' (hash: {}, size: {} bytes)",
            rel_path,
            &hash[..8.min(hash.len())],
            file_size
        );
        Ok(Some(hash))
    }
}

/// Restores a file to a specific target revision version number, verifies byte hash,
/// writes raw content back to disk, and inserts a NEW revision record for the restoration action.
pub async fn restore_file_version(
    pool: &SqlitePool,
    workspace_root: &Path,
    rel_path_str: &str,
    target_version: i64,
) -> Result<RevisionSummary, AppError> {
    let workspace_id = normalize_rel_path(workspace_root);
    let rel_path = rel_path_str.replace('\\', "/");

    let row = sqlx::query(
        "SELECT r.id, r.workspace_id, r.file_path, r.hash, 
                CASE WHEN b.content IS NOT NULL AND length(b.content) > 0 THEN b.content ELSE r.content END as content, 
                r.size_bytes, r.version_num, r.mission_id, r.agent_id, r.created_at 
         FROM file_revisions r 
         LEFT JOIN cas_blobs b ON r.hash = b.hash 
         WHERE r.workspace_id = ? AND r.file_path = ? AND r.version_num = ?"
    )
    .bind(&workspace_id)
    .bind(&rel_path)
    .bind(target_version)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Revision v{} for file '{}' not found in CAS store", target_version, rel_path)))?;

    let record = FullRevisionRecord {
        id: row.get("id"),
        workspace_id: row.get("workspace_id"),
        file_path: row.get("file_path"),
        hash: row.get("hash"),
        content: row.get("content"),
        size_bytes: row.get::<i64, _>("size_bytes") as usize,
        version_num: row.get("version_num"),
        mission_id: row.get("mission_id"),
        agent_id: row.get("agent_id"),
        created_at: row.get("created_at"),
    };

    let abs_target_path = workspace_root.join(&rel_path);
    if let Some(parent) = abs_target_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to create parent directory for restoration: {}",
                e
            ))
        })?;
    }

    // Write raw bytes to target file
    tokio::fs::write(&abs_target_path, &record.content)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to write restored content to '{}': {}",
                abs_target_path.display(),
                e
            ))
        })?;

    // Post-write integrity hash check
    let written_bytes = tokio::fs::read(&abs_target_path).await.map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to read back restored file for verification: {}",
            e
        ))
    })?;
    let written_hash = compute_sha256(&written_bytes);
    if written_hash != record.hash {
        return Err(AppError::InternalServerError(format!(
            "Restoration integrity verification failed for '{}': expected hash {}, wrote {}",
            rel_path, record.hash, written_hash
        )));
    }

    // Atomic transaction to record restoration event as a NEW revision
    let mut tx = pool.begin().await?;

    let max_ver: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_num), 0) FROM file_revisions WHERE workspace_id = ? AND file_path = ?"
    )
    .bind(&workspace_id)
    .bind(&rel_path)
    .fetch_one(&mut *tx)
    .await?;

    let new_ver = max_ver + 1;
    let created_at = chrono::Utc::now().to_rfc3339();
    let restore_mission_id = format!("restoration-from-v{}", target_version);
    let restore_agent_id = "cas_restore_operator";

    // Ensure BLOB is present in cas_blobs
    sqlx::query(
        "INSERT OR IGNORE INTO cas_blobs (hash, content, size_bytes, created_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&record.hash)
    .bind(&record.content)
    .bind(record.size_bytes as i64)
    .bind(&created_at)
    .execute(&mut *tx)
    .await?;

    let inserted_id = sqlx::query(
        "INSERT INTO file_revisions (workspace_id, file_path, hash, content, size_bytes, version_num, mission_id, agent_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&workspace_id)
    .bind(&rel_path)
    .bind(&record.hash)
    .bind(&record.content)
    .bind(record.size_bytes as i64)
    .bind(new_ver)
    .bind(&restore_mission_id)
    .bind(restore_agent_id)
    .bind(&created_at)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    tx.commit().await?;

    let summary = RevisionSummary {
        id: inserted_id,
        workspace_id: record.workspace_id,
        file_path: record.file_path,
        hash: record.hash,
        size_bytes: record.size_bytes,
        version_num: new_ver,
        mission_id: Some(restore_mission_id),
        agent_id: Some(restore_agent_id.to_string()),
        created_at,
    };

    tracing::info!(
        "🔄 [CAS] Restored file '{}' from v{} -> logged new revision v{} (hash: {})",
        rel_path,
        target_version,
        new_ver,
        &summary.hash[..8.min(summary.hash.len())]
    );

    Ok(summary)
}

/// Retrieves the revision history summary list for a file WITHOUT loading full content BLOBs into memory.
pub async fn get_file_history(
    pool: &SqlitePool,
    workspace_root: &Path,
    rel_path_str: &str,
) -> Result<Vec<RevisionSummary>, AppError> {
    let workspace_id = normalize_rel_path(workspace_root);
    let rel_path = rel_path_str.replace('\\', "/");

    let rows = sqlx::query(
        "SELECT id, workspace_id, file_path, hash, size_bytes, version_num, mission_id, agent_id, created_at FROM file_revisions WHERE workspace_id = ? AND file_path = ? ORDER BY version_num DESC"
    )
    .bind(&workspace_id)
    .bind(&rel_path)
    .fetch_all(pool)
    .await?;

    let summaries = rows
        .into_iter()
        .map(|r| RevisionSummary {
            id: r.get("id"),
            workspace_id: r.get("workspace_id"),
            file_path: r.get("file_path"),
            hash: r.get("hash"),
            size_bytes: r.get::<i64, _>("size_bytes") as usize,
            version_num: r.get("version_num"),
            mission_id: r.get("mission_id"),
            agent_id: r.get("agent_id"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(summaries)
}

// Metadata: [cas_service]
