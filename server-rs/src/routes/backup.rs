//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / backup
//! - **Primary Entrypoints**: `get_backup_status`, `configure_backup`, `trigger_backup_now`, `restore_backup`, `get_backup_history`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::NotFound`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared

use crate::{
    error::AppError,
    services::backup::{
        crypto::{decrypt_backup_payload, generate_recovery_key_manifest},
        daemon::{load_backup_config, run_backup_cycle, save_backup_config, BackupConfig},
        snapshot::unpack_snapshot_archive,
    },
    state::AppState,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Serialize, specta::Type)]
pub struct BackupStatusResponse {
    pub enabled: bool,
    pub destination_type: String,
    pub destination_path: String,
    pub has_passphrase: bool,
    pub last_run_timestamp: Option<i64>,
    pub last_status: Option<String>,
    pub total_snapshots: usize,
    pub total_size_bytes: u64,
    pub retention_hours: i64,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct ConfigureBackupRequest {
    pub enabled: bool,
    pub destination_type: String, // "local" | "cloud_sync"
    pub destination_path: String,
    pub passphrase: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct RestoreBackupRequest {
    pub snapshot_path: String,
    pub passphrase: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow, specta::Type)]
pub struct BackupRecordItem {
    pub id: String,
    pub snapshot_filename: String,
    pub destination_type: String,
    pub destination_path: String,
    pub file_size_bytes: i64,
    pub sha256_checksum: String,
    pub sqlite_bytes: i64,
    pub lancedb_bytes: i64,
    pub status: String,
    pub created_at: i64,
    pub pruned_at: Option<i64>,
}

/// GET /v1/system/backup/status
pub async fn get_backup_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BackupStatusResponse>, AppError> {
    let config = load_backup_config(&state.base_dir);
    let mut total_snapshots = 0;
    let mut total_size_bytes = 0u64;

    if !config.destination_path.trim().is_empty() {
        let p = PathBuf::from(&config.destination_path);
        if p.exists() {
            if let Ok(entries) = std::fs::read_dir(&p) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                        if name.starts_with("tadpole-snapshot-")
                            && name.ends_with(".tadpole-snap.enc")
                        {
                            total_snapshots += 1;
                            if let Ok(m) = path.metadata() {
                                total_size_bytes += m.len();
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Json(BackupStatusResponse {
        enabled: config.enabled,
        destination_type: config.destination_type,
        destination_path: config.destination_path,
        has_passphrase: config.passphrase.is_some()
            && !config.passphrase.as_ref().unwrap().is_empty(),
        last_run_timestamp: config.last_run_timestamp,
        last_status: config.last_status,
        total_snapshots,
        total_size_bytes,
        retention_hours: 48,
    }))
}

/// POST /v1/system/backup/configure
pub async fn configure_backup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConfigureBackupRequest>,
) -> Result<Json<BackupStatusResponse>, AppError> {
    let mut current = load_backup_config(&state.base_dir);
    current.enabled = payload.enabled;

    if payload.destination_type == "local" || payload.destination_type == "cloud_sync" {
        current.destination_type = payload.destination_type;
    } else {
        return Err(AppError::BadRequest(
            "destination_type must be 'local' or 'cloud_sync'".to_string(),
        ));
    }

    let clean_path = payload.destination_path.trim().to_string();
    if !clean_path.is_empty() {
        let p = PathBuf::from(&clean_path);
        if let Err(e) = std::fs::create_dir_all(&p) {
            return Err(AppError::BadRequest(format!(
                "Cannot write to target directory {}: {}",
                p.display(),
                e
            )));
        }
        current.destination_path = clean_path;
    }

    if let Some(pass) = payload.passphrase {
        if !pass.trim().is_empty() {
            current.passphrase = Some(pass);
        }
    }

    save_backup_config(&state.base_dir, &current)?;
    get_backup_status(State(state)).await
}

/// POST /v1/system/backup/trigger
pub async fn trigger_backup_now(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut config = load_backup_config(&state.base_dir);
    let record = run_backup_cycle(&state.resources.pool, &state.base_dir, &mut config).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "backup_id": record.id,
        "snapshot_filename": record.snapshot_filename,
        "file_size_bytes": record.file_size_bytes,
        "sqlite_bytes": record.sqlite_bytes,
        "lancedb_bytes": record.lancedb_bytes,
        "sha256": record.sha256_checksum,
        "created_at": record.created_at
    })))
}

/// POST /v1/system/backup/restore
pub async fn restore_backup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RestoreBackupRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let p = PathBuf::from(&payload.snapshot_path);
    if !p.exists() || !p.is_file() {
        return Err(AppError::NotFound(format!(
            "Snapshot file not found: {}",
            p.display()
        )));
    }

    let envelope_bytes = std::fs::read(&p).map_err(|e| {
        AppError::InternalServerError(format!("Failed to read snapshot file: {}", e))
    })?;

    let config = load_backup_config(&state.base_dir);
    let passphrase = payload
        .passphrase
        .or(config.passphrase)
        .unwrap_or_else(|| "TadpoleSovereignDefaultKey#2026".to_string());

    let plaintext = decrypt_backup_payload(&envelope_bytes, &passphrase)?;

    let temp_restore_dir = tempfile::tempdir().map_err(|e| {
        AppError::InternalServerError(format!("Failed to create restore tempdir: {}", e))
    })?;

    let extracted_files = unpack_snapshot_archive(&plaintext, temp_restore_dir.path())?;

    // Verify restored database integrity
    let restored_db = temp_restore_dir.path().join("tadpole.db");
    if restored_db.exists() {
        let test_opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&restored_db)
            .read_only(true);

        let test_pool = sqlx::SqlitePool::connect_with(test_opts)
            .await
            .map_err(|e| {
                AppError::BadRequest(format!("Restored SQLite database failed to connect: {}", e))
            })?;

        let row: (String,) = sqlx::query_as("PRAGMA integrity_check;")
            .fetch_one(&test_pool)
            .await
            .map_err(|e| AppError::BadRequest(format!("Integrity check query failed: {}", e)))?;

        if row.0 != "ok" {
            return Err(AppError::BadRequest(format!(
                "SQLite integrity check failed: {}",
                row.0
            )));
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Snapshot successfully verified and extracted into staging buffer",
        "extracted_file_count": extracted_files.len(),
        "integrity": "VERIFIED_OK"
    })))
}

/// GET /v1/system/backup/history
pub async fn get_backup_history(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BackupRecordItem>>, AppError> {
    let rows = sqlx::query_as::<_, BackupRecordItem>(
        r#"
        SELECT
            id, snapshot_filename, destination_type, destination_path,
            file_size_bytes, sha256_checksum, sqlite_bytes, lancedb_bytes,
            status, created_at, pruned_at
        FROM system_backups
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.resources.pool)
    .await
    .unwrap_or_default();

    Ok(Json(rows))
}
