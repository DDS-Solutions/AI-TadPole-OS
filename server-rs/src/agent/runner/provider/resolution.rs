//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / resolution
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Provider]`
//! - **Witness Tests**: none declared

use super::ProviderVariant;
use crate::agent::model_routing::is_local_endpoint;
use crate::agent::null_provider::{NullProvider, NullReason};
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::{ModelConfig, ModelProvider};

/// Resolves an API key: prefers the per-agent config override, then falls
/// back to the named environment variable. Automatically trims whitespace/newlines.
pub(crate) fn resolve_api_key(config: &ModelConfig, env_var: &str) -> Option<String> {
    config
        .api_key
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::env::var(env_var)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
}

const PRIVACY_FALLBACK_MODEL: &str = "phi3.5-safe:latest";

/// Filter out non-generative embedding, reranking, and encoder models.
fn is_plausible_chat_model(id: &str) -> bool {
    let l = id.to_ascii_lowercase();
    !l.contains("embed")
        && !l.contains("rerank")
        && !l.contains("bge")
        && !l.contains("bert")
        && !l.contains("colbert")
}

/// Dynamically queries local Ollama models to pick the first available generative chat model as fallback.
pub(crate) async fn resolve_privacy_fallback_model(
    client: &reqwest::Client,
    base_url: &str,
) -> String {
    let base_clean = base_url.trim_end_matches('/');
    let url = if base_clean.ends_with("/v1") {
        format!("{}/models", base_clean)
    } else {
        format!("{}/v1/models", base_clean)
    };

    #[derive(serde::Deserialize)]
    struct OpenAiCompatModel {
        id: String,
    }
    #[derive(serde::Deserialize)]
    struct OpenAiCompatModelsResponse {
        data: Vec<OpenAiCompatModel>,
    }

    match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(models_list) = resp.json::<OpenAiCompatModelsResponse>().await {
                if let Some(chat_model) = models_list
                    .data
                    .iter()
                    .find(|m| is_plausible_chat_model(&m.id))
                    .or_else(|| models_list.data.first())
                {
                    tracing::info!(
                        "🛡️ [Privacy Shield] Dynamically resolved local fallback model: '{}' from Ollama",
                        chat_model.id
                    );
                    return chat_model.id.clone();
                }
            }
        }
        Ok(resp) => {
            tracing::warn!(
                "🛡️ [Privacy Shield] Ollama /v1/models returned non-success status: {}",
                resp.status()
            );
        }
        Err(e) => {
            tracing::warn!("🛡️ [Privacy Shield] Ollama /v1/models probe failed: {}", e);
        }
    }

    let base_root = base_url
        .strip_suffix("/v1")
        .or_else(|| base_url.strip_suffix("/v1/"))
        .unwrap_or(base_url)
        .trim_end_matches('/');
    let api_tags_url = format!("{}/api/tags", base_root);

    #[derive(serde::Deserialize)]
    struct LegacyOllamaModel {
        name: String,
    }
    #[derive(serde::Deserialize)]
    struct LegacyOllamaResponse {
        models: Vec<LegacyOllamaModel>,
    }

    match client
        .get(&api_tags_url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(models_list) = resp.json::<LegacyOllamaResponse>().await {
                if let Some(chat_model) = models_list
                    .models
                    .iter()
                    .find(|m| is_plausible_chat_model(&m.name))
                    .or_else(|| models_list.models.first())
                {
                    tracing::info!(
                        "🛡️ [Privacy Shield] Dynamically resolved local fallback model: '{}' from legacy Ollama tags API",
                        chat_model.name
                    );
                    return chat_model.name.clone();
                }
            }
        }
        Ok(resp) => {
            tracing::warn!(
                "🛡️ [Privacy Shield] Ollama /api/tags returned non-success status: {}",
                resp.status()
            );
        }
        Err(e) => {
            tracing::warn!("🛡️ [Privacy Shield] Ollama /api/tags probe failed: {}", e);
        }
    }

    tracing::info!(
        "🛡️ [Privacy Shield] Falling back to default static model '{}'. Ensure it is pulled locally via `ollama pull {}`.",
        PRIVACY_FALLBACK_MODEL,
        PRIVACY_FALLBACK_MODEL
    );
    PRIVACY_FALLBACK_MODEL.to_string()
}

/// Helper function to resolve OpenAI-compatible providers without code duplication.
pub(crate) fn resolve_openai_provider(
    client: reqwest::Client,
    config: &ModelConfig,
    agent_id: &str,
    env_var: &'static str,
    default_url: &str,
    provider_name: &'static str,
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
                        NullReason::MissingBaseUrl {
                            provider: provider_name,
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

        // Model-Provider Alignment check: detect provider from model ID
        if let Some(detected) = ModelProvider::from_model_id(&active_config.model_id) {
            if active_config.provider == ModelProvider::Openai && detected != ModelProvider::Openai
            {
                tracing::warn!(
                    "⚠️ [Provider Alignment] Model '{}' detected as {:?}, but provider is set to OpenAI. Aligning provider to {:?}.",
                    active_config.model_id,
                    detected,
                    detected
                );
                active_config.provider = detected;
            } else if active_config.provider != detected
                && !is_local_endpoint(&active_config.provider, active_config.base_url.as_deref())
            {
                tracing::debug!(
                    "ℹ️ [Provider Alignment] Agent '{}' configured with provider {:?}, model indicates {:?}.",
                    ctx.agent_id,
                    active_config.provider,
                    detected
                );
            }
        }

        // OpenRouter Model Resolution: Apply OPENROUTER_DEFAULT_MODEL override if set
        if active_config.provider == ModelProvider::Openrouter {
            if let Ok(override_model) = std::env::var("OPENROUTER_DEFAULT_MODEL") {
                let trimmed = override_model.trim();
                if !trimmed.is_empty() && trimmed != active_config.model_id {
                    tracing::info!(
                        "🔄 [OpenRouter Resolution] Model '{}' overridden to '{}'",
                        active_config.model_id,
                        trimmed
                    );
                    active_config.model_id = trimmed.to_string();
                }
            }
        }

        // SEC-04: Privacy Mode Enforcement - Route non-local traffic to local Ollama endpoint.
        // Cluster-scoping fallback: When cluster_id is absent, the mission itself acts as an isolated cluster boundary.
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

                let port = std::env::var("OLLAMA_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(11434);
                let resolved_url =
                    crate::networking::resolver::AddressResolver::resolve_local_url(port).await;
                let fallback_base = format!("{}/v1", resolved_url);
                active_config.model_id =
                    resolve_privacy_fallback_model(&client, &fallback_base).await;

                active_config.base_url = Some(fallback_base);
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
            | ModelProvider::Deepseek
            | ModelProvider::OllamaCloud => {
                let (env_var, default_url, name) = match active_config.provider {
                    ModelProvider::Openai => {
                        ("OPENAI_API_KEY", "https://api.openai.com/v1", "OpenAI")
                    }
                    ModelProvider::Xai => ("XAI_API_KEY", "https://api.x.ai/v1", "xAI"),
                    ModelProvider::Openrouter => (
                        "OPENROUTER_API_KEY",
                        "https://openrouter.ai/api/v1",
                        "OpenRouter",
                    ),
                    ModelProvider::Mistral => {
                        ("MISTRAL_API_KEY", "https://api.mistral.ai/v1", "Mistral")
                    }
                    ModelProvider::Perplexity => (
                        "PERPLEXITY_API_KEY",
                        "https://api.perplexity.ai",
                        "Perplexity",
                    ),
                    ModelProvider::Fireworks => (
                        "FIREWORKS_API_KEY",
                        "https://api.fireworks.ai/inference/v1",
                        "Fireworks",
                    ),
                    ModelProvider::Together => (
                        "TOGETHER_API_KEY",
                        "https://api.together.xyz/v1",
                        "Together",
                    ),
                    ModelProvider::Cerebras => {
                        ("CEREBRAS_API_KEY", "https://api.cerebras.ai/v1", "Cerebras")
                    }
                    ModelProvider::Sambanova => (
                        "SAMBANOVA_API_KEY",
                        "https://api.sambanova.ai/v1",
                        "SambaNova",
                    ),
                    ModelProvider::Deepseek => (
                        "DEEPSEEK_API_KEY",
                        "https://api.deepseek.com/v1",
                        "DeepSeek",
                    ),
                    ModelProvider::OllamaCloud => (
                        "OLLAMA_CLOUD_API_KEY",
                        "https://ollama.com/v1",
                        "Ollama Cloud",
                    ),
                    _ => ("OPENAI_API_KEY", "https://api.openai.com/v1", "OpenAI"),
                };

                resolve_openai_provider(
                    client,
                    &active_config,
                    &ctx.agent_id,
                    env_var,
                    default_url,
                    name,
                )
            }
            ModelProvider::Inception => resolve_openai_provider(
                client,
                &active_config,
                &ctx.agent_id,
                "INCEPTION_API_KEY",
                "",
                "Inception",
            ),
            ModelProvider::Ollama => {
                let api_key = "ollama".to_string();
                let mut config = active_config.clone();
                if let Some(url) = config.base_url.as_ref().filter(|s| !s.trim().is_empty()) {
                    config.base_url = Some(
                        crate::networking::resolver::AddressResolver::resolve_url_if_local(url)
                            .await,
                    );
                } else {
                    let port = std::env::var("OLLAMA_PORT")
                        .ok()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(11434);
                    let resolved_url =
                        crate::networking::resolver::AddressResolver::resolve_local_url(port).await;
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
