//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Unified Schemas**: Defines the source-of-truth data contracts for agents,
//! missions, and telemetry. Ensures **Serialization Parity** with the TypeScript
//! frontend via strict `serde` renaming (snake_case/camelCase bridge).
//! Features **IMR-01 (Intelligent Model Registry)** logic for automated model 
//! discovery and capability inference (Vision, Tools, Reasoning).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: JSON deserialization mismatch (422 Unprocessable Entity),
//!   missing model config defaults leading to `None` pointer dereference
//!   logic errors, or invalid rate limit parsing from environment variables.
//! - **IMR-01 Integrity**: Verify that `ModelCapabilities` defaults match the 
//!   conservative inference logic in `capability_matrix.rs`.
//! - **Trace Scope**: `server-rs::agent::types`
//!

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

// =============================================================================
// ENUMERATIONS (For Compile-Time Safety)
// =============================================================================

/// Defines the protocols for connecting to LLM providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    #[default]
    Openai,
    Anthropic,
    Google,
    Gemini, // Alias for Google/Gemini protocol
    Ollama,
    Groq,
    Deepseek,
    Xai,
    Inception,
    Openrouter,
    // Add new providers here
}

impl ModelProvider {
    /// Helper to convert a string slug into the enum.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" | "open-ai" => Some(ModelProvider::Openai),
            "anthropic" | "claude" => Some(ModelProvider::Anthropic),
            "google" | "gemini" | "google-ai-studio" | "google-vertex" => {
                Some(ModelProvider::Google)
            }
            "ollama" => Some(ModelProvider::Ollama),
            "groq" => Some(ModelProvider::Groq),
            "deepseek" | "deep-seek" => Some(ModelProvider::Deepseek),
            "xai" | "grok" => Some(ModelProvider::Xai),
            "inception" | "mercury" => Some(ModelProvider::Inception),
            "openrouter" | "open-router" => Some(ModelProvider::Openrouter),
            _ => None,
        }
    }
}

impl std::fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ModelProvider::Openai => "openai",
            ModelProvider::Anthropic => "anthropic",
            ModelProvider::Google | ModelProvider::Gemini => "google",
            ModelProvider::Ollama => "ollama",
            ModelProvider::Groq => "groq",
            ModelProvider::Deepseek => "deepseek",
            ModelProvider::Xai => "xai",
            ModelProvider::Inception => "inception",
            ModelProvider::Openrouter => "openrouter",
        };
        write!(f, "{}", s)
    }
}

/// Supported modalities for AI models (text, image, audio, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    #[default]
    Llm,
    Vision,
    Voice,
    Reasoning,
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

/// Trait for types that can validate their own internal state.
pub trait Validatable {
    fn validate(&self) -> Result<(), String>;
}

/// Operational statistics for token consumption in an LLM turn.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Number of tokens in the prompt.
    #[serde(default, alias = "inputTokens")]
    pub input_tokens: u32,
    /// Number of tokens in the completion.
    #[serde(default, alias = "outputTokens")]
    pub output_tokens: u32,
    /// Sum of input and output tokens.
    #[serde(default, alias = "totalTokens")]
    pub total_tokens: u32,
}

/// Represents a synchronization manifest for external SME data (Phase 2).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct SyncManifest {
    pub id: String,
    pub agent_id: String,
    pub source_type: String,
    pub source_uri: String,
    pub status: String,
    pub last_sync_at: Option<DateTime<Utc>>,
}

/// Dynamic capabilities and constraints of an AI model.
/// Part of the Intelligent Model Registry (IMR-01).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    /// True if the model supports native tool-calling (function calling).
    #[serde(default, alias = "supportsTools")]
    pub supports_tools: bool,
    /// True if the model supports vision-to-text (multimodal).
    #[serde(default, alias = "supportsVision")]
    pub supports_vision: bool,
    /// True if the model supports constrained JSON output mode.
    #[serde(default, alias = "supportsStructuredOutput")]
    pub supports_structured_output: bool,
    /// True if the model is a high-latency reasoning model (e.g., o1, o3, R1).
    #[serde(default, alias = "supportsReasoning")]
    pub supports_reasoning: bool,
    /// Maximum context length in tokens.
    #[serde(default, alias = "contextWindow")]
    pub context_window: u32,
    /// Maximum output tokens per request.
    #[serde(default, alias = "maxOutputTokens")]
    pub max_output_tokens: u32,
}

