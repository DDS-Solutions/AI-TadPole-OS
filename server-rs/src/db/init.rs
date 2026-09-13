//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / init
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Database]`
//! - **Witness Tests**: none declared

use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

use crate::db::migrations::run_migrations;
use crate::db::seed::seed_default_data;

/// Initializes the SQLite database pool and executes pending migrations.
///
/// Sets high-performance defaults (WAL mode, busy timeout) and ensures that the
/// backend schema is in sync with the `migrations/` directory.
pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    let (clean_url, skip_seed_from_url) = strip_skip_seed_param(database_url);

    let options = SqliteConnectOptions::from_str(&clean_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .pragma("synchronous", "NORMAL") // Relax strict fsync for WAL speed
        .pragma("cache_size", "-64000") // Use 64MB of memory for the page cache
        .pragma("temp_store", "memory") // Keep temp tables in RAM
        .pragma("mmap_size", "268435456") // Memory-map 256MB for ultra-fast reads
        .pragma("busy_timeout", "10000") // Wait up to 10s if DB is locked
        .pragma("foreign_keys", "ON");

    let pool = SqlitePool::connect_with(options).await?;

    // Run schema migrations
    run_migrations(&pool).await?;

    // Seed default data unless explicitly skipped
    let skip_seed = skip_seed_from_url
        || std::env::var("SKIP_DB_SEED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

    if !skip_seed {
        seed_default_data(&pool).await?;
    }

    // Hydrate paired companion devices from SQLite (RED-06)
    if let Err(e) = crate::routes::remote::load_paired_devices(&pool).await {
        tracing::warn!(
            "⚠️ [Database] Failed to hydrate paired remote devices: {}",
            e
        );
    }

    tracing::info!("✅ [Database] Connection pool initialized & migrations verified.");
    Ok(pool)
}

/// Executes a passive SQLite WAL checkpoint to keep log file size small during high write throughput.
#[allow(dead_code)]
pub async fn checkpoint_wal(pool: &SqlitePool) -> Result<()> {
    sqlx::query("PRAGMA wal_checkpoint(PASSIVE);")
        .execute(pool)
        .await?;
    Ok(())
}

/// Helper to extract `skip_seed` parameter from a SQLite database URL and return `(clean_url, skip_seed)`.
///
/// Reliably extracts `skip_seed=true` or `skip_seed=1` from anywhere in query strings (start, middle, end)
/// without fragile string manipulation or byte-indexing panics.
pub fn strip_skip_seed_param(database_url: &str) -> (String, bool) {
    if let Some((base, query)) = database_url.split_once('?') {
        let mut skip_seed = false;
        let mut remaining_params = Vec::new();
        for param in query.split('&') {
            if let Some((k, v)) = param.split_once('=') {
                if k.eq_ignore_ascii_case("skip_seed") {
                    if v.eq_ignore_ascii_case("true") || v == "1" {
                        skip_seed = true;
                    }
                    continue;
                }
            } else if param.eq_ignore_ascii_case("skip_seed") {
                skip_seed = true;
                continue;
            }
            if !param.is_empty() {
                remaining_params.push(param);
            }
        }
        if skip_seed {
            let clean = if remaining_params.is_empty() {
                base.to_string()
            } else {
                format!("{}?{}", base, remaining_params.join("&"))
            };
            return (clean, true);
        }
    }
    (database_url.to_string(), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_skip_seed_no_query() {
        let (clean, skip) = strip_skip_seed_param("sqlite::memory:");
        assert_eq!(clean, "sqlite::memory:");
        assert!(!skip);
    }

    #[test]
    fn test_strip_skip_seed_only_param() {
        let (clean, skip) = strip_skip_seed_param("sqlite:test.db?skip_seed=true");
        assert_eq!(clean, "sqlite:test.db");
        assert!(skip);
    }

    #[test]
    fn test_strip_skip_seed_first_param() {
        let (clean, skip) = strip_skip_seed_param("sqlite:test.db?skip_seed=true&mode=rwc");
        assert_eq!(clean, "sqlite:test.db?mode=rwc");
        assert!(skip);
    }

    #[test]
    fn test_strip_skip_seed_middle_param() {
        let (clean, skip) =
            strip_skip_seed_param("sqlite:test.db?cache=shared&skip_seed=1&mode=rwc");
        assert_eq!(clean, "sqlite:test.db?cache=shared&mode=rwc");
        assert!(skip);
    }

    #[test]
    fn test_strip_skip_seed_last_param() {
        let (clean, skip) = strip_skip_seed_param("sqlite:test.db?cache=shared&skip_seed=true");
        assert_eq!(clean, "sqlite:test.db?cache=shared");
        assert!(skip);
    }

    #[test]
    fn test_strip_skip_seed_case_insensitive() {
        let (clean, skip) = strip_skip_seed_param("sqlite:test.db?SKIP_SEED=TRUE");
        assert_eq!(clean, "sqlite:test.db");
        assert!(skip);
    }
}
