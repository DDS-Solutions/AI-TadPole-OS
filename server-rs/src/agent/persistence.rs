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

use crate::agent::types::{EngineAgent, ModelEntry, ProviderConfig, TokenUsage};

use anyhow::{Context, Result};
use sqlx::SqlitePool;

const PROVIDERS_FILE: &str = "data/infra_providers.json";
const MODELS_FILE: &str = "data/infra_models.json";

/// Maps a provider ID to its corresponding environment variable name.
/// Returns `None` for providers that don't use API keys (e.g. ollama).
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
pub async fn load_agents_db(pool: &SqlitePool) -> Result<Vec<EngineAgent>> {
    // Avoid `SELECT *` here: SQLite statement-cache metadata can go stale across
    // schema migrations, which may panic in sqlx row decoding when column counts drift.
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
            working_memory
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
            .unwrap_or_else(|_| "google".to_string());
        let provider = crate::agent::types::ModelProvider::from_str(&provider_str)
            .unwrap_or(crate::agent::types::ModelProvider::Google);

        let agent = EngineAgent {
            id: row.get("id"),
            name: row.get("name"),
            role: row.get("role"),
            department: row.get("department"),
            description: row.get("description"),
            model_id: row.get("model_id"),
            tokens_used: row.get::<Option<i64>, _>("tokens_used").unwrap_or(0) as u32,
            status: row.get("status"),
            theme_color: row.get("theme_color"),
            budget_usd: row.get::<Option<f64>, _>("budget_usd").unwrap_or(0.0),
            cost_usd: row.get::<Option<f64>, _>("cost_usd").unwrap_or(0.0),
            voice_id: row.try_get("voice_id").ok(),
            metadata,
            skills: serde_json::from_str(&row.get::<String, _>("skills")).unwrap_or_default(),
            workflows: serde_json::from_str(&row.get::<String, _>("workflows")).unwrap_or_default(),
            mcp_tools: serde_json::from_str(&row.get::<String, _>("mcp_tools")).unwrap_or_default(),
            connector_configs: row
                .try_get::<Option<String>, _>("connector_configs")
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            model_2: row.try_get("model_2").ok(),
            model_3: row.try_get("model_3").ok(),
            model_config2: row
                .get::<Option<String>, _>("model_config2")
                .and_then(|s| serde_json::from_str(&s).ok()),
            model_config3: row
                .get::<Option<String>, _>("model_config3")
                .and_then(|s| serde_json::from_str(&s).ok()),
            active_model_slot: row.get::<Option<i32>, _>("active_model_slot"),
            voice_engine: row.try_get("voice_engine").ok(),
            category: row
                .try_get("category")
                .unwrap_or_else(|_| "user".to_string()),
            token_usage: TokenUsage {
                input_tokens,
                output_tokens,
                total_tokens: input_tokens + output_tokens,
            },
            model: crate::agent::types::ModelConfig {
                provider,
                model_id: row
                    .get::<Option<String>, _>("model_id")
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "gemini-1.5-pro".to_string()),
                api_key: row.try_get("api_key").ok(),
                base_url: row.try_get("base_url").ok(),
                system_prompt: row.try_get("system_prompt").ok(),
                temperature: row.get::<Option<f64>, _>("temperature").map(|f| f as f32),
                max_tokens: None,
                external_id: None,
                rpm: None,
                rpd: None,
                tpm: None,
                tpd: None,
                skills: None,
                workflows: None,
                mcp_tools: None,
                connector_configs: None,
                extra_parameters: None,
            },
            skill_manifest: None,
            active_mission: row
                .get::<Option<String>, _>("active_mission")
                .and_then(|s| serde_json::from_str(&s).ok()),
            current_task: row.get::<Option<String>, _>("current_task"),
            failure_count: row.get::<Option<i64>, _>("failure_count").unwrap_or(0) as u32,
            last_failure_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_failure_at"),
            created_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at"),
            heartbeat_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("heartbeat_at"),
            requires_oversight: row
                .get::<Option<bool>, _>("requires_oversight")
                .unwrap_or(false),
            working_memory: row
                .get::<Option<String>, _>("working_memory")
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| serde_json::json!({})),
        };
        agents.push(agent);
    }
    Ok(agents)
}