/// Configuration for an agent's model.
/// Kept in sync with TS `ModelConfig` in `server/types.ts`.
/// Unified configuration for an LLM model and its execution parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    /// The protocol used (e.g., "openai", "groq", "gemini").
    pub provider: ModelProvider,
    /// The unique model name/slug.
    #[serde(default, alias = "modelId")]
    pub model_id: String,
    /// Secret key (injected from vault at runtime).
    #[serde(default, alias = "apiKey")]
    pub api_key: Option<String>,
    /// Optional service endpoint.
    #[serde(default, alias = "baseUrl")]
    pub base_url: Option<String>,
    /// Permanent core personality instructions.
    #[serde(default, alias = "systemPrompt")]
    pub system_prompt: Option<String>,
    /// Sampling temperature (0.0 to 1.0).
    pub temperature: Option<f32>,
    /// Maximum completion tokens.
    #[serde(default, alias = "maxTokens")]
    pub max_tokens: Option<u32>,
    /// UUID for multi-tier provider identification.
    #[serde(default, alias = "externalId")]
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
    #[serde(default, alias = "mcpTools")]
    pub mcp_tools: Option<Vec<String>>,
    /// Active OBLITERATUS Steering Vectors applied to inference layers directly
    #[serde(default, alias = "steeringVectors")]
    pub steering_vectors: Option<Vec<String>>,
    /// Connector source configurations (Phase 2).
    #[serde(default, alias = "connectorConfigs")]
    pub connector_configs: Option<Vec<ConnectorConfig>>,
    /// Arbitrary vendor-specific parameters (e.g., json_mode, seed, thinking).
    #[serde(default, alias = "extraParameters")]
    pub extra_parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Dynamic source configuration for SME data connectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub r#type: String, // 'fs', 'slack', etc.
    pub uri: String,    // path or ID
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

    /// Merges another ModelConfig into this one, with this one taking precedence.
    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = self.clone();

        if merged.system_prompt.is_none() {
            merged.system_prompt = other.system_prompt.clone();
        }
        if merged.temperature.is_none() {
            merged.temperature = other.temperature;
        }
        if merged.max_tokens.is_none() {
            merged.max_tokens = other.max_tokens;
        }
        if merged.rpm.is_none() {
            merged.rpm = other.rpm;
        }
        if merged.rpd.is_none() {
            merged.rpd = other.rpd;
        }
        if merged.tpm.is_none() {
            merged.tpm = other.tpm;
        }
        if merged.tpd.is_none() {
            merged.tpd = other.tpd;
        }
        if merged.steering_vectors.is_none() {
            merged.steering_vectors = other.steering_vectors.clone();
        }

        // Merge extra_parameters
        if let Some(other_extras) = &other.extra_parameters {
            let mut extras = merged.extra_parameters.unwrap_or_default();
            for (k, v) in other_extras {
                // with this one (merged) taking precedence, only insert if not already present
                extras.entry(k.clone()).or_insert_with(|| v.clone());
            }
            merged.extra_parameters = Some(extras);
        }

        merged
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
    #[serde(default)]
    pub api_key: Option<String>,
    /// API base endpoint.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Protocol used for dispatch.
    pub protocol: ModelProvider,
    /// UUID for multi-tier provider identification.
    #[serde(default)]
    pub external_id: Option<String>,
    /// Arbitrary per-provider headers (e.g., for Inception/OpenRouter).
    #[serde(default)]
    pub custom_headers: Option<std::collections::HashMap<String, String>>,
    /// Default model configuration for this provider.
    #[serde(default)]
    pub default_config: Option<ModelConfig>,
    /// Whether this specific local endpoint supports Activation Addition
    #[serde(default, alias = "supportsSteeringVectors")]
    pub supports_steering_vectors: bool,
    /// Suggested model for audio transcription (optional).
    #[serde(default)]
    pub audio_model: Option<String>,
}

