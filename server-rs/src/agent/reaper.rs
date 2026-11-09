//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Swarm Reaper (Data Lifecycle & Idle Actor Hibernation)**: Orchestrates the
//! permanent deletion of mission-specific data to prevent disk saturation and
//! database bloat. Enforces the **48-Hour Retention Policy**: missions older than
//! 48 hours that are not marked as `is_pinned` are purged. Cleans up **Relational
//! Records** (Mission Logs, Context, Relationships) and the **Physical
//! Workspace** on disk (REAP-01).
//! Incorporates **Idle Actor Hibernation** (Infinity pattern): sweeps and transitions
//! inactive idle agents into suspended states to reclaim system resources.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Link**: Search `[Reaper]` in tracing logs.
//! - **Failure Path**: Filesystem permission errors during workspace purge,
//!   database locks during batch deletion, or incomplete cleanup.
//! - **Trace Scope**: `server-rs::agent::reaper`

use crate::error::AppError;
use crate::state::AppState;
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

            if let Err(e) = Self::sweep(&state).await {
                tracing::error!("🚨 [Reaper] Batch sweep failed: {:?}", e);
            }

            // Periodic Merkle Audit Ledger Integrity Verification
            match state.security.audit_trail.verify_last_n(250, None).await {
                Ok((verified, total)) => {
                    if total > 0 && verified == total {
                        tracing::info!(
                            "🛡️ [Audit Ledger] Periodic integrity verified ({} of {} records valid).",
                            verified,
                            total
                        );
                    } else if total > 0 {
                        tracing::error!(
                            "🚨 [Audit Ledger] TAMPER DETECTION ALERT: Periodic integrity check failed! ({} of {} valid)",
                            verified,
                            total
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [Audit Ledger] Periodic integrity check encountered error: {}",
                        e
                    );
                }
            }
        }
    }

    /// Performs a single cleanup sweep of the entire system.
    pub async fn sweep(state: &Arc<AppState>) -> Result<(), AppError> {
        let pool = &state.resources.pool;
        let threshold = Utc::now() - Duration::hours(48);
        tracing::info!(
            "🧹 [Reaper] Starting sweep for missions older than {}",
            threshold
        );

        // 0a. Sliding-Window Checkpoint Pruning: Retain only the latest 3 turn checkpoints for terminal missions
        let prune_res = sqlx::query(
            "DELETE FROM mission_checkpoints 
             WHERE mission_id IN (SELECT id FROM mission_history WHERE status IN ('completed', 'failed', 'cancelled'))
               AND rowid NOT IN (
                   SELECT rowid FROM (
                       SELECT rowid, ROW_NUMBER() OVER (PARTITION BY mission_id ORDER BY turn_index DESC) as rn
                       FROM mission_checkpoints
                   ) WHERE rn <= 3
               )"
        )
        .execute(pool)
        .await;
        if let Ok(res) = prune_res {
            if res.rows_affected() > 0 {
                tracing::info!(
                    "🧹 [Reaper] Pruned {} obsolete turn checkpoints.",
                    res.rows_affected()
                );
            }
        }

        // 0b. Sweep and hibernate inactive swarm agents (Infinity pattern)
        if let Err(e) = Self::reap_idle_agents(pool, 300).await {
            tracing::warn!("⚠️ [Reaper] Idle agent hibernation check failed: {:?}", e);
        }

        // 1. Identify candidate missions for reaping
        let missions: Vec<(String,)> =
            sqlx::query_as("SELECT id FROM mission_history WHERE updated_at < ? AND is_pinned = 0")
                .bind(threshold)
                .fetch_all(pool)
                .await
                .map_err(AppError::Sqlx)?;

        if missions.is_empty() {
            tracing::info!("🧹 [Reaper] Sweep complete. No stale missions found.");
            return Ok(());
        }

        let total_count = missions.len();
        tracing::info!(
            "🧹 [Reaper] Found {} stale missions to harvest.",
            total_count
        );

        for (mission_id,) in missions {
            tracing::debug!("🧹 [Reaper] Harvesting mission: {}", mission_id);

            // 2. Clear Database Records (Manual cascade since schema lacks FK CASCADE)
            // S-04: Atomic check against is_pinned race. If pinned after initial candidate selection, abort deletion.
            let mut tx = pool.begin().await.map_err(AppError::Sqlx)?;

            let deleted = sqlx::query("DELETE FROM mission_history WHERE id = ? AND is_pinned = 0")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;

            if deleted.rows_affected() == 0 {
                tracing::info!(
                    "🧹 [Reaper] Skipping mission {}: protected by pin or already reaped",
                    mission_id
                );
                let _ = tx.rollback().await;
                continue;
            }

            sqlx::query("DELETE FROM mission_logs WHERE mission_id = ?")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;
            sqlx::query("DELETE FROM swarm_context WHERE mission_id = ?")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;
            sqlx::query("DELETE FROM oversight_log WHERE mission_id = ?")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;
            sqlx::query("DELETE FROM mission_relationships WHERE from_id = ? OR to_id = ?")
                .bind(&mission_id)
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;
            sqlx::query("DELETE FROM agent_directives WHERE mission_id = ?")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;
            sqlx::query("DELETE FROM peer_reviews WHERE mission_id = ?")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;
            sqlx::query("DELETE FROM mission_nodes WHERE mission_id = ?")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;
            sqlx::query("DELETE FROM mission_checkpoints WHERE mission_id = ?")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;
            sqlx::query("DELETE FROM file_revisions WHERE mission_id = ?")
                .bind(&mission_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Sqlx)?;

            tx.commit().await.map_err(AppError::Sqlx)?;

            // 3. Clear stale active_mission pointers from in-memory registry
            state.registry.agents.iter_mut().for_each(|mut entry| {
                let is_stale = entry
                    .state
                    .active_mission
                    .as_ref()
                    .and_then(|m| m.get("id"))
                    .and_then(|v| v.as_str())
                    == Some(&mission_id);
                if is_stale {
                    entry.state.active_mission = None;
                    tracing::debug!(
                        "🧹 [Reaper] Cleared stale active_mission for agent {}",
                        entry.identity.id
                    );
                }
            });

            // 4. Purge Physical Workspace
            // Logic: data/workspaces/{cluster}/missions/{mission_id}
            Self::purge_workspace_files(&mission_id).await;
        }

        tracing::info!(
            "✅ [Reaper] Successfully harvested {} missions.",
            total_count
        );
        Ok(())
    }

    /// Reaps inactive idle swarm agents into a suspended/hibernating state,
    /// reclaiming in-memory resources and Tokio scheduler slots (Infinity pattern).
    pub async fn reap_idle_agents(
        pool: &SqlitePool,
        idle_threshold_seconds: i64,
    ) -> Result<u64, AppError> {
        let mut tx = pool.begin().await.map_err(AppError::Sqlx)?;

        let res = sqlx::query(
            "UPDATE agents 
             SET status = 'suspended' 
             WHERE status = 'idle' 
               AND (tokens_used = 0 OR tokens_used IS NULL)
               AND id NOT IN ('1', '2', 'alpha', 'general')
               AND (updated_at IS NULL OR updated_at < datetime('now', '-' || ? || ' seconds'))",
        )
        .bind(idle_threshold_seconds)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Sqlx)?;

        let suspended_count = res.rows_affected();
        tx.commit().await.map_err(AppError::Sqlx)?;

        if suspended_count > 0 {
            tracing::info!(
                "💤 [Reaper] Reaped and suspended {} idle swarm agents (inactivity > {}s).",
                suspended_count,
                idle_threshold_seconds
            );
        }

        Ok(suspended_count)
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
                    tracing::info!(
                        "📂 [Reaper] Purging physical workspace: {:?}",
                        cluster_mission_path
                    );
                    if let Err(e) = tokio::fs::remove_dir_all(&cluster_mission_path).await {
                        tracing::error!(
                            "🚨 [Reaper] Failed to purge workspace {:?}: {}",
                            cluster_mission_path,
                            e
                        );
                    }
                }
            }
        }
    }
}
