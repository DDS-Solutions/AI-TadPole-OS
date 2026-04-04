//! Mission Store — History, costs and swarm state
//!
//! Orchestrates the creation, tracking, and finality of autonomous missions, 
//! ensuring audit trails and financial ledgers remain synchronous.
//!
//! @docs ARCHITECTURE:Persistence
//!
//! @state MissionStatus: Pending -> Active -> (Completed | Failed | Paused)
//!
//! ### AI Assist Note
//! **Cost Accumulation**: `update_mission` increments `cost_usd`. Agents 
//! must log every significant turn to `log_step` for the mission's 
//! permanent audit trail. Use `share_finding` for swarm context bus.

use crate::agent::types::{Mission, MissionLog, MissionStatus, MissionRelationship, RelationshipType, SwarmGraph, GraphNode, GraphEdge};
use anyhow::Result;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
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
    let count: i64 = sqlx::query_scalar::<sqlx::Sqlite, i64>("SELECT COUNT(*) FROM agents WHERE id = ?")
        .bind(agent_id)
        .fetch_one(pool)
        .await?;

    if count == 0 {
        return Err(anyhow::anyhow!(
            "Agent ID '{}' not found in database",
            agent_id
        ));
    }

    sqlx::query::<sqlx::Sqlite>(
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

    sqlx::query::<sqlx::Sqlite>(
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

    // SEC-03: Prune large payloads before persistence
    let sanitized_metadata = metadata.map(crate::utils::serialization::sanitize_json);
    let mut text_val = serde_json::Value::String(text.to_string());
    crate::utils::serialization::prune_large_strings(&mut text_val, 2048);
    let pruned_text = text_val.as_str().unwrap_or("").to_string();

    let metadata_json = sanitized_metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());

    sqlx::query::<sqlx::Sqlite>(
        "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
    .bind(&log_id)
    .bind(mission_id)
    .bind(agent_id)
    .bind(source)
    .bind(&pruned_text)
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
        text: pruned_text,
        severity: severity.to_string(),
        timestamp: now,
        metadata: sanitized_metadata,
    })
}

