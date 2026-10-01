//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / migrations
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[migrations]`

use anyhow::{Context, Result};
use sqlx::SqlitePool;

// Migration version constants declared in strictly ascending chronological order
pub const CONNECTOR_COLUMN_FIX_MIGRATION_VERSION: i64 = 20260328000100;
pub const SWARM_GRAPH_MIGRATION_VERSION: i64 = 20260404000100;
pub const CREATED_AT_FIX_MIGRATION_VERSION: i64 = 20260405000100;
pub const CURRENT_TASK_FIX_MIGRATION_VERSION: i64 = 20260405000200;
pub const MERKLE_AUDIT_TRAIL_MIGRATION_VERSION: i64 = 20260516000100;
pub const INSTITUTIONAL_KNOWLEDGE_STORE_MIGRATION_VERSION: i64 = 20260601000100;
pub const IKS_ADD_TEXT_COLUMN_MIGRATION_VERSION: i64 = 20260601000101;

/// Executes pending database migrations and applies hotfix reconciliations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migrator = sqlx::migrate!("./migrations");

    let is_fresh = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .with_context(|| "checking if database is fresh (querying _sqlx_migrations)")?
    .is_none();

    let allow_reconcile = std::env::var("ALLOW_MIGRATION_RECONCILE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if allow_reconcile && !is_fresh {
        premark_all_hotfix_migrations(pool, &migrator)
            .await
            .with_context(|| "pre-marking hotfix migrations during reconciliation")?;
    } else if !allow_reconcile && !is_fresh {
        tracing::info!(
            "ℹ Migration reconciler is disabled (set ALLOW_MIGRATION_RECONCILE=true to enable)"
        );
    }

    migrator
        .run(pool)
        .await
        .with_context(|| "executing sqlx migrations via migrator")?;

    tracing::info!("✅ [migrations] Database migrations applied successfully");
    Ok(())
}

/// Declarative idempotency probe verifying schema state before any pre-marking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyProbe {
    TableExists(&'static str),
    ColumnExists(&'static str, &'static str),
    IndexExists(&'static str),
}

/// Configuration for a single hotfix migration pre-mark operation.
#[derive(Debug, Clone, Copy)]
pub struct HotfixMigration {
    pub version: i64,
    pub label: &'static str,
    pub probes: &'static [IdempotencyProbe],
}

/// Single Source of Truth for all hotfix migration specifications and probes.
pub fn hotfix_registry() -> &'static [HotfixMigration] {
    &[
        HotfixMigration {
            version: CONNECTOR_COLUMN_FIX_MIGRATION_VERSION,
            label: "Connector column",
            probes: &[IdempotencyProbe::ColumnExists(
                "agents",
                "connector_configs",
            )],
        },
        HotfixMigration {
            version: SWARM_GRAPH_MIGRATION_VERSION,
            label: "Swarm-graph",
            probes: &[
                IdempotencyProbe::TableExists("mission_relationships"),
                IdempotencyProbe::IndexExists("idx_mission_from"),
                IdempotencyProbe::IndexExists("idx_mission_to"),
                IdempotencyProbe::IndexExists("idx_mission_rel_type"),
            ],
        },
        HotfixMigration {
            version: CREATED_AT_FIX_MIGRATION_VERSION,
            label: "Created-at",
            probes: &[IdempotencyProbe::ColumnExists("agents", "created_at")],
        },
        HotfixMigration {
            version: CURRENT_TASK_FIX_MIGRATION_VERSION,
            label: "Current-task telemetry",
            probes: &[
                IdempotencyProbe::ColumnExists("agents", "current_task"),
                IdempotencyProbe::ColumnExists("agents", "input_tokens"),
                IdempotencyProbe::ColumnExists("agents", "output_tokens"),
            ],
        },
        HotfixMigration {
            version: MERKLE_AUDIT_TRAIL_MIGRATION_VERSION,
            label: "Merkle-audit-trail",
            probes: &[
                IdempotencyProbe::ColumnExists("mission_logs", "hash"),
                IdempotencyProbe::ColumnExists("mission_logs", "prev_hash"),
            ],
        },
        HotfixMigration {
            version: INSTITUTIONAL_KNOWLEDGE_STORE_MIGRATION_VERSION,
            label: "IKS-store",
            probes: &[
                IdempotencyProbe::TableExists("knowledge_store_meta"),
                IdempotencyProbe::IndexExists("idx_ks_topic"),
                IdempotencyProbe::IndexExists("idx_ks_cluster"),
                IdempotencyProbe::IndexExists("idx_ks_ttl"),
                IdempotencyProbe::IndexExists("idx_ks_hash"),
                IdempotencyProbe::IndexExists("idx_ks_source_node"),
                IdempotencyProbe::IndexExists("idx_ks_confidence"),
                IdempotencyProbe::IndexExists("idx_ks_created_at"),
            ],
        },
        HotfixMigration {
            version: IKS_ADD_TEXT_COLUMN_MIGRATION_VERSION,
            label: "IKS-text-column",
            probes: &[IdempotencyProbe::ColumnExists(
                "knowledge_store_meta",
                "text",
            )],
        },
    ]
}

/// Evaluates a single declarative idempotency probe against the active database.
pub async fn evaluate_probe(pool: &SqlitePool, probe: &IdempotencyProbe) -> Result<bool> {
    match probe {
        IdempotencyProbe::TableExists(table) => {
            let exists = sqlx::query_scalar::<_, i64>(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1 LIMIT 1",
            )
            .bind(table)
            .fetch_optional(pool)
            .await
            .with_context(|| format!("evaluating TableExists probe on '{}'", table))?
            .is_some();
            Ok(exists)
        }
        IdempotencyProbe::ColumnExists(table, column) => {
            let table_exists = sqlx::query_scalar::<_, i64>(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1 LIMIT 1",
            )
            .bind(table)
            .fetch_optional(pool)
            .await
            .with_context(|| {
                format!(
                    "checking table existence for ColumnExists probe on '{}.{}'",
                    table, column
                )
            })?
            .is_some();

            if !table_exists {
                return Ok(false);
            }

            let col_exists = match *table {
                "agents" => {
                    sqlx::query_scalar::<_, i64>(
                        "SELECT 1 FROM pragma_table_info('agents') WHERE name=?1 LIMIT 1",
                    )
                    .bind(column)
                    .fetch_optional(pool)
                    .await
                    .with_context(|| format!("evaluating ColumnExists probe on '{}.{}'", table, column))?
                    .is_some()
                }
                "mission_relationships" => {
                    sqlx::query_scalar::<_, i64>(
                        "SELECT 1 FROM pragma_table_info('mission_relationships') WHERE name=?1 LIMIT 1",
                    )
                    .bind(column)
                    .fetch_optional(pool)
                    .await
                    .with_context(|| format!("evaluating ColumnExists probe on '{}.{}'", table, column))?
                    .is_some()
                }
                "knowledge_store_meta" => {
                    sqlx::query_scalar::<_, i64>(
                        "SELECT 1 FROM pragma_table_info('knowledge_store_meta') WHERE name=?1 LIMIT 1",
                    )
                    .bind(column)
                    .fetch_optional(pool)
                    .await
                    .with_context(|| format!("evaluating ColumnExists probe on '{}.{}'", table, column))?
                    .is_some()
                }
                "mission_logs" => {
                    sqlx::query_scalar::<_, i64>(
                        "SELECT 1 FROM pragma_table_info('mission_logs') WHERE name=?1 LIMIT 1",
                    )
                    .bind(column)
                    .fetch_optional(pool)
                    .await
                    .with_context(|| format!("evaluating ColumnExists probe on '{}.{}'", table, column))?
                    .is_some()
                }
                "test_tbl" => {
                    sqlx::query_scalar::<_, i64>(
                        "SELECT 1 FROM pragma_table_info('test_tbl') WHERE name=?1 LIMIT 1",
                    )
                    .bind(column)
                    .fetch_optional(pool)
                    .await
                    .with_context(|| format!("evaluating ColumnExists probe on '{}.{}'", table, column))?
                    .is_some()
                }
                other => {
                    anyhow::bail!("unsupported table for ColumnExists probe: {}", other);
                }
            };
            Ok(col_exists)
        }
        IdempotencyProbe::IndexExists(index_name) => {
            let exists = sqlx::query_scalar::<_, i64>(
                "SELECT 1 FROM sqlite_master WHERE type='index' AND name=?1 LIMIT 1",
            )
            .bind(index_name)
            .fetch_optional(pool)
            .await
            .with_context(|| format!("evaluating IndexExists probe on '{}'", index_name))?
            .is_some();
            Ok(exists)
        }
    }
}

async fn premark_all_hotfix_migrations(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<()> {
    for hf in hotfix_registry() {
        premark_hotfix_migration(pool, migrator, hf).await?;
    }
    Ok(())
}

async fn premark_hotfix_migration(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
    config: &HotfixMigration,
) -> Result<()> {
    let target_migration = migrator
        .iter()
        .find(|m| m.version == config.version)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Hotfix migration {} ({}) not found in embedded migrator catalog",
                config.version,
                config.label
            )
        })?;

    // Check if migration is already recorded in _sqlx_migrations
    let existing_checksum = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT checksum FROM _sqlx_migrations WHERE version = ?1 LIMIT 1",
    )
    .bind(config.version)
    .fetch_optional(pool)
    .await
    .with_context(|| format!("querying _sqlx_migrations for version {}", config.version))?;

    if let Some(recorded_checksum) = existing_checksum {
        // Tamper verification: Recorded checksum MUST match the current binary's migration file checksum.
        if recorded_checksum != target_migration.checksum.as_ref() {
            anyhow::bail!(
                "Applied migration {} ({}) checksum mismatch! Recorded: {:?}, binary: {:?}. \
                 Migrations are immutable once applied. Checksum rewriting is permanently disabled.",
                config.version,
                config.label,
                recorded_checksum,
                target_migration.checksum.as_ref()
            );
        }
        return Ok(());
    }

    // Verify all declared probes (tables, columns, indexes) are genuinely present
    for probe in config.probes {
        let satisfied = evaluate_probe(pool, probe).await?;
        if !satisfied {
            tracing::debug!(
                "ℹ [Hotfix] Probe {:?} not satisfied for {} ({}); delegating to migrator.",
                probe,
                config.label,
                config.version
            );
            return Ok(());
        }
    }

    // All probes satisfied: pre-mark with INSERT OR IGNORE to prevent TOCTOU race conditions
    tracing::info!(
        "🔧 [Hotfix] Pre-marking {} migration ({}) as applied (all probes satisfied)...",
        config.label,
        config.version
    );
    sqlx::query(
        "INSERT OR IGNORE INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
         VALUES (?1, ?2, CURRENT_TIMESTAMP, 1, ?3, 0)",
    )
    .bind(config.version)
    .bind(&target_migration.description)
    .bind(target_migration.checksum.as_ref())
    .execute(pool)
    .await
    .with_context(|| format!("pre-marking hotfix migration {} in _sqlx_migrations", config.version))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotfix_registry_monotonic_and_unique() {
        let registry = hotfix_registry();
        assert!(!registry.is_empty(), "registry must not be empty");

        for i in 0..registry.len() {
            assert!(
                !registry[i].probes.is_empty(),
                "hotfix {} has no probes",
                registry[i].version
            );
            if i > 0 {
                assert!(
                    registry[i].version > registry[i - 1].version,
                    "hotfixes must be in strictly ascending order: {} <= {}",
                    registry[i].version,
                    registry[i - 1].version
                );
            }
        }
    }

    #[tokio::test]
    async fn test_evaluate_probe_coverage() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("open memory pool");

        // 1. Initial state: table, column, index do not exist
        assert!(
            !evaluate_probe(&pool, &IdempotencyProbe::TableExists("test_tbl"))
                .await
                .unwrap()
        );
        assert!(
            !evaluate_probe(&pool, &IdempotencyProbe::ColumnExists("test_tbl", "col_a"))
                .await
                .unwrap()
        );
        assert!(
            !evaluate_probe(&pool, &IdempotencyProbe::IndexExists("idx_test_col"))
                .await
                .unwrap()
        );

        // 2. Create table and column
        sqlx::query("CREATE TABLE test_tbl (id INTEGER PRIMARY KEY, col_a TEXT);")
            .execute(&pool)
            .await
            .unwrap();

        assert!(
            evaluate_probe(&pool, &IdempotencyProbe::TableExists("test_tbl"))
                .await
                .unwrap()
        );
        assert!(
            evaluate_probe(&pool, &IdempotencyProbe::ColumnExists("test_tbl", "col_a"))
                .await
                .unwrap()
        );
        assert!(
            !evaluate_probe(&pool, &IdempotencyProbe::ColumnExists("test_tbl", "col_b"))
                .await
                .unwrap()
        );
        assert!(
            !evaluate_probe(&pool, &IdempotencyProbe::IndexExists("idx_test_col"))
                .await
                .unwrap()
        );

        // 3. Create index
        sqlx::query("CREATE INDEX idx_test_col ON test_tbl (col_a);")
            .execute(&pool)
            .await
            .unwrap();

        assert!(
            evaluate_probe(&pool, &IdempotencyProbe::IndexExists("idx_test_col"))
                .await
                .unwrap()
        );
    }
}
