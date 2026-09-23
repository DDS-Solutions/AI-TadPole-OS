//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / Agent Models
//! - **Primary Entrypoints**: `AgentResponse`, `CreateAgentRequest`, `ChatCompletionRequest`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::{
    agent::types::{
        AgentCapabilities, AgentEconomics, AgentHealth, AgentIdentity, AgentModels, EngineAgent,
        ModelConfig, ModelProvider,
    },
    error::AppError,
};
use serde::Serialize;

pub const STATUS_IDLE: &str = "idle";
pub const STATUS_BUSY: &str = "busy";
pub const STATUS_SUSPENDED: &str = "suspended";
pub const MAX_FAILURE_COUNT: u32 = 5;
pub const DEDUP_CACHE_PRUNE_SECS: u64 = 30;
pub const DEDUP_WINDOW_SECS: u64 = 15;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub status: String,
    pub model: String,
    pub provider: String,
    pub model_config: crate::agent::types::ModelConfig,
    pub planning_slot: Option<crate::agent::types::ModelConfig>,
    pub execution_slot: Option<crate::agent::types::ModelConfig>,
    pub active_model_slot: Option<String>,
    pub budget_usd: f64,
    pub cost_usd: f64,
    pub is_healthy: bool,
    pub is_bankrupt: bool,
    pub failure_count: u32,
    pub last_failure_at: Option<chrono::DateTime<chrono::Utc>>,
    pub skills: Vec<String>,
    pub workflows: Vec<String>,
    pub mcp_tools: Vec<String>,
    pub requires_oversight: bool,
    pub shadows_human_id: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub version: u32,
    pub tokens_used: u64,
    pub token_usage: crate::agent::types::TokenUsage,
}

impl From<&EngineAgent> for AgentResponse {
    fn from(agent: &EngineAgent) -> Self {
        let model_name = if agent.models.model.model_id.trim().is_empty() {
            agent
                .models
                .model_id
                .as_deref()
                .unwrap_or_default()
                .to_string()
        } else {
            agent.models.model.model_id.clone()
        };

        if model_name.trim().is_empty() {
            tracing::warn!("⚠️ Agent {} has no configured model_id!", agent.identity.id);
        }

        Self {
            id: agent.identity.id.clone(),
            name: agent.identity.name.clone(),
            role: agent.identity.role.clone(),
            department: agent.identity.department.clone(),
            status: agent.health.status.clone(),
            model: model_name,
            provider: agent.models.model.provider.to_string(),
            // SEC: Redact api_key from REST responses to prevent credential leakage.
            // ModelConfig may contain per-agent API keys loaded from the DB.
            model_config: {
                let mut mc = agent.models.model.clone();
                mc.api_key = None;
                mc
            },
            planning_slot: agent.models.planning_slot.clone().map(|mut s| {
                s.api_key = None;
                s
            }),
            execution_slot: agent.models.execution_slot.clone().map(|mut s| {
                s.api_key = None;
                s
            }),
            active_model_slot: agent.models.active_model_slot.clone(),
            budget_usd: agent.economics.budget_usd,
            cost_usd: agent.economics.cost_usd,
            is_healthy: agent.health.failure_count < MAX_FAILURE_COUNT,
            is_bankrupt: agent.economics.cost_usd >= agent.economics.budget_usd
                && agent.economics.budget_usd > 0.0,
            failure_count: agent.health.failure_count,
            last_failure_at: agent.health.last_failure_at,
            skills: agent.capabilities.skills.clone(),
            workflows: agent.capabilities.workflows.clone(),
            mcp_tools: agent.capabilities.mcp_tools.clone(),
            requires_oversight: agent.requires_oversight,
            shadows_human_id: agent.shadows_human_id.clone(),
            created_at: agent.created_at,
            version: agent.version,
            tokens_used: agent.economics.tokens_used,
            token_usage: agent.economics.token_usage.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentRequest {
    pub id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub description: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, alias = "modelConfig")]
    pub model_config: Option<ModelConfig>,
    #[serde(default, alias = "planningSlot")]
    pub planning_slot: Option<ModelConfig>,
    #[serde(default, alias = "executionSlot")]
    pub execution_slot: Option<ModelConfig>,
    #[serde(default, alias = "activeModelSlot")]
    pub active_model_slot: Option<String>,
    #[serde(default, alias = "budget_usd")]
    pub budget_usd: Option<f64>,
    #[serde(default)]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub workflows: Option<Vec<String>>,
    #[serde(default, alias = "mcpTools")]
    pub mcp_tools: Option<Vec<String>>,
    #[serde(default, alias = "requiresOversight")]
    pub requires_oversight: Option<bool>,
    #[serde(default, alias = "shadowsHumanId")]
    pub shadows_human_id: Option<String>,
}

impl CreateAgentRequest {
    pub fn into_engine_agent(self) -> Result<EngineAgent, AppError> {
        let id = self.id.trim().to_string();
        if id.is_empty() || id.len() > 128 {
            return Err(AppError::BadRequest(
                "Agent ID must be non-empty and <= 128 characters".into(),
            ));
        }
        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AppError::BadRequest(
                "Agent ID must contain only alphanumeric characters, dashes, and underscores"
                    .into(),
            ));
        }

        let model = if let Some(mc) = self.model_config {
            mc
        } else if let Some(model_str) = self.model {
            let provider =
                ModelProvider::from_model_id(&model_str).unwrap_or(ModelProvider::Openai);
            ModelConfig {
                provider,
                model_id: model_str,
                ..Default::default()
            }
        } else {
            ModelConfig {
                provider: ModelProvider::Openai,
                model_id: "gpt-4o".to_string(),
                ..Default::default()
            }
        };

        Ok(EngineAgent {
            identity: AgentIdentity {
                id: id.clone(),
                name: self.name.trim().to_string(),
                role: self.role.trim().to_string(),
                department: self.department.trim().to_string(),
                description: self.description.trim().to_string(),
                category: self.category.unwrap_or_else(|| "general".to_string()),
                theme_color: None,
            },
            health: AgentHealth {
                status: "idle".to_string(),
                failure_count: 0,
                last_failure_at: None,
                heartbeat_at: Some(chrono::Utc::now()),
            },
            models: AgentModels {
                model_id: None,
                model,
                planning_slot: self.planning_slot,
                execution_slot: self.execution_slot,
                active_model_slot: self.active_model_slot,
            },
            economics: AgentEconomics {
                budget_usd: self.budget_usd.unwrap_or(0.0),
                cost_usd: 0.0,
                tokens_used: 0,
                token_usage: Default::default(),
            },
            capabilities: AgentCapabilities {
                skills: self.skills.unwrap_or_default(),
                workflows: self.workflows.unwrap_or_default(),
                mcp_tools: self.mcp_tools.unwrap_or_default(),
                skill_manifest: None,
            },
            state: Default::default(),
            metadata: Default::default(),
            created_at: Some(chrono::Utc::now()),
            requires_oversight: self.requires_oversight.unwrap_or(false),
            shadows_human_id: self.shadows_human_id,
            voice_id: None,
            voice_engine: None,
            connector_configs: Vec::new(),
            version: 1,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[allow(dead_code)]
    pub temperature: Option<f32>,
    #[allow(dead_code)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionChoiceMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: ChatCompletionChoiceMessage,
    pub finish_reason: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: ChatCompletionUsage,
}

#[derive(Debug, serde::Deserialize)]
pub struct CloneMissionRequest {
    pub primary_goal: Option<String>,
}
