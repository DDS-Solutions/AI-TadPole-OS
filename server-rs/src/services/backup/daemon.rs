//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Services / Backup / Daemon
//! - **Primary Entrypoints**: `run_backup_cycle`, `start_backup_daemon`, `load_backup_config`, `save_backup_config`, `prune_expired_snapshots`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[BackupDaemon]`

use super::crypto::{encrypt_backup_payload, generate_recovery_key_manifest};
use super::snapshot::create_state_snapshot;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{watch, RwLock};

const CONFIG_FILENAME: &str = "system_backup.json";
const RETENTION_HOURS: i64 = 48;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BackupConfig {
    pub enabled: bool,
    pub destination_type: String, // "local" | "cloud_sync"
    pub destination_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
    pub last_run_timestamp: Option<i64>,
    pub last_status: Option<String>,
    pub total_snapshots: u32,
    pub latest_size_bytes: u64,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            destination_type: "local".to_string(),
            destination_path: String::new(),
            passphrase: None,
            last_run_timestamp: None,
            last_status: None,
            total_snapshots: 0,
            latest_size_bytes: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupExecutionRecord {
    pub id: String,
    pub snapshot_filename: String,
    pub file_size_bytes: u64,
    pub sha256_checksum: String,
    pub sqlite_bytes: u64,
    pub lancedb_bytes: u64,
    pub destination_path: String,
    pub created_at: i64,
}

/// Resolves the canonical backup configuration file path.
pub fn get_config_path(base_dir: &Path) -> PathBuf {
    let state_cfg = base_dir.join("data").join("state").join(CONFIG_FILENAME);
    if state_cfg.parent().map(|p| p.exists()).unwrap_or(false) {
        state_cfg
    } else {
        base_dir.join("data").join(CONFIG_FILENAME)
    }
}

/// Loads the persistent backup daemon configuration from disk.
pub fn load_backup_config(base_dir: &Path) -> BackupConfig {
    let path = get_config_path(base_dir);
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<BackupConfig>(&content) {
                return cfg;
            }
        }
    }
    BackupConfig::default()
}

/// Saves the backup daemon configuration to disk.
pub fn save_backup_config(base_dir: &Path, config: &BackupConfig) -> Result<(), AppError> {
    let path = get_config_path(base_dir);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = serde_json::to_string_pretty(config).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize backup config: {}", e))
    })?;
    std::fs::write(&path, content).map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to write backup config {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}

