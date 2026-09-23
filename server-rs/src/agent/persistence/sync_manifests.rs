//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / sync_manifests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::{EngineAgent, SyncManifest};
use crate::error::AppError;
use sqlx::SqlitePool;

/// Synchronizes an agent's connector configurations with the `sync_manifest` table.
///
/// This ensures that the background data ingestion workers know which URIs to watch
/// for a specific specialist agent. It handles both "Cleanup" (deleting removed URIs)
/// and "Discovery" (adding new URIs or updating source types).
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

    // 2. Add or update manifests (updating source_type if reconfigured)
    for config in &agent.connector_configs {
        sqlx::query(
            "INSERT INTO sync_manifest (id, agent_id, source_type, source_uri, status) \
             VALUES (?, ?, ?, ?, 'idle') \
             ON CONFLICT(id) DO UPDATE SET source_type = excluded.source_type",
        )
        .bind(format!("{}-{}", agent.identity.id, config.uri))
        .bind(&agent.identity.id)
        .bind(&config.r#type)
        .bind(&config.uri)
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

/// Loads all sync manifests from the database with explicit column projection.
pub async fn load_sync_manifests(pool: &SqlitePool) -> Result<Vec<SyncManifest>, AppError> {
    let rows = sqlx::query_as::<_, SyncManifest>(
        "SELECT id, agent_id, source_type, source_uri, last_sync_at, checksum, status, metadata, file_count, total_bytes \
         FROM sync_manifest"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Updates the status of a sync manifest. Returns `NotFound` if ID does not exist.
pub async fn update_sync_status(pool: &SqlitePool, id: &str, status: &str) -> Result<(), AppError> {
    let res = sqlx::query("UPDATE sync_manifest SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Sync manifest '{}' not found",
            id
        )));
    }

    Ok(())
}

/// Retrieves all sync manifests for system-wide observability.
/// Alias for [`load_sync_manifests`] to maintain API naming convention compatibility.
#[inline]
pub async fn get_all_sync_manifests(pool: &SqlitePool) -> Result<Vec<SyncManifest>, AppError> {
    load_sync_manifests(pool).await
}

/// Records a successful sync completion. Returns `NotFound` if ID does not exist.
pub async fn complete_sync(
    pool: &SqlitePool,
    id: &str,
    last_sync: chrono::DateTime<chrono::Utc>,
    file_count: i32,
    total_bytes: i64,
) -> Result<(), AppError> {
    let res = sqlx::query(
        "UPDATE sync_manifest SET last_sync_at = ?, status = 'idle', file_count = ?, total_bytes = ? WHERE id = ?"
    )
    .bind(last_sync)
    .bind(file_count)
    .bind(total_bytes)
    .bind(id)
    .execute(pool)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Sync manifest '{}' not found",
            id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE sync_manifest (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_uri TEXT NOT NULL,
                last_sync_at DATETIME,
                checksum TEXT,
                status TEXT NOT NULL DEFAULT 'idle',
                metadata TEXT,
                file_count INTEGER DEFAULT 0,
                total_bytes INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_sync_manifests_lifecycle() {
        let pool = setup_test_db().await;
        let mut conn = pool.acquire().await.unwrap();

        let mut agent = EngineAgent::default();
        agent.identity.id = "specialist-1".to_string();
        agent.connector_configs = vec![crate::agent::types::ConnectorConfig {
            r#type: "local_dir".to_string(),
            uri: "file:///docs".to_string(),
        }];

        sync_manifests_for_agent(&mut conn, &agent).await.unwrap();
        let manifests = load_sync_manifests(&pool).await.unwrap();
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].source_type, "local_dir");

        // Update connector type
        agent.connector_configs[0].r#type = "github_repo".to_string();
        sync_manifests_for_agent(&mut conn, &agent).await.unwrap();
        let updated_manifests = load_sync_manifests(&pool).await.unwrap();
        assert_eq!(updated_manifests.len(), 1);
        assert_eq!(updated_manifests[0].source_type, "github_repo");

        // Status update
        let manifest_id = &updated_manifests[0].id;
        update_sync_status(&pool, manifest_id, "syncing")
            .await
            .unwrap();

        // Complete sync
        complete_sync(&pool, manifest_id, chrono::Utc::now(), 42, 1024)
            .await
            .unwrap();
        let completed = load_sync_manifests(&pool).await.unwrap();
        assert_eq!(completed[0].status, "idle");
        assert_eq!(completed[0].file_count, 42);
        assert_eq!(completed[0].total_bytes, 1024);

        // Removal cleanup
        agent.connector_configs.clear();
        sync_manifests_for_agent(&mut conn, &agent).await.unwrap();
        let cleared = load_sync_manifests(&pool).await.unwrap();
        assert_eq!(cleared.len(), 0);
    }
}
