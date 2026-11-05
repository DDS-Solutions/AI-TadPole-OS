//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / finalize / accounting
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none

use crate::agent::runner::{AgentRunner, RunContext};

impl AgentRunner {
    /// Returns the effective model ID used during inference, preferring the vetted/resolved
    /// model ID over the original static configuration (e.g. for Privacy Shield local fallbacks).
    pub(crate) fn effective_model_id(&self, ctx: &RunContext) -> String {
        ctx.resolved_model_id
            .lock()
            .clone()
            .unwrap_or_else(|| ctx.model_config.model_id.clone())
    }

    /// Strips internal LLM control tokens and thinking tags (including attributes) from output text before broadcasting to the UI.
    pub(crate) fn scrub_mythos_tags(text: &str) -> String {
        static TAG_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r"(?is)<thought(\s+[^>]*)?>.*?</thought>|<(thinking|/thinking|thought|/thought|halting_signal|halt)(\s+[^>]*)?\s*/?>|<\|im_end\|>|<\|thought\|>|<\|endoftext\|>").unwrap()
        });
        TAG_REGEX.replace_all(text, "").trim().to_string()
    }
}
