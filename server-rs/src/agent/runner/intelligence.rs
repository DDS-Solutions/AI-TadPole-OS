//! Step Execution — Atomic turn logic and tool dispatch
//!
//! Extracted from `runner/mod.rs` for maintainability. All methods remain 
//! on `impl AgentRunner` — this is a structural split, not a behavioral change.
//!
//! @docs ARCHITECTURE:Runner

use crate::agent::types::TaskPayload;

use super::{AgentRunner, IntelligenceOutput, RunContext};

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  INTELLIGENCE LOOP
    // ─────────────────────────────────────────────────────────

    /// Handles the prompt generation, provider calls, and tool execution loop.
    pub(crate) async fn execute_intelligence_loop(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (anyhow::Error, Option<crate::agent::types::TokenUsage>)> {
        let hierarchy_label = if ctx.agent_id == "1" {
            "CEO (Strategic Intelligence Lead)"
        } else if ctx.agent_id == "2" {
            "COO (Operations Director)"
        } else if ctx.agent_id == "alpha" {
            "ALPHA NODE (Swarm Mission Commander)"
        } else {
            match ctx.depth {
                0 => "OVERLORD (Strategic Intelligence Lead)",
                1 => "ORCHESTRATOR (Execution Lead)",
                2 => "CLUSTER COORDINATOR",
                _ => "AGENT (Task Specialist)",
            }
        };

        self.broadcast_sys(
            &format!("Agent {} starting task ({})...", ctx.name, hierarchy_label),
            "info",
            Some(ctx.mission_id.clone()),
        );
        self.update_status(
            &ctx.agent_id,
            &ctx.mission_id,
            "thinking",
            Some("Consulting intelligence model..."),
        );

        self.state
            .yield_phase_transition(&ctx.agent_id, "Execution: Prompt Generation")
            .await;

        let system_prompt = self
            .build_system_prompt(ctx, hierarchy_label, &payload.message)
            .await;
        let swarm_tool = self.build_tools(ctx).await;

        let provider_span = tracing::info_span!("ProviderCall", agent_id = %ctx.agent_id, mission_id = %ctx.mission_id, model = %ctx.model_config.model_id);
        let _provider_guard = provider_span.enter();
        let result = self
            .call_provider(
                ctx,
                &system_prompt,
                &payload.message,
                Some(vec![swarm_tool]),
            )
            .await;
        drop(_provider_guard);
        let (mut output_text, function_calls, mut usage) = match result {
            Ok(data) => data,
            Err(e) => {
                // Return usage so far (none) for budget tracking
                return Err((e, None));
            }
        };

        // Budget Enforcement
        let step_cost = crate::agent::rates::calculate_cost(
            &ctx.model_config.model_id,
            usage.as_ref().map(|u| u.input_tokens).unwrap_or(0),
            usage.as_ref().map(|u| u.output_tokens).unwrap_or(0),
        );

        if let Some(budget_msg) = self
            .check_budget(ctx, step_cost, &output_text)
            .await
            .map_err(|e| (e, usage.clone()))?
        {
            return Ok(IntelligenceOutput {
                text: budget_msg,
                usage,
            });
        }

        self.state
            .yield_phase_transition(&ctx.agent_id, "Execution: Tool Orchestration")
            .await;

        // Tool Execution Loop
        if !function_calls.is_empty() {
            let orbit_span = tracing::info_span!("ToolOrchestration", agent_id = %ctx.agent_id, mission_id = %ctx.mission_id, count = function_calls.len());
            let _orbit_guard = orbit_span.enter();

            use futures::stream::{FuturesUnordered, StreamExt};
            let mut futures = FuturesUnordered::new();
            for fc in function_calls {
                let runner = self.clone();
                let ctx_clone = ctx.clone();
                let user_msg = payload.message.clone();
                futures.push(async move {
                    runner.update_status(
                        &ctx_clone.agent_id,
                        &ctx_clone.mission_id,
                        "working",
                        Some(&format!("Executing tool: {}...", fc.name)),
                    );
                    let mut local_text = String::new();
                    let mut local_usage = None;
                    let result = runner
                        .execute_tool(
                            &ctx_clone,
                            &fc,
                            &mut local_text,
                            &mut local_usage,
                            &user_msg,
                        )
                        .await;
                    (result, local_text, local_usage)
                });
            }

            while let Some(item) = futures.next().await {
                let (result, local_text, local_usage) = item;
                self.accumulate_usage(&mut usage, local_usage);
                output_text.push_str(&local_text);

                if let Err(e) = result {
                    return Err((e, usage));
                }

                if let Ok(Some(early_return)) = result {
                    return Ok(IntelligenceOutput {
                        text: early_return,
                        usage,
                    });
                }
            }
            drop(_orbit_guard);
        }

        Ok(IntelligenceOutput {
            text: output_text,
            usage,
        })
    }
}
