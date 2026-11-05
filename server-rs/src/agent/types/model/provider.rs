//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / provider
//! - **Primary Entrypoints**: `ModelProvider`, `ProviderStatus`, `Modality`, `ProviderConfig`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::config::{ModelConfig, Validatable};
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

    /// Lenient alias lookup (`claude` -> Anthropic, `grok` -> Xai, ...).
    /// Returns `None` for unknown providers. Also available via `str::parse`.
    pub fn from_str(s: &str) -> Option<Self> {
        Self::parse_alias(s)
    }

    fn parse_alias(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("openai") || trimmed.eq_ignore_ascii_case("open-ai") {
            Some(Self::Openai)
        } else if trimmed.eq_ignore_ascii_case("anthropic")
            || trimmed.eq_ignore_ascii_case("claude")
            || trimmed.eq_ignore_ascii_case("claude-3")
        {
            Some(Self::Anthropic)
        } else if trimmed.eq_ignore_ascii_case("google")
            || trimmed.eq_ignore_ascii_case("gemini")
            || trimmed.eq_ignore_ascii_case("google-ai-studio")
            || trimmed.eq_ignore_ascii_case("google-vertex")
        {
            Some(Self::Google)
        } else if trimmed.eq_ignore_ascii_case("ollama") {
            Some(Self::Ollama)
        } else if trimmed.eq_ignore_ascii_case("groq") {
            Some(Self::Groq)
        } else if trimmed.eq_ignore_ascii_case("mistral")
            || trimmed.eq_ignore_ascii_case("mistral-ai")
            || trimmed.eq_ignore_ascii_case("mistralai")
        {
            Some(Self::Mistral)
        } else if trimmed.eq_ignore_ascii_case("perplexity") || trimmed.eq_ignore_ascii_case("pplx")
        {
            Some(Self::Perplexity)
        } else if trimmed.eq_ignore_ascii_case("fireworks")
            || trimmed.eq_ignore_ascii_case("fireworks-ai")
        {
            Some(Self::Fireworks)
        } else if trimmed.eq_ignore_ascii_case("together")
            || trimmed.eq_ignore_ascii_case("together-ai")
        {
            Some(Self::Together)
        } else if trimmed.eq_ignore_ascii_case("deepseek")
            || trimmed.eq_ignore_ascii_case("deep-seek")
        {
            Some(Self::Deepseek)
        } else if trimmed.eq_ignore_ascii_case("xai")
            || trimmed.eq_ignore_ascii_case("grok")
            || trimmed.eq_ignore_ascii_case("x-ai")
        {
            Some(Self::Xai)
        } else if trimmed.eq_ignore_ascii_case("inception")
            || trimmed.eq_ignore_ascii_case("mercury")
        {
            Some(Self::Inception)
        } else if trimmed.eq_ignore_ascii_case("openrouter")
            || trimmed.eq_ignore_ascii_case("open-router")
        {
            Some(Self::Openrouter)
        } else if trimmed.eq_ignore_ascii_case("cerebras") {
            Some(Self::Cerebras)
        } else if trimmed.eq_ignore_ascii_case("sambanova")
            || trimmed.eq_ignore_ascii_case("samba-nova")
        {
            Some(Self::Sambanova)
        } else if trimmed.eq_ignore_ascii_case("ollama-cloud")
            || trimmed.eq_ignore_ascii_case("ollama_cloud")
            || trimmed.eq_ignore_ascii_case("ollamacloud")
        {
            Some(Self::OllamaCloud)
        } else {
            None
        }
    }

    /// Infer provider from a model identifier string using keyword heuristics.
    /// Order matters: more specific patterns (e.g. "ollama-cloud") must be checked
    /// before broader ones (e.g. "ollama") to avoid false matches. OpenAI is checked
    /// last so vendor-prefixed routes (`openrouter/openai/gpt-4o`) keep their router.
    pub fn from_model_id(model_id: &str) -> Option<Self> {
        let id = model_id.trim().to_lowercase();
        // Ollama cloud tags look like `gpt-oss:120b-cloud`; anchor on the tag suffix so
        // ids such as `google-cloud-vertex/...` are not misclassified.
        let is_ollama_cloud_tag =
            id.contains("ollama-cloud") || id.ends_with("-cloud") || id.ends_with(":cloud");

        if is_ollama_cloud_tag || id.contains("gemma4") || id.contains("gemma-4") {
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
        } else if id.contains("openrouter") || id.contains("stealth") {
            Some(Self::Openrouter)
        } else if is_openai_model_id(&id) {
            Some(Self::Openai)
        } else {
            None
        }
    }
}

/// Matches first-party OpenAI ids: `gpt-*`, `chatgpt-*`, `o1`/`o3`/`o4` (+ variants),
/// and `openai/` routes. `gpt-oss` is excluded: it is an open-weight family served by
/// many hosts (Ollama, Groq, ...), so it carries no provider signal.
fn is_openai_model_id(id: &str) -> bool {
    let base = id.strip_prefix("openai/").unwrap_or(id);
    if base.starts_with("gpt-oss") {
        return false;
    }
    let is_o_series = ["o1", "o3", "o4"].iter().any(|p| {
        base.strip_prefix(p)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
    });
    id.starts_with("openai/")
        || base.starts_with("gpt-")
        || base.starts_with("chatgpt-")
        || is_o_series
}

impl std::str::FromStr for ModelProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_alias(s).ok_or_else(|| format!("unknown model provider: '{}'", s.trim()))
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
            let trimmed = url.trim();
            if !trimmed.is_empty() {
                crate::security::ssrf_guard::validate_url_security(trimmed)
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
