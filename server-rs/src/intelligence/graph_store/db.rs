//! @docs ARCHITECTURE:Core:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / graph_store / db
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling, batched transactional persistence, and zero blocking calls in async context.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `crate::error::AppError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::graph_store::tests`

use super::model::GraphSnapshot;
use crate::error::AppError;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{QueryBuilder, SqlitePool};
use std::path::Path;

/// Current schema version. Increment whenever DDL changes.
/// `ensure_schema` will detect a mismatch and drop + recreate all tables.
const SCHEMA_VERSION: u32 = 10;

/// Number of rows per batched INSERT statement.
/// SQLite's SQLITE_LIMIT_VARIABLE_NUMBER defaults to 32766;
/// a node row has 18 bound parameters → 500 × 18 = 9000, safely within limit.
const INSERT_BATCH_SIZE: usize = 500;

pub(super) async fn open_graph_pool(db_path: &Path) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?)
}

pub(super) async fn ensure_schema(pool: &SqlitePool) -> Result<(), AppError> {
    // ── Schema version migration guard ────────────────────────────────────────
    // If the persisted schema_version does not match SCHEMA_VERSION, wipe all
    // application tables and let them be recreated cleanly below.
    // If the table doesn't exist yet (fresh DB), treat as None without failing.
    let persisted: Option<u32> = match sqlx::query_scalar(
        "SELECT CAST(value AS INTEGER) FROM metadata WHERE key = 'schema_version' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    {
        Ok(val) => val,
        Err(sqlx::Error::Database(db_err)) if db_err.message().contains("no such table") => None,
        Err(e) => {
            return Err(AppError::InternalServerError(format!(
                "Failed to query schema_version from metadata: {e}"
            )));
        }
    };

    if let Some(stored) = persisted {
        if stored != SCHEMA_VERSION {
            tracing::warn!(
                "⚠️ [db] Schema version mismatch (stored={}, current={}). Recreating tables.",
                stored,
                SCHEMA_VERSION
            );
            let drop_stmts = [
                "DROP TABLE IF EXISTS flow_memberships",
                "DROP TABLE IF EXISTS flow_snapshots",
                "DROP TABLE IF EXISTS flows",
                "DROP TABLE IF EXISTS community_summaries",
                "DROP TABLE IF EXISTS communities",
                "DROP TABLE IF EXISTS risk_index",
                "DROP TABLE IF EXISTS edges",
                "DROP TABLE IF EXISTS nodes_fts",
                "DROP TABLE IF EXISTS nodes",
                "DROP TABLE IF EXISTS file_cache",
                "DROP TABLE IF EXISTS metadata",
            ];
            for stmt in drop_stmts {
                sqlx::query(stmt).execute(pool).await?;
            }
        }
    }

    let schema = [
        "CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS nodes (id INTEGER PRIMARY KEY AUTOINCREMENT, kind TEXT NOT NULL, name TEXT NOT NULL, qualified_name TEXT NOT NULL UNIQUE, file_path TEXT NOT NULL, line_start INTEGER, line_end INTEGER, language TEXT, parent_name TEXT, params TEXT, return_type TEXT, modifiers TEXT, is_test INTEGER DEFAULT 0, file_hash TEXT, extra TEXT DEFAULT '{}', updated_at REAL NOT NULL, signature TEXT, community_id INTEGER)",
        "CREATE TABLE IF NOT EXISTS edges (id INTEGER PRIMARY KEY AUTOINCREMENT, kind TEXT NOT NULL, source_qualified TEXT NOT NULL, target_qualified TEXT NOT NULL, file_path TEXT NOT NULL, line INTEGER DEFAULT 0, extra TEXT DEFAULT '{}', confidence REAL DEFAULT 1.0, confidence_tier TEXT DEFAULT 'EXTRACTED', updated_at REAL NOT NULL)",
        "CREATE TABLE IF NOT EXISTS communities (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, level INTEGER NOT NULL DEFAULT 0, parent_id INTEGER, cohesion REAL NOT NULL DEFAULT 0.0, size INTEGER NOT NULL DEFAULT 0, dominant_language TEXT, description TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
        "CREATE TABLE IF NOT EXISTS community_summaries (community_id INTEGER PRIMARY KEY, name TEXT NOT NULL, purpose TEXT DEFAULT '', key_symbols TEXT DEFAULT '[]', risk TEXT DEFAULT 'unknown', size INTEGER DEFAULT 0, dominant_language TEXT DEFAULT '')",
        "CREATE TABLE IF NOT EXISTS flows (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, entry_point_id INTEGER NOT NULL, depth INTEGER NOT NULL, node_count INTEGER NOT NULL, file_count INTEGER NOT NULL, criticality REAL NOT NULL DEFAULT 0.0, path_json TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))",
        "CREATE TABLE IF NOT EXISTS flow_memberships (flow_id INTEGER NOT NULL, node_id INTEGER NOT NULL, position INTEGER NOT NULL, PRIMARY KEY (flow_id, node_id))",
        "CREATE TABLE IF NOT EXISTS flow_snapshots (flow_id INTEGER PRIMARY KEY, name TEXT NOT NULL, entry_point TEXT NOT NULL, critical_path TEXT DEFAULT '[]', criticality REAL DEFAULT 0.0, node_count INTEGER DEFAULT 0, file_count INTEGER DEFAULT 0)",
        "CREATE TABLE IF NOT EXISTS risk_index (node_id INTEGER PRIMARY KEY, qualified_name TEXT NOT NULL, risk_score REAL DEFAULT 0.0, caller_count INTEGER DEFAULT 0, test_coverage TEXT DEFAULT 'unknown', security_relevant INTEGER DEFAULT 0, last_computed TEXT DEFAULT '')",
        "CREATE TABLE IF NOT EXISTS file_cache (file_path TEXT PRIMARY KEY, file_hash TEXT NOT NULL, cache_json TEXT NOT NULL)",
        "CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(name, qualified_name, file_path, signature, content='nodes', content_rowid='rowid', tokenize='porter unicode61')",
    ];
    for stmt in schema {
        sqlx::query(stmt).execute(pool).await?;
    }

    let indexes = [
        "CREATE INDEX IF NOT EXISTS idx_nodes_qualified ON nodes(qualified_name)",
        "CREATE INDEX IF NOT EXISTS idx_nodes_file ON nodes(file_path)",
        "CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind)",
        "CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_qualified)",
        "CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_qualified)",
        "CREATE INDEX IF NOT EXISTS idx_edges_kind ON edges(kind)",
        "CREATE INDEX IF NOT EXISTS idx_risk_index_score ON risk_index(risk_score DESC)",
        "CREATE INDEX IF NOT EXISTS idx_flows_criticality ON flows(criticality DESC)",
        "CREATE INDEX IF NOT EXISTS idx_communities_cohesion ON communities(cohesion DESC)",
    ];
    for stmt in indexes {
        sqlx::query(stmt).execute(pool).await?;
    }
    Ok(())
}

