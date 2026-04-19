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
/// 
/// Each variant represents a unique dispatch logic (OpenAI-compatible, 
/// Gemini Multimodal, or specialized Local/LPU providers). 
/// 
/// ### 🧬 Logic: Protocol Bridging
/// While many providers (Groq, Together, Deepseek) claim OpenAI compatibility, 
/// the engine maintains distinct variants for each to allow for 
/// provider-specific "In-Flight Recovery" (hallucination correction) 
/// and specialized audio/vision encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    /// Standard OAI v1 Protocol (GPT-4o, etc.)
    #[default]
    Openai,
    /// Anthropic Messages API (Claude 3.x)
    Anthropic,
    /// Google Generative AI / Vertex AI (Gemini 1.5)
    Google,
    /// Gemini-specific protocol alias.
    Gemini, 
    /// Local inference bridge via Ollama (llama.cpp)
    Ollama,
    /// High-speed LPU inference via Groq
    Groq,
    /// Native Deepseek protocol (R1/v3)
    Deepseek,
    /// X.ai Grok protocol
    Xai,
    /// Inception/Mercury specialized local cluster
    Inception,
    /// Aggregator bridge for multi-model access
    Openrouter,
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

/// ### 👁️ Cognitive Modality
/// Supported physical/data modalities for AI models.
/// 
/// Most autonomous agents utilize `Llm` (Text) for reasoning, 
/// though the swarm utilizes `Vision` for screenshot analysis 
/// and `Voice` for multimodal sovereign interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    /// Traditional text-in, text-out reasoning.
    #[default]
    Llm,
    /// Image-aware multimodal processing.
    Vision,
    /// Real-time audio transcription and synthesis.
    Voice,
    /// Chain-of-thought specialized 'Reasoning' models (e.g., o1).
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

/// ### 📊 Telemetry: TokenUsage
/// Operational statistics for token consumption in an LLM turn.
/// Used for real-time financial auditing and budget enforcement (SEC-02).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Number of tokens in the prompt (Input context).
    #[serde(default, alias = "inputTokens")]
    pub input_tokens: u32,
    /// Number of tokens in the completion (Agent response).
    #[serde(default, alias = "outputTokens")]
    pub output_tokens: u32,
    /// Sum of input and output tokens (Total payload size).
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

/// ### 🧠 Cognitive Analysis: ModelCapabilities
/// Dynamic capabilities and constraints of an AI model inferred via 
/// the `CapabilityMatrix`. 
/// 
/// Part of the **Intelligent Model Registry (IMR-01)**. 
/// These flags determine whether the engine allows the agent to 
/// perform Vision-based tool use or structured JSON emission.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    /// True if the model supports native function calling (Tool Use).
    #[serde(default, alias = "supportsTools")]
    pub supports_tools: bool,
    /// True if the model supports JPG/PNG multimodal analysis.
    #[serde(default, alias = "supportsVision")]
    pub supports_vision: bool,
    /// True if the model supports enforced schema JSON mode.
    #[serde(default, alias = "supportsStructuredOutput")]
    pub supports_structured_output: bool,
    /// True if the model utilizes an internal chain-of-thought buffer (high latency).
    #[serde(default, alias = "supportsReasoning")]
    pub supports_reasoning: bool,
    /// Physical context limit in tokens (LMT-04 boundary).
    #[serde(default, alias = "contextWindow")]
    pub context_window: u32,
    /// Target completion token limit to prevent unbounded cost.
    #[serde(default, alias = "maxOutputTokens")]
    pub max_output_tokens: u32,
}