pub async fn save_agent_db(pool: &SqlitePool, agent: &EngineAgent) -> Result<()> {
    let metadata_json = serde_json::to_string(&agent.metadata)?;
    let active_mission_json = agent
        .active_mission
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok());
    let primary_model_id = if !agent.model.model_id.trim().is_empty() {
        Some(agent.model.model_id.clone())
    } else {
        agent
            .model_id
            .as_ref()
            .filter(|id| !id.trim().is_empty())
            .cloned()
    };

    sqlx::query("INSERT INTO agents (id, name, role, department, description, model_id, tokens_used, status, current_task, input_tokens, output_tokens, theme_color, budget_usd, cost_usd, metadata, skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2, model_config3, active_model_slot, voice_id, voice_engine, failure_count, last_failure_at, created_at, heartbeat_at, active_mission, provider, api_key, base_url, system_prompt, temperature, category, requires_oversight, working_memory)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            working_memory = excluded.working_memory")
    .bind(&agent.id)
    .bind(&agent.name)
    .bind(&agent.role)
    .bind(&agent.department)
    .bind(&agent.description)
    .bind(&primary_model_id)
    .bind(agent.tokens_used as i64)
    .bind(&agent.status)
    .bind(&agent.current_task)
    .bind(agent.token_usage.input_tokens as i64)
    .bind(agent.token_usage.output_tokens as i64)
    .bind(&agent.theme_color)
    .bind(agent.budget_usd)
    .bind(agent.cost_usd)
    .bind(metadata_json)
    .bind(serde_json::to_string(&agent.skills).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&agent.workflows).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&agent.mcp_tools).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&agent.connector_configs).unwrap_or_else(|_| "[]".to_string()))
    .bind(&agent.model_2)
    .bind(&agent.model_3)
    .bind(agent.model_config2.as_ref().and_then(|c| serde_json::to_string(c).ok()))
    .bind(agent.model_config3.as_ref().and_then(|c| serde_json::to_string(c).ok()))
    .bind(agent.active_model_slot)
    .bind(&agent.voice_id)
    .bind(&agent.voice_engine)
    .bind(agent.failure_count as i64)
    .bind(agent.last_failure_at)
    .bind(agent.created_at)
    .bind(agent.heartbeat_at)
    .bind(active_mission_json)
    .bind(agent.model.provider.to_string())
    .bind(&agent.model.api_key)
    .bind(&agent.model.base_url)
    .bind(&agent.model.system_prompt)
    .bind(agent.model.temperature.map(|f| f as f64))
    .bind(&agent.category)
    .bind(agent.requires_oversight)
    .bind(serde_json::to_string(&agent.working_memory).unwrap_or_else(|_| "{}".to_string()))
    .execute(pool)
    .await?;

    // Phase 2: Synchronize SyncManifests for this agent
    sync_manifests_for_agent(pool, agent).await?;

    Ok(())
}