impl Validatable for ProviderConfig {
    /// Validates the provider configuration for production readiness.
    fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }

        if let Some(url) = &self.base_url {
            if !url.trim().is_empty() && !url.starts_with("http") {
                return Err(format!(
                    "Invalid base_url: '{}'. Must start with http:// or https://",
                    url
                ));
            }
        }

        Ok(())
    }
}

/// Definition of a specific AI model provided by a specific service.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelEntry {
    /// Unique model slug.
    pub id: String,
    /// Friendly display name.
    pub name: String,
    /// Parent provider ID.
    pub provider_id: String,
    /// Friendly provider name (Frontend Parity / Protocol).
    #[serde(default)]
    pub provider: Option<ModelProvider>,
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
    pub modality: Modality,
    /// Detailed functional capabilities (IMR-01).
    #[serde(default)]
    pub capabilities: ModelCapabilities,
}

impl Validatable for ModelEntry {
    /// Validates the model entry for swarm consistency.
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Model name cannot be empty".to_string());
        }

        if self.provider_id.trim().is_empty() {
            return Err("Model must be assigned to a Provider ID".to_string());
        }

        Ok(())
    }
}

/// Persistent state representation of an Agent.
#[derive(Debug, Clone, Default)]
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
    pub model_id: Option<String>,

    /// Default model config.
    pub model: ModelConfig,

    /// Auxiliary model ID for slot 2.
    pub model_2: Option<String>,
    /// Auxiliary model ID for slot 3.
    pub model_3: Option<String>,

    /// Model configuration for slot 2.
    pub model_config2: Option<ModelConfig>,
    /// Model configuration for slot 3.
    pub model_config3: Option<ModelConfig>,

    /// The index of the currently active model slot.
    pub active_model_slot: Option<i32>,

    /// Snapshot of the currently active mission (if any).
    pub active_mission: Option<serde_json::Value>,

    /// Current session status.
    /// @state: (idle | running | throttled | failed)
    pub status: String,

    /// Description of the task currently being executed, if any.
    pub current_task: Option<String>,

    /// Historical token count.
    pub tokens_used: u32,
    /// Detailed usage for the last turn.
    pub token_usage: TokenUsage,

    pub skills: Vec<String>,
    pub workflows: Vec<String>,
    pub mcp_tools: Vec<String>,

    pub skill_manifest: Option<crate::agent::skill_manifest::SkillManifest>,

    pub metadata: std::collections::HashMap<String, serde_json::Value>,

    pub theme_color: Option<String>,

    pub budget_usd: f64,
    pub cost_usd: f64,

    pub voice_id: Option<String>,
    pub voice_engine: Option<String>,

    pub category: String,

    pub failure_count: u32,

    pub last_failure_at: Option<chrono::DateTime<chrono::Utc>>,

    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    pub heartbeat_at: Option<chrono::DateTime<chrono::Utc>>,

    pub requires_oversight: bool,

    pub working_memory: serde_json::Value,
    pub connector_configs: Vec<ConnectorConfig>,
}

// Internal helper struct used for clean Serialization.
// Ensures that we can control the JSON schema for the frontend independently
// of the internal EngineAgent memory layout.
#[derive(Serialize)]
struct AgentResponse<'a> {
    id: &'a str,
    name: &'a str,
    role: &'a str,
    department: &'a str,
    description: &'a str,
    model_id: &'a Option<String>,
    model: &'a str,
    model_config: &'a ModelConfig,
    model_2: &'a Option<String>,
    model_config2: &'a Option<ModelConfig>,
    model_config3: &'a Option<ModelConfig>,
    active_model_slot: &'a Option<i32>,
    active_mission: &'a Option<serde_json::Value>,
    status: &'a str,
    current_task: &'a Option<String>,
    tokens_used: u32,
    token_usage: &'a TokenUsage,
    skills: &'a [String],
    workflows: &'a [String],
    mcp_tools: &'a [String],
    skill_manifest: &'a Option<crate::agent::skill_manifest::SkillManifest>,
    metadata: &'a HashMap<String, serde_json::Value>,
    theme_color: &'a Option<String>,
    budget_usd: f64,
    cost_usd: f64,
    voice_id: &'a Option<String>,
    voice_engine: &'a Option<String>,
    category: &'a str,
    failure_count: u32,
    last_failure_at: &'a Option<DateTime<Utc>>,
    created_at: &'a Option<DateTime<Utc>>,
    last_pulse: &'a Option<DateTime<Utc>>,
    requires_oversight: bool,
    working_memory: &'a serde_json::Value,
    connector_configs: &'a [ConnectorConfig],
}

