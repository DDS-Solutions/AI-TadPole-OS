//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Storage Synchronizer**: Bridges in-memory registries with persistent
//! **SQLite** and **JSON** storage. Orchestrates **Agent Reaping**
//! (`reap_stale_agents`) to ensure the swarm recovers from zombie/crashed
//! runs. Enforces **Credential Polarization** (SEC-02) by prioritizing
//! environment variables over disk-based JSON configs. Features
//! **Incremental Sync Manifests** to track external data ingestion
//! state across engine restarts.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: SQLite statement-cache staling across migrations
//!   (causing row decoding panics), JSON parsing errors in
//!   `infra_providers.json`, or heartbeat timeout misconfiguration
//!   leading to premature reaping.
//! - **Trace Scope**: `server-rs::agent::persistence`

use crate::agent::types::{
    AgentCapabilities, AgentEconomics, AgentHealth, AgentIdentity, AgentModels, AgentState,
    EngineAgent, ModelEntry, ProviderConfig, RoleBlueprint, TokenUsage,
};
use crate::error::AppError;

use sqlx::SqlitePool;

const PROVIDERS_FILE: &str = "data/infra_providers.json";
const MODELS_FILE: &str = "data/infra_models.json";

const DEFAULT_PROVIDER: &str = "google";
const DEFAULT_MODEL_ID: &str = "gemini-1.5-pro";
const DEFAULT_CATEGORY: &str = "user";

async fn read_json_file<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Option<T> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&content).ok()
}

/// ### 🔒 SEC-02: Credential Polarization Helper
/// Maps a protocol-specific provider ID to its canonical environment variable.
/// 
/// Returns `None` for providers that do not require secret keys or utilize
/// alternative authentication mechanisms (e.g., local Ollama instance).
fn provider_env_var(provider_id: &str) -> Option<&'static str> {
    match provider_id {
        "google" | "gemini" => Some("GOOGLE_API_KEY"),
        "groq" => Some("GROQ_API_KEY"),
        "openai" => Some("OPENAI_API_KEY"),
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "inception" => Some("INCEPTION_API_KEY"),
        "deepseek" => Some("DEEPSEEK_API_KEY"),
        _ => None,
    }
}

