//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / backup
//! - **Primary Entrypoints**: `get_backup_status`, `configure_backup`, `trigger_backup_now`, `restore_backup`, `get_backup_history`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared

use crate::{
    error::AppError,
    middleware::auth::RequireAdmin,
    services::backup::{
        crypto::decrypt_backup_payload,
        daemon::{
            has_configured_passphrase, load_backup_config, resolve_backup_passphrase,
            run_backup_cycle, save_backup_config, set_runtime_passphrase,
            validate_backup_directory, validate_snapshot_file,
        },
        snapshot::unpack_snapshot_archive,
    },
    state::AppState,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
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
    /// When true, atomically stages the verified database to `data/state/tadpole.db.restored`
    /// for promotion on next restart; when false (default), runs a verify-only dry-run.
    #[serde(default)]
    pub apply: bool,
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
    _admin: RequireAdmin,
) -> Result<Json<BackupStatusResponse>, AppError> {
    let base_dir = state.base_dir.clone();
    let config = load_backup_config(&base_dir);
    let mut total_snapshots = 0;
    let mut total_size_bytes = 0u64;

    if !config.destination_path.trim().is_empty() {
        if let Ok(p) = validate_backup_directory(&base_dir, &config.destination_path) {
            if let Ok(mut entries) = tokio::fs::read_dir(&p).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                        if name.starts_with("tadpole-snapshot-")
                            && name.ends_with(".tadpole-snap.enc")
                        {
                            total_snapshots += 1;
                            if let Ok(m) = tokio::fs::metadata(&path).await {
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
        destination_type: config.destination_type.clone(),
        destination_path: config.destination_path.clone(),
        has_passphrase: has_configured_passphrase(&config),
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
    _admin: RequireAdmin,
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
    let validated_dir = validate_backup_directory(&state.base_dir, &clean_path)?;
    tokio::fs::create_dir_all(&validated_dir)
        .await
        .map_err(|e| {
            AppError::BadRequest(format!(
                "Cannot write to target directory {}: {}",
                validated_dir.display(),
                e
            ))
        })?;
    current.destination_path = validated_dir.to_string_lossy().to_string();

    if let Some(pass) = payload.passphrase {
        let trimmed = pass.trim();
        if !trimmed.is_empty() {
            if trimmed.chars().count() < 12 {
                return Err(AppError::BadRequest(
                    "Backup passphrase must be at least 12 characters".to_string(),
                ));
            }
            set_runtime_passphrase(pass);
        }
    }
    // Never retain plaintext passphrase inside the serialized config struct
    current.passphrase = None;

    save_backup_config(&state.base_dir, &current)?;
    get_backup_status(State(state), RequireAdmin).await
}

/// POST /v1/system/backup/trigger
pub async fn trigger_backup_now(
    State(state): State<Arc<AppState>>,
    _admin: RequireAdmin,
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

/// Recursively copies a directory tree without following symlinks.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// POST /v1/system/backup/restore
pub async fn restore_backup(
    State(state): State<Arc<AppState>>,
    _admin: RequireAdmin,
    Json(payload): Json<RestoreBackupRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let snapshot_path = validate_snapshot_file(&state.base_dir, &payload.snapshot_path)?;
    let meta = tokio::fs::metadata(&snapshot_path).await.map_err(|_| {
        AppError::NotFound(format!(
            "Snapshot file not found: {}",
            snapshot_path.display()
        ))
    })?;
    if !meta.is_file() {
        return Err(AppError::NotFound(format!(
            "Snapshot path is not a file: {}",
            snapshot_path.display()
        )));
    }

    let envelope_bytes = tokio::fs::read(&snapshot_path).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read snapshot file: {}", e))
    })?;

    let config = load_backup_config(&state.base_dir);
    let passphrase = resolve_backup_passphrase(payload.passphrase.as_deref(), &config)?;

    let plaintext =
        tokio::task::spawn_blocking(move || decrypt_backup_payload(&envelope_bytes, &passphrase))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Decryption worker failed: {}", e))
            })??;

    let temp_restore_dir = tempfile::tempdir().map_err(|e| {
        AppError::InternalServerError(format!("Failed to create restore tempdir: {}", e))
    })?;

    let unpack_root = temp_restore_dir.path().to_path_buf();
    let extracted_files =
        tokio::task::spawn_blocking(move || unpack_snapshot_archive(&plaintext, &unpack_root))
            .await
            .map_err(|e| AppError::InternalServerError(format!("Unpack worker failed: {}", e)))??;

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
        test_pool.close().await;

        if row.0 != "ok" {
            return Err(AppError::BadRequest(format!(
                "SQLite integrity check failed: {}",
                row.0
            )));
        }
    }

    if payload.apply {
        if !restored_db.exists() {
            return Err(AppError::BadRequest(
                "Snapshot archive does not contain tadpole.db to restore".to_string(),
            ));
        }
        let state_dir = state.base_dir.join("data").join("state");
        tokio::fs::create_dir_all(&state_dir)
            .await
            .map_err(AppError::Io)?;
        let staged_tmp =
            state_dir.join(format!("tadpole.db.restored.tmp.{}", uuid::Uuid::new_v4()));
        let staged_final = state_dir.join("tadpole.db.restored");
        tokio::fs::copy(&restored_db, &staged_tmp)
            .await
            .map_err(AppError::Io)?;

        // Stage unpacked LanceDB / workspaces BEFORE committing tadpole.db.restored so a workspace
        // staging failure never leaves an orphaned DB restore without its matching vector store.
        let restored_workspaces = temp_restore_dir.path().join("workspaces");
        let staged_workspaces_path = if restored_workspaces.is_dir() {
            let ws_tmp =
                state_dir.join(format!("workspaces.restored.tmp.{}", uuid::Uuid::new_v4()));
            let ws_final = state_dir.join("workspaces.restored");
            let src_ws = restored_workspaces.clone();
            let dst_ws = ws_tmp.clone();
            let copy_res =
                tokio::task::spawn_blocking(move || copy_dir_recursive(&src_ws, &dst_ws))
                    .await
                    .map_err(|e| {
                        AppError::InternalServerError(format!(
                            "Workspace staging worker failed: {}",
                            e
                        ))
                    })
                    .and_then(|r| r.map_err(AppError::Io));
            if let Err(e) = copy_res {
                let _ = tokio::fs::remove_file(&staged_tmp).await;
                let _ = tokio::fs::remove_dir_all(&ws_tmp).await;
                return Err(e);
            }
            if tokio::fs::try_exists(&ws_final).await.unwrap_or(false) {
                let _ = tokio::fs::remove_dir_all(&ws_final).await;
            }
            if let Err(e) = tokio::fs::rename(&ws_tmp, &ws_final).await {
                let _ = tokio::fs::remove_file(&staged_tmp).await;
                let _ = tokio::fs::remove_dir_all(&ws_tmp).await;
                return Err(AppError::Io(e));
            }
            Some("data/state/workspaces.restored")
        } else {
            None
        };

        if let Err(e) = tokio::fs::rename(&staged_tmp, &staged_final).await {
            let _ = tokio::fs::remove_file(&staged_tmp).await;
            if staged_workspaces_path.is_some() {
                let _ = tokio::fs::remove_dir_all(state_dir.join("workspaces.restored")).await;
            }
            return Err(AppError::Io(e));
        }

        return Ok(Json(serde_json::json!({
            "success": true,
            "mode": "staged_for_restart",
            "applied": true,
            "requires_restart": true,
            "staged_db_path": "data/state/tadpole.db.restored",
            "staged_workspaces_path": staged_workspaces_path,
            "message": "Snapshot verified and atomically staged for promotion on next restart",
            "extracted_file_count": extracted_files.len(),
            "integrity": "VERIFIED_OK"
        })));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "mode": "verify_only",
        "applied": false,
        "requires_restart": false,
        "message": "Snapshot decrypted and verified cleanly (dry-run; pass apply=true to stage restore)",
        "extracted_file_count": extracted_files.len(),
        "integrity": "VERIFIED_OK"
    })))
}

