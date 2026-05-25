//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[pruner]` in tracing logs.

use tiktoken_rs::cl100k_base;
use crate::agent::runner::RunContext;

/// Handles context pruning based on token count (TPM Protection).
pub fn prune_context(
    ctx: &RunContext,
    identity: &str,
    memory: &str,
    repo_map: &str,
    swarm_context_str: &str,
) -> (String, String) {
    let bpe = match cl100k_base() {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(
                "🚨 [Pruning] Failed to initialize tokenizer: {:?}. Skipping pruning.",
                e
            );
            return (repo_map.to_string(), swarm_context_str.to_string());
        }
    };
    let tpm_limit = ctx.model_config.tpm.unwrap_or(100_000);
    let safe_limit = (tpm_limit as f32 * 0.85) as usize;

    let mut pruned_repo_map = repo_map.to_string();
    let mut pruned_swarm_context = swarm_context_str.to_string();

    let get_total_tokens = |repo: &str, swarm: &str| {
        let combined = format!("{}{}{}{}", identity, memory, repo, swarm);
        bpe.encode_with_special_tokens(&combined).len()
    };

    let mut current_tokens = get_total_tokens(&pruned_repo_map, &pruned_swarm_context);

    if current_tokens > safe_limit {
        tracing::warn!(
            "⚠️ [Pruning] Context size {} exceeds safe limit {}. Applying semantic weights.",
            current_tokens,
            safe_limit
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
        if current_tokens > tpm_limit as usize {
            tracing::error!("🚨 [Pruning] EMERGENCY: Context still exceeds TPM limit ({}). Hard truncating swarm context.", current_tokens);
            pruned_swarm_context = safe_truncate(&pruned_swarm_context, 500);
        }
    }

    (pruned_repo_map, pruned_swarm_context)
}

fn safe_truncate(text: &str, len: usize) -> String {
    if text.len() <= len {
        return text.to_string();
    }
    format!("{}... [TRUNCATED]", &text[..len])
}

// Metadata: [pruner]
