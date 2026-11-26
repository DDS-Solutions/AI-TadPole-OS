//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Services / Backup / Daemon
//! - **Primary Entrypoints**: `run_backup_cycle`, `load_backup_config`, `save_backup_config`, `prune_expired_snapshots`, `validate_backup_directory`, `validate_snapshot_file`, `resolve_backup_passphrase`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[BackupDaemon]`

use super::crypto::{encrypt_backup_payload, generate_recovery_key_manifest};
use super::snapshot::create_state_snapshot;
use crate::error::AppError;
use crate::security::path_guard::validate_path;
use parking_lot::RwLock as SyncRwLock;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use zeroize::Zeroizing;

const CONFIG_FILENAME: &str = "system_backup.json";
const RETENTION_HOURS: i64 = 48;

/// Process-local in-memory encryption passphrase store (never serialized to disk).
static RUNTIME_BACKUP_PASSPHRASE: LazyLock<SyncRwLock<Option<Zeroizing<String>>>> =
    LazyLock::new(|| SyncRwLock::new(None));

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BackupConfig {
    pub enabled: bool,
    pub destination_type: String, // "local" | "cloud_sync"
    pub destination_path: String,
    /// Never serialized to plaintext JSON; deserialization retained only to migrate legacy configs.
    #[serde(default, skip_serializing)]
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

/// Stores the active backup encryption passphrase strictly in memory (zeroized on drop).
pub fn set_runtime_passphrase(passphrase: String) {
    let trimmed = passphrase.trim().to_string();
    let mut guard = RUNTIME_BACKUP_PASSPHRASE.write();
    if trimmed.is_empty() {
        *guard = None;
    } else {
        *guard = Some(Zeroizing::new(trimmed));
    }
}

/// Clears the process-local runtime backup passphrase (used for deterministic test isolation).
#[cfg(test)]
pub fn clear_runtime_passphrase() {
    let mut guard = RUNTIME_BACKUP_PASSPHRASE.write();
    *guard = None;
}

/// Returns true if an encryption passphrase is available in memory, environment, or legacy config.
pub fn has_configured_passphrase(config: &BackupConfig) -> bool {
    if RUNTIME_BACKUP_PASSPHRASE
        .read()
        .as_ref()
        .is_some_and(|p| !p.trim().is_empty())
    {
        return true;
    }
    if std::env::var("TADPOLE_BACKUP_PASSPHRASE")
        .ok()
        .is_some_and(|v| !v.trim().is_empty())
    {
        return true;
    }
    config
        .passphrase
        .as_deref()
        .is_some_and(|p| !p.trim().is_empty())
}

/// Resolves the backup encryption passphrase fail-closed (no hardcoded default key).
pub fn resolve_backup_passphrase(
    explicit: Option<&str>,
    config: &BackupConfig,
) -> Result<Zeroizing<String>, AppError> {
    if let Some(p) = explicit.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(Zeroizing::new(p.to_string()));
    }
    if let Some(p) = RUNTIME_BACKUP_PASSPHRASE.read().as_ref() {
        if !p.trim().is_empty() {
            return Ok(p.clone());
        }
    }
    if let Ok(env_pass) = std::env::var("TADPOLE_BACKUP_PASSPHRASE") {
        let trimmed = env_pass.trim();
        if !trimmed.is_empty() {
            return Ok(Zeroizing::new(trimmed.to_string()));
        }
    }
    if let Some(p) = config
        .passphrase
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Ok(Zeroizing::new(p.to_string()));
    }
    Err(AppError::BadRequest(
        "Backup encryption passphrase is required. Provide a passphrase or set TADPOLE_BACKUP_PASSPHRASE.".to_string(),
    ))
}

/// Collects authorized root directories permitted for backup storage and restore operations.
fn authorized_backup_roots(base_dir: &Path) -> Vec<PathBuf> {
    let mut roots = vec![
        base_dir.join("backups"),
        base_dir.join("data").join("backups"),
    ];
    if let Ok(extra_root) = std::env::var("TADPOLE_BACKUP_ROOT") {
        let trimmed = extra_root.trim();
        if !trimmed.is_empty() {
            roots.push(PathBuf::from(trimmed));
        }
    }
    roots
}