/// Executes a single, deterministic backup cycle.
pub async fn run_backup_cycle(
    pool: &SqlitePool,
    base_dir: &Path,
    config: &mut BackupConfig,
) -> Result<BackupExecutionRecord, AppError> {
    if config.destination_path.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Backup destination path is not configured".to_string(),
        ));
    }

    let dest_dir = PathBuf::from(&config.destination_path);
    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir).map_err(|e| {
            AppError::BadRequest(format!(
                "Failed to access or create backup destination directory '{}': {}",
                dest_dir.display(),
                e
            ))
        })?;
    }

    let passphrase = config
        .passphrase
        .as_deref()
        .unwrap_or("TadpoleSovereignDefaultKey#2026");

    // 1. Create Point-In-Time Snapshot
    tracing::info!("🔒 [BackupDaemon] Creating live state snapshot...");
    let bundle = create_state_snapshot(pool, base_dir).await?;

    // 2. Encrypt with AES-256 / ChaCha20 authenticated envelope
    tracing::info!(
        "🔐 [BackupDaemon] Encrypting snapshot bundle ({} bytes)...",
        bundle.raw_archive.len()
    );
    let encrypted_payload = encrypt_backup_payload(&bundle.raw_archive, passphrase)?;

    let timestamp_str = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("tadpole-snapshot-{}.tadpole-snap.enc", timestamp_str);
    let target_file_path = dest_dir.join(&filename);

    std::fs::write(&target_file_path, &encrypted_payload).map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to write encrypted snapshot to '{}': {}",
            target_file_path.display(),
            e
        ))
    })?;

    // Save recovery key manifest if not present
    let recovery_key_path = dest_dir.join(".tadpole-recovery.key");
    if !recovery_key_path.exists() {
        if let Ok(manifest) = generate_recovery_key_manifest(passphrase) {
            if let Ok(json) = serde_json::to_string_pretty(&manifest) {
                let _ = std::fs::write(&recovery_key_path, json);
            }
        }
    }

    let backup_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let file_size = encrypted_payload.len() as u64;

    // 3. Persist record into SQLite audit table
    let dest_path_str = target_file_path.to_string_lossy().to_string();
    let _ = sqlx::query(
        r#"
        INSERT INTO system_backups (
            id, snapshot_filename, destination_type, destination_path,
            file_size_bytes, sha256_checksum, sqlite_bytes, lancedb_bytes,
            status, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'completed', ?9)
        "#,
    )
    .bind(&backup_id)
    .bind(&filename)
    .bind(&config.destination_type)
    .bind(&dest_path_str)
    .bind(file_size as i64)
    .bind(&bundle.sha256_checksum)
    .bind(bundle.sqlite_bytes as i64)
    .bind(bundle.lancedb_bytes as i64)
    .bind(now)
    .execute(pool)
    .await;

    // 4. Prune snapshots older than 48 hours
    let pruned_count = prune_expired_snapshots(pool, &dest_dir).await;
    if pruned_count > 0 {
        tracing::info!(
            "🧹 [BackupDaemon] Pruned {} snapshots older than {}h",
            pruned_count,
            RETENTION_HOURS
        );
    }

    // 5. Update runtime config state
    config.last_run_timestamp = Some(now);
    config.last_status = Some("HEALTHY".to_string());
    config.latest_size_bytes = file_size;
    let _ = save_backup_config(base_dir, config);

    tracing::info!(
        "✅ [BackupDaemon] Snapshot complete: {} ({} bytes)",
        filename,
        file_size
    );

    Ok(BackupExecutionRecord {
        id: backup_id,
        snapshot_filename: filename,
        file_size_bytes: file_size,
        sha256_checksum: bundle.sha256_checksum,
        sqlite_bytes: bundle.sqlite_bytes,
        lancedb_bytes: bundle.lancedb_bytes,
        destination_path: target_file_path.to_string_lossy().to_string(),
        created_at: now,
    })
}

/// Prunes snapshot files older than 48 hours from the destination directory and updates the database.
pub async fn prune_expired_snapshots(pool: &SqlitePool, dest_dir: &Path) -> usize {
    let mut pruned = 0;
    let cutoff = chrono::Utc::now().timestamp() - (RETENTION_HOURS * 3600);

    if let Ok(entries) = std::fs::read_dir(dest_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if file_name.starts_with("tadpole-snapshot-")
                    && file_name.ends_with(".tadpole-snap.enc")
                {
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let duration = modified.elapsed().unwrap_or_default();
                            let age_secs = duration.as_secs() as i64;
                            let file_time = chrono::Utc::now().timestamp() - age_secs;

                            if file_time < cutoff && std::fs::remove_file(&path).is_ok() {
                                pruned += 1;
                                let now = chrono::Utc::now().timestamp();
                                let _ = sqlx::query(
                                    "UPDATE system_backups SET status = 'pruned', pruned_at = ?1 WHERE snapshot_filename = ?2",
                                )
                                .bind(now)
                                .bind(file_name)
                                .execute(pool)
                                .await;
                            }
                        }
                    }
                }
            }
        }
    }
    pruned
}

/// Starts the hourly background backup daemon runner.
pub fn start_backup_daemon(
    pool: SqlitePool,
    base_dir: PathBuf,
    config_state: Arc<RwLock<BackupConfig>>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        tracing::info!(
            "🛡️ [BackupDaemon] Hourly automated state backup daemon initialized (48h retention)."
        );
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [BackupDaemon] Graceful shutdown signal received.");
                        break;
                    }
                }
                _ = interval.tick() => {
                    let mut cfg = config_state.write().await;
                    if cfg.enabled && !cfg.destination_path.trim().is_empty() {
                        tracing::info!("⏰ [BackupDaemon] Executing scheduled hourly backup cycle...");
                        if let Err(e) = run_backup_cycle(&pool, &base_dir, &mut cfg).await {
                            tracing::error!("🚨 [BackupDaemon] Hourly backup failed: {:?}", e);
                            cfg.last_status = Some(format!("FAILED: {}", e));
                        }
                    }
                }
            }
        }
    })
}
