//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / sentinel
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Sentinel]`
//! - **Witness Tests**: none declared

use crate::agent::runner::RunContext;
use crate::error::AppError;
use regex::Regex;
use std::sync::LazyLock;

static TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<(thinking|\/thinking|halting_signal|halt)\s*\/?>").unwrap());

static REPORT_CITATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:`([a-zA-Z0-9_\-/\\]+\.md)`|\b([a-zA-Z0-9_\-]+(?:_Report|_report|_audit|_drift)\.md)\b)").unwrap()
});

/// Removes internal Mythos control tags from narrative text case-insensitively.
pub fn scrub_mythos_tags(text: &str) -> String {
    TAG_REGEX.replace_all(text, "").trim().to_string()
}

/// Scans output text for ungrounded citations (e.g. fabricated reports or non-existent API routes).
pub fn find_ungrounded_citations(text: &str, workspace_root: &std::path::Path) -> Vec<String> {
    let mut violations = Vec::new();

    // 1. Check for fabricated API routes
    if text.contains("/api/v3") || text.contains("/v2/") || text.contains("/api/v2") {
        violations.push("Fabricated API versioning detected (/api/v3 or /v2). Server routes are strictly Axum /v1.".to_string());
    }

    // 2. Check for cited reports that do not exist on disk
    let has_factual_assertion = text.contains("documented in")
        || text.contains("Discovery Phase")
        || text.contains("Audit Complete")
        || text.contains("findings are")
        || text.contains("verified against")
        || text.contains("concluded")
        || text.contains("Mission Status:");

    if has_factual_assertion {
        for cap in REPORT_CITATION_REGEX.captures_iter(text) {
            let file_match = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str());
            if let Some(file_name) = file_match {
                let path = std::path::Path::new(file_name);
                let exists = workspace_root.join(path).exists()
                    || workspace_root.join("docs").join(path).exists()
                    || workspace_root.join("reports").join(path).exists()
                    || workspace_root.join("directives").join(path).exists();
                if !exists {
                    violations.push(format!(
                        "Referenced report '{}' does not exist on disk.",
                        file_name
                    ));
                }
            }
        }
    }

    violations
}

impl super::super::AgentRunner {
    /// Enforces the Sentinel Gate protocol: Specialist agents are forbidden from text-only turns,
    /// and all agents (including orchestrators) are forbidden from fabricated milestone claims.
    pub(crate) async fn enforce_sentinel_gate(
        &self,
        ctx: &RunContext,
        system_prompt: &str,
        user_directive: &str,
        output_text: &mut String,
        function_calls: &mut Vec<crate::agent::types::ToolCall>,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<(), AppError> {
        let citations_root = if ctx.workspace_root.as_os_str().is_empty() {
            &ctx.base_dir
        } else {
            &ctx.workspace_root
        };
        let mut text_to_scan = output_text.clone();
        for fc in function_calls.iter() {
            text_to_scan.push(' ');
            text_to_scan.push_str(&fc.name);
            text_to_scan.push(' ');
            text_to_scan.push_str(&serde_json::to_string(&fc.args).unwrap_or_default());
        }
        let ungrounded = find_ungrounded_citations(&text_to_scan, citations_root);
        if !ungrounded.is_empty() && !ctx.safe_mode {
            tracing::warn!(
                "🛡️ [Sentinel] Grounding violation by {}: {:?}",
                ctx.agent_id,
                ungrounded
            );
            let sentinel_directive = format!(
                "SYSTEM_SENTINEL: REALITY COLLAPSE DETECTED. Your turn asserted facts or artifacts that do not exist: {}. \
                 You are FORBIDDEN from fabricating audit reports, non-existent API routes (/api/v3), or imaginary milestones. \
                 You must state the ground truth or call available tools ('read_file', 'list_files', 'spawn_subagent') to execute the needed work.",
                ungrounded.join("; ")
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
            return Ok(());
        }

        let is_orchestrator =
            crate::agent::runner::service_traits::IdentityService::is_orchestrator(&ctx.agent_id);

        // 2. Tactical Specialist Gate: If not an orchestrator, and no tools are being called, and mission isn't completed...
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
