//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Swarm Reaper (Data Lifecycle)**: Orchestrates the permanent deletion
//! of mission-specific data to prevent disk saturation and database bloat.
//! Enforces the **48-Hour Retention Policy**: missions older than 48 hours
//! that are not marked as `is_pinned` are purged. Cleans up **Relational
//! Records** (Mission Logs, Context, Relationships) and the **Physical
//! Workspace** on disk (REAP-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Filesystem permission errors during workspace purge,
//!   database locks during batch deletion, or incomplete cleanup (orphaned logs).
//! - **Trace Scope**: `server-rs::agent::reaper`

use crate::state::AppState;
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::time::{sleep, Duration as TokioDuration};

pub struct SwarmReaper;

impl SwarmReaper {
    /// Starts the Swarm Reaper background loop.
    pub async fn start(state: Arc<AppState>) {
        tracing::info!("♻️ [Reaper] Swarm Reaper background service launched (Interval: 1h).");
        
        loop {
            // Wait for the next sweep interval
            sleep(TokioDuration::from_secs(3600)).await;

            if let Err(e) = Self::sweep(&state.resources.pool).await {
                tracing::error!("🚨 [Reaper] Batch sweep failed: {:?}", e);
            }
        }
    }

    /// Performs a single cleanup sweep of the entire system.
    pub async fn sweep(pool: &SqlitePool) -> Result<()> {
        let threshold = Utc::now() - Duration::hours(48);
        tracing::info!("🧹 [Reaper] Starting sweep for missions older than {}", threshold);

        // 1. Identify candidate missions for reaping
        let missions: Vec<(String,)> = sqlx::query_as(
            "SELECT id FROM mission_history WHERE updated_at < ? AND is_pinned = 0"
        )
        .bind(threshold)
        .fetch_all(pool)
        .await?;

        if missions.is_empty() {
            tracing::info!("🧹 [Reaper] Sweep complete. No stale missions found.");
            return Ok(());
        }

        let total_count = missions.len();
        tracing::info!("🧹 [Reaper] Found {} stale missions to harvest.", total_count);

        for (mission_id,) in missions {
            tracing::debug!("🧹 [Reaper] Harvesting mission: {}", mission_id);

            // 2. Clear Database Records (Manual cascade since schema lacks FK CASCADE)
            // SEC: Order matters to maintain referential integrity during the sweep.
            let mut tx = pool.begin().await?;

            sqlx::query("DELETE FROM mission_logs WHERE mission_id = ?").bind(&mission_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM swarm_context WHERE mission_id = ?").bind(&mission_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM oversight_log WHERE mission_id = ?").bind(&mission_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM mission_relationships WHERE from_id = ? OR to_id = ?")
                .bind(&mission_id)
                .bind(&mission_id)
                .execute(&mut *tx)
                .await?;
            
            sqlx::query("DELETE FROM mission_history WHERE id = ?").bind(&mission_id).execute(&mut *tx).await?;

            tx.commit().await?;

            // 3. Purge Physical Workspace
            // Logic: data/workspaces/{cluster}/missions/{mission_id}
            Self::purge_workspace_files(&mission_id).await;
        }

        tracing::info!("✅ [Reaper] Successfully harvested {} missions.", total_count);
        Ok(())
    }

    /// Purges all physical workspace files associated with a reaped mission.
    async fn purge_workspace_files(mission_id: &str) {
        let base_dir = std::path::Path::new("data/workspaces");
        if !base_dir.exists() {
            return;
        }

        // We iterate through all clusters to find the mission subdirectory.
        // Missions are organized as data/workspaces/{cluster_name}/missions/{mission_id}
        if let Ok(mut entries) = tokio::fs::read_dir(base_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let cluster_mission_path = entry.path().join("missions").join(mission_id);
                if cluster_mission_path.exists() && cluster_mission_path.is_dir() {
                    tracing::info!("📂 [Reaper] Purging physical workspace: {:?}", cluster_mission_path);
                    if let Err(e) = tokio::fs::remove_dir_all(&cluster_mission_path).await {
                        tracing::error!("🚨 [Reaper] Failed to purge workspace {:?}: {}", cluster_mission_path, e);
                    }
                }
            }
        }
    }
}

// Metadata: [reaper]

// Metadata: [reaper]
