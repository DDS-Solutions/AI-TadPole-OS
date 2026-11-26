//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! - Keep claim, release, reaping and deletion operations transaction-safe and status-consistent.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Persistence / agent lifecycle
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Sqlx`
//! - **Telemetry Targets**: `[Persistence]`

use crate::error::AppError;
use sqlx::SqlitePool;

/// ### 🔗 Orchestration: Atomic Resource Claiming
/// Atomically claims an agent for a mission by setting its status to 'busy' and stamping heartbeat.
/// Returns `Ok(true)` if the agent was successfully claimed via the atomic locking
/// mechanism, or `Ok(false)` if the agent is already engaged in another reasoning turn.
pub async fn claim_agent(pool: &SqlitePool, agent_id: &str) -> Result<bool, AppError> {
    let now = chrono::Utc::now();
    // 1. Try atomic claim for non-busy agents (handles 'idle', 'active', 'ready', etc.)
    let res = sqlx::query(
        "UPDATE agents SET status = 'busy', heartbeat_at = ?1 WHERE id = ?2 AND status != 'busy'",
    )
    .bind(now)
    .bind(agent_id)
    .execute(pool)
    .await?;

    if res.rows_affected() > 0 {
        return Ok(true);
    }

    // 2. Check if agent row exists in DB
    let existing_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM agents WHERE id = ?1")
            .bind(agent_id)
            .fetch_optional(pool)
            .await?;

    match existing_status {
        Some(status) => {
            if status == "busy" {
                // Agent is legitimately occupied with another mission
                Ok(false)
            } else {
                // Guarded edge-case update retry
                let res2 = sqlx::query(
                    "UPDATE agents SET status = 'busy', heartbeat_at = ?1 WHERE id = ?2 AND status != 'busy'"
                )
                .bind(now)
                .bind(agent_id)
                .execute(pool)
                .await?;
                Ok(res2.rows_affected() > 0)
            }
        }
        None => {
            // Agent does not exist in DB yet — insert base record marked as 'busy' with stamped heartbeat
            let safe_suffix: String = agent_id
                .chars()
                .take(128)
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
                .collect();
            let safe_name = format!("Agent {}", safe_suffix);

            let res_insert = sqlx::query(
                "INSERT INTO agents (id, name, role, department, description, status, created_at, heartbeat_at, category, metadata, skills, workflows, mcp_tools, version) \
                 VALUES (?1, ?2, 'Specialist', 'Core', 'Auto-created continuity agent identity', 'busy', ?3, ?3, 'user', '{}', '[]', '[]', '[]', 1)"
            )
            .bind(agent_id)
            .bind(safe_name)
            .bind(now)
            .execute(pool)
            .await;

            match res_insert {
                Ok(_) => Ok(true),
                Err(e)
                    if e.as_database_error()
                        .is_some_and(|db_error| db_error.is_unique_violation()) =>
                {
                    tracing::debug!(agent_id = %agent_id, error = %e, "[Persistence] Concurrent agent insert race; retrying guarded claim");
                    // Guarded fallback attempt if inserted concurrently
                    let res_retry = sqlx::query(
                        "UPDATE agents SET status = 'busy', heartbeat_at = ?1 WHERE id = ?2 AND status != 'busy'",
                    )
                    .bind(now)
                    .bind(agent_id)
                    .execute(pool)
                    .await?;
                    Ok(res_retry.rows_affected() > 0)
                }
                Err(e) => Err(AppError::Sqlx(e)),
            }
        }
    }
}

/// ### 🔓 Persistence: Agent Release
/// Releases a claimed agent by restoring its status to 'idle' and clearing active mission coordinates.
pub async fn release_agent(pool: &SqlitePool, agent_id: &str) -> Result<bool, AppError> {
    let res = sqlx::query("UPDATE agents SET status = 'idle', active_mission = NULL, current_task = NULL WHERE id = ? AND status = 'busy'")
        .bind(agent_id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() > 0)
}

/// ### ⚖️ Governance Rationale: The Swarm Reaper
/// Identifies and harvests agents marked as 'busy' that have exceeded their heartbeat threshold.
/// Resets status to 'idle', clears active_mission/current_task, and reconciles abandoned missions in mission_history.
pub async fn reap_stale_agents(pool: &SqlitePool, threshold_secs: i64) -> Result<u64, AppError> {
    let now = chrono::Utc::now();
    let threshold_time = now - chrono::Duration::seconds(threshold_secs);

    // 1. Identify stale agent IDs and active missions
    let stale_agents: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT id, active_mission FROM agents \
         WHERE status = 'busy' AND (heartbeat_at IS NULL OR heartbeat_at < ?)",
    )
    .bind(threshold_time)
    .fetch_all(pool)
    .await?;

    for (agent_id, active_mission_raw) in &stale_agents {
        let mut mission_ids = Vec::new();
        if let Some(ref raw) = active_mission_raw {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
                if let Some(id_str) = val.get("id").and_then(|v| v.as_str()) {
                    mission_ids.push(id_str.to_string());
                }
            } else if !raw.is_empty() {
                mission_ids.push(raw.clone());
            }
        }

        // Also query mission_history for active/pending missions under this agent
        let db_missions: Vec<(String,)> = sqlx::query_as(
            "SELECT id FROM mission_history WHERE agent_id = ? AND status IN ('active', 'pending')",
        )
        .bind(agent_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (db_mid,) in db_missions {
            if !mission_ids.contains(&db_mid) {
                mission_ids.push(db_mid);
            }
        }

        for mid in mission_ids {
            let log_id = uuid::Uuid::new_v4().to_string();
            let log_text = format!(
                "⚠️ [Reaper] Agent '{}' heartbeat timed out (>{}s). Active mission marked failed.",
                agent_id, threshold_secs
            );

            let _ = sqlx::query(
                "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp, hash) \
                 VALUES (?1, ?2, ?3, 'system', ?4, 'warning', ?5, '')",
            )
            .bind(&log_id)
            .bind(&mid)
            .bind(agent_id)
            .bind(&log_text)
            .bind(now)
            .execute(pool)
            .await;

            let _ = sqlx::query(
                "UPDATE mission_history SET status = 'failed', updated_at = ?1 WHERE id = ?2 AND status IN ('active', 'pending')",
            )
            .bind(now)
            .bind(&mid)
            .execute(pool)
            .await;
        }
    }

    let res = sqlx::query(
        "UPDATE agents SET status = 'idle', active_mission = NULL, current_task = NULL \
         WHERE status = 'busy' AND (heartbeat_at IS NULL OR heartbeat_at < ?)",
    )
    .bind(threshold_time)
    .execute(pool)
    .await?;

    let reaped = res.rows_affected();
    if reaped > 0 {
        tracing::info!("♻️ [Persistence] Reaped {} stale agent runs.", reaped);
    }
    Ok(reaped)
}

/// ### 📡 Telemetry: Heartbeat Propagation
/// Updates the heartbeat timestamp in the database for the specified agent.
pub async fn update_agent_heartbeat(pool: &SqlitePool, agent_id: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE agents SET heartbeat_at = ? WHERE id = ?")
        .bind(chrono::Utc::now())
        .bind(agent_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Transactionally deletes an agent and cascades deletions to all associated metadata tables.
/// Enforces strict child-first ordering to comply with SQLite foreign key constraints.
pub async fn delete_agent_cascade(pool: &SqlitePool, agent_id: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Sqlx)?;

    // 1. Delete granular mission execution logs and nodes (depend on mission_history)
    sqlx::query(
        "DELETE FROM mission_logs WHERE agent_id = ?1 OR mission_id IN (SELECT id FROM mission_history WHERE agent_id = ?1)",
    )
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM swarm_context WHERE agent_id = ?1 OR mission_id IN (SELECT id FROM mission_history WHERE agent_id = ?1)",
    )
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM agent_directives WHERE source_agent_id = ?1 OR target_agent_id = ?1 OR mission_id IN (SELECT id FROM mission_history WHERE agent_id = ?1)",
    )
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM peer_reviews WHERE requester_id = ?1 OR reviewer_id = ?1 OR mission_id IN (SELECT id FROM mission_history WHERE agent_id = ?1)",
    )
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM oversight_log WHERE agent_id = ?1 OR mission_id IN (SELECT id FROM mission_history WHERE agent_id = ?1)",
    )
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM mission_quotas WHERE cluster_id IN (SELECT id FROM mission_history WHERE agent_id = ?1)",
    )
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM mission_relationships WHERE from_id IN (SELECT id FROM mission_history WHERE agent_id = ?1) OR to_id IN (SELECT id FROM mission_history WHERE agent_id = ?1)",
    )
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM durable_workflows WHERE agent_id = ?1 OR mission_id IN (SELECT id FROM mission_history WHERE agent_id = ?1)",
    )
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    // 2. Delete parent missions referencing agent_id
    sqlx::query("DELETE FROM mission_history WHERE agent_id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    // 3. Delete metadata, tasks, quotas, policies, and manifests
    sqlx::query("DELETE FROM agent_quotas WHERE entity_id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM agent_permission_policies WHERE agent_id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM agent_capability_policies WHERE agent_id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM agent_balances WHERE agent_id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM agent_asset_registry WHERE owner_agent_id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM sync_manifest WHERE agent_id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM audit_trail WHERE agent_id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    // 4. Finally, delete the agent row itself (last, satisfying FKs)
    sqlx::query("DELETE FROM agents WHERE id = ?")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await.map_err(AppError::Sqlx)?;
    Ok(())
}
