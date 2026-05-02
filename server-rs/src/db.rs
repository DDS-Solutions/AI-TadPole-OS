//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Engine Persistence (Database Layer)**: Orchestrates the lifecycle
//! of the **SQLite** connection pool and schema management for the
//! Tadpole OS engine. Features **WAL (Write-Ahead Logging)** and
//! `synchronous=NORMAL`: optimized for the high-throughput state
//! updates required by autonomous agent swarms. Implements **Automated
//! Migrations** via `sqlx` and a **Hotfix Reconciler**: handles
//! out-of-band schema modifications (e.g., manual column additions) to
//! prevent migration checksum failures during system boot. AI agents
//! should monitor the `_sqlx_migrations` table to verify schema
//! integrity before executing complex data operations (DB-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Database locked (`SQLITE_BUSY`) during
//!   bursty write operations, migration checksum mismatches due to
//!   manual tampering, or path permission errors on the
//!   `DATABASE_URL` target.
//! - **Telemetry Link**: Search for `[Database]` or `[SQLx]` in
//!   `tracing` logs for query performance and migration status.
//! - **Trace Scope**: `server-rs::db`
use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

const CONNECTOR_COLUMN_FIX_MIGRATION_VERSION: i64 = 20260328000100;
const CREATED_AT_FIX_MIGRATION_VERSION: i64 = 20260405000100;
const CURRENT_TASK_FIX_MIGRATION_VERSION: i64 = 20260405000200;

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
        .pragma("temp_store", "memory") // Keep temp tables in RAM
        .pragma("mmap_size", "268435456") // 256MB memory-mapped I/O
        .busy_timeout(std::time::Duration::from_secs(10)); // Raised for bursty agent writes

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(25)
        .min_connections(3)
        .acquire_timeout(std::time::Duration::from_secs(8))
        .connect_with(options)
        .await?;

    // Run migrations automatically at startup.
    // SEC: This ensures the schema is always consistent and up-to-date.
    let migrator = sqlx::migrate!("./migrations");

    // Fresh DB detection: if migrations table doesn't exist, we skip hotfix pre-marking
    // to save overhead on every unit test boot.
    let is_fresh = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations' LIMIT 1",
    )
    .fetch_optional(&pool)
    .await?
    .is_none();

    if !is_fresh || !database_url.contains(":memory:") {
        premark_connector_column_fix_migration_if_needed(&pool, &migrator).await?;
        premark_created_at_fix_migration_if_needed(&pool, &migrator).await?;
        premark_current_task_fix_migration_if_needed(&pool, &migrator).await?;
    }

    migrator
        .run(&pool)
        .await
        .expect("❌ Failed to run database migrations");

    tracing::info!("✅ Database migrations applied successfully");

    // Seed baseline entities if the DB is fresh and NOT explicitly skipping
    // skip_seed is handled by checking a thread-local or environment variable for flexibility
    let skip_seed = std::env::var("SKIP_DB_SEED").map(|v| v == "true").unwrap_or(false);

    if !skip_seed {
        seed_baseline_agents(&pool).await?;
        seed_baseline_providers().await?;
        seed_baseline_workflows().await?;
        seed_baseline_mcp_config().await?;
    }

    Ok(pool)
}

