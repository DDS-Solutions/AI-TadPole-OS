//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / wire
//! - **Primary Entrypoints**: `EngineAgentWire`, `AgentResponse`
//!
//! ### AI Assist Note
//! - Preserve legacy economics spellings; reject disagreement between aliases.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::entity::{
    AgentCapabilities, AgentEconomics, AgentHealth, AgentIdentity, AgentModels, AgentState,
    EngineAgent, TokenUsage,
};
use crate::agent::types::model::{ConnectorConfig, ModelConfig, ModelProvider};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentResponse<'a> {
    #[serde(flatten)]
    identity: &'a AgentIdentity,
    #[serde(flatten)]
    models_raw: AgentModelsResponse<'a>,
    #[serde(flatten)]
    economics: &'a AgentEconomics,
    #[serde(flatten)]
    health: AgentHealthResponse<'a>,
    #[serde(flatten)]
    capabilities: &'a AgentCapabilities,
    #[serde(flatten)]
    state: &'a AgentState,
    metadata: &'a HashMap<String, serde_json::Value>,
    created_at: &'a Option<DateTime<Utc>>,
    requires_oversight: bool,
    shadows_human_id: &'a Option<String>,
    voice_id: &'a Option<String>,
    voice_engine: &'a Option<String>,
    connector_configs: &'a [ConnectorConfig],
    version: u32,
    #[serde(rename = "tokens_used")]
    tokens_used_snake: u64,
    #[serde(rename = "cost_usd")]
    cost_usd_snake: f64,
    #[serde(rename = "budget_usd")]
    budget_usd_snake: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentModelsResponse<'a> {
    model_id: &'a Option<String>,
    model: &'a str,
    model_config: &'a ModelConfig,
    planning_slot: &'a Option<ModelConfig>,
    execution_slot: &'a Option<ModelConfig>,
    active_model_slot: &'a Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentHealthResponse<'a> {
    status: &'a str,
    failure_count: u32,
    last_failure_at: &'a Option<DateTime<Utc>>,
    #[serde(rename = "lastPulse")]
    last_pulse: &'a Option<DateTime<Utc>>,
}

impl Serialize for EngineAgent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let model_name = if self.models.model.model_id.trim().is_empty() {
            self.models.model_id.as_deref().unwrap_or_default()
        } else {
            self.models.model.model_id.as_str()
        };

        let response = AgentResponse {
            identity: &self.identity,
            models_raw: AgentModelsResponse {
                model_id: &self.models.model_id,
                model: model_name,
                model_config: &self.models.model,
                planning_slot: &self.models.planning_slot,
                execution_slot: &self.models.execution_slot,
                active_model_slot: &self.models.active_model_slot,
            },
            economics: &self.economics,
            health: AgentHealthResponse {
                status: &self.health.status,
                failure_count: self.health.failure_count,
                last_failure_at: &self.health.last_failure_at,
                last_pulse: &self.health.heartbeat_at,
            },
            capabilities: &self.capabilities,
            state: &self.state,
            metadata: &self.metadata,
            created_at: &self.created_at,
            requires_oversight: self.requires_oversight,
            shadows_human_id: &self.shadows_human_id,
            voice_id: &self.voice_id,
            voice_engine: &self.voice_engine,
            connector_configs: &self.connector_configs,
            version: self.version,
            tokens_used_snake: self.economics.tokens_used,
            cost_usd_snake: self.economics.cost_usd,
            budget_usd_snake: self.economics.budget_usd,
        };
        response.serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EngineAgentModelInput {
    ModelId(String),
    Config(Box<ModelConfig>),
}

/// `activeModelSlot` arrives either as a legacy integer index or as the slot name
/// that `Serialize for EngineAgent` emits. Accepting both keeps the JSON round-trip lossless.
#[derive(Deserialize)]
#[serde(untagged)]
enum ActiveModelSlotInput {
    Index(i64),
    Name(String),
}

