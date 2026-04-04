//! Null Provider — Development mock for testing agent logic without API costs
//!
//! @docs ARCHITECTURE:Agent
//!
//! Provides a no-op implementation of `LlmProvider` used for graceful 
//! saturation, privacy shielding, or missing-key recovery.
//!
//! @docs ARCHITECTURE:DegradationProvider
//!
//! ### AI Assist Note
//! **Graceful Failure**: Returns `Ok` with a `[DEGRADED]` message instead 
//! of `Err` to prevent engine panics. Correctly increments mission 
//! counts but sets `is_degraded = true`. Essential for CI and Privacy Mode.

use crate::agent::gemini::GeminiTool;
use crate::agent::provider_trait::LlmProvider;
use crate::agent::types::{GeminiFunctionCall, TokenUsage};
use async_trait::async_trait;

/// The reason a NullProvider was substituted.
#[derive(Debug, Clone)]
pub enum NullReason {
    /// A required API key environment variable is missing.
    MissingApiKey { env_var: &'static str },
    /// The configured provider name is not recognised by the engine.
    UnknownProvider { name: String },
    /// Forced by the `TADPOLE_NULL_PROVIDERS=true` environment variable (CI / test mode).
    TestMode,
    /// Blocked by Privacy Shield (Privacy Mode is ON).
    PrivacyModeEnforced,
}

impl NullReason {
    pub fn as_str(&self) -> String {
        match self {
            NullReason::MissingApiKey { env_var } => format!("missing_api_key ({})", env_var),
            NullReason::UnknownProvider { name } => format!("unknown_provider ({})", name),
            NullReason::TestMode => "test_mode".to_string(),
            NullReason::PrivacyModeEnforced => "privacy_mode_enforced".to_string(),
        }
    }
}

/// A no-op LLM provider used for graceful degradation.
///
/// Activated when:
/// - A required API key is missing (returns a degraded response instead of crashing)
/// - The provider name is unknown (instead of returning an Err)
/// - `TADPOLE_NULL_PROVIDERS=true` is set (CI / integration test mode)
///
/// **Always emits a `tracing::warn!`** on every call — impossible to use silently.
/// Missions completed via NullProvider are marked `is_degraded = true` by the runner.
pub struct NullProvider {
    pub agent_id: String,
    pub reason: NullReason,
}

impl NullProvider {
    pub fn new(agent_id: impl Into<String>, reason: NullReason) -> Self {
        Self {
            agent_id: agent_id.into(),
            reason,
        }
    }
}

#[async_trait]
impl LlmProvider for NullProvider {
    async fn generate(
        &self,
        _system_prompt: &str,
        _user_message: &str,
        _tools: Option<Vec<GeminiTool>>,
    ) -> anyhow::Result<(String, Vec<GeminiFunctionCall>, Option<TokenUsage>)> {
        tracing::warn!(
            "⚠️  NULL PROVIDER ACTIVE — agent='{}' reason='{}'. \
             No LLM call was made. Set the correct API key or provider to restore full skill.",
            self.agent_id,
            self.reason.as_str()
        );

        let degraded_msg = format!(
            "[DEGRADED: {}] This agent has no configured provider. \
             Please configure a valid LLM provider and API key in Settings.",
            self.reason.as_str()
        );

        // Return Ok — not Err — so the mission records as degraded, not failed.
        Ok((degraded_msg, vec![], None))
    }

    async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
        tracing::warn!(
            "⚠️ NULL PROVIDER ACTIVE (embed) — agent='{}'",
            self.agent_id
        );
        Ok(vec![0.0; 768]) // Return a zeroed vector or fixed dimension placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn null_provider_returns_ok_not_err() {
        let provider = NullProvider::new("agent-test", NullReason::TestMode);
        let result = provider.generate("sys", "user", None).await;
        assert!(result.is_ok(), "NullProvider must return Ok, not Err");
        let (text, calls, usage) = result.unwrap();
        assert!(
            text.contains("DEGRADED"),
            "Response must contain DEGRADED marker"
        );
        assert!(calls.is_empty(), "No function calls from NullProvider");
        assert!(usage.is_none(), "No token usage from NullProvider");
    }
}
