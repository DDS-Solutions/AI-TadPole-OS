//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Services / Backup / Snapshot
//! - **Primary Entrypoints**: `create_state_snapshot`, `pack_snapshot_archive`, `unpack_snapshot_archive`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use crate::error::AppError;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};

const ARCHIVE_MAGIC: &[u8; 16] = b"TADPOLE_ARCH_V1\0";

pub struct SnapshotBundle {
    pub raw_archive: Vec<u8>,
    pub sqlite_bytes: u64,
    pub lancedb_bytes: u64,
    pub total_files: usize,
    pub sha256_checksum: String,
}

pub struct ArchiveFileEntry {
    pub rel_path: String,
    pub data: Vec<u8>,
}

/// Packs multiple named files into a deterministic byte archive.
pub fn pack_snapshot_archive(files: Vec<ArchiveFileEntry>) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(ARCHIVE_MAGIC);
    out.extend_from_slice(&(files.len() as u32).to_le_bytes());

    for file in files {
        let path_bytes = file.rel_path.as_bytes();
        out.extend_from_slice(&(path_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(path_bytes);
        out.extend_from_slice(&(file.data.len() as u64).to_le_bytes());
        out.extend_from_slice(&file.data);
    }

    out
}

/// Safely unpacks a snapshot archive into a target directory, preventing path traversal.
pub fn unpack_snapshot_archive(
    archive_bytes: &[u8],
    target_root: &Path,
) -> Result<Vec<PathBuf>, AppError> {
    if archive_bytes.len() < ARCHIVE_MAGIC.len() + 4 {
        return Err(AppError::BadRequest(
            "Archive corrupted: header too short".to_string(),
        ));
    }

    if !archive_bytes.starts_with(ARCHIVE_MAGIC) {
        return Err(AppError::BadRequest("Invalid archive header".to_string()));
    }

    let mut cursor = ARCHIVE_MAGIC.len();
    let file_count = u32::from_le_bytes(
        archive_bytes[cursor..cursor + 4]
            .try_into()
            .map_err(|_| AppError::BadRequest("Malformed file count".to_string()))?,
    ) as usize;
    cursor += 4;

    let mut extracted_paths = Vec::new();

    for _ in 0..file_count {
        if cursor + 2 > archive_bytes.len() {
            return Err(AppError::BadRequest(
                "Unexpected EOF reading path length".to_string(),
            ));
        }
        let path_len = u16::from_le_bytes(
            archive_bytes[cursor..cursor + 2]
                .try_into()
                .map_err(|_| AppError::BadRequest("Malformed path length".to_string()))?,
        ) as usize;
        cursor += 2;

        if cursor + path_len > archive_bytes.len() {
            return Err(AppError::BadRequest(
                "Unexpected EOF reading relative path".to_string(),
            ));
        }
        let rel_path_str = std::str::from_utf8(&archive_bytes[cursor..cursor + path_len])
            .map_err(|_| AppError::BadRequest("Invalid UTF-8 in archive path".to_string()))?;
        cursor += path_len;

        // Path Traversal Security Guard
        if rel_path_str.contains("..")
            || rel_path_str.starts_with('/')
            || rel_path_str.starts_with('\\')
        {
            return Err(AppError::BadRequest(format!(
                "Illegal path in archive: {}",
                rel_path_str
            )));
        }

        if cursor + 8 > archive_bytes.len() {
            return Err(AppError::BadRequest(
                "Unexpected EOF reading file size".to_string(),
            ));
        }
        let file_len = u64::from_le_bytes(
            archive_bytes[cursor..cursor + 8]
                .try_into()
                .map_err(|_| AppError::BadRequest("Malformed file size".to_string()))?,
        ) as usize;
        cursor += 8;

        if cursor + file_len > archive_bytes.len() {
            return Err(AppError::BadRequest(
                "Unexpected EOF reading file data".to_string(),
            ));
        }
        let file_data = &archive_bytes[cursor..cursor + file_len];
        cursor += file_len;

        let out_path = target_root.join(rel_path_str);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::InternalServerError(format!(
                    "Failed to create parent dir {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        std::fs::write(&out_path, file_data).map_err(|e| {
            AppError::InternalServerError(format!("Failed to write {}: {}", out_path.display(), e))
        })?;

        extracted_paths.push(out_path);
    }

    Ok(extracted_paths)
}

/// Recursively scans a directory and collects all files for archive inclusion.
fn collect_directory_files(dir: &Path, rel_prefix: &str) -> Vec<ArchiveFileEntry> {
    let mut entries = Vec::new();
    if !dir.exists() || !dir.is_dir() {
        return entries;
    }

    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            let rel = if rel_prefix.is_empty() {
                file_name.clone()
            } else {
                format!("{}/{}", rel_prefix, file_name)
            };

            if path.is_file() {
                if let Ok(data) = std::fs::read(&path) {
                    entries.push(ArchiveFileEntry {
                        rel_path: rel,
                        data,
                    });
                }
            } else if path.is_dir() {
                entries.extend(collect_directory_files(&path, &rel));
            }
        }
    }
    entries
}

/// Creates a complete point-in-time snapshot of the SQLite database and LanceDB vector memories.
pub async fn create_state_snapshot(
    pool: &SqlitePool,
    base_dir: &Path,
) -> Result<SnapshotBundle, AppError> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| AppError::InternalServerError(format!("Failed to create tempdir: {}", e)))?;

    let temp_db_path = temp_dir.path().join("vacuum_tadpole.db");
    let temp_db_str = temp_db_path.to_string_lossy().replace('\\', "/");

    // 1. Non-blocking SQLite snapshot via VACUUM INTO
    let vacuum_sql = format!("VACUUM INTO '{}'", temp_db_str.replace('\'', "''"));
    sqlx::query(sqlx::AssertSqlSafe(&*vacuum_sql))
        .execute(pool)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("SQLite VACUUM INTO snapshot failed: {}", e))
        })?;

    let sqlite_bytes = std::fs::metadata(&temp_db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let sqlite_data = std::fs::read(&temp_db_path).map_err(|e| {
        AppError::InternalServerError(format!("Failed to read vacuumed database: {}", e))
    })?;

    let mut entries = Vec::new();
    entries.push(ArchiveFileEntry {
        rel_path: "tadpole.db".to_string(),
        data: sqlite_data,
    });

    // 2. Collect workspaces / LanceDB vector memory stores
    let state_workspaces = base_dir.join("data").join("state").join("workspaces");
    let legacy_workspaces = base_dir.join("data").join("workspaces");
    let workspaces_path = if state_workspaces.exists() {
        state_workspaces
    } else {
        legacy_workspaces
    };

    let mut lancedb_bytes = 0u64;
    if workspaces_path.exists() {
        let ws_entries = collect_directory_files(&workspaces_path, "workspaces");
        for ws in ws_entries {
            lancedb_bytes += ws.data.len() as u64;
            entries.push(ws);
        }
    }

    let total_files = entries.len();
    let raw_archive = pack_snapshot_archive(entries);

    let mut hasher = Sha256::new();
    hasher.update(&raw_archive);
    let sha256_checksum = hex::encode(hasher.finalize());

    Ok(SnapshotBundle {
        raw_archive,
        sqlite_bytes,
        lancedb_bytes,
        total_files,
        sha256_checksum,
    })
}
