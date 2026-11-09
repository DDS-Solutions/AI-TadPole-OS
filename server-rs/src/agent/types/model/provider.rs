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

const ALIAS_TABLE: &[(&[&str], ModelProvider)] = &[
    (&["openai", "open-ai"], ModelProvider::Openai),
    (
        &["anthropic", "claude", "claude-3"],
        ModelProvider::Anthropic,
    ),
    (
        &["google", "gemini", "google-ai-studio", "google-vertex"],
        ModelProvider::Google,
    ),
    (&["ollama"], ModelProvider::Ollama),
    (&["groq"], ModelProvider::Groq),
    (
        &["mistral", "mistral-ai", "mistralai"],
        ModelProvider::Mistral,
    ),
    (&["perplexity", "pplx"], ModelProvider::Perplexity),
    (&["fireworks", "fireworks-ai"], ModelProvider::Fireworks),
    (&["together", "together-ai"], ModelProvider::Together),
    (&["deepseek", "deep-seek"], ModelProvider::Deepseek),
    (&["xai", "grok", "x-ai"], ModelProvider::Xai),
    (&["inception", "mercury"], ModelProvider::Inception),
    (&["openrouter", "open-router"], ModelProvider::Openrouter),
    (&["cerebras"], ModelProvider::Cerebras),
    (&["sambanova", "samba-nova"], ModelProvider::Sambanova),
    (
        &["ollama-cloud", "ollama_cloud", "ollamacloud"],
        ModelProvider::OllamaCloud,
    ),
];

const KEYWORD_TABLE: &[(&[&str], ModelProvider)] = &[
    (&["gemma4", "gemma-4"], ModelProvider::OllamaCloud),
    (&["gemini"], ModelProvider::Gemini),
    (&["claude", "anthropic"], ModelProvider::Anthropic),
    (&["ollama", "phi", "gemma"], ModelProvider::Ollama),
    (&["deepseek"], ModelProvider::Deepseek),
    (&["grok"], ModelProvider::Xai),
    (&["mistral"], ModelProvider::Mistral),
    (&["openrouter", "stealth"], ModelProvider::Openrouter),
];

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
        ALIAS_TABLE
            .iter()
            .find(|(aliases, _)| aliases.iter().any(|a| trimmed.eq_ignore_ascii_case(a)))
            .map(|(_, provider)| *provider)
    }

    /// Infer provider from a model identifier string using keyword heuristics.
    /// Order matters: more specific patterns (e.g. "ollama-cloud") must be checked
    /// before broader ones (e.g. "ollama") to avoid false matches. OpenAI is checked
    /// last so vendor-prefixed routes (`openrouter/openai/gpt-4o`) keep their router.
    pub fn from_model_id(model_id: &str) -> Option<Self> {
        let id = model_id.trim().to_lowercase();
        let is_cloud =
            id.contains("ollama-cloud") || id.ends_with("-cloud") || id.ends_with(":cloud");
        if is_cloud {
            return Some(Self::OllamaCloud);
        }
        if let Some((_, provider)) = KEYWORD_TABLE
            .iter()
            .find(|(keywords, _)| keywords.iter().any(|kw| id.contains(kw)))
        {
            return Some(*provider);
        }
        is_openai_model_id(&id).then_some(Self::Openai)
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

#[derive(Clone, Serialize, Deserialize, specta::Type)]
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

impl std::fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let redacted_key = self.api_key.as_ref().map(|_| "<REDACTED>");
        let header_keys = self
            .custom_headers
            .as_ref()
            .map(|h| h.keys().collect::<Vec<_>>());
        f.debug_struct("ProviderConfig")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("icon", &self.icon)
            .field("api_key", &redacted_key)
            .field("base_url", &self.base_url)
            .field("protocol", &self.protocol)
            .field("external_id", &self.external_id)
            .field("custom_headers", &header_keys)
            .field("default_config", &self.default_config)
            .field("supports_steering_vectors", &self.supports_steering_vectors)
            .field("audio_model", &self.audio_model)
            .finish()
    }
}

fn scrub_opt_secret(secret: &mut Option<String>) {
    use zeroize::Zeroize;
    if let Some(ref mut s) = secret {
        s.zeroize();
    }
    *secret = None;
}

impl ProviderConfig {
    /// Explicitly zeroes in-memory API keys and custom header values using `zeroize::Zeroize`.
    pub fn scrub_credentials(&mut self) {
        use zeroize::Zeroize;
        scrub_opt_secret(&mut self.api_key);
        if let Some(ref mut headers) = self.custom_headers {
            headers.values_mut().for_each(|v| v.zeroize());
        }
        if let Some(ref mut default_cfg) = self.default_config {
            scrub_opt_secret(&mut default_cfg.api_key);
        }
    }
}

impl Drop for ProviderConfig {
    fn drop(&mut self) {
        self.scrub_credentials();
    }
}

impl Validatable for ProviderConfig {
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }
        let Some(url) = self.base_url.as_deref() else {
            return Ok(());
        };
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        crate::security::ssrf_guard::validate_url_security(trimmed).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_scrub_and_debug_redaction() {
        let mut cfg = ProviderConfig {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            icon: None,
            api_key: Some("sk-super-secret-live-key-12345".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            protocol: ModelProvider::Openai,
            external_id: None,
            custom_headers: None,
            default_config: None,
            supports_steering_vectors: false,
            audio_model: None,
        };

        let debug_str = format!("{cfg:?}");
        assert!(!debug_str.contains("sk-super-secret-live-key-12345"));
        assert!(debug_str.contains("<REDACTED>"));

        cfg.scrub_credentials();
        assert!(cfg.api_key.is_none());
    }
}