/// Ensures default agents defined in `data/agents.json` exist in the system.
async fn seed_baseline_agents(pool: &SqlitePool) -> Result<()> {
    // 1. Check if we already have agents (other than maybe Alpha)
    let agent_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM agents")
        .fetch_one(pool)
        .await?;

    if agent_count > 1 {
        // We already have a populated swarm, skip seeding
        return Ok(());
    }

    tracing::info!("🌱 [Database] Seeding baseline agents from bundle...");

    // 2. Resolve data file path
    let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
    let agents_json_path = find_bundled_file(&resource_root, "data/agents.json");

    if let Some(path) = agents_json_path {
        tracing::info!("📂 [Database] Found baseline agents at {:?}", path);
        
        let content = std::fs::read_to_string(&path)?;
        let agents: Vec<serde_json::Value> = serde_json::from_str(&content)?;

        let mut tx = pool.begin().await?;

        for agent_val in agents {
            let id = agent_val["id"].as_str().unwrap_or_default();
            if id.is_empty() { continue; }

            let name = agent_val["name"].as_str().unwrap_or("Unknown");
            let role = agent_val["role"].as_str().unwrap_or("Specialist");
            let dept = agent_val["department"].as_str().unwrap_or("Swarm Core");
            let desc = agent_val["description"].as_str().unwrap_or("");
            let model_id = agent_val["model"].as_str().or_else(|| agent_val["model_id"].as_str());
            let provider = agent_val["model_config"]["provider"].as_str().unwrap_or("google");
            let theme = agent_val["theme_color"].as_str().unwrap_or("#4fd1c5");

            sqlx::query(
                "INSERT OR IGNORE INTO agents (id, name, role, department, description, status, provider, model_id, theme_color, metadata, skills, workflows, mcp_tools, active_model_slot, category)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(id)
            .bind(name)
            .bind(role)
            .bind(dept)
            .bind(desc)
            .bind("idle")
            .bind(provider)
            .bind(model_id)
            .bind(theme)
            .bind("{}")
            .bind(serde_json::to_string(&agent_val["skills"]).unwrap_or_else(|_| "[]".to_string()))
            .bind(serde_json::to_string(&agent_val["workflows"]).unwrap_or_else(|_| "[]".to_string()))
            .bind("[]") // mcp_tools
            .bind(1)
            .bind("user")
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
    } else {
        tracing::warn!("⚠️ [Database] Seed file 'agents.json' not found in bundle; falling back...");
        if agent_count == 0 {
            seed_minimal_alpha(pool).await?;
        }
    }

    Ok(())
}

async fn seed_minimal_alpha(pool: &SqlitePool) -> Result<()> {
    tracing::info!("🌱 Seeding minimal Alpha agent...");
    sqlx::query(
        "INSERT OR IGNORE INTO agents (id, name, role, department, description, status, provider, model_id, theme_color, metadata, skills, workflows, mcp_tools, active_model_slot, category)
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
    .bind("{}")
    .bind("[]")
    .bind("[]")
    .bind("[]")
    .bind(1)
    .bind("user")
    .execute(pool)
    .await?;
    Ok(())
}

/// Ensures default provider configs and model catalogs are available in the data directory.
async fn seed_baseline_providers() -> Result<()> {
    let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
    let base_dir = std::env::current_dir().unwrap_or_default();
    let data_dir = base_dir.join("data");
    
    // Ensure data dir exists
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }

    let files_to_seed = [
        "infra_providers.json",
        "infra_models.json",
        "routines.json"
    ];

    for filename in files_to_seed {
        let dest_path = data_dir.join(filename);
        if !dest_path.exists() {
            if let Some(src_path) = find_bundled_file(&resource_root, &format!("data/{}", filename)) {
                tracing::info!("🌱 [System] Seeding {} from {:?}...", filename, src_path);
                std::fs::copy(&src_path, &dest_path)?;
            }
        }
    }

    Ok(())
}

/// Ensures default workflows (directives) are loaded from the bundle.
async fn seed_baseline_workflows() -> Result<()> {
    let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
    let base_dir = std::env::current_dir().unwrap_or_default();
    let directives_dir = base_dir.join("directives");

    if !directives_dir.exists() {
        std::fs::create_dir_all(&directives_dir)?;
    }

    // Copy from bundled data/workflows to directives/
    let bundled_workflows_dir = find_bundled_file(&resource_root, "data/workflows");

    if let Some(src_dir) = bundled_workflows_dir {
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(filename) = path.file_name() {
                        let dest_path = directives_dir.join(filename);
                        if !dest_path.exists() {
                            tracing::info!("🌱 [System] Seeding workflow {:?} from {:?}...", filename, path);
                            std::fs::copy(&path, &dest_path)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Ensures default MCP configuration is available.
async fn seed_baseline_mcp_config() -> Result<()> {
    let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
    let base_dir = std::env::current_dir().unwrap_or_default();
    let agent_dir = base_dir.join(".agent");

    if !agent_dir.exists() {
        std::fs::create_dir_all(&agent_dir)?;
    }

    let mcp_filename = "mcp_config.json";
    let dest_path = agent_dir.join(mcp_filename);
    
    if !dest_path.exists() {
        if let Some(src_path) = find_bundled_file(&resource_root, &format!(".agent/{}", mcp_filename)) {
            tracing::info!("🌱 [System] Seeding MCP configuration from {:?}...", src_path);
            std::fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Helper to find a bundled file, handling Tauri v2's '_up_' prefix if needed.
fn find_bundled_file(resource_root: &str, relative_path: &str) -> Option<std::path::PathBuf> {
    let root = std::path::Path::new(resource_root);
    
    // 1. Try direct path (e.g., resources/data/agents.json)
    let direct = root.join(relative_path);
    if direct.exists() {
        return Some(direct);
    }
    
    // 2. Try _up_ path (e.g., resources/_up_/data/agents.json)
    let up_path = root.join("_up_").join(relative_path);
    if up_path.exists() {
        return Some(up_path);
    }
    
    // 3. Last ditch: check if current dir has it (usually only for dev)
    let dev_path = std::path::Path::new(".").join(relative_path);
    if dev_path.exists() {
        return Some(dev_path);
    }
    
    None
}

// ─────────────────────────────────────────────────────────
//  Hotfix Reconciler: Generic Migration Pre-Marker
// ─────────────────────────────────────────────────────────

/// Configuration for a single hotfix migration pre-mark operation.
struct HotfixMigration {
    /// The table to check for the pre-existing column.
    table: &'static str,
    /// The column name to check for.
    column: &'static str,
    /// The migration version to pre-mark.
    version: i64,
    /// Human-readable label for logging.
    label: &'static str,
    /// Whether to check that the table exists first (skip for non-initial migrations).
    check_table_exists: bool,
    /// Whether to force-update the checksum if the migration is already applied.
    force_checksum_sync: bool,
}

/// Generic migration pre-marker that handles all hotfix reconciliation scenarios.
///
/// This function prevents migration failures when schema changes were applied
/// out-of-band (e.g., manual `ALTER TABLE` commands, previous broken migrations).
/// It checks if a target column already exists, and if so, pre-marks the migration
/// as applied using the embedded checksum so SQLx validation remains intact.
async fn premark_hotfix_migration(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
    config: &HotfixMigration,
) -> Result<()> {
    // 1. Optionally check if the target table exists (fresh DB guard)
    if config.check_table_exists {
        let table_exists = sqlx::query_scalar::<_, i64>(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1 LIMIT 1",
        )
        .bind(config.table)
        .fetch_optional(pool)
        .await?
        .is_some();
        if !table_exists {
            return Ok(());
        }
    }

    // 2. Check if the column already exists
    let query = format!(
        "SELECT 1 FROM pragma_table_info('{}') WHERE name='{}' LIMIT 1",
        config.table, config.column
    );
    let column_exists = sqlx::query_scalar::<_, i64>(&query)
        .fetch_optional(pool)
        .await?
        .is_some();
    if !column_exists {
        return Ok(());
    }

    // 3. Ensure the migrations table exists
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

    // 4. Check if already applied
    let already_applied =
        sqlx::query_scalar::<_, i64>("SELECT 1 FROM _sqlx_migrations WHERE version = ?1 LIMIT 1")
            .bind(config.version)
            .fetch_optional(pool)
            .await?
            .is_some();

    // 5. Find the migration in the embedded set
    let Some(migration) = migrator.iter().find(|m| m.version == config.version) else {
        tracing::warn!(
            "⚠️ {} fix migration {} not found in embedded migrations; skipping pre-mark",
            config.label,
            config.version
        );
        return Ok(());
    };

    if already_applied {
        if config.force_checksum_sync {
            // Force-update the checksum to resolve VersionMismatch errors.
            sqlx::query("UPDATE _sqlx_migrations SET checksum = ?1 WHERE version = ?2")
                .bind(migration.checksum.as_ref())
                .bind(config.version)
                .execute(pool)
                .await?;
        }
        return Ok(());
    }

    // 6. Pre-mark as applied
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?1, ?2, TRUE, ?3, 0)",
    )
    .bind(migration.version)
    .bind(migration.description.as_ref())
    .bind(migration.checksum.as_ref())
    .execute(pool)
    .await?;

    tracing::warn!(
        "⚠️ Pre-marked migration {} because {}.{} already exists",
        config.version,
        config.table,
        config.column
    );

    Ok(())
}

/// Handles legacy hotfixed environments where `connector_configs` was added manually.
async fn premark_connector_column_fix_migration_if_needed(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<()> {
    premark_hotfix_migration(pool, migrator, &HotfixMigration {
        table: "agents",
        column: "connector_configs",
        version: CONNECTOR_COLUMN_FIX_MIGRATION_VERSION,
        label: "Connector",
        check_table_exists: true,
        force_checksum_sync: false,
    }).await
}

/// Handles environments where `created_at` column was added out-of-band.
async fn premark_created_at_fix_migration_if_needed(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<()> {
    premark_hotfix_migration(pool, migrator, &HotfixMigration {
        table: "agents",
        column: "created_at",
        version: CREATED_AT_FIX_MIGRATION_VERSION,
        label: "Created-at",
        check_table_exists: true,
        force_checksum_sync: true,
    }).await
}

/// Handles environments where `current_task` column was added out-of-band.
async fn premark_current_task_fix_migration_if_needed(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<()> {
    premark_hotfix_migration(pool, migrator, &HotfixMigration {
        table: "agents",
        column: "current_task",
        version: CURRENT_TASK_FIX_MIGRATION_VERSION,
        label: "Current-task",
        check_table_exists: false, // Table guaranteed to exist at this point
        force_checksum_sync: true,
    }).await
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

// Metadata: [db]

// Metadata: [db]