/// Loads agents from the database.
/// 
/// ### 🔍 Implementation Details: SQLx Reliability (P1)
/// This function uses a manually specified column list instead of `SELECT *`.
/// This is a **Security Pillar** requirement to prevent the `sqlx` row decoder 
/// from panicking if the underlying SQLite schema is migrated (adding/ordering columns) 
/// while the compiled binary's statement cache remains stale (ROUT-02).
/// ### 📡 Retrieval: load_agents_db
/// Reconstitutes the in-memory agent registry from the persistent SQLite store.
/// 
/// ### 🔍 Implementation Details: SQLx Reliability (P1)
/// This function utilizes a manually specified column manifest instead of `SELECT *`.
/// This is a **Security Pillar** requirement (ROUT-02) to prevent row decoding 
/// panics if the underlying schema is migrated (adding or reordering columns) 
/// while the engine binary's cached prepared statements remain stale.
/// 
/// ### 🧬 Logic: Data De-serialization
/// - **Metadata**: De-serialized from a JSONB string into an extensible HashMap.
/// - **Provider Context**: Reconstituted from raw string slugs via `ModelProvider::from_str`.
/// - **Telemetry**: Aggregated from `input_tokens` and `output_tokens` into a unified `TokenUsage` struct.
pub async fn load_agents_db(pool: &SqlitePool) -> Result<Vec<EngineAgent>, AppError> {
    let rows = sqlx::query(
        "SELECT
            id,
            name,
            role,
            department,
            description,
            model_id,
            tokens_used,
            status,
            current_task,
            input_tokens,
            output_tokens,
            theme_color,
            budget_usd,
            cost_usd,
            voice_id,
            metadata,
            skills,
            workflows,
            mcp_tools,
            connector_configs,
            model_2,
            model_3,
            model_config2,
            model_config3,
            active_model_slot,
            voice_engine,
            category,
            provider,
            api_key,
            base_url,
            system_prompt,
            temperature,
            active_mission,
            failure_count,
            last_failure_at,
            created_at,
            heartbeat_at,
            requires_oversight,
            working_memory,
            version
         FROM agents",
    )
    .fetch_all(pool)
    .await?;
    let mut agents = Vec::new();

    for row in rows {
        use sqlx::Row;
        let metadata_str: String = row.get("metadata");
        let metadata: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_str(&metadata_str).unwrap_or_default();
        let input_tokens = row.get::<Option<i64>, _>("input_tokens").unwrap_or(0) as u32;
        let output_tokens = row.get::<Option<i64>, _>("output_tokens").unwrap_or(0) as u32;

        let provider_str: String = row
            .try_get("provider")
            .unwrap_or_else(|_| DEFAULT_PROVIDER.to_string());
        let provider = crate::agent::types::ModelProvider::from_str(&provider_str)
            .unwrap_or(crate::agent::types::ModelProvider::Google);

        let agent = EngineAgent {
            identity: AgentIdentity {
                id: row.get("id"),
                name: row.get("name"),
                role: row.get("role"),
                department: row.get("department"),
                description: row.get("description"),
                category: row.try_get("category").unwrap_or_else(|_| DEFAULT_CATEGORY.to_string()),
                theme_color: row.get("theme_color"),
            },
            models: AgentModels {
                model_id: row.get("model_id"),
                model: crate::agent::types::ModelConfig {
                    provider,
                    model_id: row
                        .get::<Option<String>, _>("model_id")
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or_else(|| DEFAULT_MODEL_ID.to_string()),
                    api_key: row.try_get("api_key").ok(),
                    base_url: row.try_get("base_url").ok(),
                    system_prompt: row.try_get("system_prompt").ok(),
                    temperature: row.get::<Option<f64>, _>("temperature").map(|f| f as f32),
                    ..Default::default()
                },
                model_2: row.try_get("model_2").ok(),
                model_3: row.try_get("model_3").ok(),
                model_config2: row
                    .get::<Option<String>, _>("model_config2")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                model_config3: row
                    .get::<Option<String>, _>("model_config3")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                active_model_slot: row.get::<Option<i32>, _>("active_model_slot"),
            },
            economics: AgentEconomics {
                budget_usd: row.get::<Option<f64>, _>("budget_usd").unwrap_or(0.0),
                cost_usd: row.get::<Option<f64>, _>("cost_usd").unwrap_or(0.0),
                tokens_used: row.get::<Option<i64>, _>("tokens_used").unwrap_or(0) as u32,
                token_usage: TokenUsage {
                    input_tokens,
                    output_tokens,
                    total_tokens: input_tokens + output_tokens,
                },
            },
            health: AgentHealth {
                status: row.get("status"),
                failure_count: row.get::<Option<i64>, _>("failure_count").unwrap_or(0) as u32,
                last_failure_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_failure_at"),
                heartbeat_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("heartbeat_at"),
            },
            capabilities: AgentCapabilities {
                skills: serde_json::from_str(&row.get::<String, _>("skills")).unwrap_or_default(),
                workflows: serde_json::from_str(&row.get::<String, _>("workflows")).unwrap_or_default(),
                mcp_tools: serde_json::from_str(&row.get::<String, _>("mcp_tools")).unwrap_or_default(),
                skill_manifest: None,
            },
            state: AgentState {
                active_mission: row
                    .get::<Option<String>, _>("active_mission")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                current_task: row.get::<Option<String>, _>("current_task"),
                working_memory: row
                    .get::<Option<String>, _>("working_memory")
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({})),
                current_reasoning_turn: 0,
            },
            metadata,
            voice_id: row.try_get("voice_id").ok(),
            voice_engine: row.try_get("voice_engine").ok(),
            connector_configs: row
                .try_get::<Option<String>, _>("connector_configs")
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            created_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at"),
            requires_oversight: row
                .get::<Option<bool>, _>("requires_oversight")
                .unwrap_or(false),
            version: row.get::<i64, _>("version") as u32,
        };
        agents.push(agent);
    }
    Ok(agents)
}

