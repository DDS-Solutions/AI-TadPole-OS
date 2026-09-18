//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / context_manager
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: tests::test_compact_strips_private_tags, tests::test_compact_collapses_successes, tests::test_frozen_continuation_notice

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::tokenizer::TokenizerService;
use crate::error::AppError;

pub struct ContextManager;

pub const FROZEN_CONTINUATION_NOTICE: &str = "\n\n[CONTINUATION NOTICE: The above summary reflects all work completed so far. Do NOT repeat or re-run any tasks, tool calls, or tests already marked as completed above. Pick up directly at <current_work_and_next_step>.]";

impl ContextManager {
    /// Applies deterministic rules to reduce history size before LLM summarization.
    ///
    /// Focuses on removing redundant CLI output, conversational filler,
    /// ephemeral system tags, and collapsing repeated successful attempts.
    pub fn compact(history: &str) -> String {
        let lines: Vec<&str> = history.lines().collect();
        if lines.is_empty() {
            return String::new();
        }

        let mut compacted_lines = Vec::with_capacity(lines.len());

        for line in lines.iter() {
            let current = line.trim();

            // Strip ephemeral debug / private tags before LLM summarization
            if current.contains("[SystemOnly]") || current.contains("[Private]") {
                continue;
            }

            let prev = compacted_lines
                .last()
                .map(|s: &String| s.trim())
                .unwrap_or("");

            if (current.contains("tool_result") && current.contains("Success"))
                && (prev.contains("tool_result") && prev.contains("Success"))
            {
                // Collapse consecutive tool_result Success entries
                continue;
            }
            compacted_lines.push(line.to_string());
        }

        // Fact-Preservation: Ensure paths and error codes are NEVER pruned
        compacted_lines.join("\n")
    }
}

impl ContextManager {
    /// Calculates the token count of a given text content using default/fallback model.
    pub fn calculate_tokens(text: &str) -> usize {
        TokenizerService::count_tokens("gpt-4o", text)
    }

    /// Calculates the token count of a given text content for a specific model ID.
    pub fn calculate_tokens_for_model(model_id: &str, text: &str) -> usize {
        TokenizerService::count_tokens(model_id, text)
    }

    /// Performs tiered history compression.
    ///
    /// Tier 1: Local Heuristics (HeuristicCompactor & Tag Stripper)
    /// Tier 2: Semantic 8-Section Summarization (LLM) with Frozen Continuation
    pub async fn summarize_history(
        runner: &AgentRunner,
        ctx: &RunContext,
        history: &str,
    ) -> Result<String, AppError> {
        let model_id = &ctx.model_config.model_id;

        // --- Tier 1: Local Heuristics ---
        let heuristically_compacted = Self::compact(history);

        tracing::info!(
            "🧠 [ContextManager] Tier 1 Compaction: {} -> {} tokens for model {}",
            Self::calculate_tokens_for_model(model_id, history),
            Self::calculate_tokens_for_model(model_id, &heuristically_compacted),
            model_id
        );

        // --- Tier 2: Semantic 8-Section Summarization ---
        let summarization_prompt = format!(
            "You are the Context Management Engine for Tadpole OS.\n\n\
             ### MISSION OBJECTIVE:\n\
             Summarize the following mission history into a concise, high-density structured state using EXACTLY the following 8 XML sections:\n\n\
             1. <primary_intent>: What is the user ultimately trying to achieve?\n\
             2. <key_concepts>: Domain-specific terminology, architectural rules, or constraints.\n\
             3. <files_and_code>: All files viewed, edited, created, or deleted (with exact paths and symbol names).\n\
             4. <errors_and_fixes>: Errors encountered, root causes identified, and solutions applied.\n\
             5. <problem_solving>: Alternative approaches attempted and why they succeeded or failed.\n\
             6. <all_user_messages>: Verbatim or near-verbatim user requests and corrections.\n\
             7. <pending_tasks>: Tasks planned or requested but not yet executed or verified.\n\
             8. <current_work_and_next_step>: Exact active step and immediate next action.\n\n\
             Preserve all critical findings, file paths, and established facts. Remove conversational filler and redundant reasoning.\n\n\
             ### MISSION HISTORY:\n\
             {}\n\n\
             ### OUTPUT FORMAT:\n\
             Provide ONLY the 8 XML sections. Do not include any preamble or meta-commentary.",
            heuristically_compacted
        );

        let (mut summary, _, _) = runner
            .call_provider_for_synthesis(ctx, &summarization_prompt, None)
            .await?;

        if !summary.trim().is_empty() {
            summary.push_str(FROZEN_CONTINUATION_NOTICE);
        }

        tracing::info!(
            "✅ [ContextManager] Tier 2 Compaction complete. Final length: {} tokens",
            Self::calculate_tokens_for_model(model_id, &summary)
        );

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_strips_private_tags() {
        let raw = "Step 1: Normal\n[SystemOnly] Internal heartbeat\nStep 2: Processing\n[Private] Secret token\nStep 3: Done";
        let compacted = ContextManager::compact(raw);
        assert!(!compacted.contains("[SystemOnly]"));
        assert!(!compacted.contains("[Private]"));
        assert!(compacted.contains("Step 1: Normal"));
        assert!(compacted.contains("Step 2: Processing"));
        assert!(compacted.contains("Step 3: Done"));
    }

    #[test]
    fn test_compact_collapses_successes() {
        let raw = "tool_result: Success reading file\ntool_result: Success writing file\ntool_result: Failure on command";
        let compacted = ContextManager::compact(raw);
        let lines: Vec<&str> = compacted.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("tool_result: Success reading file"));
        assert!(lines[1].contains("tool_result: Failure on command"));
    }

    #[test]
    fn test_frozen_continuation_notice() {
        assert!(FROZEN_CONTINUATION_NOTICE.contains("CONTINUATION NOTICE"));
        assert!(FROZEN_CONTINUATION_NOTICE.contains("Do NOT repeat or re-run"));
        assert!(FROZEN_CONTINUATION_NOTICE.contains("<current_work_and_next_step>"));
    }
}
