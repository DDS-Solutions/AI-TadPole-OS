//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init Databases
//! - **Primary Entrypoints**: `init_database_pool`, `resolve_canonical_database_url`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: `[Database]`
//! - **Witness Tests**: `tests::test_resolve_canonical_database_url`

use crate::error::AppError;
use std::path::{Path, PathBuf};

/// Resolves a database URL canonically relative to `base_dir`.
///
/// If `raw_url` is unset or empty, resolves to `base_dir/data/tadpole.db`.
/// If `raw_url` is a relative path (e.g. `sqlite:tadpole.db` or `sqlite:data/tadpole.db`),
/// it is anchored to `base_dir/data/tadpole.db` (for `tadpole.db`) or `base_dir.join(rel)`.
/// If in-memory (`sqlite::memory:`), it is preserved untouched.
pub fn resolve_canonical_database_url(base_dir: &Path, raw_url: Option<&str>) -> String {
    let raw = match raw_url {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => {
            let data_dir = base_dir.join("data");
            let _ = std::fs::create_dir_all(&data_dir);
            let canonical_db = data_dir.join("tadpole.db");
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
        // Special-case bare "tadpole.db" or "./tadpole.db" -> resolve to base_dir/data/tadpole.db
        if stripped_path_str == "tadpole.db"
            || stripped_path_str == "./tadpole.db"
            || stripped_path_str == ".\\tadpole.db"
        {
            base_dir.join("data").join("tadpole.db")
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

        // Unset / None -> data/tadpole.db
        let default_url = resolve_canonical_database_url(&base, None);
        assert!(
            default_url.ends_with("data/tadpole.db") || default_url.ends_with("data\\tadpole.db")
        );

        // Memory preserved
        let memory_url = resolve_canonical_database_url(&base, Some("sqlite::memory:"));
        assert_eq!(memory_url, "sqlite::memory:");

        // Bare tadpole.db redirected to data/tadpole.db
        let bare_url = resolve_canonical_database_url(&base, Some("sqlite:tadpole.db"));
        assert!(bare_url.ends_with("data/tadpole.db") || bare_url.ends_with("data\\tadpole.db"));

        // Relative path preserved with query
        let query_url =
            resolve_canonical_database_url(&base, Some("sqlite:data/tadpole.db?skip_seed=true"));
        assert!(query_url.contains("data") && query_url.contains("tadpole.db?skip_seed=true"));
    }
}