impl Serialize for EngineAgent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let model_name = if self.model.model_id.trim().is_empty() {
            self.model_id.as_deref().unwrap_or_default()
        } else {
            self.model.model_id.as_str()
        };

        let response = AgentResponse {
            id: &self.id,
            name: &self.name,
            role: &self.role,
            department: &self.department,
            description: &self.description,
            model_id: &self.model_id,
            model: model_name,
            model_config: &self.model,
            model_2: &self.model_2,
            model_config2: &self.model_config2,
            model_config3: &self.model_config3,
            active_model_slot: &self.active_model_slot,
            active_mission: &self.active_mission,
            status: &self.status,
            current_task: &self.current_task,
            tokens_used: self.tokens_used,
            token_usage: &self.token_usage,
            skills: &self.skills,
            workflows: &self.workflows,
            mcp_tools: &self.mcp_tools,
            skill_manifest: &self.skill_manifest,
            metadata: &self.metadata,
            theme_color: &self.theme_color,
            budget_usd: self.budget_usd,
            cost_usd: self.cost_usd,
            voice_id: &self.voice_id,
            voice_engine: &self.voice_engine,
            category: &self.category,
            failure_count: self.failure_count,
            last_failure_at: &self.last_failure_at,
            created_at: &self.created_at,
            last_pulse: &self.heartbeat_at,
            requires_oversight: self.requires_oversight,
            working_memory: &self.working_memory,
            connector_configs: &self.connector_configs,
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
    #[serde(default, alias = "modelId")]
    model_id: Option<String>,
    #[serde(default)]
    model: Option<EngineAgentModelInput>,
    #[serde(default, alias = "model_config", alias = "modelConfig")]
    model_config: Option<ModelConfig>,
    #[serde(default, alias = "model2")]
    model_2: Option<String>,
    #[serde(default, alias = "model3")]
    model_3: Option<String>,
    #[serde(default, alias = "model_config2", alias = "modelConfig2")]
    model_config2: Option<ModelConfig>,
    #[serde(default, alias = "model_config3", alias = "modelConfig3")]
    model_config3: Option<ModelConfig>,
    #[serde(default, alias = "activeModelSlot")]
    active_model_slot: Option<i32>,
    #[serde(default, alias = "activeMission")]
    active_mission: Option<serde_json::Value>,
    status: String,
    #[serde(default, alias = "currentTask")]
    current_task: Option<String>,
    #[serde(default, alias = "tokensUsed")]
    tokens_used: u32,
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
}

impl<'de> Deserialize<'de> for EngineAgent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = EngineAgentWire::deserialize(deserializer)?;

        let model = match (wire.model_config, wire.model) {
            (Some(config), _) => config,
            (None, Some(EngineAgentModelInput::Config(config))) => *config,
            (None, Some(EngineAgentModelInput::ModelId(model_id))) => ModelConfig {
                provider: ModelProvider::from_str(&model_id).unwrap_or(ModelProvider::Openai),
                model_id,
                ..ModelConfig::default()
            },
            (None, None) => ModelConfig::default(),
        };

        let model_id = match wire.model_id {
            Some(model_id) => {
                if model.model_id.is_empty() {
                    // model.model_id = model_id.clone();
                }
                Some(model_id)
            }
            None if !model.model_id.is_empty() => Some(model.model_id.clone()),
            None => None,
        };

        Ok(Self {
            id: wire.id,
            name: wire.name,
            role: wire.role,
            department: wire.department,
            description: wire.description,
            model_id,
            model,
            model_2: wire.model_2,
            model_3: wire.model_3,
            model_config2: wire.model_config2,
            model_config3: wire.model_config3,
            active_model_slot: wire.active_model_slot,
            active_mission: wire.active_mission,
            status: wire.status,
            current_task: wire.current_task,
            tokens_used: wire.tokens_used,
            token_usage: wire.token_usage,
            skills: wire.skills,
            workflows: wire.workflows,
            mcp_tools: wire.mcp_tools,
            skill_manifest: wire.skill_manifest,
            metadata: wire.metadata,
            theme_color: wire.theme_color,
            budget_usd: wire.budget_usd,
            cost_usd: wire.cost_usd,
            voice_id: wire.voice_id,
            voice_engine: wire.voice_engine,
            category: wire.category,
            failure_count: wire.failure_count,
            last_failure_at: wire.last_failure_at,
            created_at: wire.created_at,
            heartbeat_at: wire.last_pulse.or(wire.heartbeat_at),
            requires_oversight: wire.requires_oversight,
            working_memory: wire.working_memory,
            connector_configs: wire.connector_configs,
        })
    }
}

