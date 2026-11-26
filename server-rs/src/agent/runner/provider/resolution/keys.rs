//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / resolution / keys
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none

use super::super::ProviderVariant;
use crate::agent::null_provider::{NullProvider, NullReason};
use crate::agent::types::ModelConfig;

/// Resolves an API key: prefers the per-agent config override, then falls
/// back to the named environment variable. Automatically trims whitespace/newlines.
pub(crate) fn resolve_api_key(config: &ModelConfig, env_var: &str) -> Option<String> {
    config
        .api_key
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !crate::agent::persistence::is_placeholder(s))
        .or_else(|| {
            std::env::var(env_var)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !crate::agent::persistence::is_placeholder(s))
        })
}

/// Generic helper to resolve provider API key with primary and optional fallback environment variables.
#[allow(clippy::result_large_err)]
pub(crate) fn resolve_provider_key(
    config: &ModelConfig,
    agent_id: &str,
    primary_env: &'static str,
    secondary_env: Option<&'static str>,
) -> Result<String, ProviderVariant> {
    let key_opt = resolve_api_key(config, primary_env)
        .or_else(|| secondary_env.and_then(|sec| resolve_api_key(config, sec)));

    match key_opt {
        Some(k) => Ok(k),
        None => Err(ProviderVariant::Null(NullProvider::new(
            agent_id,
            NullReason::MissingApiKey {
                env_var: primary_env,
            },
        ))),
    }
}