/// Validates that a backup destination directory is confined to an authorized backup root.
/// Prevents arbitrary host filesystem reads/writes.
pub fn validate_backup_directory(base_dir: &Path, raw_path: &str) -> Result<PathBuf, AppError> {
    let trimmed = raw_path.trim();
    let default_root = base_dir.join("backups");
    let _ = std::fs::create_dir_all(&default_root);

    if trimmed.is_empty() {
        return Ok(default_root);
    }
    if trimmed.contains('\0') {
        return Err(AppError::Forbidden(
            "Backup path contains forbidden NUL byte".to_string(),
        ));
    }

    let candidate = Path::new(trimmed);
    if !candidate.is_absolute() {
        let normalized = trimmed.replace('\\', "/");
        let (root, rel) = if let Some(rest) = normalized.strip_prefix("backups/") {
            (base_dir.join("backups"), rest)
        } else if normalized == "backups" {
            return Ok(default_root);
        } else if let Some(rest) = normalized.strip_prefix("data/backups/") {
            let root = base_dir.join("data").join("backups");
            let _ = std::fs::create_dir_all(&root);
            (root, rest)
        } else if normalized == "data/backups" {
            let root = base_dir.join("data").join("backups");
            let _ = std::fs::create_dir_all(&root);
            return Ok(root);
        } else {
            (default_root, normalized.as_str())
        };
        return validate_path(&root, rel).map(|p| p.to_path_buf());
    }

    for root in authorized_backup_roots(base_dir) {
        if !root.exists() {
            continue;
        }
        if let Ok(root_canon) = std::fs::canonicalize(&root) {
            if let Ok(safe) = validate_path(&root_canon, trimmed) {
                return Ok(safe.to_path_buf());
            }
        }
    }

    Err(AppError::Forbidden(
        "Backup path must reside within an authorized backup directory (backups/, data/backups/, or TADPOLE_BACKUP_ROOT).".to_string(),
    ))
}