/// ### 🏗️ Core Architecture: Model Configuration
/// Unified configuration for an LLM model and its execution parameters.
/// Kept in sync with TS `ModelConfig` in `server/types.ts`.
/// 
/// ### 🧬 Orchestration Logic
/// This struct is the "Intent Manifest" sent to the `RunContext`. It 
/// encapsulates both financial boundaries (RPM/RPD) and capability 
/// directives (Steering Vectors, MCP Tools).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    /// The physical protocol (e.g., Groq vs Gemini).
    pub provider: ModelProvider,
    /// The specific vendor ID (e.g., "llama-3.1-405b").
    #[serde(default, alias = "modelId")]
    pub model_id: String,
    /// Vault-stored secret (SEC-02: Documented as NEVER leaked to logs).
    #[serde(default, alias = "apiKey")]
    pub api_key: Option<String>,
    /// Optional service endpoint for local air-gapped clusters.
    #[serde(default, alias = "baseUrl")]
    pub base_url: Option<String>,
    /// The core identity/behavioral directive.
    #[serde(default, alias = "systemPrompt")]
    pub system_prompt: Option<String>,
    /// Variance selector (0.0 to 1.0).
    pub temperature: Option<f32>,
    /// Hard limit on generation length.
    #[serde(default, alias = "maxTokens")]
    pub max_tokens: Option<u32>,
    /// Registry ID for multi-tier lookup.
    #[serde(default, alias = "externalId")]
    pub external_id: Option<String>,
    /// Throttle: Requests Per Minute.
    #[serde(default)]
    pub rpm: Option<u32>,
    /// Throttle: Requests Per Day.
    #[serde(default)]
    pub rpd: Option<u32>,
    /// Throttle: Tokens Per Minute.
    #[serde(default)]
    pub tpm: Option<u32>,
    /// Throttle: Tokens Per Day.
    #[serde(default)]
    pub tpd: Option<u32>,
    /// List of enabled capability extensions.
    #[serde(default, alias = "skills")]
    pub skills: Option<Vec<String>>,
    /// List of assigned deterministic mission templates.
    #[serde(default)]
    pub workflows: Option<Vec<String>>,
    /// List of active MCP tool connectors.
    #[serde(default, alias = "mcpTools")]
    pub mcp_tools: Option<Vec<String>>,
    /// Active OBLITERATUS Steering Vectors (Feature Suppressors).
    #[serde(default, alias = "steeringVectors")]
    pub steering_vectors: Option<Vec<String>>,
    /// Multi-source RAG ingestion configurations.
    #[serde(default, alias = "connectorConfigs")]
    pub connector_configs: Option<Vec<ConnectorConfig>>,
    /// Vendor-specific JSON baggage.
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

/// ### 🏗️ Core Architecture: Autonomous Agent
/// Persistent state representation of an Agent within the Tadpole OS registry.
/// 
/// ### 🛰️ Orchestration Note
/// This is the "Single Source of Truth" for an agent's identity, capabilities,
/// and current mission status. It is synchronized to the Dashboard at 10Hz
/// during active missions via the binary pulse engine.
#[derive(Debug, Clone, Default)]
pub struct EngineAgent {
    /// Unique UUID identifier (Permanent).
    pub id: String,
    /// Friendly display name (e.g., "Alpha-1").
    pub name: String,
    /// High-level functional role (e.g., "Full Stack Architect").
    pub role: String,
    /// Organizational department (e.g., "Engineering").
    pub department: String,
    /// Core objective and baseline behavioral details. Used in the system prompt.
    pub description: String,

    /// Primary model ID for inference (Slot 1).
    pub model_id: Option<String>,

    /// Detailed configuration for Slot 1 inference.
    pub model: ModelConfig,

    /// Model ID override for Slot 2 (Specialist).
    pub model_2: Option<String>,
    /// Model ID override for Slot 3 (Evaluator).
    pub model_3: Option<String>,

    /// Configuration for Slot 2 specialist logic.
    pub model_config2: Option<ModelConfig>,
    /// Configuration for Slot 3 evaluator logic.
    pub model_config3: Option<ModelConfig>,

    /// Index of the currently active model slot [1, 2, 3].
    pub active_model_slot: Option<i32>,

    /// Current mission context in raw JSON (Optimized for persistence).
    pub active_mission: Option<serde_json::Value>,

    /// Current operational status (idle | running | throttled | failed).
    pub status: String,

    /// High-level description of the immediate operational task.
    pub current_task: Option<String>,

    /// Lifetime token consumption count for this agent identity.
    pub tokens_used: u32,
    /// Usage statistics for the most recent reasoning turn.
    pub token_usage: TokenUsage,

    /// Set of enabled neural skills (Capability extensions).
    pub skills: Vec<String>,
    /// Set of assigned deterministic workflows.
    pub workflows: Vec<String>,
    /// Set of active Tool Hub (MCP) connections.
    pub mcp_tools: Vec<String>,