#[allow(dead_code)]
pub async fn get_last_active_mission(pool: &SqlitePool, agent_id: &str) -> Result<Option<Mission>> {
    let mission = sqlx::query_as::<sqlx::Sqlite, Mission>(
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

    // SEC-03: Prune large findings
    let mut finding_val = serde_json::Value::String(finding.to_string());
    crate::utils::serialization::prune_large_strings(&mut finding_val, 2048);
    let pruned_finding = finding_val.as_str().unwrap_or("").to_string();

    sqlx::query::<sqlx::Sqlite>(
        "INSERT INTO swarm_context (id, mission_id, agent_id, topic, finding) VALUES (?1, ?2, ?3, ?4, ?5)")
    .bind(id)
    .bind(mission_id)
    .bind(agent_id)
    .bind(topic)
    .bind(pruned_finding)
    .execute(pool)
    .await?;
    Ok(())
}

/// Retrieves all findings for a mission to provide context to an agent.
pub async fn get_mission_context(pool: &SqlitePool, mission_id: &str) -> Result<String> {
    let rows: Vec<(String, String, String)> = sqlx::query_as::<sqlx::Sqlite, (String, String, String)>(
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
    let mission = sqlx::query_as::<sqlx::Sqlite, Mission>("SELECT * FROM mission_history WHERE id = ?1")
        .bind(mission_id)
        .fetch_optional(pool)
        .await?;

    Ok(mission)
}

/// Retrieves recent missions for financial auditing.
pub async fn get_recent_missions(pool: &SqlitePool, limit: i64) -> Result<Vec<Mission>> {
    let missions = sqlx::query_as::<sqlx::Sqlite, Mission>(
        "SELECT * FROM mission_history ORDER BY updated_at DESC LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(missions)
}

/// Retrieves all logs for a given mission.
pub async fn get_mission_logs(pool: &SqlitePool, mission_id: &str) -> Result<Vec<MissionLog>> {
    let rows =
        sqlx::query::<sqlx::Sqlite>("SELECT id, mission_id, agent_id, source, text, severity, timestamp, metadata FROM mission_logs WHERE mission_id = ?1 ORDER BY timestamp ASC")
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

/// Retrieves the most recent logs for a mission (ascending order).
pub async fn get_recent_mission_logs(
    pool: &SqlitePool,
    mission_id: &str,
    limit: i64,
) -> Result<Vec<MissionLog>> {
    let rows = sqlx::query::<sqlx::Sqlite>(
        "SELECT id, mission_id, agent_id, source, text, severity, timestamp, metadata
         FROM mission_logs
         WHERE mission_id = ?1
         ORDER BY timestamp DESC
         LIMIT ?2",
    )
    .bind(mission_id)
    .bind(limit)
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

    logs.reverse();
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

/// Creates a new relationship between two entities in the swarm graph.
/// This is a foundational operation for the Tadpole-OS Knowledge Graph (Phase 1).
pub async fn add_mission_relationship(
    pool: &SqlitePool,
    from_id: &str,
    to_id: &str,
    rel_type: RelationshipType,
    metadata: Option<serde_json::Value>,
) -> Result<MissionRelationship> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let rel_type_str = match rel_type {
        RelationshipType::Blocks => "blocks",
        RelationshipType::DependsOn => "depends_on",
        RelationshipType::AssignedTo => "assigned_to",
        RelationshipType::CreatedBy => "created_by",
        RelationshipType::ParentOf => "parent_of",
    };

    let metadata_json = metadata.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default());

    sqlx::query(
        "INSERT INTO mission_relationships (id, from_id, to_id, relationship_type, metadata, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)")
    .bind(&id)
    .bind(from_id)
    .bind(to_id)
    .bind(rel_type_str)
    .bind(metadata_json)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(MissionRelationship {
        id,
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        relationship_type: rel_type,
        metadata,
        created_at: now,
    })
}

/// Retrieves the complete swarm knowledge graph for visualization.
/// Bridges the relational SQLite data into a graph format for the React UI.
pub async fn get_swarm_graph(pool: &SqlitePool) -> Result<SwarmGraph> {
    // 1. Fetch Agents as Nodes
    let agent_rows = sqlx::query("SELECT id, name, role, status, metadata FROM agents")
        .fetch_all(pool)
        .await?;

    let mut nodes = Vec::new();
    for row in agent_rows {
        let meta_str: Option<String> = row.get("metadata");
        let metadata = meta_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(serde_json::json!({}));
        
        nodes.push(GraphNode {
            id: row.get("id"),
            label: row.get("name"),
            r#type: "agent".to_string(),
            status: row.get("status"),
            metadata,
        });
    }

    // 2. Fetch Active Missions as Nodes
    let mission_rows = sqlx::query("SELECT id, title, status FROM mission_history WHERE status IN ('pending', 'active')")
        .fetch_all(pool)
        .await?;

    for row in mission_rows {
        nodes.push(GraphNode {
            id: row.get("id"),
            label: row.get("title"),
            r#type: "mission".to_string(),
            status: row.get("status"),
            metadata: serde_json::json!({}),
        });
    }

    // 3. Fetch Relationships as Edges
    let rel_rows = sqlx::query("SELECT id, from_id, to_id, relationship_type, metadata FROM mission_relationships")
        .fetch_all(pool)
        .await?;

    let mut edges = Vec::new();
    for row in rel_rows {
        let meta_str: Option<String> = row.get("metadata");
        let metadata = meta_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(serde_json::json!({}));
        
        edges.push(GraphEdge {
            id: row.get("id"),
            source: row.get("from_id"),
            target: row.get("to_id"),
            label: row.get("relationship_type"),
            metadata,
        });
    }

    Ok(SwarmGraph { nodes, edges })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::RelationshipType;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_swarm_graph_generation() -> Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        
        // Setup required tables
        sqlx::query("CREATE TABLE agents (id TEXT PRIMARY KEY, name TEXT, role TEXT, department TEXT, description TEXT, status TEXT, metadata TEXT)").execute(&pool).await?;
        sqlx::query("CREATE TABLE mission_history (id TEXT PRIMARY KEY, agent_id TEXT, title TEXT, status TEXT, budget_usd REAL, cost_usd REAL, created_at DATETIME, updated_at DATETIME, is_degraded BOOLEAN)").execute(&pool).await?;
        sqlx::query("CREATE TABLE mission_relationships (id TEXT PRIMARY KEY, from_id TEXT, to_id TEXT, relationship_type TEXT, metadata TEXT, created_at DATETIME)").execute(&pool).await?;

        // 1. Setup Agents and Missions
        let agent_id = "agent-1";
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status) VALUES ('agent-1', 'Agent One', 'Developer', 'Engineering', 'Test Agent', 'idle')").execute(&pool).await?;
        
        let m1 = create_mission(&pool, agent_id, "Mission Alpha", 10.0).await?;
        let m2 = create_mission(&pool, agent_id, "Mission Beta", 10.0).await?;

        // 2. Add Relationship
        add_mission_relationship(&pool, &m1.id, &m2.id, RelationshipType::Blocks, None).await?;

        // 3. Verify Graph Data
        let graph = get_swarm_graph(&pool).await?;
        
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].source, m1.id);
        assert_eq!(graph.edges[0].target, m2.id);
        assert_eq!(graph.edges[0].label, "blocks");

        Ok(())
    }
}
