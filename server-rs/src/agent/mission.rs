use crate::agent::types::{Mission, MissionLog, MissionStatus};
use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Creates a new mission in the database and initializes its financial tracking.
/// 
/// # Arguments
/// * `pool` - The SQLite connection pool.
/// * `agent_id` - The unique identifier of the agent owning the mission.
/// * `title` - A descriptive title for the mission objective.
/// * `budget_usd` - The initial monetary cap for this mission's execution.
/// 
/// # Errors
/// Returns an error if the specified agent ID does not exist in the registry.
pub async fn create_mission(
    pool: &SqlitePool,
    agent_id: &str,
    title: &str,
    budget_usd: f64,
) -> Result<Mission> {
    let mission_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let mission = Mission {
        id: mission_id,
        agent_id: agent_id.to_string(),
        title: title.to_string(),
        status: MissionStatus::Pending,
        created_at: now,
        updated_at: now,
        budget_usd,
        cost_usd: 0.0,
        is_degraded: None,
    };

    // Diagnostic check: Does the agent exist?
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents WHERE id = ?")
        .bind(agent_id)
        .fetch_one(pool)
        .await?;

    if count == 0 {
        return Err(anyhow::anyhow!(
            "Agent ID '{}' not found in database",
            agent_id
        ));
    }

    sqlx::query(
        "INSERT INTO mission_history (id, agent_id, title, status, budget_usd, cost_usd, created_at, updated_at, is_degraded)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
    .bind(&mission.id)
    .bind(&mission.agent_id)
    .bind(&mission.title)
    .bind("pending")
    .bind(mission.budget_usd)
    .bind(mission.cost_usd)
    .bind(mission.created_at)
    .bind(mission.updated_at)
    .bind(mission.is_degraded)
    .execute(pool)
    .await?;

    Ok(mission)
}

/// Updates mission status and increments its cumulative cost.
/// 
/// This function is called after every agent turn to ensure the financial ledger 
/// and mission lifecycle state remain synchronous with reality.
pub async fn update_mission(
    pool: &SqlitePool,
    mission_id: &str,
    status: MissionStatus,
    cost_usd: f64,
) -> Result<()> {
    let status_str = status_to_str(&status);
    let now = Utc::now();

    sqlx::query(
        "UPDATE mission_history SET status = ?1, cost_usd = cost_usd + ?2, updated_at = ?3 WHERE id = ?4")
    .bind(status_str)
    .bind(cost_usd)
    .bind(now)
    .bind(mission_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Logs a step for a specific mission.
pub async fn log_step(
    pool: &SqlitePool,
    mission_id: &str,
    agent_id: &str,
    source: &str,
    text: &str,
    severity: &str,
    metadata: Option<serde_json::Value>,
) -> Result<MissionLog> {
    let log_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let metadata_json = metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());

    sqlx::query(
        "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
    .bind(&log_id)
    .bind(mission_id)
    .bind(agent_id)
    .bind(source)
    .bind(text)
    .bind(severity)
    .bind(now)
    .bind(metadata_json)
    .execute(pool)
    .await?;

    Ok(MissionLog {
        id: log_id,
        mission_id: mission_id.to_string(),
        agent_id: agent_id.to_string(),
        source: source.to_string(),
        text: text.to_string(),
        severity: severity.to_string(),
        timestamp: now,
        metadata,
    })
}

#[allow(dead_code)]
pub async fn get_last_active_mission(pool: &SqlitePool, agent_id: &str) -> Result<Option<Mission>> {
    let mission = sqlx::query_as::<_, Mission>(
        "SELECT * FROM mission_history WHERE agent_id = ?1 AND status IN ('pending', 'active') ORDER BY created_at DESC LIMIT 1")
    .bind(agent_id)
    .fetch_optional(pool)
    .await?;

    Ok(mission)
}

/// Shares a finding to the swarm context bus.
pub async fn share_finding(
    pool: &SqlitePool,
    mission_id: &str,
    agent_id: &str,
    topic: &str,
    finding: &str,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO swarm_context (id, mission_id, agent_id, topic, finding) VALUES (?1, ?2, ?3, ?4, ?5)")
    .bind(id)
    .bind(mission_id)
    .bind(agent_id)
    .bind(topic)
    .bind(finding)
    .execute(pool)
    .await?;
    Ok(())
}

/// Retrieves all findings for a mission to provide context to an agent.
pub async fn get_mission_context(pool: &SqlitePool, mission_id: &str) -> Result<String> {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT agent_id, topic, finding FROM swarm_context WHERE mission_id = ?1 ORDER BY timestamp ASC")
    .bind(mission_id)
    .fetch_all(pool)
    .await?;

    let mut context = String::new();
    for row in rows {
        context.push_str(&format!(
            "[Context from {} on {}]: {}\n",
            row.0, row.1, row.2
        ));
    }
    Ok(context)
}

/// Retrieves a mission by its ID.
pub async fn get_mission_by_id(pool: &SqlitePool, mission_id: &str) -> Result<Option<Mission>> {
    let mission = sqlx::query_as::<_, Mission>("SELECT * FROM mission_history WHERE id = ?1")
        .bind(mission_id)
        .fetch_optional(pool)
        .await?;

    Ok(mission)
}

/// Retrieves recent missions for financial auditing.
pub async fn get_recent_missions(pool: &SqlitePool, limit: i64) -> Result<Vec<Mission>> {
    let missions = sqlx::query_as::<_, Mission>("SELECT * FROM mission_history ORDER BY updated_at DESC LIMIT ?1")
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(missions)
}

/// Retrieves all logs for a given mission.
pub async fn get_mission_logs(pool: &SqlitePool, mission_id: &str) -> Result<Vec<MissionLog>> {
    let rows =
        sqlx::query("SELECT id, mission_id, agent_id, source, text, severity, timestamp, metadata FROM mission_logs WHERE mission_id = ?1 ORDER BY timestamp ASC")
            .bind(mission_id)
            .fetch_all(pool)
            .await?;

    let mut logs = Vec::new();
    for row in rows {
        let metadata_str: Option<String> = row.get("metadata");
        let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

        logs.push(MissionLog {
            id: row.get("id"),
            mission_id: row.get("mission_id"),
            agent_id: row.get("agent_id"),
            source: row.get("source"),
            text: row.get("text"),
            severity: row.get("severity"),
            timestamp: row.get("timestamp"),
            metadata,
        });
    }
    Ok(logs)
}

// ─────────────────────────────────────────────────────────
//  HELPERS
// ─────────────────────────────────────────────────────────

fn status_to_str(status: &MissionStatus) -> &'static str {
    match status {
        MissionStatus::Pending => "pending",
        MissionStatus::Active => "active",
        MissionStatus::Completed => "completed",
        MissionStatus::Failed => "failed",
        MissionStatus::Paused => "paused",
    }
}
