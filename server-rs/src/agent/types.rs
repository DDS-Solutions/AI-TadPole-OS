//! Core data structures and types for the Tadpole OS agent swarm.
//!
//! This module defines the foundational structs used across the engine for agent identity,
//! mission management, and infrastructure configuration. All types are optimized for
//! high-performance serialization/deserialization across the REST and WebSocket boundaries.

use serde::{Deserialize, Serialize};

/// Operational statistics for token consumption in an LLM turn.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Number of tokens in the prompt.
    #[serde(rename = "inputTokens")]
    pub input_tokens: u32,
    /// Number of tokens in the completion.
    #[serde(rename = "outputTokens")]
    pub output_tokens: u32,
    /// Sum of input and output tokens.
    #[serde(rename = "totalTokens")]
    pub total_tokens: u32,
}

/// Configuration for an agent's model.
/// Kept in sync with TS `ModelConfig` in `server/types.ts`.
/// Unified configuration for an LLM model and its execution parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    /// The protocol used (e.g., "openai", "groq", "gemini").
    pub provider: String,
    /// The unique model name/slug.
    #[serde(rename = "modelId")]
    pub model_id: String,
    /// Secret key (injected from vault at runtime).
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    /// Optional service endpoint.
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    /// Permanent core personality instructions.
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    /// Sampling temperature (0.0 to 1.0).
    pub temperature: Option<f32>,
    /// Maximum completion tokens.
    #[serde(rename = "maxTokens")]
    pub max_tokens: Option<u32>,
    /// UUID for multi-tier provider identification.
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
    /// Rate limit: Requests Per Minute.
    pub rpm: Option<u32>,
    /// Rate limit: Requests Per Day.
    pub rpd: Option<u32>,
    /// Rate limit: Tokens Per Minute.
    pub tpm: Option<u32>,
    /// Rate limit: Tokens Per Day.
    pub tpd: Option<u32>,
    /// IDs of active skills.
    #[serde(default, alias = "skills")]
    pub skills: Option<Vec<String>>,
    /// IDs of active workflows.
    #[serde(default)]
    pub workflows: Option<Vec<String>>,
    /// IDs of active MCP tools.
    #[serde(default, rename = "mcpTools")]
    pub mcp_tools: Option<Vec<String>>,
}

impl ModelConfig {
    /// Returns true if the model natively supports tool calling (function declarations).
    /// Some models (like Phi3 via Ollama) require tools to be suppressed to avoid API errors.
    pub fn supports_native_tools(&self) -> bool {
        // Suppress tools for models that are known to be incompatible with certain OpenAI-bridge implementations
        let mid = self.model_id.to_lowercase();
        if mid.contains("phi3") || mid.contains("phi-3") {
            return false;
        }
        true
    }
}

/// External LLM provider configuration (e.g., OpenAI, Groq).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Unique provider slug.
    pub id: String,
    /// Friendly display name.
    pub name: String,
    /// URL or identifier for the provider icon.
    pub icon: Option<String>,
    /// Secret key (stored in vault).
    #[serde(default, rename = "apiKey")]
    pub api_key: Option<String>,
    /// API base endpoint.
    #[serde(default, rename = "baseUrl")]
    pub base_url: Option<String>,
    /// Protocol used for dispatch ("openai", "gemini", etc).
    pub protocol: String,
    /// UUID for multi-tier provider identification.
    #[serde(default, rename = "externalId")]
    pub external_id: Option<String>,
    /// Arbitrary per-provider headers (e.g., for Inception/OpenRouter).
    #[serde(default, rename = "customHeaders")]
    pub custom_headers: Option<std::collections::HashMap<String, String>>,
    /// Suggested model for audio transcription (optional).
    #[serde(default, rename = "audioModel")]
    pub audio_model: Option<String>,
}

/// Definition of a specific AI model provided by a specific service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Unique model slug.
    pub id: String,
    /// Friendly display name.
    pub name: String,
    /// Parent provider ID.
    #[serde(rename = "providerId")]
    pub provider_id: String,
    /// Friendly provider name (Frontend Parity).
    #[serde(default)]
    pub provider: Option<String>,
    /// Rate limit: Requests Per Minute.
    #[serde(default)]
    pub rpm: Option<u32>,
    /// Rate limit: Tokens Per Minute.
    #[serde(default)]
    pub tpm: Option<u32>,
    /// Rate limit: Requests Per Day.
    #[serde(default)]
    pub rpd: Option<u32>,
    /// Rate limit: Tokens Per Day.
    #[serde(default)]
    pub tpd: Option<u32>,
    /// Supported input/output types (e.g., "TEXT", "AUDIO").
    #[serde(default)]
    pub modality: Option<String>,
}