/// GET /v1/system/backup/history
pub async fn get_backup_history(
    State(state): State<Arc<AppState>>,
    _admin: RequireAdmin,
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
    .map_err(AppError::Sqlx)?;

    Ok(Json(rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::backup::crypto::encrypt_backup_payload;
    use crate::services::backup::snapshot::{pack_snapshot_archive, ArchiveFileEntry};

    #[tokio::test]
    async fn test_configure_backup_rejects_arbitrary_host_path_and_never_writes_plaintext_passphrase(
    ) {
        let _guard = crate::services::backup::daemon::tests::PASSPHRASE_TEST_MUTEX
            .lock()
            .await;
        let _env_guard = crate::services::backup::daemon::tests::PassphraseEnvGuard::clear();

        let state = Arc::new(AppState::new_minimal_mock().await);
        let escape = configure_backup(
            State(state.clone()),
            RequireAdmin,
            Json(ConfigureBackupRequest {
                enabled: true,
                destination_type: "local".to_string(),
                destination_path: "../../outside_escape".to_string(),
                passphrase: Some("SecretBackupPass!99".to_string()),
            }),
        )
        .await;
        assert!(matches!(escape, Err(AppError::Forbidden(_))));

        let valid = configure_backup(
            State(state.clone()),
            RequireAdmin,
            Json(ConfigureBackupRequest {
                enabled: true,
                destination_type: "local".to_string(),
                destination_path: "backups/nightly".to_string(),
                passphrase: Some("SecretBackupPass!99".to_string()),
            }),
        )
        .await
        .expect("authorized backup subdir should succeed");
        assert!(valid.0.has_passphrase);

        let cfg_path = crate::services::backup::daemon::get_config_path(&state.base_dir);
        let raw_json = std::fs::read_to_string(cfg_path).expect("config written");
        assert!(
            !raw_json.contains("SecretBackupPass!99"),
            "passphrase must never be persisted in plaintext JSON"
        );
    }

    #[tokio::test]
    async fn test_restore_backup_rejects_traversal_missing_passphrase_and_stages_apply() {
        let _guard = crate::services::backup::daemon::tests::PASSPHRASE_TEST_MUTEX
            .lock()
            .await;
        let _env_guard = crate::services::backup::daemon::tests::PassphraseEnvGuard::clear();

        let state = Arc::new(AppState::new_minimal_mock().await);
        let bad = restore_backup(
            State(state.clone()),
            RequireAdmin,
            Json(RestoreBackupRequest {
                snapshot_path: "../../etc/passwd".to_string(),
                passphrase: Some("RestoreSecret!2026".to_string()),
                apply: false,
            }),
        )
        .await;
        assert!(matches!(bad, Err(AppError::Forbidden(_))));

        // Build a real SQLite database file + workspaces entry to exercise PRAGMA integrity_check & staging
        let temp_src = tempfile::tempdir().unwrap();
        let src_db = temp_src.path().join("source.db");
        let src_opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&src_db)
            .create_if_missing(true);
        let src_pool = sqlx::SqlitePool::connect_with(src_opts).await.unwrap();
        sqlx::query("CREATE TABLE restored_marker (id INTEGER PRIMARY KEY, label TEXT);")
            .execute(&src_pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO restored_marker (id, label) VALUES (1, 'verified');")
            .execute(&src_pool)
            .await
            .unwrap();
        src_pool.close().await;
        let sqlite_bytes = std::fs::read(&src_db).unwrap();

        let backups_dir = state.base_dir.join("backups");
        std::fs::create_dir_all(&backups_dir).unwrap();
        let archive = pack_snapshot_archive(vec![
            ArchiveFileEntry {
                rel_path: "tadpole.db".to_string(),
                data: sqlite_bytes,
            },
            ArchiveFileEntry {
                rel_path: "workspaces/note.txt".to_string(),
                data: b"restored-workspace-ok".to_vec(),
            },
        ]);
        let enc = encrypt_backup_payload(&archive, "RestoreSecret!2026").unwrap();
        let snap_path = backups_dir.join("tadpole-snapshot-test.tadpole-snap.enc");
        std::fs::write(&snap_path, enc).unwrap();

        // Missing passphrase must fail closed with BadRequest
        let missing_pass = restore_backup(
            State(state.clone()),
            RequireAdmin,
            Json(RestoreBackupRequest {
                snapshot_path: "backups/tadpole-snapshot-test.tadpole-snap.enc".to_string(),
                passphrase: None,
                apply: false,
            }),
        )
        .await;
        assert!(matches!(missing_pass, Err(AppError::BadRequest(_))));

        // Verify-only dry-run succeeds without staging
        let verify_res = restore_backup(
            State(state.clone()),
            RequireAdmin,
            Json(RestoreBackupRequest {
                snapshot_path: "backups/tadpole-snapshot-test.tadpole-snap.enc".to_string(),
                passphrase: Some("RestoreSecret!2026".to_string()),
                apply: false,
            }),
        )
        .await
        .expect("verify-only restore should pass");
        assert_eq!(verify_res.0["mode"], "verify_only");
        assert_eq!(verify_res.0["applied"], false);
        assert!(!state
            .base_dir
            .join("data/state/tadpole.db.restored")
            .exists());

        // Apply=true stages both tadpole.db.restored and workspaces.restored
        let apply_res = restore_backup(
            State(state.clone()),
            RequireAdmin,
            Json(RestoreBackupRequest {
                snapshot_path: "backups/tadpole-snapshot-test.tadpole-snap.enc".to_string(),
                passphrase: Some("RestoreSecret!2026".to_string()),
                apply: true,
            }),
        )
        .await
        .expect("apply restore should stage files");
        assert_eq!(apply_res.0["mode"], "staged_for_restart");
        assert_eq!(apply_res.0["applied"], true);
        assert!(state
            .base_dir
            .join("data/state/tadpole.db.restored")
            .exists());
        assert_eq!(
            std::fs::read_to_string(
                state
                    .base_dir
                    .join("data/state/workspaces.restored/note.txt")
            )
            .unwrap(),
            "restored-workspace-ok"
        );
    }
}
