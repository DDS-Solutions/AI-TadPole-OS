//! Database Pool Initialization & Connection Management
//!
//! Handles SQLite connection pool creation, WAL mode configuration,
//! and high-performance connection settings for Tadpole OS.
//!
//! ### AI Assist Note
//! **Database Initialization**: Configures SQLite WAL mode and executes migrations.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Database locking, migration failure, or I/O error.
//! - **Telemetry Link**: Search `[Database]` in tracing logs.

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
    let mut clean_url = database_url.to_string();
    let mut skip_seed_from_url = false;

    if let Ok(mut parsed_url) = url::Url::parse(database_url) {
        let mut has_skip_seed = false;
        let query_pairs: Vec<(String, String)> = parsed_url
            .query_pairs()
            .filter_map(|(k, v)| {
                if k.eq_ignore_ascii_case("skip_seed") {
                    if v.eq_ignore_ascii_case("true") {
                        has_skip_seed = true;
                    }
                    None
                } else {
                    Some((k.into_owned(), v.into_owned()))
                }
            })
            .collect();

        if has_skip_seed {
            skip_seed_from_url = true;
            if query_pairs.is_empty() {
                parsed_url.set_query(None);
            } else {
                let mut serializer = url::form_urlencoded::Serializer::new(String::new());
                for (k, v) in query_pairs {
                    serializer.append_pair(&k, &v);
                }
                parsed_url.set_query(Some(&serializer.finish()));
            }
            clean_url = parsed_url.to_string();
        }
    } else {
        let lower_url = database_url.to_lowercase();
        if lower_url.contains("skip_seed=true") {
            skip_seed_from_url = true;
            if let Some(idx) = lower_url.find("skip_seed=true") {
                let mut stripped = database_url.to_string();
                if idx > 0 {
                    let prev_char = lower_url.chars().nth(idx - 1);
                    if prev_char == Some('?') || prev_char == Some('&') {
                        stripped.remove(idx - 1);
                        stripped.replace_range((idx - 1)..(idx - 1 + "skip_seed=true".len()), "");
                    } else {
                        stripped.replace_range(idx..(idx + "skip_seed=true".len()), "");
                    }
                } else {
                    stripped.replace_range(idx..(idx + "skip_seed=true".len()), "");
                }
                clean_url = stripped;
            }
        }
    }

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

// Metadata: [db_init]
