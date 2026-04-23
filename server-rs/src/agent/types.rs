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

/// ### 📡 Protocol: ModelProvider
/// Defines the set of supported LLM backend protocols for the Tadpole OS engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, Default, specta::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    #[default]
    Openai,
    Anthropic,
    Google,
    Gemini, // Alias for Google
    Ollama,
    Groq,
    Mistral,
    Perplexity,
    Fireworks,
    Together,
    Deepseek,
    Xai,
    Inception,
    Openrouter,
    Cerebras,
    Sambanova,
}

impl ModelProvider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" | "open-ai" => Some(Self::Openai),
            "anthropic" | "claude" | "claude-3" => Some(Self::Anthropic),
            "google" | "gemini" | "google-ai-studio" | "google-vertex" => Some(Self::Google),
            "ollama" => Some(Self::Ollama),
            "groq" => Some(Self::Groq),
            "mistral" | "mistral-ai" | "mistralai" => Some(Self::Mistral),
            "perplexity" | "pplx" => Some(Self::Perplexity),
            "fireworks" | "fireworks-ai" => Some(Self::Fireworks),
            "together" | "together-ai" => Some(Self::Together),
            "deepseek" | "deep-seek" => Some(Self::Deepseek),
            "xai" | "grok" | "x-ai" => Some(Self::Xai),
            "inception" | "mercury" => Some(Self::Inception),
            "openrouter" | "open-router" => Some(Self::Openrouter),
            "cerebras" => Some(Self::Cerebras),
            "sambanova" | "samba-nova" => Some(Self::Sambanova),
            _ => None,
        }
    }
}

impl std::fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
            Self::Google | Self::Gemini => "google",
            Self::Ollama => "ollama",
            Self::Groq => "groq",
            Self::Mistral => "mistral",
            Self::Perplexity => "perplexity",
            Self::Fireworks => "fireworks",
            Self::Together => "together",
            Self::Deepseek => "deepseek",
            Self::Xai => "xai",
            Self::Inception => "inception",
            Self::Openrouter => "openrouter",
            Self::Cerebras => "cerebras",
            Self::Sambanova => "sambanova",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    #[default]
    Llm,
    Vision,
    Voice,
    Audio,
    Reasoning,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MissionStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Paused,
}

pub trait Validatable {
    fn validate(&self) -> Result<(), String>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct TokenUsage {
    #[serde(default, alias = "inputTokens")]
    pub input_tokens: u32,
    #[serde(default, alias = "outputTokens")]
    pub output_tokens: u32,
    #[serde(default, alias = "totalTokens")]
    pub total_tokens: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Default, specta::Type)]
pub struct SyncManifest {
    pub id: String,
    pub agent_id: String,
    pub source_type: String,
    pub source_uri: String,
    pub status: String,
    pub last_sync_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct ModelCapabilities {
    #[serde(default, alias = "supportsTools")]
    pub supports_tools: bool,
    #[serde(default, alias = "supportsVision")]
    pub supports_vision: bool,
    #[serde(default, alias = "supportsStructuredOutput")]
    pub supports_structured_output: bool,
    #[serde(default, alias = "supportsReasoning")]
    pub supports_reasoning: bool,
    #[serde(default, alias = "supportsHaltingTool")]
    pub supports_halting_tool: bool,
    #[serde(default, alias = "contextWindow")]
    pub context_window: u32,
    #[serde(default, alias = "maxOutputTokens")]
    pub max_output_tokens: u32,
}

#[derive(Debug, Clone)]
struct CapabilityPattern {
    slugs: &'static [&'static str],
    update: fn(&mut ModelCapabilities),
}

