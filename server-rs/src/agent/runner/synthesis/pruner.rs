//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / pruner
//!
//! ### AI Assist Note
//! - Enforce bounded token budgets against model context window without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::runner::RunContext;
use crate::agent::tokenizer::TokenizerService;

/// Handles context pruning based on model-aware token count (TPM Protection).
pub fn prune_context(
    ctx: &RunContext,
    identity: &str,
    memory: &str,
    repo_map: &str,
    swarm_context_str: &str,
    extra_context: &[&str],
) -> (String, String) {
    let model_id = &ctx.model_config.model_id;
    let context_window = ctx.model_config.context_window.unwrap_or(128_000) as usize;
    // Base prompt budget on model context window, reserving 25% headroom for input payload and output generation
    let effective_limit = context_window;
    let safe_limit = (context_window as f32 * 0.75) as usize;

    let mut pruned_repo_map = repo_map.to_string();
    let mut pruned_swarm_context = swarm_context_str.to_string();

    let extra_combined = extra_context.join("");
    let get_total_tokens = |repo: &str, swarm: &str| {
        let combined = format!("{}{}{}{}{}", identity, memory, repo, swarm, extra_combined);
        TokenizerService::count_tokens(model_id, &combined)
    };

    let mut current_tokens = get_total_tokens(&pruned_repo_map, &pruned_swarm_context);

    if current_tokens > safe_limit {
        tracing::warn!(
            "⚠️ [Pruning] Context size {} tokens exceeds safe limit {} for model {} (context_window={}). Applying semantic weights.",
            current_tokens,
            safe_limit,
            model_id,
            context_window
        );

        // 1. Prune Repo Map (Weight: 0.8)
        if current_tokens > safe_limit {
            let weight = ctx.resource_weights.get("repo_map").cloned().unwrap_or(0.8);
            if weight < 1.0 {
                pruned_repo_map =
                    "⚠️ Repo Map pruned due to context limits. Use 'list_files' for discovery."
                        .to_string();
                current_tokens = get_total_tokens(&pruned_repo_map, &pruned_swarm_context);
            }
        }

        // 2. Prune Swarm Context (Weight: 1.0)
        if current_tokens > safe_limit {
            let target_len = (pruned_swarm_context.len() as f32 * 0.5) as usize;
            if target_len > 500 {
                pruned_swarm_context = safe_truncate(&pruned_swarm_context, target_len);
                current_tokens = get_total_tokens(&pruned_repo_map, &pruned_swarm_context);
            }
        }

        // 3. Emergency Truncation
        if current_tokens > effective_limit {
            tracing::error!(
                "🚨 [Pruning] EMERGENCY: Context still exceeds limit ({} > {}). Hard truncating swarm context.",
                current_tokens,
                effective_limit
            );
            pruned_swarm_context = safe_truncate(&pruned_swarm_context, 500);
            let final_tokens = get_total_tokens(&pruned_repo_map, &pruned_swarm_context);
            if final_tokens > effective_limit {
                tracing::warn!(
                    "⚠️ [Pruning] Context remains at {} tokens after emergency truncation (identity + memory = {} tokens).",
                    final_tokens,
                    TokenizerService::count_tokens(model_id, &format!("{}{}", identity, memory))
                );
            }
        }
    }

    (pruned_repo_map, pruned_swarm_context)
}

/// UTF-8-safe truncation that walks backward to a valid character boundary.
pub fn safe_truncate(text: &str, len: usize) -> String {
    if text.len() <= len {
        return text.to_string();
    }
    let mut cut = len.min(text.len());
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}... [TRUNCATED]", &text[..cut])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_truncate_ascii() {
        let text = "Hello world this is a test";
        assert_eq!(safe_truncate(text, 5), "Hello... [TRUNCATED]");
        assert_eq!(safe_truncate(text, 100), text);
    }

    #[test]
    fn test_safe_truncate_multibyte_utf8() {
        // CJK and emoji strings with multi-byte characters
        let cjk_text = "你好世界，这是一个测试";
        // Each Chinese character is 3 bytes. Slicing at byte 4 would land inside the 2nd char.
        let truncated = safe_truncate(cjk_text, 4);
        assert_eq!(truncated, "你... [TRUNCATED]");

        let emoji_text = "🚀🦀🤖✨🔥";
        // Each emoji is 4 bytes. Slicing at byte 6 would land inside the 2nd emoji.
        let truncated_emoji = safe_truncate(emoji_text, 6);
        assert_eq!(truncated_emoji, "🚀... [TRUNCATED]");
    }
}