    /// Resolved manifest containing executable skill definitions.
    pub skill_manifest: Option<crate::agent::skill_manifest::SkillManifest>,

    /// Extensible property bag for non-schema persistent data.
    pub metadata: std::collections::HashMap<String, serde_json::Value>,

    /// Hex triplet for UI visualization (e.g., #00BCD4).
    pub theme_color: Option<String>,

    /// Monthly USD budget cap (Enforced during agent recruitment).
    pub budget_usd: f64,
    /// Total cost incurred in the current billing cycle.
    pub cost_usd: f64,

    /// ID of the voice assigned for synthesis.
    pub voice_id: Option<String>,
    /// Rendering engine for the voice (e.g., Groq, Azure, Browser).
    pub voice_engine: Option<String>,

    /// Operational category (SME | USER | SYSTEM).
    pub category: String,

    /// Number of sequential mission failures (triggers safety reset).
    pub failure_count: u32,

    /// Timestamp of the last fatal exception (SEC-04).
    pub last_failure_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Resource creation timestamp.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Last pulse heartbeat (used by the Swarm Reaper for cleanup).
    pub heartbeat_at: Option<chrono::DateTime<chrono::Utc>>,

    /// If true, requires human approval (Overlord) before tool execution.
    pub requires_oversight: bool,

    /// Short-term working memory buffer (JSON). Used for session continuity.
    pub working_memory: serde_json::Value,
    /// SME Data Connector source configurations for RAG.
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

/// ### 🏗️ Core Architecture: Task Orchestration
/// Request payload for starting an autonomous task or recruiting a sub-agent.
///
/// ### 🧬 Recruitment Propagation
/// When a parent agent recruits a sub-agent, the `swarm_depth` and `swarm_lineage`
/// are used to ensure that the mission stays within the mission-graph safety
/// limits (SEC-01) and traceparent links.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskPayload {
    /// The mission objective or user prompt message.
    pub message: String,
    /// Targeted cluster/workspace identifier (Local or Remote).
    pub cluster_id: Option<String>,
    /// Targeted department (used for specialized capability lookup).
    pub department: Option<String>,
    /// Model provider override for this specific branch of the swarm.
    pub provider: Option<ModelProvider>,
    /// Exact model ID override (e.g., "claude-3-5-sonnet").
    pub model_id: Option<String>,
    /// Vault-stored secret key override (Used for cross-user recruitment).
    pub api_key: Option<String>,
    /// Custom service URL override.
    pub base_url: Option<String>,
    /// Concurrency throttle (Requests Per Minute) override.
    pub rpm: Option<u32>,
    /// Throughput throttle (Tokens Per Minute) override.
    pub tpm: Option<u32>,
    /// Daily request limit override.
    pub rpd: Option<u32>,
    /// Daily token limit override.
    pub tpd: Option<u32>,
    /// Hard limit on USD cost for this task execution (Enforcement: FIN-01).
    pub budget_usd: Option<f64>,
    /// Swarm recursion depth (0 = Root / Human Initiated).
    pub swarm_depth: Option<u32>,
    /// Ordered ancestry of agent IDs participating in this swarm chain. 
    /// Used to prevent circular recruitment deadlocks and ensure trace integrity.
    pub swarm_lineage: Option<Vec<String>>,
    /// Multi-tier provider identifier for delegated authentication.
    pub external_id: Option<String>,
    /// Safety flag: If true, the agent cannot perform destructive tool operations (e.g., shell write).
    pub safe_mode: Option<bool>,
    /// Analysis flag: If true, enables iterative reasoning and deep-research protocols.
    pub analysis: Option<bool>,
    /// W3C TraceContext for distributed observability across the cluster.
    pub traceparent: Option<String>,
    /// Owning user identifier for final accountability.
    pub user_id: Option<String>,
    /// Relevant file paths for context propagation (Knowledge Bridge).
    #[serde(default)]
    pub context_files: Option<Vec<String>>,
    /// Summary findings from the parent agent (Memory Palace shortcut).
    #[serde(default)]
    pub recent_findings: Option<String>,
    /// Requirement for model response to strictly adhere to JSON schemas (IMR-01).
    #[serde(default)]
    pub structured_output: Option<bool>,
    /// The overarching mission objective, shared across all recruited child agents.
    #[serde(default, alias = "primaryGoal")]
    pub primary_goal: Option<String>,
}

