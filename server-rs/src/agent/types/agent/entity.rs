//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / entity
//! - **Primary Entrypoints**: `EngineAgent`, `AgentIdentity`, `AgentEconomics`, `AgentHealth`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::types::model::{ConnectorConfig, ModelConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, specta::Type)]
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentEconomics {
    pub budget_usd: f64,
    pub cost_usd: f64,
    #[specta(type = f64)]
    pub tokens_used: u64,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentHealth {
    pub status: String,
    pub failure_count: u32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub heartbeat_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    pub skills: Vec<String>,
    pub workflows: Vec<String>,
    pub mcp_tools: Vec<String>,
    pub skill_manifest: Option<crate::agent::skill_manifest::SkillManifest>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentState {
    pub active_mission: Option<serde_json::Value>,
    pub current_task: Option<String>,
    pub working_memory: serde_json::Value,
    pub current_reasoning_turn: u32,
}

#[derive(Debug, Clone, PartialEq, Default, specta::Type)]
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
    pub shadows_human_id: Option<String>,
    pub voice_id: Option<String>,
    pub voice_engine: Option<String>,
    pub connector_configs: Vec<ConnectorConfig>,
    pub version: u32,
}

impl EngineAgent {
    /// Merges volatile, runtime-only fields from live memory state into self if they were not
    /// modified by an update payload (comparing self against original snapshot).
    pub fn apply_runtime_state(&mut self, live_state: &EngineAgent, snapshot: &EngineAgent) {
        if self.health.status == snapshot.health.status {
            self.health.status = live_state.health.status.clone();
        }
        if self.health.heartbeat_at == snapshot.health.heartbeat_at {
            self.health.heartbeat_at = live_state.health.heartbeat_at;
        }
        if self.health.failure_count == snapshot.health.failure_count {
            self.health.failure_count = live_state.health.failure_count;
        }
        if self.health.last_failure_at == snapshot.health.last_failure_at {
            self.health.last_failure_at = live_state.health.last_failure_at;
        }
        if self.state.current_task == snapshot.state.current_task {
            self.state.current_task = live_state.state.current_task.clone();
        }
        if self.state.active_mission == snapshot.state.active_mission {
            self.state.active_mission = live_state.state.active_mission.clone();
        }
        if self.economics.cost_usd == snapshot.economics.cost_usd {
            self.economics.cost_usd = live_state.economics.cost_usd;
        }
        if self.economics.tokens_used == snapshot.economics.tokens_used {
            self.economics.tokens_used = live_state.economics.tokens_used;
        }
        if self.economics.token_usage.input_tokens == snapshot.economics.token_usage.input_tokens {
            self.economics.token_usage.input_tokens = live_state.economics.token_usage.input_tokens;
        }
        if self.economics.token_usage.output_tokens == snapshot.economics.token_usage.output_tokens
        {
            self.economics.token_usage.output_tokens =
                live_state.economics.token_usage.output_tokens;
        }
        if self.economics.token_usage.total_tokens == snapshot.economics.token_usage.total_tokens {
            self.economics.token_usage.total_tokens = live_state.economics.token_usage.total_tokens;
        }
    }

    /// Returns true if the agent has a positive budget and has exhausted it.
    pub fn is_bankrupt(&self) -> bool {
        self.economics.cost_usd >= self.economics.budget_usd && self.economics.budget_usd > 0.0
    }

    #[allow(dead_code)]
    pub fn is_suspended(&self) -> bool {
        self.health.status == "suspended"
    }
}
