//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Monologue**: Manages context compression, log truncation, and fallback summarizers.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Monologue compression failures or recursive summarizer errors.
//!

use crate::agent::runner::RunContext;
use crate::error::AppError;

impl super::super::AgentRunner {
    /// Truncates embedded markdown code blocks in a string if they exceed 2,000 characters.
    pub(crate) fn truncate_embedded_tool_logs(content: &str) -> String {
        let mut result = String::new();
        let mut current_pos = 0;
        while let Some(start_idx) = content[current_pos..].find("```") {
            let abs_start = current_pos + start_idx;
            result.push_str(&content[current_pos..abs_start]);

            let rest = &content[abs_start + 3..];
            if let Some(end_idx) = rest.find("```") {
                let abs_end = abs_start + 3 + end_idx;
                let code_block_content = &content[abs_start + 3..abs_end];

                let newline_pos = code_block_content.find('\n').unwrap_or(0);
                let header = &code_block_content[..newline_pos];
                let body = &code_block_content[newline_pos..];

                if body.len() > 2000 {
                    result.push_str(&format!(
                        "```{}\n[Raw tool result evicted to save context — was {} bytes]\n```",
                        header,
                        body.len()
                    ));
                } else {
                    result.push_str(&content[abs_start..abs_end + 3]);
                }
                current_pos = abs_end + 3;
            } else {
                result.push_str(&content[abs_start..]);
                current_pos = content.len();
                break;
            }
        }
        if current_pos < content.len() {
            result.push_str(&content[current_pos..]);
        }
        result
    }

    /// Fallback summarizer that runs deterministically without calling LLMs.
    /// Keeps header structures and short lines, removing code blocks entirely.
    pub(crate) fn deterministic_fallback_summarize(history: &str) -> String {
        let mut lines = Vec::new();
        let mut in_code_block = false;
        for line in history.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                lines.push("[Code block header omitted]".to_string());
                continue;
            }
            if in_code_block {
                continue;
            }
            if trimmed.len() > 150 {
                lines.push(format!("{}... [truncated]", &trimmed[..150]));
            } else if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }
        format!("DETERMINISTIC FALLBACK SUMMARY:\n{}", lines.join("\n"))
    }

    /// Compresses the internal monologue via recursive summarization if it grows too large.
    pub(crate) async fn compress_monologue(
        &self,
        ctx: &RunContext,
        monologue: &mut Vec<String>,
    ) -> Result<(), AppError> {
        let total_chars: usize = monologue.iter().map(|s| s.len()).sum();
        if total_chars < 8192 {
            return Ok(());
        }

        tracing::info!(
            "✂️ [Mythos] Monologue threshold reached ({} chars) for agent {}. Summarizing...",
            total_chars,
            ctx.agent_id
        );

        let tail_count = 4;
        let monologue_len = monologue.len();

        let (older_turns, tail_turns) = if monologue_len > tail_count {
            let split_idx = monologue_len - tail_count;
            (
                monologue[..split_idx].to_vec(),
                monologue[split_idx..].to_vec(),
            )
        } else {
            (Vec::new(), monologue.to_vec())
        };

        if older_turns.is_empty() {
            return Ok(());
        }

        let mut processed_older_turns = Vec::new();
        for turn in older_turns {
            processed_older_turns.push(Self::truncate_embedded_tool_logs(&turn));
        }

        let history = processed_older_turns.join("\n\n");
        let prompt = format!(
            "SUMMARIZE YOUR PREVIOUS REASONING STEPS INTO A SINGLE CONCISE PARAGRAPH. \
             RETAIN ALL KEY INSIGHTS, VARIABLES, AND HYPOTHESES. \
             \n\nPREVIOUS REASONING:\n{}",
            history
        );

        let summary_text = match self
            .call_provider(
                ctx,
                "You are an expert reasoning summarizer. Be technical, dense, and objective.",
                &prompt,
                None,
            )
            .await
        {
            Ok((text, _, _)) => format!("CONSOLIDATED REASONING: {}", text),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Mythos] Summarizer call failed ({:?}). Falling back to deterministic summarization.",
                    e
                );
                Self::deterministic_fallback_summarize(&history)
            }
        };

        monologue.clear();
        monologue.push(summary_text);
        monologue.extend(tail_turns);

        Ok(())
    }
}