/// ### 🏗️ Core Architecture: Configuration Delta
/// Delta payload for updating an agent's persistent configuration via the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigUpdate {
    /// New friendly display name.
    pub name: Option<String>,
    /// New high-level functional role.
    pub role: Option<String>,
    /// New organizational department.
    pub department: Option<String>,
    /// Model provider protocol identifier.
    pub provider: Option<ModelProvider>,
    /// Primary model ID for inference.
    pub model_id: Option<String>,
    /// Specialist model ID for Slot 2.
    #[serde(default, alias = "model2")]
    pub model_2: Option<String>,
    /// Evaluator model ID for Slot 3.
    #[serde(default, alias = "model3")]
    pub model_3: Option<String>,
    /// Vault secret key override.
    pub api_key: Option<String>,
    /// Core personality directive override.
    pub system_prompt: Option<String>,
    /// LLM sampling variance override.
    pub temperature: Option<f32>,
    /// Optional local or private service endpoint.
    pub base_url: Option<String>,
    /// UI theme color (hexadecimal string).
    pub theme_color: Option<String>,
    /// Physical USD budget cap for the billing cycle.
    pub budget_usd: Option<f64>,
    /// External provider identifier for delegated authentication.
    pub external_id: Option<String>,
    /// List of skill names to enable/disable.
    pub skills: Option<Vec<String>>,
    /// List of workflow names to enable/disable.
    pub workflows: Option<Vec<String>>,
    /// List of MCP tools to enable/disable.
    pub mcp_tools: Option<Vec<String>>,
    /// Currently active Slot index [1, 2, 3].
    pub active_model_slot: Option<i32>,
    /// Structured configuration for Slot 2.
    pub model_config2: Option<ModelConfig>,
    /// Structured configuration for Slot 3.
    pub model_config3: Option<ModelConfig>,
    /// Synthetic voice ID for TTS synthesis.
    pub voice_id: Option<String>,
    /// TTS rendering engine provider.
    pub voice_engine: Option<String>,
    /// UI categorization for dashboard layout.
    pub category: Option<String>,
    /// Governance: Whether the agent requires human-in-the-loop (HITL) approval.
    pub requires_oversight: Option<bool>,
    /// SME Data Connector source configurations for knowledge ingestion.
    pub connector_configs: Option<Vec<ConnectorConfig>>,
    /// Telemetry reset: Override input token count.
    pub input_tokens: Option<u32>,
    /// Telemetry reset: Override output token count.
    pub output_tokens: Option<u32>,
    /// Telemetry reset: Override cumulative token count.
    pub total_tokens: Option<u32>,
    /// Telemetry reset: Override legacy aggregate token count.
    #[serde(alias = "tokensUsed")]
    pub tokens_used: Option<u32>,
    /// Administrative: Creation timestamp override.
    #[serde(alias = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Administrative: Heartbeat update override.
    #[serde(alias = "lastPulse", alias = "heartbeatAt")]
    pub last_pulse: Option<chrono::DateTime<chrono::Utc>>,
    /// UI State: Current task summary shown on the visualizer.
    #[serde(alias = "currentTask")]
    pub current_task: Option<String>,
}

/// ### 🏗️ Core Architecture: Tool Execution
/// Record of a tool invocation request from an agent to the system bus.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCallAudit {
    /// Unique identifier for the specific call instance.
    pub id: String,
    /// Parent mission context ID.
    pub mission_id: Option<String>,
    /// UUID of the originating agent identity.
    #[serde(rename = "agent_id")]
    pub agent_id: String,
    /// Name of the skill/capability being requested (e.g., "list_dir").
    pub skill: String,
    /// JSON payload containing the function arguments.
    pub params: serde_json::Value,
    /// Organizational department of the actor agent.
    pub department: String,
    /// Natural language explanation of the tool's purpose in the reasoning chain.
    pub description: String,
    /// ISO 8601 temporal marker for when the call was issued.
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SkillType {
    Skill,
    Workflow,
    Hook,
}

