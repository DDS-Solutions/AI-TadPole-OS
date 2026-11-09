//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! - Coordinate agent row mapping, database writes and lifecycle operations through focused modules.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / agent_db
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Sqlx`, `AppError::Conflict`
//! - **Telemetry Targets**: none declared

mod lifecycle;
mod reads;
mod row;

pub use lifecycle::{
    claim_agent, delete_agent_cascade, reap_stale_agents, release_agent, update_agent_heartbeat,
};
pub use reads::{load_agent_by_id, load_agent_by_id_db, load_agents_db};
pub use row::FlatAgentRow;

use super::sync_manifests::sync_manifests_for_agent;
use crate::agent::types::EngineAgent;
use crate::error::AppError;
use sqlx::SqlitePool;

/// Saves an agent using optimistic concurrency version checks. Returns next version upon success.
pub async fn execute_save_agent<'c, E>(executor: E, agent: &EngineAgent) -> Result<u32, AppError>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    let primary_model_id = if agent.models.model.model_id.trim().is_empty() {
        None
    } else {
        Some(agent.models.model.model_id.clone())
    }
    .or_else(|| {
        agent
            .models
            .model_id
            .as_ref()
            .filter(|id| !id.trim().is_empty())
            .cloned()
    });

    let next_version = agent.version.checked_add(1).ok_or_else(|| {
        AppError::Conflict(format!("Agent '{}' version overflow", agent.identity.id))
    })?;

    let mut query = sqlx::query_as::<_, (i64,)>("INSERT INTO agents (id, name, role, department, description, model_id, tokens_used, status, current_task, input_tokens, output_tokens, theme_color, budget_usd, cost_usd, metadata, skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2, model_config3, active_model_slot, voice_id, voice_engine, failure_count, last_failure_at, created_at, heartbeat_at, active_mission, provider, api_key, base_url, system_prompt, temperature, category, requires_oversight, shadows_human_id, working_memory, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            role = excluded.role,
            department = excluded.department,
            description = excluded.description,
            model_id = excluded.model_id,
            tokens_used = excluded.tokens_used,
            status = excluded.status,
            current_task = excluded.current_task,
            input_tokens = excluded.input_tokens,
            output_tokens = excluded.output_tokens,
            theme_color = excluded.theme_color,
            budget_usd = excluded.budget_usd,
            cost_usd = excluded.cost_usd,
            metadata = excluded.metadata,
            skills = excluded.skills,
            workflows = excluded.workflows,
            mcp_tools = excluded.mcp_tools,
            connector_configs = excluded.connector_configs,
            model_2 = excluded.model_2,
            model_3 = excluded.model_3,
            model_config2 = excluded.model_config2,
            model_config3 = excluded.model_config3,
            active_model_slot = excluded.active_model_slot,
            voice_id = excluded.voice_id,
            voice_engine = excluded.voice_engine,
            failure_count = excluded.failure_count,
            last_failure_at = excluded.last_failure_at,
            created_at = COALESCE(agents.created_at, excluded.created_at),
            heartbeat_at = excluded.heartbeat_at,
            active_mission = excluded.active_mission,
            provider = excluded.provider,
            api_key = excluded.api_key,
            base_url = excluded.base_url,
            system_prompt = excluded.system_prompt,
            temperature = excluded.temperature,
            category = excluded.category,
            requires_oversight = excluded.requires_oversight,
            shadows_human_id = excluded.shadows_human_id,
            working_memory = excluded.working_memory,
            version = agents.version + 1
            WHERE agents.id = excluded.id AND agents.version = ?
            RETURNING version");

    query = query
        .bind(&agent.identity.id)
        .bind(&agent.identity.name)
        .bind(&agent.identity.role)
        .bind(&agent.identity.department)
        .bind(&agent.identity.description)
        .bind(&primary_model_id)
        .bind(agent.economics.tokens_used as i64)
        .bind(&agent.health.status)
        .bind(&agent.state.current_task)
        .bind(agent.economics.token_usage.input_tokens as i64)
        .bind(agent.economics.token_usage.output_tokens as i64)
        .bind(&agent.identity.theme_color)
        .bind(agent.economics.budget_usd)
        .bind(agent.economics.cost_usd)
        .bind(sqlx::types::Json(&agent.metadata))
        .bind(sqlx::types::Json(&agent.capabilities.skills))
        .bind(sqlx::types::Json(&agent.capabilities.workflows))
        .bind(sqlx::types::Json(&agent.capabilities.mcp_tools))
        .bind(sqlx::types::Json(&agent.connector_configs))
        .bind(
            agent
                .models
                .planning_slot
                .as_ref()
                .map(|s| s.model_id.clone()),
        )
        .bind(
            agent
                .models
                .execution_slot
                .as_ref()
                .map(|s| s.model_id.clone()),
        )
        .bind(agent.models.planning_slot.as_ref().map(sqlx::types::Json))
        .bind(agent.models.execution_slot.as_ref().map(sqlx::types::Json))
        .bind(
            agent
                .models
                .active_model_slot
                .as_ref()
                .map(|s| match s.as_str() {
                    "planning" => 2,
                    "execution" => 3,
                    _ => 1,
                }),
        )
        .bind(&agent.voice_id)
        .bind(&agent.voice_engine)
        .bind(agent.health.failure_count as i64)
        .bind(agent.health.last_failure_at)
        .bind(agent.created_at)
        .bind(agent.health.heartbeat_at)
        .bind(agent.state.active_mission.as_ref().map(sqlx::types::Json))
        .bind(agent.models.model.provider.to_string())
        .bind(&agent.models.model.api_key)
        .bind(&agent.models.model.base_url)
        .bind(&agent.models.model.system_prompt)
        .bind(agent.models.model.temperature.map(|f| f as f64))
        .bind(&agent.identity.category)
        .bind(agent.requires_oversight)
        .bind(&agent.shadows_human_id)
        .bind(sqlx::types::Json(&agent.state.working_memory))
        .bind(next_version as i64)
        .bind(agent.version as i64);

    let row: Option<(i64,)> = query
        .fetch_optional(executor)
        .await
        .map_err(AppError::Sqlx)?;

    match row {
        Some((persisted_version,)) => Ok(persisted_version as u32),
        None => Err(AppError::Conflict(format!(
            "Agent '{}' update failed due to version mismatch (concurrency conflict).",
            agent.identity.id
        ))),
    }
}

pub async fn save_agent_db(pool: &SqlitePool, agent: &mut EngineAgent) -> Result<(), AppError> {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) if attempts < 10 && crate::db::is_sqlite_transient(&e) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(10 * attempts)).await;
                continue;
            }
            Err(e) => return Err(AppError::Sqlx(e)),
        };

        match execute_save_agent(&mut *tx, agent).await {
            Ok(next_version) => {
                if let Err(e) = sync_manifests_for_agent(&mut tx, agent).await {
                    let _ = tx.rollback().await;
                    if attempts < 10 {
                        if let AppError::Sqlx(ref se) = e {
                            if crate::db::is_sqlite_transient(se) {
                                tokio::time::sleep(tokio::time::Duration::from_millis(
                                    10 * attempts,
                                ))
                                .await;
                                continue;
                            }
                        }
                    }
                    return Err(e);
                }
                if let Err(e) = tx.commit().await {
                    if attempts < 10 && crate::db::is_sqlite_transient(&e) {
                        tokio::time::sleep(tokio::time::Duration::from_millis(10 * attempts)).await;
                        continue;
                    }
                    return Err(AppError::Sqlx(e));
                }
                agent.version = next_version;
                return Ok(());
            }
            Err(AppError::Sqlx(ref e)) if attempts < 10 && crate::db::is_sqlite_transient(e) => {
                let _ = tx.rollback().await;
                tokio::time::sleep(tokio::time::Duration::from_millis(10 * attempts)).await;
                continue;
            }
            Err(e) => {
                let _ = tx.rollback().await;
                return Err(e);
            }
        }
    }
}

/// Transaction-compatible variant of `save_agent_db`.
///
/// NOTE: In accordance with `[Structural]` invariants, the in-memory `agent.version`
/// is NOT incremented inside this function, as the transaction has not yet committed.
/// Returns the authoritative `next_version` upon successful execution within the transaction.
/// Callers must update `agent.version = next_version` strictly AFTER their transaction commit succeeds.
pub async fn save_agent_db_in_tx(
    conn: &mut sqlx::SqliteConnection,
    agent: &mut EngineAgent,
) -> Result<u32, AppError> {
    let next_version = execute_save_agent(&mut *conn, agent).await?;
    sync_manifests_for_agent(conn, agent).await?;
    Ok(next_version)
}

/// Persists all agent entries to disk sorted by ID, stripping API keys and writing atomically.
pub async fn save_agents_json(
    base_dir: &std::path::Path,
    mut agents: Vec<EngineAgent>,
) -> Result<(), AppError> {
    agents.sort_by(|a, b| a.identity.id.cmp(&b.identity.id));
    let agents_file = crate::utils::security::validate_path(base_dir, "data/agents.json")?;

    // SEC-02 Credential Polarization: Redact API keys and authorization headers before writing to disk
    for agent in &mut agents {
        // 1. Primary slot
        agent.models.model.api_key = None;
        agent.models.model.custom_headers = None;

        // 2. Planning slot
        if let Some(ref mut slot) = agent.models.planning_slot {
            slot.api_key = None;
            slot.custom_headers = None;
        }

        // 3. Execution slot
        if let Some(ref mut slot) = agent.models.execution_slot {
            slot.api_key = None;
            slot.custom_headers = None;
        }

        // 4. Connector URIs (sanitize embedded basic auth userinfo / tokens)
        for config in &mut agent.connector_configs {
            if let Some(at_idx) = config.uri.find('@') {
                if let Some(scheme_end) = config.uri[..at_idx].find("://") {
                    config.uri = format!(
                        "{}***@{}",
                        &config.uri[..scheme_end + 3],
                        &config.uri[at_idx + 1..]
                    );
                }
            }
        }
    }

    if let Some(parent) = agents_file.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(AppError::Io)?;
    }

    let content = serde_json::to_string_pretty(&agents)?;
    let tmp_file = agents_file.with_extension("tmp");
    tokio::fs::write(&tmp_file, content)
        .await
        .map_err(AppError::Io)?;
    tokio::fs::rename(&tmp_file, agents_file)
        .await
        .map_err(AppError::Io)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_agent_persistence_round_trip() -> Result<(), AppError> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        // Setup Schema
        sqlx::query(
            "CREATE TABLE agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            role TEXT NOT NULL,
            department TEXT NOT NULL,
            description TEXT NOT NULL,
             model_id TEXT,
             tokens_used INTEGER DEFAULT 0,
             status TEXT NOT NULL,
             current_task TEXT,
             input_tokens INTEGER DEFAULT 0,
             output_tokens INTEGER DEFAULT 0,
             theme_color TEXT,
            budget_usd REAL DEFAULT 0.0,
            cost_usd REAL DEFAULT 0.0,
            metadata TEXT NOT NULL,
            skills TEXT,
            workflows TEXT,
            mcp_tools TEXT,
            connector_configs TEXT,
            model_2 TEXT,
            model_3 TEXT,
            model_config2 TEXT,
            model_config3 TEXT,
            active_model_slot INTEGER DEFAULT 1,
            failure_count INTEGER DEFAULT 0,
            last_failure_at DATETIME,
            heartbeat_at DATETIME,
            active_mission TEXT,
            provider TEXT,
            api_key TEXT,
            base_url TEXT,
            system_prompt TEXT,
            temperature REAL,
            voice_id TEXT,
            voice_engine TEXT,
            category TEXT,
            requires_oversight BOOLEAN DEFAULT 0,
            shadows_human_id TEXT,
            working_memory TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            version INTEGER NOT NULL DEFAULT 1
        );
        CREATE TABLE sync_manifest (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_uri TEXT NOT NULL,
            last_sync_at DATETIME,
            status TEXT NOT NULL DEFAULT 'idle',
            file_count INTEGER DEFAULT 0,
            total_bytes INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
        )
        .execute(&pool)
        .await?;

        let mut agent = EngineAgent::default();
        agent.identity.id = "test-agent-1".to_string();
        agent.identity.name = "Test Agent".to_string();
        agent.models.model.model_id = "gemini-1.5-flash".to_string();
        agent.version = 1;

        // Save Agent
        save_agent_db(&pool, &mut agent).await?;
        assert_eq!(
            agent.version, 2,
            "Agent version should increment on successful commit"
        );

        // Load Agent
        let loaded = load_agent_by_id(&pool, "test-agent-1")
            .await?
            .expect("Agent should exist");
        assert_eq!(loaded.identity.id, "test-agent-1");
        assert_eq!(loaded.identity.name, "Test Agent");
        assert_eq!(loaded.models.model.model_id, "gemini-1.5-flash");
        assert_eq!(loaded.version, 2);

        // Optimistic Concurrency Failure Test
        let mut stale_agent = loaded.clone();
        stale_agent.version = 1; // Simulate stale version
        let err = save_agent_db(&pool, &mut stale_agent).await;
        assert!(matches!(err, Err(AppError::Conflict(_))));

        Ok(())
    }

    #[tokio::test]
    async fn test_claim_agent_and_reaper_lifecycle() -> Result<(), AppError> {
        let pool = crate::db::init_db("sqlite::memory:")
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // 1. Claim auto-creates and claims agent against full production migration schema
        let claimed = claim_agent(&pool, "auto-agent").await?;
        assert!(claimed);

        let row: (String, Option<chrono::DateTime<chrono::Utc>>) =
            sqlx::query_as("SELECT status, heartbeat_at FROM agents WHERE id = 'auto-agent'")
                .fetch_one(&pool)
                .await?;
        assert_eq!(row.0, "busy");
        assert!(
            row.1.is_some(),
            "Heartbeat timestamp must be stamped on claim"
        );

        // 2. Second claim fails (agent is busy)
        let second_claim = claim_agent(&pool, "auto-agent").await?;
        assert!(!second_claim);

        // 3. Stale agent reap test
        let past = chrono::Utc::now() - chrono::Duration::seconds(400);
        sqlx::query("UPDATE agents SET heartbeat_at = ?, active_mission = '\"m-1\"', current_task = 'running' WHERE id = 'auto-agent'")
            .bind(past)
            .execute(&pool)
            .await?;

        let reaped = reap_stale_agents(&pool, 300).await?;
        assert_eq!(reaped, 1);

        let reaped_row: (String, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT status, active_mission, current_task FROM agents WHERE id = 'auto-agent'",
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(reaped_row.0, "idle");
        assert_eq!(reaped_row.1, None, "Active mission must be cleared on reap");
        assert_eq!(reaped_row.2, None, "Current task must be cleared on reap");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_agent_cascade_foreign_keys_on() -> Result<(), AppError> {
        let pool = crate::db::init_db("sqlite::memory:").await?;
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;

        // 1. Insert an agent
        let mut agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "cascade-agent-1".to_string(),
                name: "Cascade Agent".to_string(),
                role: "Tester".to_string(),
                ..Default::default()
            },
            version: 1,
            ..Default::default()
        };
        save_agent_db(&pool, &mut agent).await?;

        // 2. Insert dependent mission, mission_log, swarm_context, directives
        let mid = "test-mission-123";
        sqlx::query("INSERT INTO mission_history (id, agent_id, title, status, budget_usd, cost_usd) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(mid)
            .bind("cascade-agent-1")
            .bind("Test Mission")
            .bind("pending")
            .bind(10.0)
            .bind(0.0)
            .execute(&pool)
            .await?;

        sqlx::query("INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity) VALUES (?, ?, ?, ?, ?, ?)")
            .bind("log-1")
            .bind(mid)
            .bind("cascade-agent-1")
            .bind("test")
            .bind("sample text")
            .bind("info")
            .execute(&pool)
            .await?;

        sqlx::query("INSERT INTO swarm_context (id, mission_id, agent_id, topic, finding) VALUES (?, ?, ?, ?, ?)")
            .bind("ctx-1")
            .bind(mid)
            .bind("cascade-agent-1")
            .bind("testing")
            .bind("sample finding")
            .execute(&pool)
            .await?;

        sqlx::query("INSERT INTO agent_directives (id, mission_id, source_agent_id, target_agent_id, instruction, status) VALUES (?, ?, ?, ?, ?, ?)")
            .bind("dir-1")
            .bind(mid)
            .bind("cascade-agent-1")
            .bind("cascade-agent-1")
            .bind("self-instruction")
            .bind("pending")
            .execute(&pool)
            .await?;

        // 3. Delete cascade must succeed child-first without foreign key constraint failure
        delete_agent_cascade(&pool, "cascade-agent-1").await?;

        // 4. Verify all records are wiped cleanly
        let agent_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM agents WHERE id = 'cascade-agent-1'")
                .fetch_one(&pool)
                .await?;
        assert_eq!(agent_count, 0);

        let mission_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM mission_history WHERE id = ?")
                .bind(mid)
                .fetch_one(&pool)
                .await?;
        assert_eq!(mission_count, 0);

        let log_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM mission_logs WHERE mission_id = ?")
                .bind(mid)
                .fetch_one(&pool)
                .await?;
        assert_eq!(log_count, 0);

        let context_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM swarm_context WHERE mission_id = ?")
                .bind(mid)
                .fetch_one(&pool)
                .await?;
        assert_eq!(context_count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_save_agent_in_tx_rollback_does_not_advance_version() -> Result<(), AppError> {
        let pool = crate::db::init_db("sqlite::memory:")
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "tx-agent-1".to_string(),
                name: "Tx Agent".to_string(),
                ..Default::default()
            },
            version: 1,
            ..Default::default()
        };

        save_agent_db(&pool, &mut agent).await?;
        let original_version = agent.version;
        assert_eq!(original_version, 2);

        // Begin transaction, execute save in tx, then abort/rollback
        let mut tx = pool.begin().await?;
        let _staged_ver = save_agent_db_in_tx(&mut tx, &mut agent).await?;
        // Crucial: save_agent_db_in_tx must not mutate agent.version before commit!
        assert_eq!(agent.version, original_version);
        tx.rollback().await?;

        assert_eq!(agent.version, original_version);
        let db_version: i64 =
            sqlx::query_scalar("SELECT version FROM agents WHERE id = 'tx-agent-1'")
                .fetch_one(&pool)
                .await?;
        assert_eq!(db_version, original_version as i64);

        // Next save must succeed without conflict
        save_agent_db(&pool, &mut agent).await?;
        assert_eq!(agent.version, original_version + 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_save_agents_json_multi_slot_credential_redaction() -> Result<(), AppError> {
        let temp_dir = tempfile::tempdir().map_err(AppError::Io)?;
        let mut agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "redact-agent".to_string(),
                name: "Redact Agent".to_string(),
                ..Default::default()
            },
            version: 1,
            ..Default::default()
        };

        agent.models.model.api_key = Some("SECRET_PRIMARY_KEY_12345".to_string());
        agent.models.planning_slot = Some(crate::agent::types::ModelConfig {
            model_id: "claude-3-5-sonnet".to_string(),
            api_key: Some("SECRET_PLANNING_KEY_67890".to_string()),
            ..Default::default()
        });
        agent.models.execution_slot = Some(crate::agent::types::ModelConfig {
            model_id: "gpt-4o".to_string(),
            api_key: Some("SECRET_EXECUTION_KEY_ABCDE".to_string()),
            ..Default::default()
        });
        agent
            .connector_configs
            .push(crate::agent::types::ConnectorConfig {
                r#type: "github".to_string(),
                uri: "https://user:ghp_SUPERSECRET123@github.com/org/repo".to_string(),
            });

        save_agents_json(temp_dir.path(), vec![agent]).await?;

        let written_path = temp_dir.path().join("data/agents.json");
        let content = tokio::fs::read_to_string(&written_path)
            .await
            .map_err(AppError::Io)?;

        assert!(!content.contains("SECRET_PRIMARY_KEY_12345"));
        assert!(!content.contains("SECRET_PLANNING_KEY_67890"));
        assert!(!content.contains("SECRET_EXECUTION_KEY_ABCDE"));
        assert!(!content.contains("ghp_SUPERSECRET123"));
        assert!(content.contains("***@github.com/org/repo"));

        Ok(())
    }

    #[tokio::test]
    async fn test_release_agent_clears_active_mission_and_task() -> Result<(), AppError> {
        let pool = crate::db::init_db("sqlite::memory:")
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        claim_agent(&pool, "release-test-agent").await?;

        // Simulate active mission assignment
        sqlx::query("UPDATE agents SET active_mission = '{\"id\": \"m-99\"}', current_task = 'analyzing' WHERE id = 'release-test-agent'")
            .execute(&pool)
            .await?;

        let released = release_agent(&pool, "release-test-agent").await?;
        assert!(released);

        let row: (String, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT status, active_mission, current_task FROM agents WHERE id = 'release-test-agent'",
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(row.0, "idle");
        assert_eq!(row.1, None, "Active mission must be cleared on release");
        assert_eq!(row.2, None, "Current task must be cleared on release");

        Ok(())
    }
}
