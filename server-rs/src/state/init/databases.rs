//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init Databases
//! - **Primary Entrypoints**: `init_database_pool`, `resolve_canonical_database_url`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[Database]`

use crate::error::AppError;
use std::path::{Path, PathBuf};

/// Promotes verified staged restore artifacts (`tadpole.db.restored` and `workspaces.restored`)
/// into the active target paths before SQLite opens the database.
/// Preserves existing DB + WAL/SHM into `.pre-restore.<ts>` backups and removes stale WAL/SHM
/// strictly AFTER promotion succeeds.
pub fn promote_staged_restore_if_present(target_db: &Path, base_dir: &Path) {
    let state_dir = base_dir.join("data").join("state");
    let canonical_staged = state_dir.join("tadpole.db.restored");
    let sibling_staged = target_db.with_file_name("tadpole.db.restored");

    let staged_restore = if canonical_staged.exists() {
        Some(canonical_staged)
    } else if sibling_staged.exists() {
        Some(sibling_staged)
    } else {
        None
    };

    if let Some(staged_path) = staged_restore {
        if let Some(parent) = target_db.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let ts = chrono::Utc::now().timestamp();
        let db_name = target_db
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("tadpole.db");
        let wal_path = target_db.with_file_name(format!("{}-wal", db_name));
        let shm_path = target_db.with_file_name(format!("{}-shm", db_name));

        // Backup existing DB + WAL/SHM before replacing
        if target_db.exists() {
            let backup_db = target_db.with_file_name(format!("{}.pre-restore.{}", db_name, ts));
            let _ = std::fs::copy(target_db, &backup_db);
            if wal_path.exists() {
                let _ = std::fs::copy(
                    &wal_path,
                    target_db.with_file_name(format!("{}-wal.pre-restore.{}", db_name, ts)),
                );
            }
            if shm_path.exists() {
                let _ = std::fs::copy(
                    &shm_path,
                    target_db.with_file_name(format!("{}-shm.pre-restore.{}", db_name, ts)),
                );
            }
        }

        // Promote atomically (fallback to copy+remove if across mount boundaries)
        let promoted = if std::fs::rename(&staged_path, target_db).is_ok() {
            true
        } else if std::fs::copy(&staged_path, target_db).is_ok() {
            let _ = std::fs::remove_file(&staged_path);
            true
        } else {
            false
        };

        if promoted {
            // Remove stale WAL/SHM only AFTER the restored DB file is safely in place
            let _ = std::fs::remove_file(&wal_path);
            let _ = std::fs::remove_file(&shm_path);
            tracing::info!(
                "🔄 [Database] Promoted verified staged backup restore to {}",
                target_db.display()
            );
        } else {
            tracing::error!(
                "🚨 [Database] Failed to promote staged backup restore from {} to {}; existing database preserved.",
                staged_path.display(),
                target_db.display()
            );
        }
    }

    // Also promote staged LanceDB / workspaces restore if present, rolling back on rename failure
    let staged_workspaces = state_dir.join("workspaces.restored");
    if staged_workspaces.is_dir() {
        let active_workspaces = state_dir.join("workspaces");
        let backup_ws = if active_workspaces.exists() {
            let candidate = state_dir.join(format!(
                "workspaces.pre-restore.{}",
                chrono::Utc::now().timestamp()
            ));
            if std::fs::rename(&active_workspaces, &candidate).is_ok() {
                Some(candidate)
            } else {
                tracing::error!(
                    "🚨 [Database] Failed to move existing workspaces aside before restore promotion; keeping existing workspaces."
                );
                return;
            }
        } else {
            None
        };
        if let Err(e) = std::fs::rename(&staged_workspaces, &active_workspaces) {
            tracing::error!(
                "🚨 [Database] Failed to promote staged workspaces restore ({}): rolling back previous workspaces.",
                e
            );
            if let Some(prev) = backup_ws {
                let _ = std::fs::rename(&prev, &active_workspaces);
            }
        } else {
            tracing::info!(
                "🔄 [Database] Promoted verified staged workspaces restore to {}",
                active_workspaces.display()
            );
        }
    }
}