const CAPABILITY_PATTERNS: &[CapabilityPattern] = &[
    CapabilityPattern {
        slugs: &["phi-3", "phi3", "stable-code"],
        update: |c| c.supports_tools = false,
    },
    CapabilityPattern {
        slugs: &["vision", "-v", "lava", "gpt-4o", "claude-3", "gemini-1.5", "phi-3.5-vision", "pixtral"],
        update: |c| c.supports_vision = true,
    },
    CapabilityPattern {
        slugs: &["gpt-4o", "gpt-3.5-turbo", "gemini", "-pro", "-flash"],
        update: |c| c.supports_structured_output = true,
    },
    CapabilityPattern {
        slugs: &["reasoning", "-o1", "-o3", "deepseek-r1", "-r1"],
        update: |c| {
            c.supports_reasoning = true;
            c.supports_tools = false;
        },
    },
    // Granular Family Overrides
    CapabilityPattern {
        slugs: &["gpt-4o"],
        update: |c| {
            c.context_window = 128_000;
            c.max_output_tokens = 16_384;
            c.supports_tools = true;
            c.supports_vision = true;
            c.supports_structured_output = true;
        },
    },
    CapabilityPattern {
        slugs: &["gemini-1.5"],
        update: |c| {
            c.context_window = 1_000_000;
            c.max_output_tokens = 8_192;
            c.supports_vision = true;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["claude-3"],
        update: |c| {
            c.context_window = 200_000;
            c.max_output_tokens = 4_096;
            c.supports_vision = true;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["deepseek-r1"],
        update: |c| {
            c.context_window = 64_000;
            c.supports_reasoning = true;
            c.supports_vision = false;
        },
    },
    CapabilityPattern {
        slugs: &["llama-3", "llama3", "mistral"],
        update: |c| {
            c.context_window = 128_000;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["gemma-4", "gemma4"],
        update: |c| {
            c.supports_tools = true;
            c.context_window = 128_000;
        },
    },
];

impl ModelCapabilities {
    /// ### IMR-01: Intelligent Inference
    /// Automatically infers capabilities based on the Model ID.
    pub fn infer_from_id(model_id: &str) -> Self {
        let id = model_id.to_lowercase();
        let mut caps = Self {
            context_window: 32_768,
            max_output_tokens: 4_096,
            supports_tools: true,
            supports_halting_tool: true,
            ..Self::default()
        };

        for pattern in CAPABILITY_PATTERNS {
            if pattern.slugs.iter().any(|s| id.contains(s)) {
                (pattern.update)(&mut caps);
            }
        }

        // Final Edge Case logic not easily captured by slugs
        if id.contains("gemini-1.5-pro") {
            caps.context_window = 2_000_000;
        }
        if id.contains("gemma-4") && (id.contains("26b") || id.contains("moe") || id.contains("31b")) {
            caps.context_window = 256_000;
        }

        caps
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct ModelConfig {
    pub provider: ModelProvider,
    #[serde(default, alias = "modelId")]
    pub model_id: String,
    #[serde(default, alias = "apiKey")]
    pub api_key: Option<String>,
    #[serde(default, alias = "baseUrl")]
    pub base_url: Option<String>,
    #[serde(default, alias = "systemPrompt")]
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    #[serde(default, alias = "maxTokens")]
    pub max_tokens: Option<u32>,
    #[serde(default, alias = "externalId")]
    pub external_id: Option<String>,
    #[serde(default)]
    pub rpm: Option<u32>,
    #[serde(default)]
    pub rpd: Option<u32>,
    #[serde(default)]
    pub tpm: Option<u32>,
    #[serde(default)]
    pub tpd: Option<u32>,
    #[serde(default, alias = "skills")]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub workflows: Option<Vec<String>>,
    #[serde(default, alias = "mcpTools")]
    pub mcp_tools: Option<Vec<String>>,
    #[serde(default, alias = "steeringVectors")]
    pub steering_vectors: Option<Vec<String>>,
    #[serde(default, alias = "reasoningDepth")]
    pub reasoning_depth: Option<u32>,
    #[serde(default, alias = "actThreshold")]
    pub act_threshold: Option<f32>,
    #[serde(default, alias = "maxTurns")]
    pub max_turns: Option<u32>,
    #[serde(default, alias = "connectorConfigs")]
    pub connector_configs: Option<Vec<ConnectorConfig>>,
    #[serde(default, alias = "extraParameters")]
    pub extra_parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ConnectorConfig {
    pub r#type: String, 
    pub uri: String,    
}

impl ModelConfig {
    pub fn supports_native_tools(&self) -> bool {
        let mid = self.model_id.to_lowercase();
        if mid.contains("phi3") || mid.contains("phi-3") {
            return false;
        }
        true
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = self.clone();

        macro_rules! merge_option {
            ($field:ident) => {
                if merged.$field.is_none() {
                    merged.$field = other.$field.clone();
                }
            };
        }

        merge_option!(system_prompt);
        merge_option!(temperature);
        merge_option!(max_tokens);
        merge_option!(rpm);
        merge_option!(rpd);
        merge_option!(tpm);
        merge_option!(tpd);
        merge_option!(steering_vectors);
        merge_option!(reasoning_depth);
        merge_option!(act_threshold);

        if let Some(other_extras) = &other.extra_parameters {
            let mut extras = merged.extra_parameters.unwrap_or_default();
            for (k, v) in other_extras {
                extras.entry(k.clone()).or_insert_with(|| v.clone());
            }
            merged.extra_parameters = Some(extras);
        }

        merged
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    pub protocol: ModelProvider,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub custom_headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub default_config: Option<ModelConfig>,
    #[serde(default, alias = "supportsSteeringVectors")]
    pub supports_steering_vectors: bool,
    #[serde(default)]
    pub audio_model: Option<String>,
}

impl Validatable for ProviderConfig {
    fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }
        if let Some(url) = &self.base_url {
            if !url.trim().is_empty() && !url.starts_with("http") {
                return Err(format!("Invalid base_url: '{}'. Must start with http:// or https://", url));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    #[serde(default)]
    pub provider: Option<ModelProvider>,
    #[serde(default)]
    pub rpm: Option<u32>,
    #[serde(default)]
    pub tpm: Option<u32>,
    #[serde(default)]
    pub rpd: Option<u32>,
    #[serde(default)]
    pub tpd: Option<u32>,
    #[serde(default)]
    pub modality: Modality,
    #[serde(default)]
    pub capabilities: ModelCapabilities,
}

impl Validatable for ModelEntry {
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RoleBlueprint {
    pub id: String,
    pub name: String,
    pub department: String,
    pub description: String,
    #[serde(default)]
    pub skills: String, 
    #[serde(default)]
    pub workflows: String, 
    #[serde(default, alias = "mcpTools")]
    pub mcp_tools: String, 
    #[serde(default, alias = "requiresOversight")]
    pub requires_oversight: bool,
    #[serde(default, alias = "modelId")]
    pub model_id: Option<String>,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
}

impl Validatable for RoleBlueprint {
    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Blueprint ID cannot be empty".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("Blueprint name cannot be empty".to_string());
        }
        Ok(())
    }
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
    pub tokens_used: u32,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentHealth {
    pub status: String,
    pub failure_count: u32,
    pub last_failure_at: Option<chrono::DateTime<chrono::Utc>>,
    pub heartbeat_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentModels {
    pub model_id: Option<String>,
    pub model: ModelConfig,
    pub model_2: Option<String>,
    pub model_3: Option<String>,
    pub model_config2: Option<ModelConfig>,
    pub model_config3: Option<ModelConfig>,
    pub active_model_slot: Option<i32>,
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
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
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
    model_2: &'a Option<String>,
    model_config2: &'a Option<ModelConfig>,
    model_config3: &'a Option<ModelConfig>,
    active_model_slot: &'a Option<i32>,
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
                model_2: &self.models.model_2,
                model_config2: &self.models.model_config2,
                model_config3: &self.models.model_config3,
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
                model_2: wire.model_2,
                model_3: wire.model_3,
                model_config2: wire.model_config2,
                model_config3: wire.model_config3,
                active_model_slot: wire.active_model_slot,
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
    pub fn resolve_provider_context(&self, base_dir: std::path::PathBuf) -> crate::agent::runner::RunContext {
        let workspace_root = base_dir.join("data/workspaces/default");
        crate::agent::runner::RunContext {
            agent_id: self.identity.id.clone(),
            name: self.identity.name.clone(),
            role: self.identity.role.clone(),
            department: self.identity.department.clone(),
            description: self.identity.description.clone(),
            model_config: self.models.model.clone(),
            skills: self.capabilities.skills.clone(),
            workflows: self.capabilities.workflows.clone(),
            mcp_tools: self.capabilities.mcp_tools.clone(),
            mission_id: "system-internal".to_string(),
            depth: 0,
            lineage: vec![],
            provider_name: self.models.model.provider.to_string(),
            workspace_root: workspace_root.clone(),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(workspace_root),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: self.state.working_memory.clone(),
            base_dir,
            summarized_history: None,
            structured_output: false,
            backlog: None,
            primary_goal: None,
            budget_usd: self.economics.budget_usd,
            current_cost_usd: self.economics.cost_usd,
            reasoning_depth: self.models.model.reasoning_depth.unwrap_or(1),
            act_threshold: self.models.model.act_threshold.unwrap_or(0.95),
            max_turns: self.models.model.max_turns.unwrap_or(20),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct TaskPayload {
    pub message: String,
    pub cluster_id: Option<String>,
    pub department: Option<String>,
    pub provider: Option<ModelProvider>,
    pub model_id: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub rpm: Option<u32>,
    pub tpm: Option<u32>,
    pub rpd: Option<u32>,
    pub tpd: Option<u32>,
    pub budget_usd: Option<f64>,
    pub swarm_depth: Option<u32>,
    pub swarm_lineage: Option<Vec<String>>,
    pub external_id: Option<String>,
    pub safe_mode: Option<bool>,
    pub analysis: Option<bool>,
    pub traceparent: Option<String>,
    pub user_id: Option<String>,
    #[serde(default)]
    pub context_files: Option<Vec<String>>,
    #[serde(default)]
    pub recent_findings: Option<String>,
    #[serde(default)]
    pub structured_output: Option<bool>,
    #[serde(default, alias = "primaryGoal")]
    pub primary_goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct ToolCallAudit {
    pub id: String,
    pub mission_id: Option<String>,
    #[serde(rename = "agent_id")]
    pub agent_id: String,
    pub skill: String,
    pub params: serde_json::Value,
    pub department: String,
    pub description: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum SkillType {
    Skill,
    Workflow,
    Hook,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SkillProposal {
    pub r#type: SkillType,
    pub name: String,
    pub description: String,
    pub execution_command: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub content: Option<String>,
    pub full_instructions: Option<String>,
    pub negative_constraints: Option<Vec<String>>,
    pub verification_script: Option<String>,
    #[serde(default = "default_category")]
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OversightEntry {
    pub id: String,
    pub mission_id: Option<String>,
    pub tool_call: Option<ToolCallAudit>,
    #[serde(alias = "capability_proposal")]
    pub skill_proposal: Option<SkillProposal>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct Mission {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub status: MissionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub budget_usd: f64,
    pub cost_usd: f64,
    pub is_degraded: Option<bool>,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct MissionLog {
    pub id: String,
    pub mission_id: String,
    pub agent_id: String,
    pub source: String,
    pub text: String,
    pub severity: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Enhanced memory entry with scoring metadata for Advanced RAG.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntryDetailed {
    pub id: String,
    pub text: String,
    pub mission_id: String,
    pub timestamp: i64,
    /// Raw semantic distance from vector search.
    pub distance: f32,
    /// Final calculated Multi-Factor Score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<crate::types::rag_scoring::RagScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolDefinition {
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNode {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightDecision {
    pub decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub r#type: String,
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

pub use crate::agent::merge::AgentConfigUpdate;

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

        assert_eq!(agent.identity.id, "test-agent");
        assert_eq!(agent.economics.token_usage.total_tokens, 0);
        assert!(agent.capabilities.skills.is_empty());
        assert!(agent.capabilities.workflows.is_empty());
        assert!(agent.metadata.is_empty());
        assert_eq!(agent.economics.budget_usd, 10.0);
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

        assert_eq!(agent.capabilities.skills, vec!["coding"]);
        assert_eq!(agent.capabilities.workflows, vec!["deploy"]);
        assert!(agent.capabilities.mcp_tools.is_empty());
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

#[cfg(test)]
mod contract_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_agent_config_update_serialization_parity() {
        let update = AgentConfigUpdate {
            name: Some("Test Agent".to_string()),
            role: Some("Analyst".to_string()),
            department: Some("QA".to_string()),
            budget_usd: Some(500.0),
            metadata: Some(std::collections::HashMap::from([("key".to_string(), json!("value"))])),
            input_tokens: Some(100),
            ..Default::default()
        };

        let serialized = serde_json::to_value(&update).unwrap();
        
        // Assert camelCase serialization (as required by frontend mappers)
        assert_eq!(serialized["name"], "Test Agent");
        assert_eq!(serialized["budgetUsd"], 500.0);
        assert_eq!(serialized["inputTokens"], 100);
        assert!(serialized.get("metadata").is_some());
    }

    #[test]
    fn test_engine_agent_serialization_parity() {
        let agent = EngineAgent {
            identity: AgentIdentity {
                id: "agent-1".to_string(),
                name: "Test Agent".to_string(),
                ..AgentIdentity::default()
            },
            economics: AgentEconomics {
                budget_usd: 100.0,
                ..AgentEconomics::default()
            },
            ..EngineAgent::default()
        };

        let serialized = serde_json::to_value(&agent).unwrap();
        
        // Assert camelCase serialization
        assert_eq!(serialized["id"], "agent-1");
        assert_eq!(serialized["name"], "Test Agent");
        assert_eq!(serialized["budgetUsd"], 100.0);
    }
}

// Metadata: [types]

// Metadata: [types]
