//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[db]` in tracing logs.

use super::GraphSnapshot;
use crate::error::AppError;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;

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
    for table in [
        "flow_memberships",
        "flow_snapshots",
        "flows",
        "community_summaries",
        "communities",
        "risk_index",
        "edges",
        "nodes",
        "metadata",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut *tx)
            .await?;
    }
    let _ = sqlx::query("INSERT INTO nodes_fts(nodes_fts) VALUES('delete-all')")
        .execute(&mut *tx)
        .await;

    let now = Utc::now().timestamp_millis() as f64 / 1000.0;
    for node in &snapshot.nodes {
        sqlx::query("INSERT INTO nodes (id, kind, name, qualified_name, file_path, line_start, line_end, language, parent_name, params, return_type, modifiers, is_test, file_hash, extra, updated_at, signature, community_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)")
            .bind(node.id)
            .bind(&node.kind)
            .bind(&node.name)
            .bind(&node.qualified_name)
            .bind(&node.file_path)
            .bind(node.line_start)
            .bind(node.line_end)
            .bind(&node.language)
            .bind(&node.parent_name)
            .bind(&node.params)
            .bind(&node.return_type)
            .bind(&node.modifiers)
            .bind(if node.is_test { 1 } else { 0 })
            .bind(&node.file_hash)
            .bind(&node.extra)
            .bind(now)
            .bind(&node.signature)
            .bind(node.community_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO nodes_fts(nodes_fts) VALUES('rebuild')")
        .execute(&mut *tx)
        .await?;

    for edge in &snapshot.edges {
        sqlx::query("INSERT INTO edges (kind, source_qualified, target_qualified, file_path, line, extra, confidence, confidence_tier, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1.0, 'EXTRACTED', ?7)")
            .bind(&edge.kind)
            .bind(&edge.source_qualified)
            .bind(&edge.target_qualified)
            .bind(&edge.file_path)
            .bind(edge.line)
            .bind(&edge.extra)
            .bind(now)
            .execute(&mut *tx)
            .await?;
    }

    let computed_at = Utc::now().to_rfc3339();
    for risk in &snapshot.risks {
        sqlx::query("INSERT INTO risk_index (node_id, qualified_name, risk_score, caller_count, test_coverage, security_relevant, last_computed) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
            .bind(risk.node_id)
            .bind(&risk.qualified_name)
            .bind(risk.risk_score)
            .bind(risk.caller_count)
            .bind(&risk.test_coverage)
            .bind(if risk.security_relevant { 1 } else { 0 })
            .bind(&computed_at)
            .execute(&mut *tx)
            .await?;
    }

    for community in &snapshot.communities {
        sqlx::query("INSERT INTO communities (id, name, level, parent_id, cohesion, size, dominant_language, description, created_at) VALUES (?1, ?2, 0, NULL, ?3, ?4, ?5, ?6, datetime('now'))")
            .bind(community.id)
            .bind(&community.name)
            .bind(community.cohesion)
            .bind(community.size)
            .bind(&community.dominant_language)
            .bind(&community.description)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO community_summaries (community_id, name, purpose, key_symbols, risk, size, dominant_language) VALUES (?1, ?2, ?3, '[]', ?4, ?5, ?6)")
            .bind(community.id)
            .bind(&community.name)
            .bind(&community.description)
            .bind(&community.risk)
            .bind(community.size)
            .bind(&community.dominant_language)
            .execute(&mut *tx)
            .await?;
    }

    for flow in &snapshot.flows {
        let path_json = serde_json::to_string(&flow.node_ids)?;
        let critical_path = serde_json::to_string(&flow.critical_path)?;
        sqlx::query("INSERT INTO flows (id, name, entry_point_id, depth, node_count, file_count, criticality, path_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), datetime('now'))")
            .bind(flow.id)
            .bind(&flow.name)
            .bind(flow.entry_point_id)
            .bind(flow.depth)
            .bind(flow.node_count)
            .bind(flow.file_count)
            .bind(flow.criticality)
            .bind(&path_json)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO flow_snapshots (flow_id, name, entry_point, critical_path, criticality, node_count, file_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
            .bind(flow.id)
            .bind(&flow.name)
            .bind(&flow.entry_point)
            .bind(&critical_path)
            .bind(flow.criticality)
            .bind(flow.node_count)
            .bind(flow.file_count)
            .execute(&mut *tx)
            .await?;
        for (position, node_id) in flow.node_ids.iter().enumerate() {
            sqlx::query("INSERT OR IGNORE INTO flow_memberships (flow_id, node_id, position) VALUES (?1, ?2, ?3)")
                .bind(flow.id)
                .bind(*node_id)
                .bind(position as i64)
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
        ("schema_version", "9".to_string()),
        ("last_build_type", "startup_full".to_string()),
        ("postprocess_level", "full".to_string()),
        ("last_updated", timestamp.clone()),
        ("last_postprocessed_at", timestamp),
        (
            "git_branch",
            git_output(&snapshot.root, &["branch", "--show-current"]).unwrap_or_default(),
        ),
        (
            "git_head_sha",
            git_output(&snapshot.root, &["rev-parse", "HEAD"]).unwrap_or_default(),
        ),
    ];
    for (key, value) in metadata {
        sqlx::query("INSERT INTO metadata (key, value) VALUES (?1, ?2)")
            .bind(key)
            .bind(value)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Metadata: [db]
