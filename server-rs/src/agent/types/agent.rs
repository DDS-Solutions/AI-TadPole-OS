//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[agent]` in tracing logs.

use super::model::{ConnectorConfig, ModelConfig, ModelProvider};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// ### 📡 Protocol: RoleAuthorityLevel
/// Defines the authority level of an agent in the swarm.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum RoleAuthorityLevel {
    /// Executive level (CEO, COO) - Strategic oversight and delegation.
    Executive,
    /// Management level (Alpha Node) - Tactical coordination.
    Management,
    /// Specialist level - Task execution.
    #[default]
    Specialist,
    /// Observer level - Read-only oversight.
    Observer,
}

impl RoleAuthorityLevel {
    pub fn from_role(role: &str) -> Self {
        let r = role.to_lowercase();
        if r.contains("ceo") || r.contains("overlord") || r.contains("executive") {
            Self::Executive
        } else if r.contains("coo")
            || r.contains("cto")
            || r.contains("orchestrator")
            || r.contains("commander")
            || r.contains("alpha")
            || r.contains("lead")
            || r.contains("director")
            || r.contains("manager")
            || r.contains("pm")
        {
            Self::Management
        } else if r.contains("observer") || r.contains("auditor") {
            Self::Observer
        } else {
            Self::Specialist
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    #[serde(default, alias = "input_tokens")]
    #[specta(type = f64)]
    pub input_tokens: u64,
    #[serde(default, alias = "output_tokens")]
    #[specta(type = f64)]
    pub output_tokens: u64,
    #[serde(default, alias = "total_tokens")]
    #[specta(type = f64)]
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Default, specta::Type)]
pub struct SyncManifest {
    pub id: String,
    pub agent_id: String,
    pub source_type: String,
    pub source_uri: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub checksum: Option<String>,
    pub status: String,
    pub metadata: Option<String>,
    #[serde(default)]
    pub file_count: i32,
    #[serde(default)]
    pub total_bytes: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentIdentity {
    pub id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub description: String,
    pub category: String,
    pub theme_color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentEconomics {
    pub budget_usd: f64,
    pub cost_usd: f64,
    #[specta(type = f64)]
    pub tokens_used: u64,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentHealth {
    pub status: String,
    pub failure_count: u32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub heartbeat_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentModels {
    #[serde(default, alias = "modelId")]
    pub model_id: Option<String>,
    pub model: ModelConfig,

    #[serde(default, alias = "planningSlot")]
    pub planning_slot: Option<ModelConfig>,

    #[serde(default, alias = "executionSlot")]
    pub execution_slot: Option<ModelConfig>,

    #[serde(default, alias = "activeModelSlot")]
    pub active_model_slot: Option<String>, // "planning" | "execution" | "default"
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    pub skills: Vec<String>,
    pub workflows: Vec<String>,
    pub mcp_tools: Vec<String>,
    pub skill_manifest: Option<crate::agent::skill_manifest::SkillManifest>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentState {
    pub active_mission: Option<serde_json::Value>,
    pub current_task: Option<String>,
    pub working_memory: serde_json::Value,
    pub current_reasoning_turn: u32,
}

#[derive(Debug, Clone, Default, specta::Type)]
pub struct EngineAgent {
    pub identity: AgentIdentity,
    pub models: AgentModels,
    pub economics: AgentEconomics,
    pub health: AgentHealth,
    pub capabilities: AgentCapabilities,
    pub state: AgentState,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub requires_oversight: bool,
    pub voice_id: Option<String>,
    pub voice_engine: Option<String>,
    pub connector_configs: Vec<ConnectorConfig>,
    pub version: u32,
}

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
    voice_id: &'a Option<String>,
    voice_engine: &'a Option<String>,
    connector_configs: &'a [ConnectorConfig],
    version: u32,
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
            voice_id: &self.voice_id,
            voice_engine: &self.voice_engine,
            connector_configs: &self.connector_configs,
            version: self.version,
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
    #[serde(default, alias = "model_config2", alias = "modelConfig2", alias = "planningSlot", alias = "planning_slot")]
    model_config2: Option<ModelConfig>,
    #[serde(default, alias = "model_config3", alias = "modelConfig3", alias = "executionSlot", alias = "execution_slot")]
    model_config3: Option<ModelConfig>,
    #[serde(default, alias = "activeModelSlot")]
    active_model_slot: Option<i32>,
    #[serde(default, alias = "system_prompt", alias = "systemPrompt")]
    system_prompt: Option<String>,
    #[serde(default, alias = "activeMission")]
    active_mission: Option<serde_json::Value>,
    status: String,
    #[serde(default, alias = "currentTask")]
    current_task: Option<String>,
    #[serde(default, alias = "tokensUsed")]
    tokens_used: u64,
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
    #[serde(default, alias = "budgetUsd")]
    budget_usd: f64,
    #[serde(default, alias = "costUsd")]
    cost_usd: f64,
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
                active_model_slot: wire.active_model_slot.map(|s| match s {
                    1 => "planning".to_string(),
                    2 => "execution".to_string(),
                    _ => "default".to_string(),
                }),
            },
            economics: AgentEconomics {
                budget_usd: wire.budget_usd,
                cost_usd: wire.cost_usd,
                tokens_used: wire.tokens_used,
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

fn default_category() -> String {
    "user".to_string()
}

impl EngineAgent {
    #[allow(dead_code)]
    pub fn is_suspended(&self) -> bool {
        self.health.status == "suspended"
    }

    #[allow(dead_code)]
    pub fn resolve_provider_context(
        &self,
        base_dir: std::path::PathBuf,
        cluster_id: Option<&str>,
    ) -> crate::agent::runner::RunContext {
        let workspace_id = cluster_id.unwrap_or("default");
        let workspace_root = base_dir.join(format!("data/workspaces/{}", workspace_id));
        let fs_adapter = crate::adapter::filesystem::FilesystemAdapter::new(workspace_root.clone());
        let authority_level = crate::agent::types::RoleAuthorityLevel::from_role(&self.identity.role);

        let identity = crate::agent::runner::AgentIdentity {
            agent_id: self.identity.id.clone(),
            name: self.identity.name.clone(),
            role: self.identity.role.clone(),
            department: self.identity.department.clone(),
            description: self.identity.description.clone(),
            authority_level,
        };

        let mission_state = crate::agent::runner::MissionState {
            mission_id: "system-internal".to_string(),
            cluster_id: cluster_id.map(|s| s.to_string()),
            user_id: None,
            depth: 0,
            lineage: vec![],
            primary_goal: None,
            budget_usd: self.economics.budget_usd,
            current_cost_usd: self.economics.cost_usd,
            sub_budget_usd: None,
        };

        let env = crate::agent::runner::Environment {
            workspace_root: workspace_root.clone(),
            fs_adapter: fs_adapter.clone(),
            base_dir: base_dir.clone(),
        };

        crate::agent::runner::RunContext {
            identity,
            mission_state,
            env,
            agent_id: self.identity.id.clone(),
            name: self.identity.name.clone(),
            role: self.identity.role.clone(),
            department: self.identity.department.clone(),
            description: self.identity.description.clone(),
            model_config: self.models.model.clone(),
            skills: self.capabilities.skills.clone(),
            workflows: self.capabilities.workflows.clone(),
            agent_models: self.models.clone(),
            mcp_tools: self.capabilities.mcp_tools.clone(),
            mission_id: "system-internal".to_string(),
            cluster_id: cluster_id.map(|s| s.to_string()),
            depth: 0,
            lineage: vec![],
            provider_name: self.models.model.provider.to_string(),
            workspace_root,
            fs_adapter,
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            modified_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            commands_run: std::sync::Arc::new(parking_lot::Mutex::new(
                std::collections::HashSet::new(),
            )),
            allowed_files: None,
            recent_findings: None,
            working_memory: self.state.working_memory.clone(),
            base_dir,
            summarized_history: None,
            structured_output: false,
            backlog: None,
            primary_goal: None,
            visible_transcript: None,
            conductor_plan: None,
            budget_usd: self.economics.budget_usd,
            current_cost_usd: self.economics.cost_usd,
            sub_budget_usd: None,
            reasoning_depth: self.models.model.reasoning_depth.unwrap_or(1),
            act_threshold: self.models.model.act_threshold.unwrap_or(0.95),
            max_turns: self.models.model.max_turns.unwrap_or(20),
            authority_level,
            resource_weights: std::collections::HashMap::new(),
            graph_context: None,
            verification_passed: false,
        }
    }
}

// Metadata: [agent]
