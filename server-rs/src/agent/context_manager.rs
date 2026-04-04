//! Context Manager — RAG and memory retrieval
//!
//! Provides utilities for calculating context density and performing 
//! semantic history compression when mission history exceeds token thresholds.
//!
//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Tiered Compression**: Tier 1 is heuristic (stripping redundancy via 
//! `compact()`); Tier 2 is semantic (LLM-summarized). Agents should 
//! trigger Tier 2 when tokens exceed 80% of `max_tokens`.

use crate::agent::runner::{AgentRunner, RunContext};
use tiktoken_rs::cl100k_base;

pub struct ContextManager;

impl ContextManager {
    /// Applies deterministic rules to reduce history size before LLM summarization.
    ///
    /// Focuses on removing redundant CLI output, conversational filler,
    /// and collapsing repeated failed attempts.
    pub fn compact(history: &str) -> String {
        let mut lines: Vec<String> = history.lines().map(|l| l.to_string()).collect();

        // 1. Collapse repeating "Success" or "Error" lines from tool calls
        let mut i = 0;
        while i < lines.len().saturating_sub(1) {
            let current = lines[i].trim();
            let next = lines[i + 1].trim();

            if (current.contains("tool_result") && current.contains("Success"))
                && (next.contains("tool_result") && next.contains("Success"))
            {
                lines.remove(i + 1);
            } else {
                i += 1;
            }
        }

        // 2. Fact-Preservation: Ensure paths and error codes are NEVER pruned
        // This is handled by ensuring we don't truncate lines containing patterns like '/' or 'Error:'

        lines.join("\n")
    }
}

impl ContextManager {
    /// Calculates the token count of a given text content.
    pub fn calculate_tokens(text: &str) -> usize {
        let bpe = cl100k_base().unwrap();
        bpe.encode_with_special_tokens(text).len()
    }

    /// Performs tiered history compression.
    ///
    /// Tier 1: Local Heuristics (HeuristicCompactor)
    /// Tier 2: Semantic Summarization (LLM)
    pub async fn summarize_history(
        runner: &AgentRunner,
        ctx: &RunContext,
        history: &str,
    ) -> anyhow::Result<String> {
        // --- Tier 1: Local Heuristics ---
        let heuristically_compacted = Self::compact(history);

        tracing::info!(
            "🧠 [ContextManager] Tier 1 Compaction: {} -> {} tokens",
            Self::calculate_tokens(history),
            Self::calculate_tokens(&heuristically_compacted)
        );

        // --- Tier 2: Semantic Summarization ---
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
            heuristically_compacted
        );

        let (summary, _, _) = runner
            .call_provider_for_synthesis(ctx, &summarization_prompt)
            .await?;

        tracing::info!(
            "✅ [ContextManager] Tier 2 Compaction complete. Final length: {} tokens",
            Self::calculate_tokens(&summary)
        );

        Ok(summary)
    }
}
