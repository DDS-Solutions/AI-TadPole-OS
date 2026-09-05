//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / agent_db
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` In-memory `agent.version` is incremented strictly after transaction commit succeeds.
//! - `[Behavioral]` Optimistic concurrency check rejects concurrent writes on version mismatch.
//!   - enforced_by: `test_agent_persistence_round_trip`
//! - `[Behavioral]` Stale agent reaping clears active mission locks and sets status to idle.
//!   - enforced_by: `test_claim_agent_and_reaper_lifecycle`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Persistence]`
//! - **Witness Tests**: `test_agent_persistence_round_trip`, `test_claim_agent_and_reaper_lifecycle`

use super::sync_manifests::sync_manifests_for_agent;
use crate::agent::types::{
    AgentCapabilities, AgentEconomics, AgentHealth, AgentIdentity, AgentModels, AgentState,
    EngineAgent, TokenUsage,
};
use crate::error::AppError;
use sqlx::SqlitePool;

const DEFAULT_PROVIDER: &str = "google";
const DEFAULT_MODEL_ID: &str = "gemini-1.5-pro";
const DEFAULT_CATEGORY: &str = "user";

fn parse_json_field<T: serde::de::DeserializeOwned + Default>(
    col_name: &str,
    agent_id: &str,
    raw_str: &str,
) -> T {
    let trimmed = raw_str.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return T::default();
    }
    if let Ok(val) = serde_json::from_str::<T>(trimmed) {
        return val;
    }
    // Handle double-escaped JSON strings stored in legacy DB rows
    if let Ok(unescaped_str) = serde_json::from_str::<String>(trimmed) {
        if let Ok(val) = serde_json::from_str::<T>(&unescaped_str) {
            tracing::debug!(
                "ℹ️ [Persistence] Rescued double-escaped JSON for {} in agent {}",
                col_name,
                agent_id
            );
            return val;
        }
    }
    let sanitized = trimmed.replace(r#"\""#, r#"""#);
    if let Ok(val) = serde_json::from_str::<T>(&sanitized) {
        tracing::debug!(
            "ℹ️ [Persistence] Rescued escaped quote JSON for {} in agent {}",
            col_name,
            agent_id
        );
        return val;
    }
    // Handle pseudo-array strings like `[" item1\,item2\]`
    if (trimmed.starts_with('[') || trimmed.starts_with(r#"[""#))
        && (trimmed.ends_with(']') || trimmed.ends_with(r#"\]"#))
    {
        let inner = trimmed
            .trim_start_matches(|c| c == '[' || c == '"' || c == ' ')
            .trim_end_matches(|c| c == ']' || c == '\\' || c == '"' || c == ' ');
        let items: Vec<String> = inner
            .split(&[',', '\\'][..])
            .map(|s| {
                s.trim()
                    .trim_matches('"')
                    .replace('\\', "")
                    .trim()
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();
        if let Ok(serialized) = serde_json::to_string(&items) {
            if let Ok(val) = serde_json::from_str::<T>(&serialized) {
                tracing::debug!(
                    "ℹ️ [Persistence] Rescued pseudo-array format for {} in agent {}",
                    col_name,
                    agent_id
                );
                return val;
            }
        }
    }
    tracing::warn!(
        "⚠️ [Persistence] Failed to parse {} for agent {}. Falling back to default.",
        col_name,
        agent_id
    );
    T::default()
}

/// ### 🏢 Type-Safe Flat Agent Row Mapping
/// Flat representation of the SQLite `agents` table row for type-safe query decoding.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FlatAgentRow {
    pub id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub description: String,
    pub model_id: Option<String>,
    pub tokens_used: Option<i64>,
    pub status: String,
    pub current_task: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub theme_color: Option<String>,
    pub budget_usd: Option<f64>,
    pub cost_usd: Option<f64>,
    pub voice_id: Option<String>,
    pub voice_engine: Option<String>,
    pub metadata: Option<String>,
    pub skills: Option<String>,
    pub workflows: Option<String>,
    pub mcp_tools: Option<String>,
    pub connector_configs: Option<String>,
    pub model_2: Option<String>,
    pub model_3: Option<String>,
    pub model_config2: Option<String>,
    pub model_config3: Option<String>,
    pub active_model_slot: Option<i32>,
    pub category: Option<String>,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f64>,
    pub active_mission: Option<String>,
    pub failure_count: Option<i64>,
    pub last_failure_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub heartbeat_at: Option<chrono::DateTime<chrono::Utc>>,
    pub requires_oversight: Option<bool>,
    pub shadows_human_id: Option<String>,
    pub working_memory: Option<String>,
    pub version: i64,
}

impl FlatAgentRow {
    pub fn to_engine_agent(&self) -> EngineAgent {
        let agent_id = self.id.clone();
        let metadata_str = self.metadata.as_deref().unwrap_or("{}");
        let metadata = parse_json_field("metadata", &agent_id, metadata_str);

        let input_tokens = self.input_tokens.unwrap_or(0) as u64;
        let output_tokens = self.output_tokens.unwrap_or(0) as u64;

        let provider_str = self.provider.as_deref().unwrap_or(DEFAULT_PROVIDER);
        let provider = match crate::agent::types::ModelProvider::from_str(provider_str) {
            Some(p) => p,
            None => {
                tracing::warn!(
                    agent_id = %agent_id,
                    provider = %provider_str,
                    "⚠️ [Persistence] Unknown model provider '{}' for agent {}; falling back to default ({})",
                    provider_str, agent_id, DEFAULT_PROVIDER
                );
                crate::agent::types::ModelProvider::Google
            }
        };

        let skills_str = self.skills.as_deref().unwrap_or("[]");
        let workflows_str = self.workflows.as_deref().unwrap_or("[]");
        let mcp_tools_str = self.mcp_tools.as_deref().unwrap_or("[]");

        let connector_configs = self
            .connector_configs
            .as_deref()
            .map(|s| parse_json_field("connector_configs", &agent_id, s))
            .unwrap_or_default();

        EngineAgent {
            identity: AgentIdentity {
                id: agent_id.clone(),
                name: self.name.clone(),
                role: self.role.clone(),
                department: self.department.clone(),
                description: self.description.clone(),
                category: self
                    .category
                    .clone()
                    .unwrap_or_else(|| DEFAULT_CATEGORY.to_string()),
                theme_color: self.theme_color.clone(),
            },
            models: AgentModels {
                model_id: self.model_id.clone(),
                model: crate::agent::types::ModelConfig {
                    provider,
                    model_id: self
                        .model_id
                        .as_deref()
                        .unwrap_or(DEFAULT_MODEL_ID)
                        .to_string(),
                    api_key: self.api_key.clone(),
                    base_url: self.base_url.clone(),
                    system_prompt: self.system_prompt.clone(),
                    temperature: self.temperature.map(|f| f as f32),
                    ..Default::default()
                },
                planning_slot: self
                    .model_config2
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok()),
                execution_slot: self
                    .model_config3
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok()),
                active_model_slot: self.active_model_slot.map(|slot| match slot {
                    1 => "planning".to_string(),
                    2 => "execution".to_string(),
                    _ => "primary".to_string(),
                }),
            },
            economics: AgentEconomics {
                budget_usd: self.budget_usd.unwrap_or(0.0),
                cost_usd: self.cost_usd.unwrap_or(0.0),
                tokens_used: self.tokens_used.unwrap_or(0) as u64,
                token_usage: TokenUsage {
                    input_tokens,
                    output_tokens,
                    total_tokens: input_tokens + output_tokens,
                },
            },
            health: AgentHealth {
                status: self.status.clone(),
                failure_count: self.failure_count.unwrap_or(0) as u32,
                last_failure_at: self.last_failure_at,
                heartbeat_at: self.heartbeat_at,
            },
            state: AgentState {
                current_task: self.current_task.clone(),
                active_mission: self
                    .active_mission
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok()),
                working_memory: self
                    .working_memory
                    .as_deref()
                    .map(|s| parse_json_field("working_memory", &agent_id, s))
                    .unwrap_or_default(),
                current_reasoning_turn: 0,
            },
            capabilities: AgentCapabilities {
                skills: parse_json_field("skills", &agent_id, skills_str),
                workflows: parse_json_field("workflows", &agent_id, workflows_str),
                mcp_tools: parse_json_field("mcp_tools", &agent_id, mcp_tools_str),
                skill_manifest: None,
            },
            connector_configs,
            voice_id: self.voice_id.clone(),
            voice_engine: self.voice_engine.clone(),
            requires_oversight: self.requires_oversight.unwrap_or(false),
            shadows_human_id: self.shadows_human_id.clone(),
            created_at: self.created_at,
            metadata,
            version: self.version as u32,
        }
    }
}

const SELECT_AGENTS_QUERY: &str = "SELECT id, name, role, department, description, model_id, tokens_used, status, current_task,
            input_tokens, output_tokens, theme_color, budget_usd, cost_usd, voice_id, metadata,
            skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2,
            model_config3, active_model_slot, voice_engine, category, provider, api_key, base_url,
            system_prompt, temperature, active_mission, failure_count, last_failure_at, created_at,
            heartbeat_at, requires_oversight, shadows_human_id, working_memory, version FROM agents ORDER BY id ASC";

const SELECT_AGENT_BY_ID_QUERY: &str = "SELECT id, name, role, department, description, model_id, tokens_used, status, current_task,
            input_tokens, output_tokens, theme_color, budget_usd, cost_usd, voice_id, metadata,
            skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2,
            model_config3, active_model_slot, voice_engine, category, provider, api_key, base_url,
            system_prompt, temperature, active_mission, failure_count, last_failure_at, created_at,
            heartbeat_at, requires_oversight, shadows_human_id, working_memory, version FROM agents WHERE id = ?";

/// Loads all agents from the SQLite database.
pub async fn load_agents_db(pool: &SqlitePool) -> Result<Vec<EngineAgent>, AppError> {
    let rows = sqlx::query_as::<_, FlatAgentRow>(SELECT_AGENTS_QUERY)
        .fetch_all(pool)
        .await
        .map_err(AppError::Sqlx)?;

    Ok(rows.into_iter().map(|r| r.to_engine_agent()).collect())
}

/// Loads a single agent by ID from the SQLite database.
pub async fn load_agent_by_id(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<Option<EngineAgent>, AppError> {
    let row = sqlx::query_as::<_, FlatAgentRow>(SELECT_AGENT_BY_ID_QUERY)
        .bind(agent_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Sqlx)?;

    Ok(row.map(|r| r.to_engine_agent()))
}

/// Alias for [`load_agent_by_id`].
#[inline]
pub async fn load_agent_by_id_db(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<Option<EngineAgent>, AppError> {
    load_agent_by_id(pool, agent_id).await
}

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

    let next_version = agent.version + 1;

    let mut query = sqlx::query("INSERT INTO agents (id, name, role, department, description, model_id, tokens_used, status, current_task, input_tokens, output_tokens, theme_color, budget_usd, cost_usd, metadata, skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2, model_config3, active_model_slot, voice_id, voice_engine, failure_count, last_failure_at, created_at, heartbeat_at, active_mission, provider, api_key, base_url, system_prompt, temperature, category, requires_oversight, shadows_human_id, working_memory, version)
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
            WHERE agents.id = excluded.id AND agents.version = ?");

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
                    "planning" => 1,
                    "execution" => 2,
                    _ => 0,
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

    let result = query.execute(executor).await.map_err(AppError::Sqlx)?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(format!(
            "Agent '{}' update failed due to version mismatch (concurrency conflict).",
            agent.identity.id
        )));
    }

    Ok(next_version)
}

pub async fn save_agent_db(pool: &SqlitePool, agent: &mut EngineAgent) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Sqlx)?;
    let next_version = execute_save_agent(&mut *tx, agent).await?;
    sync_manifests_for_agent(&mut tx, agent).await?;
    tx.commit().await.map_err(AppError::Sqlx)?;
    agent.version = next_version;
    Ok(())
}