fn default_category() -> String {
    "user".to_string()
}

impl EngineAgent {
    /// Returns true if the agent is currently in a suspended state.
    #[allow(dead_code)]
    pub fn is_suspended(&self) -> bool {
        self.status == "suspended"
    }

    #[allow(dead_code)]
    pub fn resolve_provider_context(
        &self,
        base_dir: std::path::PathBuf,
    ) -> crate::agent::runner::RunContext {
        let workspace_root = base_dir.join("data/workspaces/default");
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
            provider_name: self.model.provider.to_string(),
            workspace_root: workspace_root.clone(),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(workspace_root),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: self.working_memory.clone(),
            base_dir,
            summarized_history: None,
            structured_output: false,
            backlog: None,
            primary_goal: None,
            budget_usd: self.budget_usd,
            current_cost_usd: self.cost_usd,
        }
    }
}

/// Request payload for starting an autonomous task.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskPayload {
    /// The mission objective message.
    pub message: String,
    /// Targeted cluster/workspace.
    pub cluster_id: Option<String>,
    /// Department designation.
    pub department: Option<String>,
    /// Model provider override.
    pub provider: Option<ModelProvider>,
    /// Model ID override.
    pub model_id: Option<String>,
    /// Secret key override.
    pub api_key: Option<String>,
    /// Base URL override.
    pub base_url: Option<String>,
    /// Requests Per Minute rate limit override.
    pub rpm: Option<u32>,
    /// Tokens Per Minute rate limit override.
    pub tpm: Option<u32>,
    /// Requests Per Day rate limit override.
    pub rpd: Option<u32>,
    /// Tokens Per Day rate limit override.
    pub tpd: Option<u32>,
    /// Max USD budget for this task.
    pub budget_usd: Option<f64>,
    /// Swarm recursion depth.
    pub swarm_depth: Option<u32>,
    /// Ancestry path of nodes leading to this recruitment.
    pub swarm_lineage: Option<Vec<String>>,
    /// Multi-tier provider identifier.
    pub external_id: Option<String>,
    /// If true, disables mutation/recruitment.
    pub safe_mode: Option<bool>,
    /// If true, enables iterative analysis.
    pub analysis: Option<bool>,
    pub traceparent: Option<String>,
    pub user_id: Option<String>,
    /// Relevant file paths for context propagation.
    #[serde(default)]
    pub context_files: Option<Vec<String>>,
    /// Summary of recent findings from parent agent.
    #[serde(default)]
    pub recent_findings: Option<String>,
    /// If true, enforces JSON mode for the provider response.
    #[serde(default)]
    pub structured_output: Option<bool>,
    /// The overarching mission objective, shared across the swarm.
    #[serde(default, alias = "primaryGoal")]
    pub primary_goal: Option<String>,
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
    /// Model provider identifier (enum).
    pub provider: Option<ModelProvider>,
    /// Primary model ID.
    pub model_id: Option<String>,
    /// Model ID for slot 2.
    #[serde(default, alias = "model2")]
    pub model_2: Option<String>,
    /// Model ID for slot 3.
    #[serde(default, alias = "model3")]
    pub model_3: Option<String>,
    /// Secret key override.
    pub api_key: Option<String>,
    /// Global personality instructions.
    pub system_prompt: Option<String>,
    /// LLM sampling temperature.
    pub temperature: Option<f32>,
    /// Optional service endpoint.
    pub base_url: Option<String>,
    /// Hexadecimal theme color.
    pub theme_color: Option<String>,
    /// Maximum USD budget.
    pub budget_usd: Option<f64>,
    /// Provider-specific external ID.
    pub external_id: Option<String>,
    /// Enabled skill names.
    pub skills: Option<Vec<String>>,
    /// Enabled workflow names.
    pub workflows: Option<Vec<String>>,
    /// Enabled MCP tool names.
    pub mcp_tools: Option<Vec<String>>,
    /// Active slot index.
    pub active_model_slot: Option<i32>,
    /// Inline config for slot 2.
    pub model_config2: Option<ModelConfig>,
    /// Inline config for slot 3.
    pub model_config3: Option<ModelConfig>,
    /// ID for TTS synthesis.
    pub voice_id: Option<String>,
    /// Logic engine for TTS.
    pub voice_engine: Option<String>,
    /// Optimization: Categorize agent for UI filtering.
    pub category: Option<String>,
    /// Persistence: Whether the agent requires human oversight.
    pub requires_oversight: Option<bool>,
    /// Connector source configurations (Phase 2).
    pub connector_configs: Option<Vec<ConnectorConfig>>,
    /// Telemetry: External input token count.
    pub input_tokens: Option<u32>,
    /// Telemetry: External output token count.
    pub output_tokens: Option<u32>,
    /// Telemetry: Cumulative total tokens.
    pub total_tokens: Option<u32>,
    /// Telemetry: Legacy aggregate token count.
    #[serde(alias = "tokensUsed")]
    pub tokens_used: Option<u32>,
    /// Authoritative creation timestamp override.
    #[serde(alias = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Heartbeat/last pulse timestamp override.
    #[serde(alias = "lastPulse", alias = "heartbeatAt")]
    pub last_pulse: Option<chrono::DateTime<chrono::Utc>>,
    /// Task description currently shown in the UI.
    #[serde(alias = "currentTask")]
    pub current_task: Option<String>,
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
    Hook,
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
    pub tool_call: Option<ToolCall>,
    /// Proposed architectural change (if applicable).
    #[serde(alias = "capability_proposal")]
    pub skill_proposal: Option<SkillProposal>,
    /// Current decision state ("pending", "approved", "rejected").
    pub status: String,
    /// Formal creation timestamp.
    pub created_at: String,
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

