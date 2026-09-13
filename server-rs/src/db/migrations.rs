//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / migrations
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[migrations]`
//! - **Witness Tests**: none declared

use anyhow::Result;
use sqlx::SqlitePool;

// Migration version constants
pub const CONNECTOR_COLUMN_FIX_MIGRATION_VERSION: i64 = 20260328000100;
pub const CREATED_AT_FIX_MIGRATION_VERSION: i64 = 20260405000100;
pub const CURRENT_TASK_FIX_MIGRATION_VERSION: i64 = 20260405000200;
pub const MERKLE_AUDIT_TRAIL_MIGRATION_VERSION: i64 = 20260516000100;
pub const INSTITUTIONAL_KNOWLEDGE_STORE_MIGRATION_VERSION: i64 = 20260601000100;
pub const IKS_ADD_TEXT_COLUMN_MIGRATION_VERSION: i64 = 20260601000101;
pub const SWARM_GRAPH_MIGRATION_VERSION: i64 = 20260404000100;

/// Executes pending database migrations and applies hotfix reconciliations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migrator = sqlx::migrate!("./migrations");

    let is_fresh = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?
    .is_none();

    let allow_reconcile = std::env::var("ALLOW_MIGRATION_RECONCILE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if allow_reconcile && !is_fresh {
        premark_all_hotfix_migrations(pool, &migrator).await?;
    } else if !allow_reconcile && !is_fresh {
        tracing::info!(
            "ℹ Migration reconciler is disabled (set ALLOW_MIGRATION_RECONCILE=true to enable)"
        );
    }

    migrator.run(pool).await?;
    tracing::info!("✅ [migrations] Database migrations applied successfully");
    Ok(())
}

/// Configuration for a single hotfix migration pre-mark operation.
struct HotfixMigration {
    table: &'static str,
    column: &'static str,
    version: i64,
    label: &'static str,
    check_table_exists: bool,
    force_checksum_sync: bool,
}

async fn premark_all_hotfix_migrations(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<()> {
    let hotfixes = [
        HotfixMigration {
            table: "agents",
            column: "connector_configs",
            version: CONNECTOR_COLUMN_FIX_MIGRATION_VERSION,
            label: "Connector column",
            check_table_exists: true,
            force_checksum_sync: false,
        },
        HotfixMigration {
            table: "agents",
            column: "created_at",
            version: CREATED_AT_FIX_MIGRATION_VERSION,
            label: "Created-at",
            check_table_exists: true,
            force_checksum_sync: false,
        },
        HotfixMigration {
            table: "agents",
            column: "current_task",
            version: CURRENT_TASK_FIX_MIGRATION_VERSION,
            label: "Current-task",
            check_table_exists: true,
            force_checksum_sync: false,
        },
        HotfixMigration {
            table: "mission_logs",
            column: "hash",
            version: MERKLE_AUDIT_TRAIL_MIGRATION_VERSION,
            label: "Merkle-audit-trail",
            check_table_exists: true,
            force_checksum_sync: true,
        },
        HotfixMigration {
            table: "knowledge_store_meta",
            column: "id",
            version: INSTITUTIONAL_KNOWLEDGE_STORE_MIGRATION_VERSION,
            label: "IKS-store",
            check_table_exists: false,
            force_checksum_sync: true,
        },
        HotfixMigration {
            table: "institutional_knowledge_store",
            column: "raw_text",
            version: IKS_ADD_TEXT_COLUMN_MIGRATION_VERSION,
            label: "IKS-raw-text",
            check_table_exists: true,
            force_checksum_sync: true,
        },
        HotfixMigration {
            table: "mission_relationships",
            column: "id",
            version: SWARM_GRAPH_MIGRATION_VERSION,
            label: "Swarm-graph",
            check_table_exists: true,
            force_checksum_sync: true,
        },
    ];

    for hf in &hotfixes {
        premark_hotfix_migration(pool, migrator, hf).await?;
    }

    Ok(())
}

async fn premark_hotfix_migration(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
    config: &HotfixMigration,
) -> Result<()> {
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

    let query = match (config.table, config.column) {
        ("agents", "connector_configs") => {
            "SELECT 1 FROM pragma_table_info('agents') WHERE name='connector_configs' LIMIT 1"
        }
        ("agents", "created_at") => {
            "SELECT 1 FROM pragma_table_info('agents') WHERE name='created_at' LIMIT 1"
        }
        ("agents", "current_task") => {
            "SELECT 1 FROM pragma_table_info('agents') WHERE name='current_task' LIMIT 1"
        }
        ("mission_logs", "hash") => {
            "SELECT 1 FROM pragma_table_info('mission_logs') WHERE name='hash' LIMIT 1"
        }
        ("knowledge_store_meta", "id") => {
            "SELECT 1 FROM pragma_table_info('knowledge_store_meta') WHERE name='id' LIMIT 1"
        }
        ("institutional_knowledge_store", "raw_text") => {
            "SELECT 1 FROM pragma_table_info('institutional_knowledge_store') WHERE name='raw_text' LIMIT 1"
        }
        ("mission_relationships", "id") => {
            "SELECT 1 FROM pragma_table_info('mission_relationships') WHERE name='id' LIMIT 1"
        }
        _ => return Ok(()),
    };

    let col_exists = sqlx::query_scalar::<_, i64>(query)
        .fetch_optional(pool)
        .await?
        .is_some();

    if col_exists {
        if let Some(target_migration) = migrator.iter().find(|m| m.version == config.version) {
            let is_premarked = sqlx::query_scalar::<_, i64>(
                "SELECT 1 FROM _sqlx_migrations WHERE version = ?1 LIMIT 1",
            )
            .bind(config.version)
            .fetch_optional(pool)
            .await?
            .is_some();

            if !is_premarked {
                tracing::info!(
                    "🔧 [Hotfix] Pre-marking {} migration ({}) as applied...",
                    config.label,
                    config.version
                );
                sqlx::query(
                    "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
                     VALUES (?1, ?2, CURRENT_TIMESTAMP, 1, ?3, 0)"
                )
                .bind(config.version)
                .bind(&target_migration.description)
                .bind(target_migration.checksum.as_ref())
                .execute(pool)
                .await?;
            } else if config.force_checksum_sync {
                sqlx::query(
                    "UPDATE _sqlx_migrations SET checksum = ?1, success = 1 WHERE version = ?2",
                )
                .bind(target_migration.checksum.as_ref())
                .bind(config.version)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}
