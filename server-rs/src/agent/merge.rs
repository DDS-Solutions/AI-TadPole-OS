//! @docs ARCHITECTURE:Domain
//! 
//! ### AI Assist Note
//! **Domain Merge Logic**: Authoritative state transition engine for AI Agents. 
//! Orchestrates partial updates, status changes, and telemetry resets. 
//! Shifted from imperative route handlers to domain methods to enforce 
//! structural integrity and prevent state corruption during concurrent updates.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Field collision during concurrent `AgentConfigUpdate` applications or mismatch in CamelCase deserialization from frontend.
//! - **Telemetry Link**: Search `[AgentMerge]` in server traces.
//!

use serde::{Deserialize, Serialize};
use crate::agent::types::{EngineAgent, ModelConfig, ModelProvider, ConnectorConfig};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

impl EngineAgent {
    /// Pauses a running agent, changing its operational status.
    pub fn pause(&mut self) -> bool {
        if self.health.status != "suspended" {
            self.health.status = "suspended".to_string();
            return true;
        }
        false
    }
 
    /// Resumes a suspended agent, returning it to idle.
    pub fn resume(&mut self) -> bool {
        if self.health.status == "suspended" {
            self.health.status = "idle".to_string();
            return true;
        }
        false
    }
 
    /// Resets agent failure telemetry and status.
    pub fn reset(&mut self) {
        self.health.failure_count = 0;
        self.health.last_failure_at = None;
        self.health.status = "idle".to_string();
    }
 
