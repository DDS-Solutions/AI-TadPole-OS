//! @docs ARCHITECTURE:Core:Storage:CAS
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / cas
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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

/// Captures the current content state of a workspace file into `cas_blobs` and logs a new revision record in `file_revisions`
/// before an agent modifies or deletes it. Uses deduplicated BLOB storage and atomic DB transactions.
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

    // Read bytes directly to prevent TOCTOU race between metadata check and file read
    let raw_bytes = match tokio::fs::read(target_path).await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(
                target: "cas_service",
                path = %target_path.display(),
                error = %e,
                "⚠️ [CAS] Failed to read pre-mutation file"
            );
            return Err(AppError::Io(e));
        }
    };

    let file_size = raw_bytes.len();
    if file_size > MAX_CAS_FILE_SIZE {
        tracing::warn!(
            target: "cas_service",
            path = %target_path.display(),
            size_mb = file_size / (1024 * 1024),
            "⚠️ [CAS] Skipping pre-mutation capture: file size exceeds safety ceiling (50 MB)"
        );
        return Ok(None);
    }

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

    let (next_ver, is_duplicate) = match latest_opt {
        Some((latest_ver, latest_hash)) => (latest_ver + 1, latest_hash == hash),
        None => (0, false),
    };

    if is_duplicate {
        tx.rollback().await?;
        return Ok(Some(hash));
    }

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

    // 2. Insert revision metadata into file_revisions referencing the content hash (empty content stub for schema compat)
    sqlx::query(
        "INSERT INTO file_revisions (workspace_id, file_path, hash, content, size_bytes, version_num, mission_id, agent_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&workspace_id)
    .bind(&rel_path)
    .bind(&hash)
    .bind(&[] as &[u8])
    .bind(file_size as i64)
    .bind(next_ver)
    .bind(mission_id)
    .bind(agent_id)
    .bind(&created_at)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!(
        target: "cas_service",
        version = next_ver,
        path = %rel_path,
        hash = &hash[..8.min(hash.len())],
        size_bytes = file_size,
        "📸 [CAS] Captured file revision"
    );
    Ok(Some(hash))
}

/// Restores a file to a specific target revision version number, verifies byte hash,
/// atomically writes content back to disk via temporary file rename, and logs a new revision record.
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

    let safe_target = crate::utils::security::validate_path(workspace_root, &rel_path)?;
    let abs_target_path = safe_target.to_path_buf();
    if let Some(parent) = abs_target_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to create parent directory for restoration: {}",
                e
            ))
        })?;
    }

    // Atomic disk restoration: write to temporary sibling file and rename to prevent corruption on crash
    let temp_restore_path =
        abs_target_path.with_extension(format!("tmp_cas_restore_{}", uuid::Uuid::new_v4()));
    tokio::fs::write(&temp_restore_path, &record.content)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to write temporary restored content to '{}': {}",
                temp_restore_path.display(),
                e
            ))
        })?;

    // Post-write integrity hash check on temp file
    let written_bytes = tokio::fs::read(&temp_restore_path).await.map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to read back temporary restored file for verification: {}",
            e
        ))
    })?;
    let written_hash = compute_sha256(&written_bytes);
    if written_hash != record.hash {
        let _ = tokio::fs::remove_file(&temp_restore_path).await;
        return Err(AppError::InternalServerError(format!(
            "Restoration integrity verification failed for '{}': expected hash {}, wrote {}",
            rel_path, record.hash, written_hash
        )));
    }

    // Atomic filesystem swap
    tokio::fs::rename(&temp_restore_path, &abs_target_path)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to atomically rename restored file to '{}': {}",
                abs_target_path.display(),
                e
            ))
        })?;

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
    .bind(&[] as &[u8])
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
        target: "cas_service",
        path = %rel_path,
        source_version = target_version,
        new_version = new_ver,
        hash = &summary.hash[..8.min(summary.hash.len())],
        "🔄 [CAS] Restored file from prior version"
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
    let _ = crate::utils::security::validate_path(workspace_root, &rel_path)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS cas_blobs (
                hash TEXT PRIMARY KEY,
                content BLOB NOT NULL,
                size_bytes INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS file_revisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workspace_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                hash TEXT NOT NULL,
                content BLOB NOT NULL,
                size_bytes INTEGER NOT NULL,
                version_num INTEGER NOT NULL,
                mission_id TEXT,
                agent_id TEXT,
                created_at TEXT NOT NULL
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_cas_capture_restore_and_deduplication() {
        let pool = setup_test_db().await;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("code.rs");

        // Write initial version
        tokio::fs::write(&file_path, b"fn main() { println!(\"v0\"); }")
            .await
            .unwrap();

        let hash_v0 =
            capture_pre_mutation(&pool, dir.path(), &file_path, Some("m1"), Some("agent-1"))
                .await
                .unwrap()
                .expect("Must capture initial version");

        // Modify file
        tokio::fs::write(&file_path, b"fn main() { println!(\"v1\"); }")
            .await
            .unwrap();

        let hash_v1 =
            capture_pre_mutation(&pool, dir.path(), &file_path, Some("m2"), Some("agent-2"))
                .await
                .unwrap()
                .expect("Must capture modified version");

        assert_ne!(hash_v0, hash_v1);

        // Check history
        let history = get_file_history(&pool, dir.path(), "code.rs")
            .await
            .unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version_num, 1);
        assert_eq!(history[1].version_num, 0);

        // Restore back to v0
        let restored = restore_file_version(&pool, dir.path(), "code.rs", 0)
            .await
            .unwrap();
        assert_eq!(restored.version_num, 2);
        assert_eq!(restored.hash, hash_v0);

        // Verify disk content is restored
        let current_bytes = tokio::fs::read(&file_path).await.unwrap();
        assert_eq!(current_bytes, b"fn main() { println!(\"v0\"); }");
    }
}
