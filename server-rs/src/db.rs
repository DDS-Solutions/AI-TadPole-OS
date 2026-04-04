//! Engine Persistence — SQLite and schema management
//!
//! Handles the lifecycle of the SQLite connection pool, including WAL mode
//! configuration, busy timeouts, and automated schema migrations via `sqlx`.
//!
//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Legacy Hotfixes**: `premark_connector_column_fix_migration_if_needed` handles 
//! cases where the schema was modified out-of-band by early development agents. 
//! This is a one-time "catch-up" mechanism to prevent SQLx from panicing.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: DB locked (busy timeout), migration checksum mismatch, or `DATABASE_URL` path permission error.
//! - **Telemetry Link**: Search for `[Database]` or `[SQLx]` in `tracing` logs.
//! - **Trace Scope**: `server-rs::db`
use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

const CONNECTOR_COLUMN_FIX_MIGRATION_VERSION: i64 = 20260328000100;

/// Initializes the SQLite database pool and executes pending migrations.
///
/// Sets high-performance defaults (WAL mode, busy timeout) and ensures that the
/// backend schema is in sync with the `migrations/` directory.
pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .pragma("synchronous", "NORMAL") // Relax strict fsync for WAL speed
        .pragma("cache_size", "-64000") // Use 64MB of memory for the page cache
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations automatically at startup.
    // SEC: This ensures the schema is always consistent and up-to-date.
    let migrator = sqlx::migrate!("./migrations");
    premark_connector_column_fix_migration_if_needed(&pool, &migrator).await?;
    migrator
        .run(&pool)
        .await
        .expect("❌ Failed to run database migrations");

    tracing::info!("✅ Database migrations applied successfully");

    // Seed baseline entities if the DB is fresh
    seed_baseline_agents(&pool).await?;

    Ok(pool)
}

/// Ensures default agents like 'Agent 1' (Alpha) exist in the system.
async fn seed_baseline_agents(pool: &SqlitePool) -> Result<()> {
    let has_alpha = sqlx::query_scalar::<_, i64>("SELECT 1 FROM agents WHERE id = '1' LIMIT 1")
        .fetch_optional(pool)
        .await?
        .is_some();

    if !has_alpha {
        tracing::info!("🌱 Seeding baseline Agent '1' (Alpha)...");
        sqlx::query(
            "INSERT INTO agents (id, name, role, department, description, status, provider, model_id, theme_color, metadata, skills, workflows, mcp_tools, active_model_slot, category)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("1")
        .bind("Alpha")
        .bind("Agent of Nine")
        .bind("Swarm Core")
        .bind("The primary intelligence node of the Tadpole OS network.")
        .bind("idle")
        .bind("google")
        .bind("gemini-1.5-flash")
        .bind("#4fd1c5")
        .bind("{}") // metadata
        .bind("[]") // skills
        .bind("[]") // workflows
        .bind("[]") // mcp_tools
        .bind(1)    // active_model_slot
        .bind("user")
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Handles legacy hotfixed environments where `connector_configs` was added manually
/// before migration `20260328000100` was introduced.
///
/// In those environments, applying that migration would fail with a duplicate-column
/// error and block startup. If the column is already present, we pre-mark only this
/// migration as applied using the exact embedded checksum so SQLx validation remains intact.
async fn premark_connector_column_fix_migration_if_needed(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<()> {
    // If `agents` table doesn't exist yet, this is likely a fresh DB; let normal migrations run.
    let agents_table_exists = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='agents' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?
    .is_some();
    if !agents_table_exists {
        return Ok(());
    }

    let connector_column_exists = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM pragma_table_info('agents') WHERE name='connector_configs' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?
    .is_some();
    if !connector_column_exists {
        return Ok(());
    }

    // Mirror SQLx SQLite migration table schema so we can safely record the migration.
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);
        "#,
    )
    .execute(pool)
    .await?;

    let already_applied =
        sqlx::query_scalar::<_, i64>("SELECT 1 FROM _sqlx_migrations WHERE version = ?1 LIMIT 1")
            .bind(CONNECTOR_COLUMN_FIX_MIGRATION_VERSION)
            .fetch_optional(pool)
            .await?
            .is_some();
    if already_applied {
        return Ok(());
    }

    let Some(migration) = migrator
        .iter()
        .find(|m| m.version == CONNECTOR_COLUMN_FIX_MIGRATION_VERSION)
    else {
        tracing::warn!(
            "⚠️ Connector fix migration {} not found in embedded migrations; skipping pre-mark",
            CONNECTOR_COLUMN_FIX_MIGRATION_VERSION
        );
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?1, ?2, TRUE, ?3, 0)",
    )
    .bind(migration.version)
    .bind(migration.description.as_ref())
    .bind(migration.checksum.as_ref())
    .execute(pool)
    .await?;

    tracing::warn!(
        "⚠️ Pre-marked migration {} because agents.connector_configs already exists",
        CONNECTOR_COLUMN_FIX_MIGRATION_VERSION
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        premark_connector_column_fix_migration_if_needed, CONNECTOR_COLUMN_FIX_MIGRATION_VERSION,
    };
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn premarks_connector_fix_migration_when_column_exists() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("failed to open sqlite memory database");
        let migrator = sqlx::migrate!("./migrations");

        sqlx::query("CREATE TABLE agents (id TEXT PRIMARY KEY, connector_configs TEXT)")
            .execute(&pool)
            .await
            .expect("failed to create agents table");

        premark_connector_column_fix_migration_if_needed(&pool, &migrator)
            .await
            .expect("premark helper failed");

        let applied: Option<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE version = ?1 LIMIT 1")
                .bind(CONNECTOR_COLUMN_FIX_MIGRATION_VERSION)
                .fetch_optional(&pool)
                .await
                .expect("failed to query _sqlx_migrations");

        assert_eq!(applied, Some(CONNECTOR_COLUMN_FIX_MIGRATION_VERSION));
    }

    #[tokio::test]
    async fn does_not_premark_when_connector_column_missing() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("failed to open sqlite memory database");
        let migrator = sqlx::migrate!("./migrations");

        sqlx::query("CREATE TABLE agents (id TEXT PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("failed to create agents table");

        premark_connector_column_fix_migration_if_needed(&pool, &migrator)
            .await
            .expect("premark helper failed");

        let migrations_table_exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations' LIMIT 1",
        )
        .fetch_optional(&pool)
        .await
        .expect("failed to inspect sqlite_master");

        assert!(
            migrations_table_exists.is_none(),
            "_sqlx_migrations should not be created when connector column is absent"
        );
    }

    #[tokio::test]
    async fn premark_is_idempotent_for_connector_fix_migration() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("failed to open sqlite memory database");
        let migrator = sqlx::migrate!("./migrations");

        sqlx::query("CREATE TABLE agents (id TEXT PRIMARY KEY, connector_configs TEXT)")
            .execute(&pool)
            .await
            .expect("failed to create agents table");

        premark_connector_column_fix_migration_if_needed(&pool, &migrator)
            .await
            .expect("first premark failed");
        premark_connector_column_fix_migration_if_needed(&pool, &migrator)
            .await
            .expect("second premark failed");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ?1")
                .bind(CONNECTOR_COLUMN_FIX_MIGRATION_VERSION)
                .fetch_one(&pool)
                .await
                .expect("failed to count migration rows");

        assert_eq!(count, 1);
    }
}
