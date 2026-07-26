//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Sentinel**: Blocks narrative leaks from specialist nodes and scrubs control tags.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Re-prompt API failures or control tag leaks in output text.
//!

use crate::error::AppError;
use crate::agent::runner::RunContext;

/// Removes internal Mythos control tags from narrative text.
pub fn scrub_mythos_tags(text: &str) -> String {
    text.replace("<halting_signal/>", "")
        .replace("<halt/>", "")
        .replace("<thinking>", "")
        .replace("</thinking>", "")
        .trim()
        .to_string()
}

impl super::super::AgentRunner {
    /// Enforces the Sentinel Gate protocol: Specialist agents are forbidden from text-only turns.
    pub(crate) async fn enforce_sentinel_gate(
        &self,
        ctx: &RunContext,
        system_prompt: &str,
        user_directive: &str,
        output_text: &mut String,
        function_calls: &mut Vec<crate::agent::types::ToolCall>,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<(), AppError> {
        let is_orchestrator =
            crate::agent::runner::service_traits::IdentityService::is_orchestrator(&ctx.agent_id);

        // If not an orchestrator, and no tools are being called, and mission isn't completed...
        // 🚨 OVERLORD BYPASS: If safe_mode is active, we allow specialists to be conversational.
        if !is_orchestrator
            && !ctx.safe_mode
            && function_calls.is_empty()
            && !output_text.contains("complete_mission")
        {
            tracing::warn!("🛡️ [Sentinel] Specialist {} attempted narrative leak. Enforcing tactical autonomy...", ctx.agent_id);

            // Fix 8: Don't re-inject the full raw user directive into the sentinel re-prompt.
            // This prevents injection amplification by truncating and sanitizing.
            let sanitized_objective = crate::agent::sanitizer::Sanitizer::sanitize_for_prompt(
                &user_directive.chars().take(200).collect::<String>(),
            );
            let sentinel_directive = format!(
                "SYSTEM_SENTINEL: Your turn resulted in a narrative-only response. As an AGENT (Task Specialist), you are FORBIDDEN from text-only progress reports or roadblock apologies. \
                 You MUST execute tools or call 'complete_mission' with results. Mission objective (summarized): {}",
                sanitized_objective
            );

            let swarm_tool = self.build_tools(ctx).await;
            let sentinel_result = self
                .call_provider(
                    ctx,
                    system_prompt,
                    &sentinel_directive,
                    Some(vec![swarm_tool]),
                )
                .await;

            let (sent_text, sent_calls, sent_usage) = sentinel_result?;
            *output_text = sent_text;
            *function_calls = sent_calls;
            self.accumulate_usage(usage, sent_usage);
        }
        Ok(())
    }
}