pub(super) async fn write_snapshot(
    pool: &SqlitePool,
    snapshot: &GraphSnapshot,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // Safety: The table list below is a static compile-time constant array of known schema identifiers.
    const TABLES_TO_TRUNCATE: &[&str] = &[
        "flow_memberships",
        "flow_snapshots",
        "flows",
        "community_summaries",
        "communities",
        "risk_index",
        "edges",
        "nodes",
        "metadata",
    ];
    for table in TABLES_TO_TRUNCATE {
        sqlx::query(sqlx::AssertSqlSafe(format!("DELETE FROM {table}")))
            .execute(&mut *tx)
            .await?;
    }

    // FTS5 external content table sync: delete-all cleans stale index tokens prior to rebuild.
    let _ = sqlx::query("INSERT INTO nodes_fts(nodes_fts) VALUES('delete-all')")
        .execute(&mut *tx)
        .await;

    let now = Utc::now().timestamp_millis() as f64 / 1000.0;

    // ── Batched node INSERTs ─────────────────────────────────────────────────
    for chunk in snapshot.nodes.chunks(INSERT_BATCH_SIZE) {
        let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT INTO nodes (id, kind, name, qualified_name, file_path, line_start, line_end, \
             language, parent_name, params, return_type, modifiers, is_test, file_hash, extra, \
             updated_at, signature, community_id) ",
        );
        qb.push_values(chunk, |mut b, node| {
            b.push_bind(node.id)
                .push_bind(&node.kind)
                .push_bind(&node.name)
                .push_bind(&node.qualified_name)
                .push_bind(&node.file_path)
                .push_bind(node.line_start)
                .push_bind(node.line_end)
                .push_bind(&node.language)
                .push_bind(&node.parent_name)
                .push_bind(&node.params)
                .push_bind(&node.return_type)
                .push_bind(&node.modifiers)
                .push_bind(if node.is_test { 1i32 } else { 0i32 })
                .push_bind(&node.file_hash)
                .push_bind(&node.extra)
                .push_bind(now)
                .push_bind(&node.signature)
                .push_bind(node.community_id);
        });
        qb.build().execute(&mut *tx).await?;
    }

    sqlx::query("INSERT INTO nodes_fts(nodes_fts) VALUES('rebuild')")
        .execute(&mut *tx)
        .await?;

    // ── Batched edge INSERTs ─────────────────────────────────────────────────
    // Edges have 9 bound parameters: 500 × 9 = 4500, well within SQLite limits.
    for chunk in snapshot.edges.chunks(INSERT_BATCH_SIZE) {
        let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT INTO edges (kind, source_qualified, target_qualified, file_path, line, extra, \
             confidence, confidence_tier, updated_at) ",
        );
        qb.push_values(chunk, |mut b, edge| {
            b.push_bind(edge.kind.as_str())
                .push_bind(&edge.source_qualified)
                .push_bind(&edge.target_qualified)
                .push_bind(&edge.file_path)
                .push_bind(edge.line)
                .push_bind(&edge.extra)
                .push_bind(1.0f64)
                .push_bind("EXTRACTED")
                .push_bind(now);
        });
        qb.build().execute(&mut *tx).await?;
    }

    // ── Batched risk_index INSERTs ───────────────────────────────────────────
    let computed_at = Utc::now().to_rfc3339();
    for chunk in snapshot.risks.chunks(INSERT_BATCH_SIZE) {
        let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT INTO risk_index (node_id, qualified_name, risk_score, caller_count, test_coverage, security_relevant, last_computed) ",
        );
        qb.push_values(chunk, |mut b, risk| {
            b.push_bind(risk.node_id)
                .push_bind(&risk.qualified_name)
                .push_bind(risk.risk_score)
                .push_bind(risk.caller_count)
                .push_bind(&risk.test_coverage)
                .push_bind(if risk.security_relevant { 1i32 } else { 0i32 })
                .push_bind(&computed_at);
        });
        qb.build().execute(&mut *tx).await?;
    }

    // ── Batched communities & summaries INSERTs ──────────────────────────────
    for chunk in snapshot.communities.chunks(INSERT_BATCH_SIZE) {
        let mut qb_com: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT INTO communities (id, name, level, parent_id, cohesion, size, dominant_language, description, created_at) ",
        );
        qb_com.push_values(chunk, |mut b, community| {
            b.push_bind(community.id)
                .push_bind(&community.name)
                .push_bind(0i32)
                .push_bind(None::<i64>)
                .push_bind(community.cohesion)
                .push_bind(community.size)
                .push_bind(&community.dominant_language)
                .push_bind(&community.description)
                .push_bind(&computed_at);
        });
        qb_com.build().execute(&mut *tx).await?;

        let mut qb_sum: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT INTO community_summaries (community_id, name, purpose, key_symbols, risk, size, dominant_language) ",
        );
        qb_sum.push_values(chunk, |mut b, community| {
            b.push_bind(community.id)
                .push_bind(&community.name)
                .push_bind(&community.description)
                .push_bind("[]")
                .push_bind(&community.risk)
                .push_bind(community.size)
                .push_bind(&community.dominant_language);
        });
        qb_sum.build().execute(&mut *tx).await?;
    }

    // ── Batched flows, snapshots & memberships INSERTs ───────────────────────
    for chunk in snapshot.flows.chunks(INSERT_BATCH_SIZE) {
        let mut qb_flows: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT INTO flows (id, name, entry_point_id, depth, node_count, file_count, criticality, path_json, created_at, updated_at) ",
        );
        qb_flows.push_values(chunk, |mut b, flow| {
            let path_json =
                serde_json::to_string(&flow.node_ids).unwrap_or_else(|_| "[]".to_string());
            b.push_bind(flow.id)
                .push_bind(&flow.name)
                .push_bind(flow.entry_point_id)
                .push_bind(flow.depth)
                .push_bind(flow.node_count)
                .push_bind(flow.file_count)
                .push_bind(flow.criticality)
                .push_bind(path_json)
                .push_bind(&computed_at)
                .push_bind(&computed_at);
        });
        qb_flows.build().execute(&mut *tx).await?;

        let mut qb_snap: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT INTO flow_snapshots (flow_id, name, entry_point, critical_path, criticality, node_count, file_count) ",
        );
        qb_snap.push_values(chunk, |mut b, flow| {
            let critical_path =
                serde_json::to_string(&flow.critical_path).unwrap_or_else(|_| "[]".to_string());
            b.push_bind(flow.id)
                .push_bind(&flow.name)
                .push_bind(&flow.entry_point)
                .push_bind(critical_path)
                .push_bind(flow.criticality)
                .push_bind(flow.node_count)
                .push_bind(flow.file_count);
        });
        qb_snap.build().execute(&mut *tx).await?;
    }

    // Batched flow memberships
    let mut memberships = Vec::new();
    for flow in &snapshot.flows {
        for (position, node_id) in flow.node_ids.iter().enumerate() {
            memberships.push((flow.id, *node_id, position as i64));
        }
    }
    for chunk in memberships.chunks(INSERT_BATCH_SIZE) {
        let mut qb_mem: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT OR IGNORE INTO flow_memberships (flow_id, node_id, position) ",
        );
        qb_mem.push_values(chunk, |mut b, (flow_id, node_id, pos)| {
            b.push_bind(flow_id).push_bind(node_id).push_bind(pos);
        });
        qb_mem.build().execute(&mut *tx).await?;
    }

    // ── Batched file_cache updates ───────────────────────────────────────────
    for chunk in snapshot.cache_updates.chunks(INSERT_BATCH_SIZE) {
        let mut qb_cache: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT OR REPLACE INTO file_cache (file_path, file_hash, cache_json) ",
        );
        qb_cache.push_values(chunk, |mut b, (path, hash, json)| {
            b.push_bind(path).push_bind(hash).push_bind(json);
        });
        qb_cache.build().execute(&mut *tx).await?;
    }

    let rows = sqlx::query("SELECT file_path FROM file_cache")
        .fetch_all(&mut *tx)
        .await?;
    let mut cached_paths = Vec::new();
    for row in rows {
        use sqlx::Row;
        let path: String = row.get(0);
        cached_paths.push(path);
    }
    let files_set: std::collections::HashSet<&str> =
        snapshot.files_present.iter().map(|s| s.as_str()).collect();
    for path in cached_paths {
        if !files_set.contains(path.as_str()) {
            sqlx::query("DELETE FROM file_cache WHERE file_path = ?1")
                .bind(&path)
                .execute(&mut *tx)
                .await?;
        }
    }

    write_metadata(&mut tx, snapshot).await?;
    tx.commit().await?;
    Ok(())
}