// Types removed during zero-warning cleanup.

/// A unified graph element for the frontend visualizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub r#type: String, // "agent", "mission", "resource"
    pub status: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub metadata: serde_json::Value,
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

        assert_eq!(agent.skills, vec!["coding"]);
        assert_eq!(agent.workflows, vec!["deploy"]);
        assert!(agent.mcp_tools.is_empty());
        assert_eq!(agent.metadata.get("key").unwrap(), &json!("value"));
    }

    #[test]
    fn test_model_config_merge() {
        let mut base_extras = std::collections::HashMap::new();
        base_extras.insert("json_mode".to_string(), json!(true));
        base_extras.insert("seed".to_string(), json!(42));

        let base = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "gpt-4".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            extra_parameters: Some(base_extras),
            ..Default::default()
        };

        let mut override_extras = std::collections::HashMap::new();
        override_extras.insert("seed".to_string(), json!(123)); // Should take precedence
        override_extras.insert("thinking".to_string(), json!(true)); // Should be added

        let overrides = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "gpt-4".to_string(),
            temperature: Some(0.0), // Should take precedence
            extra_parameters: Some(override_extras),
            ..Default::default()
        };

        let merged = overrides.merge(&base);

        assert_eq!(merged.temperature, Some(0.0));
        assert_eq!(merged.max_tokens, Some(1000)); // From base

        let extras = merged.extra_parameters.unwrap();
        assert_eq!(extras.get("json_mode").unwrap(), &json!(true)); // From base
        assert_eq!(extras.get("seed").unwrap(), &json!(123)); // From overrides
        assert_eq!(extras.get("thinking").unwrap(), &json!(true)); // From overrides
    }
}
