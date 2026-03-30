//! Context Management Engine for history summarization and token optimization.
//!
//! Provides utilities for calculating context density and performing
//! semantic history compression when mission history exceeds token thresholds.

use tiktoken_rs::cl100k_base;
use crate::agent::runner::{AgentRunner, RunContext};

pub struct ContextManager;

impl ContextManager {
    /// Calculates the token count of a given text content.
    pub fn calculate_tokens(text: &str) -> usize {
        let bpe = cl100k_base().unwrap();
        bpe.encode_with_special_tokens(text).len()
    }

    /// Performs semantic summarization of the mission history.
    ///
    /// This method calls the agent's primary model to condense the 
    /// accumulated findings and conversation history into a high-density summary.
    pub async fn summarize_history(
        runner: &AgentRunner,
        ctx: &RunContext,
        history: &str,
    ) -> anyhow::Result<String> {
        tracing::info!(
            "🧠 [ContextManager] Summarizing history for agent {} (History length: {} tokens)",
            ctx.agent_id,
            Self::calculate_tokens(history)
        );

        let summarization_prompt = format!(
            "You are the Context Management Engine for Tadpole OS.\n\n\
             ### MISSION OBJECTIVE:\n\
             Summarize the following mission history into a concise, high-density 'Condensed State'. \
             Preserve all critical findings, file paths, and established facts. \
             Remove conversational filler and redundant reasoning.\n\n\
             ### MISSION HISTORY:\n\
             {}\n\n\
             ### OUTPUT FORMAT:\n\
             Provide ONLY the condensed summary. Do not include any meta-commentary.",
            history
        );

        // Call the primary model for summarization
        let (summary, _, _) = runner
            .call_provider_for_synthesis(ctx, &summarization_prompt)
            .await?;

        tracing::info!(
            "✅ [ContextManager] History condensed (Summary length: {} tokens)",
            Self::calculate_tokens(&summary)
        );

        Ok(summary)
    }
}