    /// Updates the active mission context.
    pub fn set_mission(&mut self, mission: serde_json::Value) {
        self.state.active_mission = Some(mission);
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfigUpdate {
    pub name: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
    pub provider: Option<ModelProvider>,
    pub model_id: Option<String>,
    #[serde(default, alias = "model2")]
    pub model_2: Option<String>,
    #[serde(default, alias = "model3")]
    pub model_3: Option<String>,
    pub api_key: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub base_url: Option<String>,
    pub reasoning_depth: Option<u32>,
    pub act_threshold: Option<f32>,
    pub theme_color: Option<String>,
    pub budget_usd: Option<f64>,
    pub external_id: Option<String>,
    pub skills: Option<Vec<String>>,
    pub workflows: Option<Vec<String>>,
    pub mcp_tools: Option<Vec<String>>,
    pub active_model_slot: Option<i32>,
    pub model_config2: Option<ModelConfig>,
    pub model_config3: Option<ModelConfig>,
    pub voice_id: Option<String>,
    pub voice_engine: Option<String>,
    pub category: Option<String>,
    pub requires_oversight: Option<bool>,
    pub connector_configs: Option<Vec<ConnectorConfig>>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub tokens_used: Option<u32>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_pulse: Option<DateTime<Utc>>,
    pub current_task: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl AgentConfigUpdate {
    pub fn apply_to(&self, agent: &mut EngineAgent) -> bool {
        let mut changed = false;
 
        if let Some(name) = &self.name {
            agent.identity.name = name.clone();
            changed = true;
        }
        if let Some(role) = &self.role {
            agent.identity.role = role.clone();
            changed = true;
        }
        if let Some(dept) = &self.department {
            agent.identity.department = dept.clone();
            changed = true;
        }
        if let Some(model_id) = &self.model_id {
            agent.models.model_id = Some(model_id.clone());
            agent.models.model.model_id = model_id.clone();
            changed = true;
        }
        if let Some(provider) = self.provider {
            agent.models.model.provider = provider;
            changed = true;
        }
        if let Some(temp) = self.temperature {
            agent.models.model.temperature = Some(temp);
            changed = true;
        }
        if let Some(prompt) = &self.system_prompt {
            agent.models.model.system_prompt = Some(prompt.clone());
            changed = true;
        }
        if let Some(reasoning_depth) = self.reasoning_depth {
            agent.models.model.reasoning_depth = Some(reasoning_depth);
            changed = true;
        }
        if let Some(act_threshold) = self.act_threshold {
            agent.models.model.act_threshold = Some(act_threshold);
            changed = true;
        }
        if let Some(api_key) = &self.api_key {
            agent.models.model.api_key = Some(api_key.clone());
            changed = true;
        }
        if let Some(color) = &self.theme_color {
            agent.identity.theme_color = Some(color.clone());
            changed = true;
        }
        if let Some(budget) = self.budget_usd {
            agent.economics.budget_usd = budget;
            changed = true;
        }
        if let Some(skills) = &self.skills {
            agent.capabilities.skills = skills.clone();
            changed = true;
        }
        if let Some(workflow) = &self.workflows {
            agent.capabilities.workflows = workflow.clone();
            changed = true;
        }
        if let Some(mcp_tools) = &self.mcp_tools {
            agent.capabilities.mcp_tools = mcp_tools.clone();
            changed = true;
        }
        if let Some(m2) = &self.model_2 {
            agent.models.model_2 = Some(m2.clone());
            changed = true;
        }
        if let Some(m3) = &self.model_3 {
            agent.models.model_3 = Some(m3.clone());
            changed = true;
        }
        if let Some(active_slot) = self.active_model_slot {
            agent.models.active_model_slot = Some(active_slot);
            changed = true;
        }
        if let Some(mc2) = &self.model_config2 {
            agent.models.model_config2 = Some(mc2.clone());
            changed = true;
        }
        if let Some(mc3) = &self.model_config3 {
            agent.models.model_config3 = Some(mc3.clone());
            changed = true;
        }
        if let Some(connector_configs) = &self.connector_configs {
            agent.connector_configs = connector_configs.clone();
            changed = true;
        }
        if let Some(voice_id) = &self.voice_id {
            agent.voice_id = Some(voice_id.clone());
            changed = true;
        }
        if let Some(voice_engine) = &self.voice_engine {
            agent.voice_engine = Some(voice_engine.clone());
            changed = true;
        }
        if let Some(base_url) = &self.base_url {
            agent.models.model.base_url = Some(base_url.clone());
            changed = true;
        }
        if let Some(oversight) = self.requires_oversight {
            agent.requires_oversight = oversight;
            changed = true;
        }
        if let Some(category) = &self.category {
            agent.identity.category = category.clone();
            changed = true;
        }
        if let Some(created_at) = self.created_at {
            agent.created_at = Some(created_at);
            changed = true;
        }
        if let Some(last_pulse) = self.last_pulse {
            agent.health.heartbeat_at = Some(last_pulse);
            changed = true;
        }
        if let Some(current_task) = &self.current_task {
            agent.state.current_task = Some(current_task.clone());
            changed = true;
        }
        if let Some(metadata) = &self.metadata {
            agent.metadata.extend(metadata.clone());
            changed = true;
        }
 
        if let Some(it) = self.input_tokens {
            agent.economics.token_usage.input_tokens = it;
            changed = true;
        }
        if let Some(ot) = self.output_tokens {
            agent.economics.token_usage.output_tokens = ot;
            changed = true;
        }
        if let Some(tt) = self.total_tokens {
            agent.economics.token_usage.total_tokens = tt;
            changed = true;
        }
        if let Some(tu) = self.tokens_used {
            agent.economics.tokens_used = tu;
            changed = true;
        }
 
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::EngineAgent;

    #[test]
    fn test_agent_state_logic() {
        let mut agent = EngineAgent {
            health: crate::agent::types::AgentHealth {
                status: "idle".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
 
        assert!(agent.pause());
        assert_eq!(agent.health.status, "suspended");
        assert!(!agent.pause()); // Should not change if already suspended
 
        assert!(agent.resume());
        assert_eq!(agent.health.status, "idle");
        assert!(!agent.resume()); // Should not change if idle
 
        agent.health.failure_count = 5;
        agent.health.status = "failed".to_string();
        agent.reset();
        assert_eq!(agent.health.failure_count, 0);
        assert_eq!(agent.health.status, "idle");
    }

    #[test]
    fn test_agent_config_merge() {
        let mut agent = EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                name: "Original".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
 
        let update = AgentConfigUpdate {
            name: Some("Updated".to_string()),
            budget_usd: Some(500.0),
            skills: Some(vec!["rust".to_string()]),
            input_tokens: Some(100),
            ..Default::default()
        };
 
        let changed = update.apply_to(&mut agent);
        assert!(changed);
        assert_eq!(agent.identity.name, "Updated");
        assert_eq!(agent.economics.budget_usd, 500.0);
        assert_eq!(agent.capabilities.skills, vec!["rust".to_string()]);
        assert_eq!(agent.economics.token_usage.input_tokens, 100);
    }

    #[test]
    fn test_update_deserialization_parity() {
        // Enforce that we can deserialize from camelCase matching frontend serializers
        let json = r#"{
            "name": "CamelCaseAgent",
            "budgetUsd": 123.45,
            "inputTokens": 1000
        }"#;

        let update: AgentConfigUpdate = serde_json::from_str(json).expect("Deserialization failed");
        assert_eq!(update.name, Some("CamelCaseAgent".to_string()));
        assert_eq!(update.budget_usd, Some(123.45));
        assert_eq!(update.input_tokens, Some(1000));
    }
}

// Metadata: [AgentMerge]

// Metadata: [merge]

// Metadata: [merge]
