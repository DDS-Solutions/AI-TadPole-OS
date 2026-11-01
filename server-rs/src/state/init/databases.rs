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

/// Helper to ensure legacy database files are safely migrated to `data/state/` if present.
fn migrate_legacy_db_if_needed(base_dir: &Path) -> PathBuf {
    let state_dir = base_dir.join("data").join("state");
    let state_db = state_dir.join("tadpole.db");
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
}
