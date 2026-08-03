//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Provider Resolution**: Instantiates specific provider clients and handles API key resolution.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing or malformed environment API keys, or invalid local/remote address routing.
//!

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::model_routing::{is_local_endpoint};
use crate::agent::null_provider::{NullProvider, NullReason};
use crate::agent::types::{ModelConfig, ModelProvider};
use super::ProviderVariant;

/// Resolves an API key: prefers the per-agent config override, then falls
/// back to the named environment variable.
pub(crate) fn resolve_api_key(config: &ModelConfig, env_var: &str) -> Option<String> {
    config
        .api_key
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .or_else(|| std::env::var(env_var).ok().filter(|s| !s.trim().is_empty()))
}

const PRIVACY_FALLBACK_MODEL: &str = "phi3.5-safe:latest";
const CLOUD_MODEL_MARKERS: &[&str] = &[
    "gpt",
    "claude",
    "gemini",
    "deepseek",
    "o1",
    "o3",
    "o4",
    "grok",
    "mistral",
    "mixtral",
    "pixtral",
    "llama-3.3-70b-versatile",
    "mixtral-8x7b-32768",
];

/// Dynamically queries local Ollama models to pick the first available model as fallback.
pub(crate) async fn resolve_privacy_fallback_model(client: &reqwest::Client, base_url: &str) -> String {
    let url = if base_url.ends_with("/v1") {
        format!("{}/models", base_url)
    } else if base_url.ends_with("/v1/") {
        format!("{}models", base_url)
    } else {
        format!("{}/v1/models", base_url)
    };

    #[derive(serde::Deserialize)]
    struct OllamaModel {
        id: String,
    }
    #[derive(serde::Deserialize)]
    struct OllamaModelsResponse {
        data: Vec<OllamaModel>,
    }

    if let Ok(resp) = client.get(&url).send().await {
        if resp.status().is_success() {
            if let Ok(models_list) = resp.json::<OllamaModelsResponse>().await {
                if let Some(first_model) = models_list.data.first() {
                    tracing::info!("🛡️ [Privacy Shield] Dynamically resolved local fallback model: '{}' from Ollama", first_model.id);
                    return first_model.id.clone();
                }
            }
        }
    }

    let api_tags_url = if let Some(stripped) = base_url.strip_suffix("/v1") {
        format!("{}/api/tags", stripped)
    } else if let Some(stripped) = base_url.strip_suffix("/v1/") {
        format!("{}/api/tags", stripped)
    } else {
        format!("{}/api/tags", base_url)
    };

    #[derive(serde::Deserialize)]
    struct LegacyOllamaModel {
        name: String,
    }
    #[derive(serde::Deserialize)]
    struct LegacyOllamaResponse {
        models: Vec<LegacyOllamaModel>,
    }

    if let Ok(resp) = client.get(&api_tags_url).send().await {
        if resp.status().is_success() {
            if let Ok(models_list) = resp.json::<LegacyOllamaResponse>().await {
                if let Some(first_model) = models_list.models.first() {
                    tracing::info!("🛡️ [Privacy Shield] Dynamically resolved local fallback model: '{}' from legacy Ollama tags API", first_model.name);
                    return first_model.name.clone();
                }
            }
        }
    }

    PRIVACY_FALLBACK_MODEL.to_string()
}

/// Helper function to resolve OpenAI-compatible providers without code duplication.
pub(crate) fn resolve_openai_provider(
    client: reqwest::Client,
    config: &ModelConfig,
    agent_id: &str,
    env_var: &'static str,
    default_url: &str,
) -> ProviderVariant {
    match resolve_api_key(config, env_var) {
        Some(key) => {
            let mut config = config.clone();
            if config
                .base_url
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty()
            {
                if default_url.is_empty() {
                    return ProviderVariant::Null(NullProvider::new(
                        agent_id,
                        NullReason::MissingApiKey {
                            env_var: "INCEPTION_BASE_URL",
                        },
                    ));
                }
                config.base_url = Some(default_url.to_string());
            }
            ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                client, key, config,
            ))
        }
        None => ProviderVariant::Null(NullProvider::new(
            agent_id,
            NullReason::MissingApiKey { env_var },
        )),
    }
}