/// Helper to migrate legacy database files to `data/state/` and promote any staged restore.
fn migrate_legacy_db_if_needed(base_dir: &Path) -> PathBuf {
    let state_dir = base_dir.join("data").join("state");
    let state_db = state_dir.join("tadpole.db");
    promote_staged_restore_if_present(&state_db, base_dir);
    if state_db.exists() {
        return state_db;
    }

    let legacy_db = base_dir.join("data").join("tadpole.db");
    if legacy_db.exists() {
        let _ = std::fs::create_dir_all(&state_dir);
        // Atomically or safely copy legacy database and WAL artifacts to state dir
        if std::fs::copy(&legacy_db, &state_db).is_ok() {
            let legacy_wal = base_dir.join("data").join("tadpole.db-wal");
            let state_wal = state_dir.join("tadpole.db-wal");
            if legacy_wal.exists() {
                let _ = std::fs::copy(&legacy_wal, &state_wal);
            }
            let legacy_shm = base_dir.join("data").join("tadpole.db-shm");
            let state_shm = state_dir.join("tadpole.db-shm");
            if legacy_shm.exists() {
                let _ = std::fs::copy(&legacy_shm, &state_shm);
            }
            tracing::info!(
                "📦 [Database] Migrated legacy tadpole.db to durable location: {}",
                state_db.display()
            );
            return state_db;
        }
    }

    let _ = std::fs::create_dir_all(&state_dir);
    state_db
}

/// Resolves a database URL canonically relative to `base_dir`.
///
/// If `raw_url` is unset or empty, resolves to `base_dir/data/state/tadpole.db`.
/// If `raw_url` is a relative path (e.g. `sqlite:tadpole.db` or `sqlite:data/state/tadpole.db`),
/// it is anchored to `base_dir/data/state/tadpole.db` (for `tadpole.db`) or `base_dir.join(rel)`.
/// If in-memory (`sqlite::memory:`), it is preserved untouched.
pub fn resolve_canonical_database_url(base_dir: &Path, raw_url: Option<&str>) -> String {
    let raw = match raw_url {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => {
            let canonical_db = migrate_legacy_db_if_needed(base_dir);
            return format!("sqlite:{}", canonical_db.display());
        }
    };

    if raw.starts_with("sqlite::memory:") {
        return raw.to_string();
    }

    // Split query parameters if any
    let (url_without_query, query_part) = match raw.split_once('?') {
        Some((u, q)) => (u, Some(q)),
        None => (raw, None),
    };

    let stripped_path_str = if let Some(stripped) = url_without_query.strip_prefix("sqlite://") {
        stripped
    } else if let Some(stripped) = url_without_query.strip_prefix("sqlite:") {
        stripped
    } else {
        url_without_query
    };

    let path = Path::new(stripped_path_str);
    let resolved_path: PathBuf = if path.is_relative() {
        // Special-case bare "tadpole.db" or "./tadpole.db" -> resolve to base_dir/data/state/tadpole.db
        if stripped_path_str == "tadpole.db"
            || stripped_path_str == "./tadpole.db"
            || stripped_path_str == ".\\tadpole.db"
        {
            migrate_legacy_db_if_needed(base_dir)
        } else {
            base_dir.join(path)
        }
    } else {
        path.to_path_buf()
    };

    if let Some(parent) = resolved_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Ensure verified staged restores promote even when DATABASE_URL is explicitly configured
    // (e.g., sqlite://data/state/tadpole.db or sqlite:/app/data/state/tadpole.db)
    promote_staged_restore_if_present(&resolved_path, base_dir);

    match query_part {
        Some(q) => format!("sqlite:{}?{}", resolved_path.display(), q),
        None => format!("sqlite:{}", resolved_path.display()),
    }
}