/// Transaction-compatible variant of `save_agent_db`.
/// Accepts a mutable connection reference for batched saves within a single transaction.
/// Used by `AppState::save_agents()` to persist all agents in 1 commit instead of N.
pub async fn save_agent_db_in_tx(
    conn: &mut sqlx::SqliteConnection,
    agent: &EngineAgent,
) -> Result<()> {
    let metadata_json = serde_json::to_string(&agent.metadata)?;
    let active_mission_json = agent
        .active_mission
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok());
    let primary_model_id = if !agent.model.model_id.trim().is_empty() {
        Some(agent.model.model_id.clone())
    } else {
        agent
            .model_id
            .as_ref()
            .filter(|id| !id.trim().is_empty())
            .cloned()
    };

    sqlx::query("INSERT INTO agents (id, name, role, department, description, model_id, tokens_used, status, current_task, input_tokens, output_tokens, theme_color, budget_usd, cost_usd, metadata, skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2, model_config3, active_model_slot, voice_id, voice_engine, failure_count, last_failure_at, created_at, heartbeat_at, active_mission, provider, api_key, base_url, system_prompt, temperature, category, requires_oversight, working_memory)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            working_memory = excluded.working_memory")
    .bind(&agent.id)
    .bind(&agent.name)
    .bind(&agent.role)
    .bind(&agent.department)
    .bind(&agent.description)
    .bind(&primary_model_id)
    .bind(agent.tokens_used as i64)
    .bind(&agent.status)
    .bind(&agent.current_task)
    .bind(agent.token_usage.input_tokens as i64)
    .bind(agent.token_usage.output_tokens as i64)
    .bind(&agent.theme_color)
    .bind(agent.budget_usd)
    .bind(agent.cost_usd)
    .bind(&metadata_json)
    .bind(serde_json::to_string(&agent.skills).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&agent.workflows).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&agent.mcp_tools).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&agent.connector_configs).unwrap_or_else(|_| "[]".to_string()))
    .bind(&agent.model_2)
    .bind(&agent.model_3)
    .bind(agent.model_config2.as_ref().and_then(|c| serde_json::to_string(c).ok()))
    .bind(agent.model_config3.as_ref().and_then(|c| serde_json::to_string(c).ok()))
    .bind(agent.active_model_slot)
    .bind(&agent.voice_id)
    .bind(&agent.voice_engine)
    .bind(agent.failure_count as i64)
    .bind(agent.last_failure_at)
    .bind(agent.created_at)
    .bind(agent.heartbeat_at)
    .bind(&active_mission_json)
    .bind(agent.model.provider.to_string())
    .bind(&agent.model.api_key)
    .bind(&agent.model.base_url)
    .bind(&agent.model.system_prompt)
    .bind(agent.model.temperature.map(|f| f as f64))
    .bind(&agent.category)
    .bind(agent.requires_oversight)
    .bind(serde_json::to_string(&agent.working_memory).unwrap_or_else(|_| "{}".to_string()))
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn sync_manifests_for_agent(pool: &SqlitePool, agent: &EngineAgent) -> Result<()> {
    // 1. Delete manifests that are no longer in the agent config
    let current_uris: Vec<String> = agent
        .connector_configs
        .iter()
        .map(|c| c.uri.clone())
        .collect();
    sqlx::query("DELETE FROM sync_manifest WHERE agent_id = ? AND source_uri NOT IN (SELECT value FROM json_each(?))")
        .bind(&agent.id)
        .bind(serde_json::to_string(&current_uris)?)
        .execute(pool)
        .await?;

    // 2. Add new manifests
    for config in &agent.connector_configs {
        sqlx::query("INSERT OR IGNORE INTO sync_manifest (id, agent_id, source_type, source_uri, status) VALUES (?, ?, ?, ?, 'idle')")
            .bind(format!("{}-{}", agent.id, config.uri))
            .bind(&agent.id)
            .bind(&config.r#type)
            .bind(&config.uri)
            .execute(pool)
            .await?;
    }

    Ok(())
}

/// Loads provider configurations from disk and overlays security context.
pub async fn load_providers(base_dir: &std::path::Path) -> Vec<ProviderConfig> {
    let providers_file = crate::utils::security::validate_path(base_dir, PROVIDERS_FILE)
        .unwrap_or_else(|_| base_dir.join(PROVIDERS_FILE));
    let mut providers = if providers_file.exists() {
        match tokio::fs::read_to_string(&providers_file).await {
            Ok(content) => match serde_json::from_str::<Vec<ProviderConfig>>(&content) {
                Ok(providers) => providers,
                Err(e) => {
                    tracing::error!(
                        file = ?providers_file,
                        error = %e,
                        "❌ [Persistence] Provider JSON parse failure — falling back to defaults"
                    );
                    crate::agent::registry::get_default_providers()
                }
            },
            Err(e) => {
                tracing::error!(
                    file = ?providers_file,
                    error = %e,
                    "❌ [Persistence] Provider file read failure — falling back to defaults"
                );
                crate::agent::registry::get_default_providers()
            }
        }
    } else {
        // Fallback: Check RESOURCE_ROOT (e.g. bundled data)
        let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
        let bundled_file = std::path::Path::new(&resource_root).join(PROVIDERS_FILE);
        
        if bundled_file.exists() {
            match tokio::fs::read_to_string(&bundled_file).await {
                Ok(content) => match serde_json::from_str::<Vec<ProviderConfig>>(&content) {
                    Ok(p) => p,
                    Err(_) => crate::agent::registry::get_default_providers()
                },
                Err(_) => crate::agent::registry::get_default_providers()
            }
        } else {
            crate::agent::registry::get_default_providers()
        }
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

/// Persists provider configurations to disk after sanitizing credentials.
pub async fn save_providers(
    base_dir: &std::path::Path,
    providers: Vec<ProviderConfig>,
) -> Result<()> {
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
        .context("Failed to save providers")?;
    Ok(())
}

/// Loads the model registry from disk.
pub async fn load_models(base_dir: &std::path::Path) -> Vec<ModelEntry> {
    let models_file = crate::utils::security::validate_path(base_dir, MODELS_FILE)
        .unwrap_or_else(|_| base_dir.join(MODELS_FILE));
    if models_file.exists() {
        match tokio::fs::read_to_string(&models_file).await {
            Ok(content) => match serde_json::from_str::<Vec<ModelEntry>>(&content) {
                Ok(models) => return models,
                Err(e) => tracing::error!(
                    file = ?models_file,
                    error = %e,
                    "❌ [Persistence] Model JSON parse failure — falling back to defaults"
                ),
            },
            Err(e) => tracing::error!(
                file = ?models_file,
                error = %e,
                "❌ [Persistence] Model file read failure — falling back to defaults"
            ),
        }
    } else {
        // Fallback: Check RESOURCE_ROOT
        let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
        let bundled_file = std::path::Path::new(&resource_root).join(MODELS_FILE);
        
        if bundled_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&bundled_file) {
                if let Ok(models) = serde_json::from_str::<Vec<ModelEntry>>(&content) {
                    return models;
                }
            }
        }
    }
    crate::agent::registry::get_default_models()
}

/// Persists all model entries to disk.
pub async fn save_models(base_dir: &std::path::Path, models: Vec<ModelEntry>) -> Result<()> {
    let models_file = crate::utils::security::validate_path(base_dir, MODELS_FILE)?;
    let content = serde_json::to_string_pretty(&models)?;
    tokio::fs::write(models_file, content)
        .await
        .context("Failed to save models")?;
    Ok(())
}

/// Atomically claims an agent for a mission by setting its status to 'busy'.
/// Returns Ok(true) if the claim was successful, Ok(false) if the agent was already busy.
pub async fn claim_agent(pool: &SqlitePool, agent_id: &str) -> Result<bool> {
    let res = sqlx::query("UPDATE agents SET status = 'busy' WHERE id = ? AND status = 'idle'")
        .bind(agent_id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() > 0)
}

/// Identifies agents marked as 'busy' that haven't updated their heartbeat
/// within the specified threshold (in seconds) and resets them to 'idle'.
pub async fn reap_stale_agents(pool: &SqlitePool, threshold_secs: i64) -> Result<u64> {
    let now = chrono::Utc::now();
    // We use a safe subtraction since durations can be large
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

pub async fn update_agent_heartbeat(pool: &SqlitePool, agent_id: &str) -> Result<()> {
    sqlx::query("UPDATE agents SET heartbeat_at = ? WHERE id = ?")
        .bind(chrono::Utc::now())
        .bind(agent_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Loads all sync manifests from the database.
pub async fn load_sync_manifests(pool: &SqlitePool) -> Result<Vec<crate::agent::SyncManifest>> {
    let rows = sqlx::query_as::<_, crate::agent::SyncManifest>("SELECT * FROM sync_manifest")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// Updates the status of a sync manifest.
pub async fn update_sync_status(pool: &SqlitePool, id: &str, status: &str) -> Result<()> {
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
) -> Result<()> {
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
    async fn test_agent_persistence_round_trip() -> Result<()> {
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
            id: "test-agent-1".to_string(),
            name: "Test Agent".to_string(),
            role: "Tester".to_string(),
            department: "QA".to_string(),
            description: "A test agent".to_string(),
            status: "idle".to_string(),
            theme_color: Some("#ff0000".to_string()),
            skills: vec!["testing".to_string()],
            model_id: Some("gemini-1.5-pro".to_string()),
            ..Default::default()
        };

        agent.model.provider = crate::agent::types::ModelProvider::Anthropic;
        agent.model.model_id = "claude-3-5-sonnet".to_string();
        agent.model.api_key = Some("sk-test-123".to_string());
        agent.model.base_url = Some("https://api.anthropic.com".to_string());
        agent.model.system_prompt = Some("You are a tester.".to_string());
        agent.model.temperature = Some(0.7);

        agent.current_task = Some("Validating persistence parity".to_string());
        agent.token_usage = TokenUsage {
            input_tokens: 321,
            output_tokens: 123,
            total_tokens: 444,
        };
        agent.tokens_used = 2222;
        agent.working_memory = serde_json::json!({"milestone": "test-passed"});

        // 3. Save
        save_agent_db(&pool, &agent).await?;

        // 4. Load
        let agents = load_agents_db(&pool).await?;
        let loaded = agents
            .iter()
            .find(|a| a.id == "test-agent-1")
            .expect("Agent not found");

        // 5. Assert Parity
        assert_eq!(loaded.name, agent.name);
        assert_eq!(loaded.model.provider, agent.model.provider);
        assert_eq!(loaded.model.model_id, agent.model.model_id);
        assert_eq!(loaded.model.api_key, agent.model.api_key);
        assert_eq!(loaded.model.base_url, agent.model.base_url);
        assert_eq!(loaded.model.system_prompt, agent.model.system_prompt);
        assert_eq!(loaded.model.temperature, agent.model.temperature);
        assert_eq!(loaded.theme_color, agent.theme_color);
        assert_eq!(loaded.skills, agent.skills);
        assert_eq!(loaded.requires_oversight, agent.requires_oversight);
        assert_eq!(loaded.current_task, agent.current_task);
        assert_eq!(
            loaded.token_usage.input_tokens,
            agent.token_usage.input_tokens
        );
        assert_eq!(
            loaded.token_usage.output_tokens,
            agent.token_usage.output_tokens
        );
        assert_eq!(
            loaded.token_usage.total_tokens,
            agent.token_usage.total_tokens
        );
        assert_eq!(loaded.tokens_used, agent.tokens_used);
        assert_eq!(loaded.working_memory["milestone"], "test-passed");

        Ok(())
    }

    #[tokio::test]
    async fn test_atomic_claiming_and_reaping() -> Result<()> {
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
