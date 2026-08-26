//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / model
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ### 📡 Protocol: ModelProvider
/// Defines the set of supported LLM backend protocols for the Tadpole OS engine.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    sqlx::Type,
    Default,
    specta::Type,
)]
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
    #[serde(rename = "ollama-cloud")]
    #[sqlx(rename = "ollama-cloud")]
    OllamaCloud,
}

impl ModelProvider {
    pub fn default_env_key(&self) -> &'static str {
        match self {
            Self::Openai => "OPENAI_API_KEY",
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::Google | Self::Gemini => "GOOGLE_API_KEY",
            Self::Groq => "GROQ_API_KEY",
            Self::Xai => "XAI_API_KEY",
            Self::Openrouter => "OPENROUTER_API_KEY",
            Self::Mistral => "MISTRAL_API_KEY",
            Self::Perplexity => "PERPLEXITY_API_KEY",
            Self::Fireworks => "FIREWORKS_API_KEY",
            Self::Together => "TOGETHER_API_KEY",
            Self::Cerebras => "CEREBRAS_API_KEY",
            Self::Sambanova => "SAMBANOVA_API_KEY",
            Self::OllamaCloud => "OLLAMA_CLOUD_API_KEY",
            Self::Deepseek => "DEEPSEEK_API_KEY",
            Self::Inception => "INCEPTION_API_KEY",
            Self::Ollama => "OLLAMA_API_KEY",
        }
    }

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
            "ollama-cloud" | "ollama_cloud" | "ollamacloud" => Some(Self::OllamaCloud),
            _ => None,
        }
    }

    /// Infer provider from a model identifier string using keyword heuristics.
    /// Order matters: more specific patterns (e.g. "ollama-cloud") must be checked
    /// before broader ones (e.g. "ollama") to avoid false matches.
    pub fn from_model_id(model_id: &str) -> Option<Self> {
        let id = model_id.to_lowercase();
        // Cloud-hosted Ollama variants (must precede the generic "ollama" check)
        if id.contains("ollama-cloud") || id.contains("-cloud") || id.contains(":cloud") {
            Some(Self::OllamaCloud)
        } else if id.contains("gemini") {
            Some(Self::Gemini)
        } else if id.contains("claude") || id.contains("anthropic") {
            Some(Self::Anthropic)
        } else if id.contains("ollama") || id.contains("phi") || id.contains("gemma") {
            Some(Self::Ollama)
        } else if id.contains("deepseek") {
            Some(Self::Deepseek)
        } else if id.contains("grok") {
            Some(Self::Xai)
        } else if id.contains("mistral") {
            Some(Self::Mistral)
        } else if id.contains("stealth")
            || id.contains("openrouter")
            || id.contains("ox-alpha")
            || id.contains("oxalpha")
        {
            Some(Self::Openrouter)
        } else {
            None
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
            Self::OllamaCloud => "ollama-cloud",
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

/// ### 📡 Protocol: ProviderStatus
/// Represents the health state of an LLM provider.
/// - Green: Healthy, all models functional.
/// - Amber: Degraded, high failure rate or rate limited. Diverts to secondary.
/// - Red: Down, critical failures. Fails over to fallback or NullProvider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum ProviderStatus {
    Green,
    Amber,
    Red,
}

pub trait Validatable {
    fn validate(&self) -> Result<(), String>;
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
        slugs: &[
            "vision",
            "-v",
            "lava",
            "gpt-4o",
            "gpt-5",
            "claude-3",
            "claude-4",
            "claude-5",
            "gemini-1.5",
            "gemini-2.0",
            "gemini-3",
            "phi-3.5-vision",
            "pixtral",
        ],
        update: |c| c.supports_vision = true,
    },
    CapabilityPattern {
        slugs: &[
            "gpt-4o",
            "gpt-5",
            "gpt-3.5-turbo",
            "gemini",
            "-pro",
            "-flash",
        ],
        update: |c| c.supports_structured_output = true,
    },
    CapabilityPattern {
        slugs: &[
            "reasoning",
            "-o1",
            "-o3",
            "deepseek-r1",
            "-r1",
            "mai-thinking",
        ],
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
        slugs: &["gpt-5"],
        update: |c| {
            c.context_window = 256_000;
            c.max_output_tokens = 16_384;
            c.supports_tools = true;
            c.supports_vision = true;
            c.supports_structured_output = true;
        },
    },
    CapabilityPattern {
        slugs: &["gemini-1.5", "gemini-2.0"],
        update: |c| {
            c.context_window = 1_000_000;
            c.max_output_tokens = 8_192;
            c.supports_vision = true;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["gemini-3"],
        update: |c| {
            c.context_window = 2_000_000;
            c.max_output_tokens = 16_384;
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
        slugs: &["claude-4", "claude-5"],
        update: |c| {
            c.context_window = 500_000;
            c.max_output_tokens = 8_192;
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
        slugs: &["llama-3", "llama3", "llama-4", "llama4", "mistral"],
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
        if id.contains("gemma-4")
            && (id.contains("12b")
                || id.contains("26b")
                || id.contains("moe")
                || id.contains("31b"))
        {
            caps.context_window = 256_000;
        }

        caps
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ConnectorConfig {
    pub r#type: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
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
    pub extra_parameters: Option<HashMap<String, serde_json::Value>>,
}

impl ModelConfig {
    #[allow(dead_code)]
    pub fn is_openai_compatible(&self) -> bool {
        match self.provider {
            ModelProvider::Openai
            | ModelProvider::Groq
            | ModelProvider::Perplexity
            | ModelProvider::Together
            | ModelProvider::Fireworks
            | ModelProvider::Deepseek
            | ModelProvider::Xai
            | ModelProvider::Mistral
            | ModelProvider::Openrouter
            | ModelProvider::Cerebras
            | ModelProvider::Sambanova
            | ModelProvider::Inception => true,
            _ => {
                let model_lower = self.model_id.to_lowercase();
                let base_url_is_openai = self
                    .base_url
                    .as_ref()
                    .map(|url| url.contains("api.openai.com") || url.contains("/v1"))
                    .unwrap_or(false);

                base_url_is_openai
                    || model_lower.contains("gpt")
                    || model_lower.contains("o1-")
                    || model_lower.contains("o3-")
            }
        }
    }

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
    pub custom_headers: Option<HashMap<String, String>>,
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
                return Err(format!(
                    "Invalid base_url: '{}'. Must start with http:// or https://",
                    url
                ));
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
    #[serde(default)]
    pub provider_model_id: Option<String>,
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
