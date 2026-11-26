//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / resolution / adapter
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none

use super::super::ProviderVariant;
use super::keys::resolve_api_key;
use crate::agent::model_routing::is_local_endpoint;
use crate::agent::null_provider::{NullProvider, NullReason};
use crate::agent::types::ModelConfig;

/// Helper function to resolve OpenAI-compatible providers without code duplication.
pub(crate) fn resolve_openai_provider(
    client: reqwest::Client,
    config: &ModelConfig,
    agent_id: &str,
    env_var: &'static str,
    default_url: &str,
    provider_name: &'static str,
) -> ProviderVariant {
    let key_opt = resolve_api_key(config, env_var);
    let is_local = is_local_endpoint(&config.provider, config.base_url.as_deref());

    // Local OpenAI-compatible endpoints (vLLM, LM Studio, llama.cpp, local proxies)
    // frequently do not require an API key; provide a dummy token if key is omitted.
    let effective_key = match key_opt {
        Some(k) => k,
        None if is_local => "local-no-key".to_string(),
        None => {
            return ProviderVariant::Null(NullProvider::new(
                agent_id,
                NullReason::MissingApiKey { env_var },
            ));
        }
    };

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
        client,
        effective_key,
        config,
    ))
}