/// Persistent state representation of an Agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineAgent {
    /// Unique UUID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Current functional role.
    pub role: String,
    /// Home department.
    pub department: String,
    /// Objective and behavior details.
    pub description: String,

    /// Primary model ID lookup.
    #[serde(rename = "model")]
    pub model_id: Option<String>,

    /// Default model config.
    #[serde(rename = "modelConfig")]
    pub model: ModelConfig,

    /// Auxiliary model ID for slot 2.
    #[serde(default, rename = "model2")]
    pub model_2: Option<String>,
    /// Auxiliary model ID for slot 3.
    #[serde(default, rename = "model3")]
    pub model_3: Option<String>,

    /// Model configuration for slot 2.
    #[serde(default, rename = "modelConfig2")]
    pub model_config2: Option<ModelConfig>,
    /// Model configuration for slot 3.
    #[serde(default, rename = "modelConfig3")]
    pub model_config3: Option<ModelConfig>,

    /// The index of the currently active model slot.
    #[serde(default, rename = "activeModelSlot")]
    pub active_model_slot: Option<i32>,

    /// Snapshot of the currently active mission (if any).
    #[serde(default, rename = "activeMission")]
    pub active_mission: Option<serde_json::Value>,

    /// Current session status.
    pub status: String,

    /// Description of the task currently being executed, if any.
    #[serde(
        default,
        rename = "currentTask",
        skip_serializing_if = "Option::is_none"
    )]
    pub current_task: Option<String>,

    /// Historical token count.
    #[serde(rename = "tokensUsed")]
    pub tokens_used: u32,
    /// Detailed usage for the last turn.
    #[serde(default, rename = "tokenUsage")]
    pub token_usage: TokenUsage,

    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub workflows: Vec<String>,
    #[serde(default, rename = "mcpTools")]
    pub mcp_tools: Vec<String>,

    #[serde(default)]
    pub skill_manifest: Option<crate::agent::skill_manifest::SkillManifest>,

    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,

    #[serde(default, rename = "themeColor")]
    pub theme_color: Option<String>,

    #[serde(default, rename = "budgetUsd")]
    pub budget_usd: f64,
    #[serde(default, rename = "costUsd")]
    pub cost_usd: f64,

    #[serde(default, rename = "voiceId")]
    pub voice_id: Option<String>,
    #[serde(default, rename = "voiceEngine")]
    pub voice_engine: Option<String>,

    #[serde(default = "default_category")]
    pub category: String,

    #[serde(default, rename = "failureCount")]
    pub failure_count: u32,

    #[serde(default, rename = "lastFailureAt")]
    pub last_failure_at: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(default, rename = "requiresOversight")]
    pub requires_oversight: bool,

    #[serde(default, rename = "workingMemory")]
    pub working_memory: serde_json::Value,
}

fn default_category() -> String {
    "user".to_string()
}

impl EngineAgent {
    pub fn resolve_provider_context(&self) -> crate::agent::runner::RunContext {
        crate::agent::runner::RunContext {
            agent_id: self.id.clone(),
            name: self.name.clone(),
            role: self.role.clone(),
            department: self.department.clone(),
            description: self.description.clone(),
            model_config: self.model.clone(),
            skills: self.skills.clone(),
            workflows: self.workflows.clone(),
            mcp_tools: self.mcp_tools.clone(),
            mission_id: "system-internal".to_string(),
            depth: 0,
            lineage: vec![],
            provider_name: self.model.provider.clone(),
            workspace_root: std::path::PathBuf::from("data/workspaces/default"),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("data/workspaces/default"),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: self.working_memory.clone(),
        }
    }
}

/// Request payload for starting an autonomous task.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskPayload {
    /// The mission objective message.
    pub message: String,
    /// Targeted cluster/workspace.
    #[serde(rename = "clusterId")]
    pub cluster_id: Option<String>,
    /// Department designation.
    pub department: Option<String>,
    /// Model provider override.
    pub provider: Option<String>,
    /// Model ID override.
    #[serde(rename = "modelId")]
    pub model_id: Option<String>,
    /// Secret key override.
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    /// Base URL override.
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    /// Requests Per Minute rate limit override.
    pub rpm: Option<u32>,
    /// Tokens Per Minute rate limit override.
    pub tpm: Option<u32>,
    /// Max USD budget for this task.
    #[serde(rename = "budgetUsd")]
    pub budget_usd: Option<f64>,
    /// Swarm recursion depth.
    #[serde(rename = "swarmDepth")]
    pub swarm_depth: Option<u32>,
    /// Ancestry path of nodes leading to this recruitment.
    #[serde(rename = "swarmLineage")]
    pub swarm_lineage: Option<Vec<String>>,
    /// Multi-tier provider identifier.
    pub external_id: Option<String>,
    /// If true, disables mutation/recruitment.
    #[serde(rename = "safeMode")]
    pub safe_mode: Option<bool>,
    /// If true, enables iterative analysis.
    #[serde(rename = "analysis")]
    pub analysis: Option<bool>,
    #[serde(rename = "traceparent")]
    pub traceparent: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    /// Relevant file paths for context propagation.
    #[serde(default, rename = "contextFiles")]
    pub context_files: Option<Vec<String>>,
    /// Summary of recent findings from parent agent.
    #[serde(default, rename = "recentFindings")]
    pub recent_findings: Option<String>,
}