/// Transaction-compatible variant of `save_agent_db`.
pub async fn save_agent_db_in_tx(
    conn: &mut sqlx::SqliteConnection,
    agent: &mut EngineAgent,
) -> Result<(), AppError> {
    let next_version = execute_save_agent(&mut *conn, agent).await?;
    sync_manifests_for_agent(conn, agent).await?;
    agent.version = next_version;
    Ok(())
}

/// Persists all agent entries to disk sorted by ID, stripping API keys and writing atomically.
pub async fn save_agents_json(
    base_dir: &std::path::Path,
    mut agents: Vec<EngineAgent>,
) -> Result<(), AppError> {
    if cfg!(test) {
        return Ok(());
    }
    agents.sort_by(|a, b| a.identity.id.cmp(&b.identity.id));
    let agents_file = crate::utils::security::validate_path(base_dir, "data/agents.json")?;

    // SEC-02 Credential Polarization: Redact API keys before writing to disk
    for agent in &mut agents {
        agent.models.model.api_key = None;
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
            let res_insert = sqlx::query(
                "INSERT INTO agents (id, name, role, department, description, status, created_at, heartbeat_at, category) \
                 VALUES (?1, ?2, 'Specialist', 'Core', 'Auto-created continuity agent identity', 'busy', ?3, ?3, 'user')"
            )
            .bind(agent_id)
            .bind(format!("Agent {}", agent_id))
            .bind(now)
            .execute(pool)
            .await;

            match res_insert {
                Ok(_) => Ok(true),
                Err(e) => {
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
            }
        }
    }
}