async fn execute_save_agent<'c, E>(executor: E, agent: &EngineAgent) -> Result<(), AppError>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    let primary_model_id = Some(&agent.models.model.model_id)
        .filter(|id| !id.trim().is_empty())
        .cloned()
        .or_else(|| {
            agent
                .models
                .model_id
                .as_ref()
                .filter(|id| !id.trim().is_empty())
                .cloned()
        });

    sqlx::query("INSERT INTO agents (id, name, role, department, description, model_id, tokens_used, status, current_task, input_tokens, output_tokens, theme_color, budget_usd, cost_usd, metadata, skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2, model_config3, active_model_slot, voice_id, voice_engine, failure_count, last_failure_at, created_at, heartbeat_at, active_mission, provider, api_key, base_url, system_prompt, temperature, category, requires_oversight, working_memory, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            created_at = excluded.created_at,
            heartbeat_at = excluded.heartbeat_at,
            active_mission = excluded.active_mission,
            provider = excluded.provider,
            api_key = excluded.api_key,
            base_url = excluded.base_url,
            system_prompt = excluded.system_prompt,
            temperature = excluded.temperature,
            category = excluded.category,
            requires_oversight = excluded.requires_oversight,
            working_memory = excluded.working_memory,
            version = agents.version + 1
            WHERE agents.id = excluded.id AND agents.version = ?")
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
    .bind(&agent.models.model_2)
    .bind(&agent.models.model_3)
    .bind(agent.models.model_config2.as_ref().map(sqlx::types::Json))
    .bind(agent.models.model_config3.as_ref().map(sqlx::types::Json))
    .bind(agent.models.active_model_slot)
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
    .bind(sqlx::types::Json(&agent.state.working_memory))
    .bind(agent.version as i64)
    .bind(agent.version as i64)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn save_agent_db(pool: &SqlitePool, agent: &EngineAgent) -> Result<(), AppError> {
    let mut conn = pool.acquire().await?;
    execute_save_agent(&mut *conn, agent).await?;
    sync_manifests_for_agent(&mut conn, agent).await?;
    Ok(())
}

/// Transaction-compatible variant of `save_agent_db`.
pub async fn save_agent_db_in_tx(
    conn: &mut sqlx::SqliteConnection,
    agent: &EngineAgent,
) -> Result<(), AppError> {
    execute_save_agent(&mut *conn, agent).await?;
    sync_manifests_for_agent(&mut *conn, agent).await?;
    Ok(())
}

/// Synchronizes an agent's connector configurations with the `sync_manifest` table.
/// 
/// This ensures that the background data ingestion workers know which URIs to watch
/// for a specific specialist agent. It handles both "Cleanup" (deleting removed URIs)
/// and "Discovery" (adding new URIs).
/// Synchronizes an agent's connector configurations with the `sync_manifest` table.
/// Synchronizes an agent's connector configurations with the `sync_manifest` table.
async fn sync_manifests_for_agent(
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

/// Loads provider configurations from disk and overlays security context.
/// 
/// ### 🔒 SEC-02: Credential Polarization
/// API keys are NEVER loaded from the raw JSON file if a corresponding environment 
/// variable (e.g., `GOOGLE_API_KEY`) is present. Environment variables are treated 
/// as the Sovereign Root of Truth for credentials.
pub async fn load_providers(base_dir: &std::path::Path) -> Vec<ProviderConfig> {
    let providers_file = crate::utils::security::validate_path(base_dir, PROVIDERS_FILE)
        .unwrap_or_else(|_| crate::utils::security::SafePath::from_trusted(base_dir.join(PROVIDERS_FILE)));
    let mut providers = if providers_file.exists() {
        read_json_file::<Vec<ProviderConfig>>(&providers_file).await.unwrap_or_else(|| {
            tracing::error!(
                file = ?providers_file,
                "❌ [Persistence] Provider JSON parse failure — falling back to defaults"
            );
            crate::agent::registry::get_default_providers()
        })
    } else {
        // Fallback: Check RESOURCE_ROOT (e.g. bundled data)
        let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
        let bundled_file = std::path::Path::new(&resource_root).join(PROVIDERS_FILE);
        read_json_file::<Vec<ProviderConfig>>(&bundled_file).await.unwrap_or_else(crate::agent::registry::get_default_providers)
    };

    // SEC-02: Override api_key from environment variables.
    for provider in &mut providers {
        if let Some(env_var) = provider_env_var(&provider.id) {
            if let Ok(key) = std::env::var(env_var) {
                if !key.trim().is_empty() {
                    provider.api_key = Some(key);
                }
            }
        }
    }

    providers
}

/// ### 🔒 SEC-02: Credential Redaction Pass
/// Persists provider configurations to disk after sanitizing sensitive tokens.
/// 
/// ### 🛰️ Security Note: Identity Leakage Prevention
/// API keys are stripped and replaced with `serde_json::Value::Null` before disk write.
/// This ensures that repo-wide exports or cloud backups do not accidentally contain
/// the `NEURAL_TOKEN` or cloud provider secrets.
pub async fn save_providers(
    base_dir: &std::path::Path,
    providers: Vec<ProviderConfig>,
) -> Result<(), AppError> {
    let providers_file = crate::utils::security::validate_path(base_dir, PROVIDERS_FILE)?;
    let sanitized: Vec<serde_json::Value> = providers
        .iter()
        .map(|p| {
            let mut val = serde_json::to_value(p).unwrap_or_default();
            if let Some(obj) = val.as_object_mut() {
                obj.insert("api_key".to_string(), serde_json::Value::Null);
                obj.remove("apiKey");
            }
            val
        })
        .collect();
    let content = serde_json::to_string_pretty(&sanitized)?;
    tokio::fs::write(providers_file, content)
        .await
        .map_err(AppError::Io)?;
    Ok(())
}

/// Loads the model registry from disk.
pub async fn load_models(base_dir: &std::path::Path) -> Vec<ModelEntry> {
    let models_file = crate::utils::security::validate_path(base_dir, MODELS_FILE)
        .unwrap_or_else(|_| crate::utils::security::SafePath::from_trusted(base_dir.join(MODELS_FILE)));
    if models_file.exists() {
        if let Some(models) = read_json_file::<Vec<ModelEntry>>(&models_file).await {
            return models;
        }
        tracing::error!(file = ?models_file, "❌ [Persistence] Model JSON parse failure — falling back to defaults");
    } else {
        // Fallback: Check RESOURCE_ROOT
        let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
        let bundled_file = std::path::Path::new(&resource_root).join(MODELS_FILE);
        if let Some(models) = read_json_file::<Vec<ModelEntry>>(&bundled_file).await {
            return models;
        }
    }
    crate::agent::registry::get_default_models()
}

/// Persists all model entries to disk.
pub async fn save_models(base_dir: &std::path::Path, models: Vec<ModelEntry>) -> Result<(), AppError> {
    let models_file = crate::utils::security::validate_path(base_dir, MODELS_FILE)?;
    let content = serde_json::to_string_pretty(&models)?;
    tokio::fs::write(models_file, content)
        .await
        .map_err(AppError::Io)?;
    Ok(())
}

/// ### 🔗 Orchestration: Atomic Resource Claiming
/// Atomically claims an agent for a mission by setting its status to 'busy'.
/// Returns `Ok(true)` if the agent was successfully claimed via the atomic locking 
/// mechanism, or `Ok(false)` if the agent is already engaged in another reasoning turn.
pub async fn claim_agent(pool: &SqlitePool, agent_id: &str) -> Result<bool, AppError> {
    let res = sqlx::query("UPDATE agents SET status = 'busy' WHERE id = ? AND status = 'idle'")
        .bind(agent_id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() > 0)
}

/// ### ⚖️ Governance Rationale: The Swarm Reaper
/// Identifies and harvests agents marked as 'busy' that have exceeded their heartbeat threshold.
/// 
/// This is the system's "Safety Valve." If an agent process crashes, hangs, or 
/// context-overflows without completing its mission, this reaper returns 
/// the agent to the available pool (`idle`) so the swarm can re-negotiate the task. 
/// 
/// Prevents permanent "Busy" locks in the database (LIF-03) and ensuring 
/// swarm availability across high-concurrency mission cycles.
pub async fn reap_stale_agents(pool: &SqlitePool, threshold_secs: i64) -> Result<u64, AppError> {
    let now = chrono::Utc::now();
    // Use safe subtraction to determine the high-water mark for zombie processes.
    let threshold_time = now - chrono::Duration::seconds(threshold_secs);

    let res = sqlx::query("UPDATE agents SET status = 'idle' WHERE status = 'busy' AND (heartbeat_at IS NULL OR heartbeat_at < ?)")
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
/// This prevents the `reap_stale_agents` safety valve from prematurely harvesting
/// an active long-running mission.
pub async fn update_agent_heartbeat(pool: &SqlitePool, agent_id: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE agents SET heartbeat_at = ? WHERE id = ?")
        .bind(chrono::Utc::now())
        .bind(agent_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Loads all sync manifests from the database.
pub async fn load_sync_manifests(pool: &SqlitePool) -> Result<Vec<crate::agent::SyncManifest>, AppError> {
    let rows = sqlx::query_as::<_, crate::agent::SyncManifest>("SELECT * FROM sync_manifest")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// ### ⚖️ Governance: Blueprint Discovery
/// Loads all registered Role Blueprints from the database.
pub async fn load_blueprints(pool: &SqlitePool) -> Result<Vec<RoleBlueprint>, AppError> {
    let rows = sqlx::query_as::<_, RoleBlueprint>(
        "SELECT id, name, department, description, skills, workflows, mcp_tools, requires_oversight, model_id, created_at FROM role_blueprints"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// ### ⚖️ Governance: Promote to Role
/// Persists a Role Blueprint to the database.
pub async fn save_blueprint(pool: &SqlitePool, blueprint: &RoleBlueprint) -> Result<(), AppError> {
    execute_save_blueprint(pool, blueprint).await
}

/// Transaction-compatible variant of `save_blueprint`.
pub async fn execute_save_blueprint<'c, E>(executor: E, blueprint: &RoleBlueprint) -> Result<(), AppError>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    sqlx::query(
        "INSERT INTO role_blueprints (id, name, department, description, skills, workflows, mcp_tools, requires_oversight, model_id, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            department = excluded.department,
            description = excluded.description,
            skills = excluded.skills,
            workflows = excluded.workflows,
            mcp_tools = excluded.mcp_tools,
            requires_oversight = excluded.requires_oversight,
            model_id = excluded.model_id"
    )
    .bind(&blueprint.id)
    .bind(&blueprint.name)
    .bind(&blueprint.department)
    .bind(&blueprint.description)
    .bind(&blueprint.skills)
    .bind(&blueprint.workflows)
    .bind(&blueprint.mcp_tools)
    .bind(blueprint.requires_oversight)
    .bind(&blueprint.model_id)
    .bind(blueprint.created_at.unwrap_or_else(chrono::Utc::now))
    .execute(executor)
    .await?;
    Ok(())
}

/// ### ⚖️ Governance: Role Retirement
/// Deletes a Role Blueprint from the system.
pub async fn delete_blueprint<'c, E>(executor: E, id: &str) -> Result<(), AppError>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    sqlx::query("DELETE FROM role_blueprints WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await?;
    Ok(())
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

/// Records a successful sync completion.
pub async fn complete_sync(
    pool: &SqlitePool,
    id: &str,
    last_sync: chrono::DateTime<chrono::Utc>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE sync_manifest SET last_sync_at = ?, status = 'idle' WHERE id = ?")
        .bind(last_sync)
        .bind(id)
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

        // 1. Setup Schema (Matching exactly the final state after migrations)
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
            working_memory TEXT DEFAULT '{}',
            version INTEGER DEFAULT 1,
            created_at DATETIME
        )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE sync_manifest (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_uri TEXT NOT NULL,
                status TEXT NOT NULL,
                last_sync_at DATETIME
            )",
        )
        .execute(&pool)
        .await?;

        // 2. Create Agent with full config
        let mut agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "test-agent-1".to_string(),
                name: "Test Agent".to_string(),
                role: "Tester".to_string(),
                department: "QA".to_string(),
                description: "A test agent".to_string(),
                theme_color: Some("#ff0000".to_string()),
                ..Default::default()
            },
            health: crate::agent::types::AgentHealth {
                status: "idle".to_string(),
                ..Default::default()
            },
            capabilities: crate::agent::types::AgentCapabilities {
                skills: vec!["testing".to_string()],
                ..Default::default()
            },
            models: crate::agent::types::AgentModels {
                model_id: Some("gemini-1.5-pro".to_string()),
                ..Default::default()
            },
            version: 1,
            ..Default::default()
        };

        agent.models.model.provider = crate::agent::types::ModelProvider::Anthropic;
        agent.models.model.model_id = "claude-3-5-sonnet".to_string();
        agent.models.model.api_key = Some("sk-test-123".to_string());
        agent.models.model.base_url = Some("https://api.anthropic.com".to_string());
        agent.models.model.system_prompt = Some("You are a tester.".to_string());
        agent.models.model.temperature = Some(0.7);

        agent.state.current_task = Some("Validating persistence parity".to_string());
        agent.economics.token_usage = TokenUsage {
            input_tokens: 321,
            output_tokens: 123,
            total_tokens: 444,
        };
        agent.economics.tokens_used = 2222;
        agent.state.working_memory = serde_json::json!({"milestone": "test-passed"});

        // 3. Save
        save_agent_db(&pool, &agent).await?;

        // 4. Load
        let agents = load_agents_db(&pool).await?;
        let loaded = agents
            .iter()
            .find(|a| a.identity.id == "test-agent-1")
            .expect("Agent not found");

        // 5. Assert Parity
        assert_eq!(loaded.identity.name, agent.identity.name);
        assert_eq!(loaded.models.model.provider, agent.models.model.provider);
        assert_eq!(loaded.models.model.model_id, agent.models.model.model_id);
        assert_eq!(loaded.models.model.api_key, agent.models.model.api_key);
        assert_eq!(loaded.models.model.base_url, agent.models.model.base_url);
        assert_eq!(loaded.models.model.system_prompt, agent.models.model.system_prompt);
        assert_eq!(loaded.models.model.temperature, agent.models.model.temperature);
        assert_eq!(loaded.identity.theme_color, agent.identity.theme_color);
        assert_eq!(loaded.capabilities.skills, agent.capabilities.skills);
        assert_eq!(loaded.requires_oversight, agent.requires_oversight);
        assert_eq!(loaded.state.current_task, agent.state.current_task);
        assert_eq!(
            loaded.economics.token_usage.input_tokens,
            agent.economics.token_usage.input_tokens
        );
        assert_eq!(
            loaded.economics.token_usage.output_tokens,
            agent.economics.token_usage.output_tokens
        );
        assert_eq!(
            loaded.economics.token_usage.total_tokens,
            agent.economics.token_usage.total_tokens
        );
        assert_eq!(loaded.economics.tokens_used, agent.economics.tokens_used);
        assert_eq!(loaded.state.working_memory["milestone"], "test-passed");

        Ok(())
    }

    #[tokio::test]
    async fn test_atomic_claiming_and_reaping() -> Result<(), AppError> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        // 1. Setup Schema (abbreviated for the test)
        sqlx::query("CREATE TABLE agents (id TEXT PRIMARY KEY, status TEXT NOT NULL, heartbeat_at DATETIME)")
            .execute(&pool)
            .await?;

        sqlx::query(
            "INSERT INTO agents (id, status, heartbeat_at) VALUES ('agent-1', 'idle', NULL)",
        )
        .execute(&pool)
        .await?;

        // 2. Test Claiming
        let success = claim_agent(&pool, "agent-1").await?;
        assert!(success, "First claim should succeed");

        let success_retry = claim_agent(&pool, "agent-1").await?;
        assert!(!success_retry, "Second claim on busy agent should fail");

        // 3. Test Reaping (Set heartbeat to 600s ago)
        let old_heartbeat = chrono::Utc::now() - chrono::Duration::seconds(600);
        sqlx::query("UPDATE agents SET heartbeat_at = ? WHERE id = 'agent-1'")
            .bind(old_heartbeat)
            .execute(&pool)
            .await?;

        let reaped = reap_stale_agents(&pool, 300).await?;
        assert_eq!(reaped, 1, "Should reap 1 stale agent");

        let final_status: String =
            sqlx::query_scalar("SELECT status FROM agents WHERE id = 'agent-1'")
                .fetch_one(&pool)
                .await?;
        assert_eq!(final_status, "idle", "Reaped agent should be idle");

        Ok(())
    }
}

// Metadata: [persistence]

// Metadata: [persistence]
