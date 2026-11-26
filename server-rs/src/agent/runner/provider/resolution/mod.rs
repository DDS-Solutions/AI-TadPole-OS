//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - Grouped execution context owns identity, mission state and environment; retain scope across callers.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / resolution
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Provider]`, `[Privacy Shield]`, `[Provider Alignment]`

pub mod adapter;
pub mod keys;
pub mod privacy_shield;

#[cfg(test)]
mod tests;

pub(crate) use adapter::*;
pub(crate) use keys::*;
pub(crate) use privacy_shield::*;

use super::ProviderVariant;
use crate::agent::model_routing::is_local_endpoint;
use crate::agent::null_provider::{NullProvider, NullReason};
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::ModelProvider;

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
            ctx.identity.agent_id
        );

        if self
            .state
            .governance
            .null_providers_test_mode
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            tracing::info!("[Provider] Null mode forced by test flag");
            return ProviderVariant::Null(NullProvider::new(
                &ctx.identity.agent_id,
                NullReason::TestMode,
            ));
        }

        let mut active_config = ctx.model_config.clone();

        // Model-Provider Alignment check:
        // Only align when:
        // 1. A detected provider exists from model_id heuristics
        // 2. The detected provider differs from configured provider
        // 3. The configured provider is NOT OpenRouter (which routes heterogeneous vendor models by design)
        // 4. The endpoint is NOT a local endpoint (vLLM, LM Studio, llama.cpp, etc.)
        // 5. The base_url is official OpenAI (empty, None, or api.openai.com)
        if let Some(detected) = ModelProvider::from_model_id(&active_config.model_id) {
            let is_local =
                is_local_endpoint(&active_config.provider, active_config.base_url.as_deref());
            let is_official_openai = is_official_openai_endpoint(active_config.base_url.as_deref());
            let is_multi_vendor_proxy = active_config.provider == ModelProvider::Openrouter;

            if active_config.provider != detected {
                if is_local || !is_official_openai || is_multi_vendor_proxy {
                    tracing::debug!(
                        "ℹ️ [Provider Alignment] Preserving provider {:?} for model '{}' (local={}, official_openai={}, multi_vendor={})",
                        active_config.provider,
                        active_config.model_id,
                        is_local,
                        is_official_openai,
                        is_multi_vendor_proxy
                    );
                } else if active_config.provider == ModelProvider::Openai {
                    tracing::warn!(
                        "⚠️ [Provider Alignment] Model '{}' detected as {:?}, but provider is set to OpenAI. Aligning provider to {:?}.",
                        active_config.model_id,
                        detected,
                        detected
                    );
                    active_config.provider = detected;
                } else {
                    tracing::debug!(
                        "ℹ️ [Provider Alignment] Agent '{}' configured with provider {:?}, model indicates {:?}.",
                        ctx.identity.agent_id,
                        active_config.provider,
                        detected
                    );
                }
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

        // SEC-04: Privacy Mode Enforcement - Route non-local traffic to local Ollama endpoint,
        // and cap oversized (>15B) models on local endpoints to safe local alternatives.
        let is_privacy = self.is_privacy_active(ctx);

        if is_privacy {
            let is_local =
                is_local_endpoint(&active_config.provider, active_config.base_url.as_deref());
            let is_oversized = is_oversized_model(&active_config.model_id);

            if !is_local || is_oversized {
                if !is_local {
                    tracing::info!(
                        "🔒 [Privacy Shield] Routing cloud provider {:?} (model: {}) for agent '{}' to local Ollama endpoint",
                        active_config.provider,
                        active_config.model_id,
                        ctx.identity.agent_id
                    );
                } else {
                    tracing::warn!(
                        "⚠️ [Privacy Shield] Agent '{}' configured with oversized local model '{}'. Enforcing safe local fallback <= 15B.",
                        ctx.identity.agent_id,
                        active_config.model_id
                    );
                }

                let port = std::env::var("OLLAMA_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(11434);
                let resolved_url =
                    crate::networking::resolver::AddressResolver::resolve_local_url(port).await;
                let fallback_base = format!("{}/v1", resolved_url);

                match resolve_privacy_fallback_model(
                    &client,
                    &fallback_base,
                    if is_oversized {
                        None
                    } else {
                        Some(&active_config.model_id)
                    },
                )
                .await
                {
                    Some(chosen_model) => {
                        active_config.provider = ModelProvider::Ollama;
                        active_config.model_id = chosen_model;
                        active_config.base_url = Some(fallback_base);
                        active_config.api_key = None;
                    }
                    None => {
                        tracing::error!(
                            "🔒 [Privacy Shield] Blocked execution for agent '{}': no reachable local model <= 15B found under Privacy Mode",
                            ctx.identity.agent_id
                        );
                        return ProviderVariant::Null(NullProvider::new(
                            &ctx.identity.agent_id,
                            NullReason::PrivacyModeEnforced,
                        ));
                    }
                }
            }
        }

        *ctx.resolved_model_id.lock() = Some(active_config.model_id.clone());

        match active_config.provider {
            ModelProvider::Google | ModelProvider::Gemini => {
                match resolve_provider_key(
                    &active_config,
                    &ctx.identity.agent_id,
                    "GOOGLE_API_KEY",
                    Some("GEMINI_API_KEY"),
                ) {
                    Ok(key) => ProviderVariant::Gemini(crate::agent::gemini::GeminiProvider::new(
                        client,
                        key,
                        active_config.clone(),
                    )),
                    Err(null_variant) => null_variant,
                }
            }
            ModelProvider::Groq => {
                match resolve_provider_key(
                    &active_config,
                    &ctx.identity.agent_id,
                    "GROQ_API_KEY",
                    None,
                ) {
                    Ok(key) => ProviderVariant::Groq(crate::agent::groq::GroqProvider::new(
                        client,
                        key,
                        active_config.clone(),
                    )),
                    Err(null_variant) => null_variant,
                }
            }
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
                    &ctx.identity.agent_id,
                    env_var,
                    default_url,
                    name,
                )
            }
            ModelProvider::Inception => resolve_openai_provider(
                client,
                &active_config,
                &ctx.identity.agent_id,
                "INCEPTION_API_KEY",
                "",
                "Inception",
            ),
            ModelProvider::Ollama => {
                let api_key = resolve_api_key(&active_config, "OLLAMA_API_KEY")
                    .unwrap_or_else(|| "ollama".to_string());
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
                match resolve_provider_key(
                    &active_config,
                    &ctx.identity.agent_id,
                    "ANTHROPIC_API_KEY",
                    None,
                ) {
                    Ok(key) => {
                        ProviderVariant::Anthropic(crate::agent::anthropic::AnthropicProvider::new(
                            client,
                            key,
                            active_config.clone(),
                        ))
                    }
                    Err(null_variant) => null_variant,
                }
            }
        }
    }
}