/// Delta payload for updating an agent's persistent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigUpdate {
    /// New display name.
    pub name: Option<String>,
    /// New functional role.
    pub role: Option<String>,
    /// New department designation.
    pub department: Option<String>,
    /// Model provider identifier ("openai", etc).
    pub provider: Option<String>,
    /// Primary model ID.
    #[serde(rename = "modelId")]
    pub model_id: Option<String>,
    /// Model ID for slot 2.
    pub model2: Option<String>,
    /// Model ID for slot 3.
    pub model3: Option<String>,
    /// Secret key override.
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    /// Global personality instructions.
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    /// LLM sampling temperature.
    pub temperature: Option<f32>,
    /// Optional service endpoint.
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    /// Hexadecimal theme color.
    #[serde(rename = "themeColor")]
    pub theme_color: Option<String>,
    /// Maximum USD budget.
    #[serde(rename = "budgetUsd")]
    pub budget_usd: Option<f64>,
    /// Provider-specific external ID.
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
    /// Enabled skill names.
    pub skills: Option<Vec<String>>,
    /// Enabled workflow names.
    pub workflows: Option<Vec<String>>,
    /// Enabled MCP tool names.
    #[serde(rename = "mcpTools")]
    pub mcp_tools: Option<Vec<String>>,
    /// Active slot index.
    #[serde(rename = "activeModelSlot")]
    pub active_model_slot: Option<i32>,
    /// Inline config for slot 2.
    #[serde(rename = "modelConfig2")]
    pub model_config2: Option<ModelConfig>,
    /// Inline config for slot 3.
    #[serde(rename = "modelConfig3")]
    pub model_config3: Option<ModelConfig>,
    /// ID for TTS synthesis.
    #[serde(rename = "voiceId")]
    pub voice_id: Option<String>,
    /// Logic engine for TTS.
    #[serde(rename = "voiceEngine")]
    pub voice_engine: Option<String>,
    /// Optimization: Categorize agent for UI filtering.
    pub category: Option<String>,
    /// Persistence: Whether the agent requires human oversight.
    #[serde(rename = "requiresOversight")]
    pub requires_oversight: Option<bool>,
}

/// Record of a tool invocation request from an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique call ID.
    pub id: String,
    /// Parent mission ID.
    pub mission_id: Option<String>,
    /// Originating agent ID.
    #[serde(rename = "agentId")]
    pub agent_id: String,
    /// Name of the skill/tool requested.
    pub skill: String,
    /// JSON-encoded arguments for the tool.
    pub params: serde_json::Value,
    /// Department of the originating agent.
    pub department: String,
    /// Human-friendly explanation of why the tool is being used.
    pub description: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SkillType {
    Skill,
    Workflow,
}

/// Request for a new system skill (skill or workflow).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProposal {
    /// Type of skill (active tool or passive workflow).
    pub r#type: SkillType,
    /// Unique skill name (lowercase, snake_case).
    pub name: String,
    /// LLM-facing description of utility.
    pub description: String,
    /// Command line to execute for skills.
    pub execution_command: Option<String>,
    /// Input validation schema for skills.
    pub schema: Option<serde_json::Value>,
    /// Markdown content for workflows.
    pub content: Option<String>,
    /// Long-form operating instructions.
    pub full_instructions: Option<String>,
    /// Prohibited behaviors/outputs.
    pub negative_constraints: Option<Vec<String>>,
    /// Integration test script.
    pub verification_script: Option<String>,
    /// Optimization: Categorize skill for UI filtering.
    #[serde(default = "default_category")]
    pub category: String,
}

