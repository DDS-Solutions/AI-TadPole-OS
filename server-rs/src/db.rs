//! Database Persistence & Migration Engine
//!
//! Handles the lifecycle of the SQLite connection pool, including WAL mode
//! configuration, busy timeouts, and automated schema migrations via `sqlx`.
//!
use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

/// Initializes the SQLite database pool and executes pending migrations.
///
/// Sets high-performance defaults (WAL mode, busy timeout) and ensures that the
/// backend schema is in sync with the `migrations/` directory.
pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations automatically at startup.
    // SEC: This ensures the schema is always consistent and up-to-date.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("❌ Failed to run database migrations");

    tracing::info!("✅ Database migrations applied successfully");

    Ok(pool)
}