async fn write_metadata(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    snapshot: &GraphSnapshot,
) -> Result<(), AppError> {
    let timestamp = Utc::now().to_rfc3339();
    let metadata = [
        ("schema_version", SCHEMA_VERSION.to_string()),
        ("last_build_type", "startup_full".to_string()),
        ("postprocess_level", "full".to_string()),
        ("last_updated", timestamp.clone()),
        ("last_postprocessed_at", timestamp),
        (
            "git_branch",
            snapshot.git_branch.clone().unwrap_or_default(),
        ),
        (
            "git_head_sha",
            snapshot.git_head_sha.clone().unwrap_or_default(),
        ),
    ];
    for (key, value) in metadata {
        // INSERT OR REPLACE prevents UNIQUE constraint failures on repeated refresh runs.
        sqlx::query("INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)")
            .bind(key)
            .bind(value)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

pub(super) async fn read_file_cache(
    pool: &SqlitePool,
) -> Result<std::collections::HashMap<String, (String, String)>, AppError> {
    let rows = sqlx::query("SELECT file_path, file_hash, cache_json FROM file_cache")
        .fetch_all(pool)
        .await?;
    let mut cache = std::collections::HashMap::new();
    for row in rows {
        use sqlx::Row;
        let file_path: String = row.get(0);
        let file_hash: String = row.get(1);
        let cache_json: String = row.get(2);
        cache.insert(file_path, (file_hash, cache_json));
    }
    Ok(cache)
}
