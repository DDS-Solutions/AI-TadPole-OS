//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! - Keep the SQLite row shape and tolerant legacy decoding aligned with the engine agent model.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Persistence / agent row mapping
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: malformed legacy JSON falls back to typed defaults
//! - **Telemetry Targets**: `[Persistence]`

use crate::agent::types::{
    AgentCapabilities, AgentEconomics, AgentHealth, AgentIdentity, AgentModels, AgentState,
    EngineAgent, TokenUsage,
};

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
            .trim_start_matches(['[', '"', ' '])
            .trim_end_matches([']', '\\', '"', ' ']);
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
    if !trimmed.is_empty() && trimmed != "[]" && trimmed != "{}" {
        tracing::error!(
            col_name = %col_name,
            agent_id = %agent_id,
            raw_len = trimmed.len(),
            "❌ [Persistence] Corrupt JSON detected for column '{}' in agent '{}'. Malformed payload could not be rescued; returning default fallback with active alert.",
            col_name,
            agent_id
        );
    }
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

        let input_tokens = self
            .input_tokens
            .map(|t| u64::try_from(t).unwrap_or(0))
            .unwrap_or(0);
        let output_tokens = self
            .output_tokens
            .map(|t| u64::try_from(t).unwrap_or(0))
            .unwrap_or(0);
        let tokens_used = self
            .tokens_used
            .map(|t| u64::try_from(t).unwrap_or(0))
            .unwrap_or(0);
        let failure_count = self
            .failure_count
            .map(|f| u32::try_from(f).unwrap_or(0))
            .unwrap_or(0);

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
                    2 => "planning".to_string(),
                    3 => "execution".to_string(),
                    _ => "primary".to_string(),
                }),
            },
            economics: AgentEconomics {
                budget_usd: self.budget_usd.unwrap_or(0.0),
                cost_usd: self.cost_usd.unwrap_or(0.0),
                tokens_used,
                token_usage: TokenUsage {
                    input_tokens,
                    output_tokens,
                    total_tokens: input_tokens + output_tokens,
                },
            },
            health: AgentHealth {
                status: self.status.clone(),
                failure_count,
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
            version: u32::try_from(self.version).unwrap_or(0),
        }
    }
}

pub(super) const SELECT_AGENTS_QUERY: &str = "SELECT id, name, role, department, description, model_id, tokens_used, status, current_task,
            input_tokens, output_tokens, theme_color, budget_usd, cost_usd, voice_id, metadata,
            skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2,
            model_config3, active_model_slot, voice_engine, category, provider, api_key, base_url,
            system_prompt, temperature, active_mission, failure_count, last_failure_at, created_at,
            heartbeat_at, requires_oversight, shadows_human_id, working_memory, version FROM agents ORDER BY id ASC";

pub(super) const SELECT_AGENT_BY_ID_QUERY: &str = "SELECT id, name, role, department, description, model_id, tokens_used, status, current_task,
            input_tokens, output_tokens, theme_color, budget_usd, cost_usd, voice_id, metadata,
            skills, workflows, mcp_tools, connector_configs, model_2, model_3, model_config2,
            model_config3, active_model_slot, voice_engine, category, provider, api_key, base_url,
            system_prompt, temperature, active_mission, failure_count, last_failure_at, created_at,
            heartbeat_at, requires_oversight, shadows_human_id, working_memory, version FROM agents WHERE id = ?";