impl AgentRunner {
    /// Resolves the correct `ProviderVariant` for the given context.
    pub(crate) async fn resolve_provider(
        &self,
        ctx: &RunContext,
        client: reqwest::Client,
    ) -> ProviderVariant {
        tracing::debug!(
            "🔍 [Provider] Resolving provider '{}' for agent '{}'",
            ctx.provider_name,
            ctx.agent_id
        );

        if self
            .state
            .governance
            .null_providers_test_mode
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            tracing::info!("[Provider] Null mode forced by test flag");
            return ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::TestMode));
        }

        let mut active_config = ctx.model_config.clone();

        // Model-Provider Alignment check
        if active_config.provider == ModelProvider::Openai {
            if let Some(detected) = ModelProvider::from_model_id(&active_config.model_id) {
                if active_config.provider != detected {
                    tracing::warn!(
                        "⚠️ [Provider Alignment] Model '{}' detected but provider is set to OpenAI. Aligning provider to {:?}.",
                        active_config.model_id,
                        detected
                    );
                    active_config.provider = detected;
                }
            }
        }

        // SEC-04: Privacy Mode Enforcement - Route to local endpoints
        let effective_cluster_id = ctx.cluster_id.as_deref().or(Some(&ctx.mission_id));
        if self
            .state
            .governance
            .is_privacy_mode_enabled(effective_cluster_id)
        {
            let is_local =
                is_local_endpoint(&active_config.provider, active_config.base_url.as_deref());

            if !is_local {
                tracing::info!(
                    "🔒 [Privacy Shield] Routing cloud provider {:?} (model: {}) for agent '{}' to local Ollama endpoint",
                    active_config.provider,
                    active_config.model_id,
                    ctx.agent_id
                );
                active_config.provider = ModelProvider::Ollama;

                // Map common cloud-only model IDs to local fallback models to avoid Ollama failures
                let model_lower = active_config.model_id.to_lowercase();
                if CLOUD_MODEL_MARKERS
                    .iter()
                    .any(|&marker| model_lower.contains(marker))
                {
                    let resolved_url =
                        crate::networking::resolver::AddressResolver::resolve_local_url(11434)
                            .await;
                    let fallback_url = format!("{}/v1", resolved_url);
                    active_config.model_id =
                        resolve_privacy_fallback_model(&client, &fallback_url).await;
                }

                active_config.base_url = None;
                active_config.api_key = None;
            }
        }

        match active_config.provider {
            ModelProvider::Google | ModelProvider::Gemini => {
                match resolve_api_key(&active_config, "GOOGLE_API_KEY") {
                    Some(key) => {
                        ProviderVariant::Gemini(crate::agent::gemini::GeminiProvider::new(
                            client,
                            key,
                            active_config.clone(),
                        ))
                    }
                    None => ProviderVariant::Null(NullProvider::new(
                        &ctx.agent_id,
                        NullReason::MissingApiKey {
                            env_var: "GOOGLE_API_KEY",
                        },
                    )),
                }
            }
            ModelProvider::Groq => match resolve_api_key(&active_config, "GROQ_API_KEY") {
                Some(key) => ProviderVariant::Groq(crate::agent::groq::GroqProvider::new(
                    client,
                    key,
                    active_config.clone(),
                )),
                None => ProviderVariant::Null(NullProvider::new(
                    &ctx.agent_id,
                    NullReason::MissingApiKey {
                        env_var: "GROQ_API_KEY",
                    },
                )),
            },
            ModelProvider::Openai
            | ModelProvider::Xai
            | ModelProvider::Openrouter
            | ModelProvider::Mistral
            | ModelProvider::Perplexity
            | ModelProvider::Fireworks
            | ModelProvider::Together
            | ModelProvider::Cerebras
            | ModelProvider::Sambanova
            | ModelProvider::OllamaCloud => {
                let (env_var, default_url) = match active_config.provider {
                    ModelProvider::Xai => ("XAI_API_KEY", "https://api.x.ai/v1"),
                    ModelProvider::Openrouter => {
                        ("OPENROUTER_API_KEY", "https://openrouter.ai/api/v1")
                    }
                    ModelProvider::Mistral => ("MISTRAL_API_KEY", "https://api.mistral.ai/v1"),
                    ModelProvider::Perplexity => {
                        ("PERPLEXITY_API_KEY", "https://api.perplexity.ai")
                    }
                    ModelProvider::Fireworks => {
                        ("FIREWORKS_API_KEY", "https://api.fireworks.ai/inference/v1")
                    }
                    ModelProvider::Together => ("TOGETHER_API_KEY", "https://api.together.xyz/v1"),
                    ModelProvider::Cerebras => ("CEREBRAS_API_KEY", "https://api.cerebras.ai/v1"),
                    ModelProvider::Sambanova => {
                        ("SAMBANOVA_API_KEY", "https://api.sambanova.ai/v1")
                    }
                    ModelProvider::OllamaCloud => ("OLLAMA_CLOUD_API_KEY", "https://ollama.com/v1"),
                    _ => ("OPENAI_API_KEY", "https://api.openai.com/v1"),
                };

                resolve_openai_provider(client, &active_config, &ctx.agent_id, env_var, default_url)
            }
            ModelProvider::Inception => resolve_openai_provider(
                client,
                &active_config,
                &ctx.agent_id,
                "INCEPTION_API_KEY",
                "",
            ),
            ModelProvider::Deepseek => {
                let api_key = resolve_api_key(&active_config, "DEEPSEEK_API_KEY");
                match api_key {
                    Some(key) => {
                        ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                            client,
                            key,
                            active_config.clone(),
                        ))
                    }
                    None => ProviderVariant::Null(NullProvider::new(
                        &ctx.agent_id,
                        NullReason::MissingApiKey {
                            env_var: "DEEPSEEK_API_KEY",
                        },
                    )),
                }
            }
            ModelProvider::Ollama => {
                let api_key = active_config
                    .api_key
                    .as_ref()
                    .filter(|s| !s.trim().is_empty())
                    .cloned()
                    .unwrap_or_else(|| "ollama".to_string());
                let mut config = active_config.clone();
                if let Some(url) = config.base_url.as_ref().filter(|s| !s.trim().is_empty()) {
                    config.base_url = Some(
                        crate::networking::resolver::AddressResolver::resolve_url_if_local(url)
                            .await,
                    );
                } else {
                    let resolved_url =
                        crate::networking::resolver::AddressResolver::resolve_local_url(11434)
                            .await;
                    config.base_url = Some(format!("{}/v1", resolved_url));
                }
                ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                    client, api_key, config,
                ))
            }
            ModelProvider::Anthropic => {
                match resolve_api_key(&active_config, "ANTHROPIC_API_KEY") {
                    Some(key) => {
                        ProviderVariant::Anthropic(crate::agent::anthropic::AnthropicProvider::new(
                            client,
                            key,
                            active_config.clone(),
                        ))
                    }
                    None => ProviderVariant::Null(NullProvider::new(
                        &ctx.agent_id,
                        NullReason::MissingApiKey {
                            env_var: "ANTHROPIC_API_KEY",
                        },
                    )),
                }
            }
        }
    }
}
