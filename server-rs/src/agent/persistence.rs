//! High-level persistence logic for agent and infrastructure records.
//!
//! Handles synchronization between the in-memory `AppState` and the persistent
//! SQLite database or JSON configuration files.

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
    let rows = sqlx::query("SELECT * FROM agents").fetch_all(pool).await?;
    let mut agents = Vec::new();

    for row in rows {
        use sqlx::Row;
        let metadata_str: String = row.get("metadata");
        let metadata: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_str(&metadata_str).unwrap_or_default();

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
            voice_id: Some(
                row.try_get("voice_id")
                    .unwrap_or_else(|_| "alloy".to_string()),
            ),
            metadata,
            skills: serde_json::from_str(&row.get::<String, _>("skills")).unwrap_or_default(),
            workflows: serde_json::from_str(&row.get::<String, _>("workflows")).unwrap_or_default(),
            mcp_tools: serde_json::from_str(&row.get::<String, _>("mcp_tools")).unwrap_or_default(),
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
            token_usage: TokenUsage::default(),
            model: crate::agent::types::ModelConfig {
                provider: row
                    .try_get("provider")
                    .unwrap_or_else(|_| "gemini".to_string()),
                model_id: row
                    .get::<Option<String>, _>("model_id")
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
            },
            skill_manifest: None,
            active_mission: row
                .get::<Option<String>, _>("active_mission")
                .and_then(|s| serde_json::from_str(&s).ok()),
            current_task: None,
            failure_count: row.get::<Option<i64>, _>("failure_count").unwrap_or(0) as u32,
            last_failure_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_failure_at"),
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

    sqlx::query("INSERT INTO agents (id, name, role, department, description, model_id, tokens_used, status, theme_color, budget_usd, cost_usd, metadata, skills, workflows, mcp_tools, model_2, model_3, model_config2, model_config3, active_model_slot, voice_id, voice_engine, failure_count, last_failure_at, active_mission, provider, api_key, base_url, system_prompt, temperature, category, requires_oversight, working_memory)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            role = excluded.role,
            department = excluded.department,
            description = excluded.description,
            model_id = excluded.model_id,
            tokens_used = excluded.tokens_used,
            status = excluded.status,
            theme_color = excluded.theme_color,
            budget_usd = excluded.budget_usd,
            cost_usd = excluded.cost_usd,
            metadata = excluded.metadata,
            skills = excluded.skills,
            workflows = excluded.workflows,
            mcp_tools = excluded.mcp_tools,
            model_2 = excluded.model_2,
            model_3 = excluded.model_3,
            model_config2 = excluded.model_config2,
            model_config3 = excluded.model_config3,
            active_model_slot = excluded.active_model_slot,
            voice_id = excluded.voice_id,
            voice_engine = excluded.voice_engine,
            failure_count = excluded.failure_count,
            last_failure_at = excluded.last_failure_at,
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
    .bind(&agent.model_id)
    .bind(agent.tokens_used as i64)
    .bind(&agent.status)
    .bind(&agent.theme_color)
    .bind(agent.budget_usd)
    .bind(agent.cost_usd)
    .bind(metadata_json)
    .bind(serde_json::to_string(&agent.skills).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&agent.workflows).unwrap_or_else(|_| "[]".to_string()))
    .bind(serde_json::to_string(&agent.mcp_tools).unwrap_or_else(|_| "[]".to_string()))
    .bind(&agent.model_2)
    .bind(&agent.model_3)
    .bind(agent.model_config2.as_ref().and_then(|c| serde_json::to_string(c).ok()))
    .bind(agent.model_config3.as_ref().and_then(|c| serde_json::to_string(c).ok()))
    .bind(agent.active_model_slot)
    .bind(&agent.voice_id)
    .bind(&agent.voice_engine)
    .bind(agent.failure_count as i64)
    .bind(agent.last_failure_at)
    .bind(active_mission_json)
    .bind(&agent.model.provider)
    .bind(&agent.model.api_key)
    .bind(&agent.model.base_url)
    .bind(&agent.model.system_prompt)
    .bind(agent.model.temperature.map(|f| f as f64))
    .bind(&agent.category)
    .bind(agent.requires_oversight)
    .bind(serde_json::to_string(&agent.working_memory).unwrap_or_else(|_| "{}".to_string()))
    .execute(pool)
    .await?;

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
        crate::agent::registry::get_default_providers()
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
                obj.insert("apiKey".to_string(), serde_json::Value::Null);
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
            theme_color TEXT,
            budget_usd REAL DEFAULT 0.0,
            cost_usd REAL DEFAULT 0.0,
            metadata TEXT NOT NULL,
            skills TEXT,
            workflows TEXT,
            mcp_tools TEXT,
            model_2 TEXT,
            model_3 TEXT,
            model_config2 TEXT,
            model_config3 TEXT,
            active_model_slot INTEGER DEFAULT 1,
            failure_count INTEGER DEFAULT 0,
            last_failure_at DATETIME,
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
            working_memory TEXT DEFAULT '{}'
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
            requires_oversight: true,
            ..Default::default()
        };

        agent.model.provider = "anthropic".to_string();
        agent.model.model_id = "claude-3-5-sonnet".to_string();
        agent.model_id = Some("claude-3-5-sonnet".to_string());
        agent.model.api_key = Some("sk-test-123".to_string());
        agent.model.base_url = Some("https://api.anthropic.com".to_string());
        agent.model.system_prompt = Some("You are a tester.".to_string());
        agent.model.temperature = Some(0.7);

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
        assert_eq!(loaded.working_memory["milestone"], "test-passed");

        Ok(())
    }
}