impl ActiveModelSlotInput {
    /// Legacy wire indices: `1` = planning, `2` = execution, anything else = default.
    /// This matches `AgentConfigUpdate` (`agent/merge.rs`). Note that the SQLite
    /// persistence layer uses its own, different integer encoding.
    fn into_slot_name(self) -> String {
        let name = match self {
            Self::Index(1) => "planning",
            Self::Index(2) => "execution",
            Self::Index(_) => "default",
            Self::Name(name) => match name.trim().to_ascii_lowercase().as_str() {
                "planning" => "planning",
                "execution" => "execution",
                _ => "default",
            },
        };
        name.to_string()
    }
}

#[derive(Deserialize, Default)]
struct EngineAgentWire {
    id: String,
    name: String,
    role: String,
    department: String,
    description: String,
    #[serde(default, alias = "modelId", alias = "primary_model")]
    model_id: Option<String>,
    #[serde(default)]
    model: Option<EngineAgentModelInput>,
    #[serde(default, alias = "model_config", alias = "modelConfig")]
    model_config: Option<ModelConfig>,
    #[serde(default, alias = "model2")]
    model_2: Option<String>,
    #[serde(default, alias = "model3")]
    model_3: Option<String>,
    #[serde(
        default,
        alias = "model_config2",
        alias = "modelConfig2",
        alias = "planningSlot",
        alias = "planning_slot"
    )]
    model_config2: Option<ModelConfig>,
    #[serde(
        default,
        alias = "model_config3",
        alias = "modelConfig3",
        alias = "executionSlot",
        alias = "execution_slot"
    )]
    model_config3: Option<ModelConfig>,
    #[serde(default, alias = "activeModelSlot")]
    active_model_slot: Option<ActiveModelSlotInput>,
    #[serde(default, alias = "system_prompt", alias = "systemPrompt")]
    system_prompt: Option<String>,
    #[serde(default, alias = "activeMission")]
    active_mission: Option<serde_json::Value>,
    status: String,
    #[serde(default, alias = "currentTask")]
    current_task: Option<String>,
    #[serde(default)]
    tokens_used: Option<u64>,
    #[serde(default, rename = "tokensUsed")]
    tokens_used_camel: Option<u64>,
    #[serde(default, alias = "tokenUsage")]
    token_usage: TokenUsage,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    workflows: Vec<String>,
    #[serde(default, alias = "mcpTools")]
    mcp_tools: Vec<String>,
    #[serde(default, alias = "skillManifest")]
    skill_manifest: Option<crate::agent::skill_manifest::SkillManifest>,
    #[serde(default)]
    metadata: HashMap<String, serde_json::Value>,
    #[serde(default, alias = "themeColor")]
    theme_color: Option<String>,
    #[serde(default)]
    budget_usd: Option<f64>,
    #[serde(default, rename = "budgetUsd")]
    budget_usd_camel: Option<f64>,
    #[serde(default)]
    cost_usd: Option<f64>,
    #[serde(default, rename = "costUsd")]
    cost_usd_camel: Option<f64>,
    #[serde(default, alias = "voiceId")]
    voice_id: Option<String>,
    #[serde(default, alias = "voiceEngine")]
    voice_engine: Option<String>,
    #[serde(default = "default_category")]
    category: String,
    #[serde(default, alias = "failureCount")]
    failure_count: u32,
    #[serde(default, alias = "lastFailureAt")]
    last_failure_at: Option<DateTime<Utc>>,
    #[serde(default, alias = "createdAt")]
    created_at: Option<DateTime<Utc>>,
    #[serde(default, alias = "heartbeatAt")]
    heartbeat_at: Option<DateTime<Utc>>,
    #[serde(default, alias = "lastPulse", alias = "last_pulse")]
    last_pulse: Option<DateTime<Utc>>,
    #[serde(default, alias = "requiresOversight")]
    requires_oversight: bool,
    #[serde(default, alias = "shadowsHumanId", alias = "shadows_human_id")]
    shadows_human_id: Option<String>,
    #[serde(default, alias = "workingMemory")]
    working_memory: serde_json::Value,
    #[serde(default, alias = "connectorConfigs")]
    connector_configs: Vec<ConnectorConfig>,
    #[serde(default, alias = "currentReasoningTurn")]
    current_reasoning_turn: u32,
    #[serde(default = "default_version")]
    version: u32,
}