/// ### 🔓 Persistence: Agent Release
/// Releases a claimed agent by restoring its status to 'idle'.
pub async fn release_agent(pool: &SqlitePool, agent_id: &str) -> Result<bool, AppError> {
    let res = sqlx::query("UPDATE agents SET status = 'idle' WHERE id = ? AND status = 'busy'")
        .bind(agent_id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() > 0)
}

/// ### ⚖️ Governance Rationale: The Swarm Reaper
/// Identifies and harvests agents marked as 'busy' that have exceeded their heartbeat threshold.
/// Resets status to 'idle' and clears active_mission and current_task to eliminate ghosts.
pub async fn reap_stale_agents(pool: &SqlitePool, threshold_secs: i64) -> Result<u64, AppError> {
    let now = chrono::Utc::now();
    let threshold_time = now - chrono::Duration::seconds(threshold_secs);

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
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        sqlx::query(
            "CREATE TABLE agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'Specialist',
                department TEXT NOT NULL DEFAULT 'Core',
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL,
                category TEXT NOT NULL DEFAULT 'user',
                active_mission TEXT,
                current_task TEXT,
                heartbeat_at DATETIME,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                version INTEGER NOT NULL DEFAULT 1
            )",
        )
        .execute(&pool)
        .await?;

        // 1. Claim auto-creates and claims agent
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
}