/// A pending decision point in the Oversight sector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightEntry {
    /// Unique registry ID for the entry.
    pub id: String,
    /// Parent mission context.
    pub mission_id: Option<String>,
    /// Targeted tool call details (if applicable).
    #[serde(rename = "toolCall")]
    pub tool_call: Option<ToolCall>,
    /// Proposed architectural change (if applicable).
    #[serde(rename = "skillProposal", alias = "capabilityProposal")]
    pub skill_proposal: Option<SkillProposal>,
    /// Current decision state ("pending", "approved", "rejected").
    pub status: String,
    /// Formal creation timestamp.
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Standardized mission status lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MissionStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Paused,
}

/// Represents a mission execution record stored in the persistence layer.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mission {
    /// Unique identifier for the mission.
    pub id: String,
    /// ID of the agent assigned to this mission.
    pub agent_id: String,
    /// Human-readable title or objective of the mission.
    pub title: String,
    /// Current execution state of the mission.
    pub status: MissionStatus,
    /// ISO 8601 timestamp of mission initialization.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// ISO 8601 timestamp of the last status or cost update.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Total budget allocated to this mission in USD.
    pub budget_usd: f64,
    /// Cumulative monetary cost incurred by this mission so far.
    pub cost_usd: f64,
    /// Flag indicating if the mission is operating in a degraded state (e.g., API failures).
    pub is_degraded: Option<bool>,
}

/// Log entry specific to a mission's permanent execution history.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MissionLog {
    /// Unique log ID.
    pub id: String,
    /// Associated mission ID.
    pub mission_id: String,
    /// Originating agent ID (or "System").
    pub agent_id: String,
    /// Symbolic name of the actor (e.g., "Developer").
    pub source: String,
    /// Content of the message/action.
    pub text: String,
    /// Operational importance ("info", "warning", "error", "success").
    pub severity: String,
    /// Exact UTC timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional metadata payload for rich UI rendering.
    #[sqlx(skip)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

/// Registry entry for a remote infrastructure node in the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNode {
    /// Unique node UUID.
    pub id: String,
    /// Friendly node name.
    pub name: String,
    /// Network address (IP or hostname).
    pub address: String,
    /// Connection state ("online", "offline", "deploying").
    pub status: String,
    /// Last detected heart-beat.
    #[serde(rename = "lastSeen")]
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Hardware and software skills of the node.
    pub metadata: std::collections::HashMap<String, String>,
}

/// User decision payload for resolving an oversight entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightDecision {
    /// The final verdict ("approved" or "rejected").
    pub decision: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_engine_agent_deserialization_defaults() {
        let agent_json = json!({
            "id": "test-agent",
            "name": "Test Agent",
            "role": "Tester",
            "department": "QA",
            "description": "Tests things",
            "status": "active",
            "model": "gpt-4",
            "modelConfig": {
                "provider": "openai",
                "modelId": "gpt-4"
            },
            "tokensUsed": 0,
            "budgetUsd": 10.0,
            "costUsd": 0.0
        });

        let agent_str = agent_json.to_string();
        let agent: EngineAgent = serde_json::from_str(&agent_str)
            .expect("Failed to deserialize agent with missing fields");

        assert_eq!(agent.id, "test-agent");
        assert_eq!(agent.token_usage.total_tokens, 0);
        assert!(agent.skills.is_empty());
        assert!(agent.workflows.is_empty());
        assert!(agent.metadata.is_empty());
        assert_eq!(agent.budget_usd, 10.0);
    }

    #[test]
    fn test_engine_agent_deserialization_full() {
        let agent_json = json!({
            "id": "full-agent",
            "name": "Full Agent",
            "role": "Lead",
            "department": "Engineering",
            "description": "Full description",
            "status": "active",
            "model": "gpt-4o",
            "modelConfig": {
                "provider": "openai",
                "modelId": "gpt-4o"
            },
            "tokensUsed": 100,
            "tokenUsage": {
                "inputTokens": 40,
                "outputTokens": 60,
                "totalTokens": 100
            },
            "skills": ["coding"],
            "workflows": ["deploy"],
            "metadata": {"key": "value"},
            "budgetUsd": 100.0,
            "costUsd": 0.5
        });

        let agent_str = agent_json.to_string();
        let agent: EngineAgent =
            serde_json::from_str(&agent_str).expect("Failed to deserialize full agent");

        assert_eq!(agent.id, "full-agent");
        assert_eq!(agent.token_usage.total_tokens, 100);
        assert_eq!(agent.skills, vec!["coding"]);
        assert_eq!(agent.workflows, vec!["deploy"]);
        assert!(agent.mcp_tools.is_empty());
        assert_eq!(agent.metadata.get("key").unwrap(), &json!("value"));
    }
}