pub async fn init_database_pool(base_dir: &Path) -> Result<sqlx::SqlitePool, AppError> {
    let database_url = if cfg!(test) {
        std::env::var("DATABASE_URL")
            .ok()
            .map(|u| resolve_canonical_database_url(base_dir, Some(&u)))
            .unwrap_or_else(|| "sqlite::memory:".to_string())
    } else {
        let env_url = std::env::var("DATABASE_URL").ok();
        resolve_canonical_database_url(base_dir, env_url.as_deref())
    };

    tracing::info!("🗄️ [Database] Connecting to: {}", database_url);
    match crate::db::init_db(&database_url).await {
        Ok(p) => {
            tracing::info!("✅ [Database] Pool established successfully.");
            Ok(p)
        }
        Err(e) => {
            tracing::error!(
                "🚨 [Database] FATAL: Failed to initialize database pool at {}: {:?}",
                database_url,
                e
            );
            Err(AppError::from(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_resolve_canonical_database_url() {
        let base = PathBuf::from("/mock/root");

        // Unset / None -> data/state/tadpole.db
        let default_url = resolve_canonical_database_url(&base, None);
        assert!(
            default_url.ends_with("data/state/tadpole.db")
                || default_url.ends_with("data\\state\\tadpole.db")
        );

        // Memory preserved
        let memory_url = resolve_canonical_database_url(&base, Some("sqlite::memory:"));
        assert_eq!(memory_url, "sqlite::memory:");

        // Bare tadpole.db redirected to data/state/tadpole.db
        let bare_url = resolve_canonical_database_url(&base, Some("sqlite:tadpole.db"));
        assert!(
            bare_url.ends_with("data/state/tadpole.db")
                || bare_url.ends_with("data\\state\\tadpole.db")
        );

        // Relative path preserved with query
        let query_url = resolve_canonical_database_url(
            &base,
            Some("sqlite:data/state/tadpole.db?skip_seed=true"),
        );
        assert!(query_url.contains("data") && query_url.contains("tadpole.db?skip_seed=true"));
    }

    #[test]
    fn test_promote_staged_restore_works_for_explicit_database_urls_and_workspaces() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();
        let state_dir = base.join("data").join("state");
        std::fs::create_dir_all(&state_dir).unwrap();

        let active_db = state_dir.join("tadpole.db");
        let wal = state_dir.join("tadpole.db-wal");
        let staged_db = state_dir.join("tadpole.db.restored");
        std::fs::write(&active_db, b"old-db-bytes").unwrap();
        std::fs::write(&wal, b"stale-wal").unwrap();
        std::fs::write(&staged_db, b"new-restored-db").unwrap();

        let staged_ws = state_dir.join("workspaces.restored");
        std::fs::create_dir_all(&staged_ws).unwrap();
        std::fs::write(staged_ws.join("index.bin"), b"restored-vectors").unwrap();

        // Explicit sqlite://data/state/tadpole.db (matching .env.example) MUST still promote!
        let resolved = resolve_canonical_database_url(base, Some("sqlite://data/state/tadpole.db"));
        assert!(resolved.ends_with("tadpole.db"));
        assert_eq!(std::fs::read(&active_db).unwrap(), b"new-restored-db");
        assert!(!staged_db.exists());
        assert!(!wal.exists(), "stale WAL must be removed after promotion");
        assert_eq!(
            std::fs::read(state_dir.join("workspaces").join("index.bin")).unwrap(),
            b"restored-vectors"
        );
        assert!(!staged_ws.exists());

        // Pre-restore backup of old DB must exist
        let has_pre_restore = std::fs::read_dir(&state_dir).unwrap().flatten().any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("tadpole.db.pre-restore.")
        });
        assert!(has_pre_restore, "pre-restore DB backup must be preserved");
    }
}
