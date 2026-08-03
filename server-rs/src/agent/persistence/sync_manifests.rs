//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Incremental Sync Manifest Persistence**: Manages external source tracking and connector synchronization state in SQLite.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Failed URI manifest query or background worker sync state tracking failure.
//! - **Telemetry Link**: Search for `[SyncManifest]` in server logs.

use crate::agent::types::{EngineAgent, SyncManifest};
use crate::error::AppError;
use sqlx::SqlitePool;

/// Synchronizes an agent's connector configurations with the `sync_manifest` table.
///
/// This ensures that the background data ingestion workers know which URIs to watch
/// for a specific specialist agent. It handles both "Cleanup" (deleting removed URIs)
/// and "Discovery" (adding new URIs).
pub async fn sync_manifests_for_agent(
    conn: &mut sqlx::SqliteConnection,
    agent: &EngineAgent,
) -> Result<(), AppError> {
    // 1. Delete manifests that are no longer in the agent config
    let current_uris: Vec<String> = agent
        .connector_configs
        .iter()
        .map(|c| c.uri.clone())
        .collect();

    sqlx::query("DELETE FROM sync_manifest WHERE agent_id = ? AND source_uri NOT IN (SELECT value FROM json_each(?))")
        .bind(&agent.identity.id)
        .bind(serde_json::to_string(&current_uris).map_err(|e| AppError::InternalServerError(e.to_string()))?)
        .execute(&mut *conn)
        .await?;

    // 2. Add new manifests
    for config in &agent.connector_configs {
        sqlx::query("INSERT OR IGNORE INTO sync_manifest (id, agent_id, source_type, source_uri, status) VALUES (?, ?, ?, ?, 'idle')")
            .bind(format!("{}-{}", agent.identity.id, config.uri))
            .bind(&agent.identity.id)
            .bind(&config.r#type)
            .bind(&config.uri)
            .execute(&mut *conn)
            .await?;
    }

    Ok(())
}

/// Loads all sync manifests from the database.
pub async fn load_sync_manifests(pool: &SqlitePool) -> Result<Vec<SyncManifest>, AppError> {
    let rows = sqlx::query_as::<_, SyncManifest>("SELECT * FROM sync_manifest")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// Updates the status of a sync manifest.
pub async fn update_sync_status(pool: &SqlitePool, id: &str, status: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE sync_manifest SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Retrieves all sync manifests for system-wide observability.
/// Alias for [`load_sync_manifests`] to maintain API naming convention compatibility.
#[inline]
pub async fn get_all_sync_manifests(pool: &SqlitePool) -> Result<Vec<SyncManifest>, AppError> {
    load_sync_manifests(pool).await
}

/// Records a successful sync completion.
pub async fn complete_sync(
    pool: &SqlitePool,
    id: &str,
    last_sync: chrono::DateTime<chrono::Utc>,
    file_count: i32,
    total_bytes: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE sync_manifest SET last_sync_at = ?, status = 'idle', file_count = ?, total_bytes = ? WHERE id = ?"
    )
    .bind(last_sync)
    .bind(file_count)
    .bind(total_bytes)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