/// Validates that a restore snapshot file path is confined to an authorized backup root
/// and points to an existing `.tadpole-snap.enc` file.
pub fn validate_snapshot_file(base_dir: &Path, raw_path: &str) -> Result<PathBuf, AppError> {
    let safe_path = validate_backup_directory(base_dir, raw_path)?;
    let name = safe_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if !name.ends_with(".tadpole-snap.enc") {
        return Err(AppError::BadRequest(
            "Snapshot path must reference a '.tadpole-snap.enc' archive inside an authorized backup directory.".to_string(),
        ));
    }
    Ok(safe_path)
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

/// Saves the backup daemon configuration to disk without plaintext passphrases.
pub fn save_backup_config(base_dir: &Path, config: &BackupConfig) -> Result<(), AppError> {
    let path = get_config_path(base_dir);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = serde_json::to_string_pretty(config).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize backup config: {}", e))
    })?;
    let tmp_path = path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
    std::fs::write(&tmp_path, content).map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to write temporary backup config {}: {}",
            tmp_path.display(),
            e
        ))
    })?;
    if let Err(e) = std::fs::rename(&tmp_path, &path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(AppError::InternalServerError(format!(
            "Failed to atomically commit backup config {}: {}",
            path.display(),
            e
        )));
    }
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

    let dest_dir = validate_backup_directory(base_dir, &config.destination_path)?;
    if !dest_dir.exists() {
        tokio::fs::create_dir_all(&dest_dir).await.map_err(|e| {
            AppError::BadRequest(format!(
                "Failed to access or create backup destination directory '{}': {}",
                dest_dir.display(),
                e
            ))
        })?;
    }

    let passphrase = resolve_backup_passphrase(None, config)?;

    // 1. Create Point-In-Time Snapshot
    tracing::info!("🔒 [BackupDaemon] Creating live state snapshot...");
    let bundle = create_state_snapshot(pool, base_dir).await?;

    // 2. Encrypt with ChaCha20 + RFC 2104 HMAC-SHA256 authenticated envelope
    tracing::info!(
        "🔐 [BackupDaemon] Encrypting snapshot bundle ({} bytes)...",
        bundle.raw_archive.len()
    );
    let raw_archive = bundle.raw_archive;
    let pass_for_enc = passphrase.clone();
    let encrypted_payload =
        tokio::task::spawn_blocking(move || encrypt_backup_payload(&raw_archive, &pass_for_enc))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Encryption worker failed: {}", e))
            })??;

    let timestamp_str = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("tadpole-snapshot-{}.tadpole-snap.enc", timestamp_str);
    let target_file_path = dest_dir.join(&filename);

    tokio::fs::write(&target_file_path, &encrypted_payload)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to write encrypted snapshot to '{}': {}",
                target_file_path.display(),
                e
            ))
        })?;

    // Always refresh recovery key manifest atomically so passphrase rotations stay synchronized
    let recovery_key_path = dest_dir.join(".tadpole-recovery.key");
    let pass_for_manifest = passphrase.clone();
    if let Ok(Ok(manifest)) =
        tokio::task::spawn_blocking(move || generate_recovery_key_manifest(&pass_for_manifest))
            .await
    {
        if let Ok(json) = serde_json::to_string_pretty(&manifest) {
            let tmp_manifest = dest_dir.join(format!(
                ".tadpole-recovery.key.tmp.{}",
                uuid::Uuid::new_v4()
            ));
            if tokio::fs::write(&tmp_manifest, json).await.is_ok()
                && tokio::fs::rename(&tmp_manifest, &recovery_key_path)
                    .await
                    .is_err()
            {
                let _ = tokio::fs::remove_file(&tmp_manifest).await;
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

    // 5. Update runtime config state (passphrase is never persisted to JSON)
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

    if let Ok(mut entries) = tokio::fs::read_dir(dest_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if file_name.starts_with("tadpole-snapshot-")
                    && file_name.ends_with(".tadpole-snap.enc")
                {
                    if let Ok(metadata) = tokio::fs::metadata(&path).await {
                        if let Ok(modified) = metadata.modified() {
                            let duration = modified.elapsed().unwrap_or_default();
                            let age_secs = duration.as_secs() as i64;
                            let file_time = chrono::Utc::now().timestamp() - age_secs;

                            if file_time < cutoff && tokio::fs::remove_file(&path).await.is_ok() {
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) static PASSPHRASE_TEST_MUTEX: LazyLock<tokio::sync::Mutex<()>> =
        LazyLock::new(|| tokio::sync::Mutex::new(()));

    /// Panic-safe RAII guard that clears runtime passphrase and restores `TADPOLE_BACKUP_PASSPHRASE` on drop.
    pub(crate) struct PassphraseEnvGuard {
        prev_env: Option<String>,
    }

    impl PassphraseEnvGuard {
        pub(crate) fn clear() -> Self {
            clear_runtime_passphrase();
            let prev_env = std::env::var("TADPOLE_BACKUP_PASSPHRASE").ok();
            std::env::remove_var("TADPOLE_BACKUP_PASSPHRASE");
            Self { prev_env }
        }
    }

    impl Drop for PassphraseEnvGuard {
        fn drop(&mut self) {
            clear_runtime_passphrase();
            if let Some(ref val) = self.prev_env {
                std::env::set_var("TADPOLE_BACKUP_PASSPHRASE", val);
            } else {
                std::env::remove_var("TADPOLE_BACKUP_PASSPHRASE");
            }
        }
    }

    #[test]
    fn test_validate_backup_directory_confines_to_authorized_roots() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        let default_dir = validate_backup_directory(base, "").unwrap();
        assert_eq!(default_dir, base.join("backups"));

        let sub = validate_backup_directory(base, "backups/daily").unwrap();
        assert!(sub.ends_with(Path::new("backups").join("daily")));

        let data_sub = validate_backup_directory(base, "data/backups/nightly").unwrap();
        assert!(data_sub.ends_with(Path::new("data").join("backups").join("nightly")));

        assert!(matches!(
            validate_backup_directory(base, "../../etc/shadow"),
            Err(AppError::Forbidden(_))
        ));
        assert!(matches!(
            validate_backup_directory(base, "/etc/passwd"),
            Err(AppError::Forbidden(_))
        ));
    }

    #[test]
    fn test_validate_snapshot_file_requires_extension_and_confinement() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        let ok = validate_snapshot_file(base, "backups/tadpole-snapshot-1.tadpole-snap.enc");
        assert!(ok.is_ok());

        let bad_ext = validate_snapshot_file(base, "backups/not_a_snapshot.db");
        assert!(matches!(bad_ext, Err(AppError::BadRequest(_))));

        let escape = validate_snapshot_file(base, "../escape.tadpole-snap.enc");
        assert!(matches!(escape, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_resolve_backup_passphrase_fails_closed_without_config() {
        let _guard = PASSPHRASE_TEST_MUTEX.lock().await;
        let _env_guard = PassphraseEnvGuard::clear();

        let cfg = BackupConfig::default();
        let res = resolve_backup_passphrase(None, &cfg);
        assert!(
            matches!(res, Err(AppError::BadRequest(_))),
            "Missing passphrase must fail closed with BadRequest"
        );

        let explicit = resolve_backup_passphrase(Some("StrongPassphrase!2026"), &cfg).unwrap();
        assert_eq!(explicit.as_str(), "StrongPassphrase!2026");
    }

    #[tokio::test]
    async fn test_run_backup_cycle_end_to_end_and_recovery_manifest() {
        let _guard = PASSPHRASE_TEST_MUTEX.lock().await;
        let _env_guard = PassphraseEnvGuard::clear();

        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();
        let state_dir = base.join("data").join("state");
        std::fs::create_dir_all(state_dir.join("workspaces")).unwrap();
        std::fs::write(
            state_dir.join("workspaces").join("memory.txt"),
            b"vector-memory-sample",
        )
        .unwrap();

        let db_path = state_dir.join("tadpole.db");
        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query(
            "CREATE TABLE system_backups (
                id TEXT PRIMARY KEY,
                snapshot_filename TEXT NOT NULL,
                destination_type TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                file_size_bytes INTEGER NOT NULL,
                sha256_checksum TEXT NOT NULL,
                sqlite_bytes INTEGER NOT NULL,
                lancedb_bytes INTEGER NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                pruned_at INTEGER
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        set_runtime_passphrase("EndToEndBackupPass!2026".to_string());
        let mut cfg = BackupConfig {
            enabled: true,
            destination_type: "local".to_string(),
            destination_path: "backups/e2e".to_string(),
            ..Default::default()
        };

        let record = run_backup_cycle(&pool, base, &mut cfg)
            .await
            .expect("backup cycle must succeed");
        assert!(record.snapshot_filename.ends_with(".tadpole-snap.enc"));
        assert!(record.sqlite_bytes > 0);
        assert!(record.lancedb_bytes > 0);
        assert_eq!(cfg.last_status.as_deref(), Some("HEALTHY"));

        let snap_bytes = std::fs::read(&record.destination_path).unwrap();
        let decrypted = crate::services::backup::crypto::decrypt_backup_payload(
            &snap_bytes,
            "EndToEndBackupPass!2026",
        )
        .expect("snapshot must decrypt cleanly");
        assert!(!decrypted.is_empty());

        let manifest_path = base
            .join("backups")
            .join("e2e")
            .join(".tadpole-recovery.key");
        assert!(manifest_path.exists(), "recovery manifest must be created");
        let row_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM system_backups")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row_count.0, 1);

        // Fresh snapshot (< 48h old) must NOT be pruned by prune_expired_snapshots
        let dest_dir = base.join("backups").join("e2e");
        let pruned_fresh = prune_expired_snapshots(&pool, &dest_dir).await;
        assert_eq!(
            pruned_fresh, 0,
            "fresh snapshot must be retained within 48h window"
        );
        assert!(std::path::Path::new(&record.destination_path).exists());

        // Expired snapshot (> 48h old) MUST be deleted from disk and marked status = 'pruned'
        let expired_name = "tadpole-snapshot-20200101-000000.tadpole-snap.enc";
        let expired_path = dest_dir.join(expired_name);
        std::fs::write(&expired_path, b"expired-ciphertext").unwrap();
        let expired_mtime =
            std::time::SystemTime::now() - std::time::Duration::from_secs(72 * 3600);
        std::fs::OpenOptions::new()
            .write(true)
            .open(&expired_path)
            .unwrap()
            .set_modified(expired_mtime)
            .unwrap();
        sqlx::query(
            "INSERT INTO system_backups (id, snapshot_filename, destination_type, destination_path, file_size_bytes, sha256_checksum, sqlite_bytes, lancedb_bytes, status, created_at) \
             VALUES ('bkp-expired-1', ?1, 'local_folder', ?2, 18, 'deadbeef', 10, 8, 'completed', ?3)",
        )
        .bind(expired_name)
        .bind(expired_path.to_string_lossy().to_string())
        .bind(chrono::Utc::now().timestamp() - (72 * 3600))
        .execute(&pool)
        .await
        .unwrap();

        let pruned_expired = prune_expired_snapshots(&pool, &dest_dir).await;
        assert_eq!(pruned_expired, 1, "expired snapshot >48h must be pruned");
        assert!(
            !expired_path.exists(),
            "expired snapshot file must be removed from disk"
        );
        assert!(
            std::path::Path::new(&record.destination_path).exists(),
            "fresh snapshot must remain untouched when pruning expired files"
        );
        let expired_status: (String,) =
            sqlx::query_as("SELECT status FROM system_backups WHERE snapshot_filename = ?1")
                .bind(expired_name)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(expired_status.0, "pruned");

        pool.close().await;
    }

    #[test]
    fn test_save_backup_config_never_serializes_passphrase() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = BackupConfig {
            enabled: true,
            destination_type: "local".to_string(),
            destination_path: "backups/local".to_string(),
            passphrase: Some("MustNeverBeWritten!2026".to_string()),
            ..Default::default()
        };
        save_backup_config(tmp.path(), &cfg).unwrap();
        let path = get_config_path(tmp.path());
        let raw = std::fs::read_to_string(path).unwrap();
        assert!(!raw.contains("MustNeverBeWritten"));
    }
}