/// ### 🏗️ Core Architecture: Capability Evolution
/// Request for a new system skill (active tool) or workflow (passive SOP).
/// Part of the Self-Evolving Swarm (SES-01) logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProposal {
    /// Functional type: Skill (Executable) | Workflow (Instructional).
    pub r#type: SkillType,
    /// Unique identifier for the proposed skill (snake_case).
    pub name: String,
    /// High-level meta-description for LLM discovery.
    pub description: String,
    /// The physical binary or script to execute when called (Draft).
    pub execution_command: Option<String>,
    /// JSON schema for parameter validation and LLM steering.
    pub schema: Option<serde_json::Value>,
    /// Markdown source for the Standard Operating Procedure (Workflows).
    pub content: Option<String>,
    /// Detailed functional requirements and edge-case handling logic.
    pub full_instructions: Option<String>,
    /// Negative constraints (e.g., "Do not use 'rm -rf'").
    pub negative_constraints: Option<Vec<String>>,
    /// Unit test or verification logic for the proposal.
    pub verification_script: Option<String>,
    /// Organizational category for dashboard grouping.
    #[serde(default = "default_category")]
    pub category: String,
}

/// ### 🏗️ Core Architecture: Oversight & Governance
/// Represents a pending decision point requiring human-in-the-loop (HITL) approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightEntry {
    /// Unique registry identifier for the oversight request.
    pub id: String,
    /// Parent mission ID linked to this decision point.
    pub mission_id: Option<String>,
    /// Reference to the specific tool call awaiting approval (if applicable).
    pub tool_call: Option<ToolCallAudit>,
    /// Reference to a proposed system-level capability change (if applicable).
    #[serde(alias = "capability_proposal")]
    pub skill_proposal: Option<SkillProposal>,
    /// Operational status: (pending | approved | rejected | expired).
    pub status: String,
    /// RFC 3339 timestamp of the request initialization.
    pub created_at: String,
}

/// ### 🏗️ Core Architecture: Mission State
/// Comprehensive record of a mission's active execution and persistent history.
/// Represents the unit of work assigned by a user to the swarm.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mission {
    /// Unique UUID identifier for the mission record in SQLite.
    pub id: String,
    /// UUID of the agent owning the primary recruitment (The Root Actor).
    pub agent_id: String,
    /// Human-readable title of the objective.
    pub title: String,
    /// Current execution status (Pending, Active, Completed, Failed).
    pub status: MissionStatus,
    /// Backend-authoritative creation timestamp (UTC).
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp of the most recent trace pulse or budget deduction.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Total soft-limit budget assigned for the entire swarm tree in USD.
    pub budget_usd: f64,
    /// Cumulative monetary cost currently incurred by all participants.
    pub cost_usd: f64,
    /// Recovery flag: True if the mission is continuing despite non-fatal system faults.
    pub is_degraded: Option<bool>,
    /// Lifecycle flag: True if the mission is pinned and explicitly excluded from the Swarm Reaper.
    pub is_pinned: bool,
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

/// ### 🏗️ Core Architecture: Swarm Intelligence Graph
/// Unified graph structure for the Frontend Visualizer (SwarmVisualizer component).
/// Powered by D3 force-directed simulation in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmGraph {
    /// Ordered set of active entities (Agents, Missions, Knowledge Clusters) in the graph.
    pub nodes: Vec<GraphNode>,
    /// Functional and sequential relationships (dependencies) between nodes.
    pub edges: Vec<GraphEdge>,
}

/// Represents a single active entity in the swarm's spatial workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// UUID of the resource or agent identity.
    pub id: String,
    /// Human-friendly display label used in the visualizer.
    pub label: String,
    /// Entity classification: (agent | mission | resource | cluster).
    pub r#type: String,
    /// Live operational state (Active, Idle, etc.).
    pub status: String,
    /// Extensible metadata bag for tooltips, icons, and theme coloring.
    pub metadata: serde_json::Value,
}

/// Represents a functional link between two swarm entities (Neural Connection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Unique identifier for the relationship instance.
    pub id: String,
    /// Originating node UUID (The Parent or Recruiter).
    pub source: String,
    /// Destination node UUID (The Child or Recruitee).
    pub target: String,
    /// Relationship description (e.g., "Assigned", "Observes", "Calls").
    pub label: String,
    /// Auxiliary data for edge styling (weight, directionality, and pulse state).
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