impl<'de> Deserialize<'de> for EngineAgent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = EngineAgentWire::deserialize(deserializer)?;

        let mut model = match (wire.model_config, wire.model) {
            (Some(config), _) => config,
            (None, Some(EngineAgentModelInput::Config(config))) => *config,
            (None, Some(EngineAgentModelInput::ModelId(model_id))) => ModelConfig {
                provider: ModelProvider::from_str(&model_id).unwrap_or(ModelProvider::Openai),
                model_id,
                ..ModelConfig::default()
            },
            (None, None) => ModelConfig::default(),
        };

        if model.system_prompt.is_none() {
            model.system_prompt = wire.system_prompt;
        }

        let model_id = match wire.model_id {
            Some(model_id) => {
                if model.model_id.is_empty() {
                    model.model_id = model_id.clone();
                }
                Some(model_id)
            }
            None if !model.model_id.is_empty() => Some(model.model_id.clone()),
            None => None,
        };

        Ok(Self {
            identity: AgentIdentity {
                id: wire.id,
                name: wire.name,
                role: wire.role,
                department: wire.department,
                description: wire.description,
                category: wire.category,
                theme_color: wire.theme_color,
            },
            models: AgentModels {
                model_id,
                model,
                planning_slot: wire.model_config2.map(|mut m| {
                    if let Some(ref m2) = wire.model_2 {
                        if !m2.trim().is_empty() {
                            m.model_id = m2.clone();
                        }
                    }
                    m
                }),
                execution_slot: wire.model_config3.map(|mut m| {
                    if let Some(ref m3) = wire.model_3 {
                        if !m3.trim().is_empty() {
                            m.model_id = m3.clone();
                        }
                    }
                    m
                }),
                active_model_slot: wire
                    .active_model_slot
                    .map(ActiveModelSlotInput::into_slot_name),
            },
            economics: AgentEconomics {
                budget_usd: merge_economics_alias::<_, D::Error>(
                    wire.budget_usd,
                    wire.budget_usd_camel,
                    "budget_usd",
                )?,
                cost_usd: merge_economics_alias::<_, D::Error>(
                    wire.cost_usd,
                    wire.cost_usd_camel,
                    "cost_usd",
                )?,
                tokens_used: merge_economics_alias::<_, D::Error>(
                    wire.tokens_used,
                    wire.tokens_used_camel,
                    "tokens_used",
                )?,
                token_usage: wire.token_usage,
            },
            health: AgentHealth {
                status: wire.status,
                failure_count: wire.failure_count,
                last_failure_at: wire.last_failure_at,
                heartbeat_at: wire.last_pulse.or(wire.heartbeat_at),
            },
            capabilities: AgentCapabilities {
                skills: wire.skills,
                workflows: wire.workflows,
                mcp_tools: wire.mcp_tools,
                skill_manifest: wire.skill_manifest,
            },
            state: AgentState {
                active_mission: wire.active_mission,
                current_task: wire.current_task,
                working_memory: wire.working_memory,
                current_reasoning_turn: wire.current_reasoning_turn,
            },
            metadata: wire.metadata,
            created_at: wire.created_at,
            requires_oversight: wire.requires_oversight,
            shadows_human_id: wire.shadows_human_id,
            voice_id: wire.voice_id,
            voice_engine: wire.voice_engine,
            connector_configs: wire.connector_configs,
            version: wire.version,
        })
    }
}

fn default_version() -> u32 {
    1
}

fn merge_economics_alias<T: Default + PartialEq, E: serde::de::Error>(
    snake: Option<T>,
    camel: Option<T>,
    name: &str,
) -> Result<T, E> {
    match (snake, camel) {
        (Some(snake), Some(camel)) if snake != camel => Err(E::custom(format!(
            "Conflicting economics aliases for {name}"
        ))),
        (Some(value), _) | (_, Some(value)) => Ok(value),
        (None, None) => Ok(T::default()),
    }
}

fn default_category() -> String {
    "user".to_string()
}
