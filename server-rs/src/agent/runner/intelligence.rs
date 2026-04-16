//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Intelligence Loop**: The heartbeat of an agent turn. Manages the
//! **Think->Act->Respond** cycle. Handles automatic hierarchy labeling
//! (CEO/COO/Alpha) and orchestrates concurrent tool execution using
//! `FuturesUnordered`. Enforces real-time **Financial Guardrails** (SEC-02)
//! by calculating neural costs per step.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Provider API timeout, tool execution panic, budget breach
//!   during a long chain, or tokenizer failure during prompt assembly.
//! - **Trace Scope**: `server-rs::agent::runner::intelligence`

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
        let hierarchy_label = if ctx.agent_id == "1" || ctx.role.to_lowercase().contains("ceo") {
            "CEO (Strategic Intelligence Lead)"
        } else if ctx.agent_id == "2" || ctx.role.to_lowercase().contains("coo") {
            "COO (Operations Director)"
        } else if ctx.agent_id == "alpha" || ctx.name.to_lowercase().contains("alpha") {
            "ALPHA NODE (Swarm Mission Commander)"
        } else {
            "AGENT (Task Specialist)"
        };

        self.broadcast_agent(
            ctx,
            &format!("starting task ({})...", hierarchy_label),
            "info",
        );
        self.update_status(
            &ctx.agent_id,
            &ctx.mission_id,
            "thinking",
            Some("Consulting intelligence model..."),
        );

        let system_prompt = self
            .build_system_prompt(ctx, hierarchy_label, &payload.message)
            .await;

        let mut output_text = String::new();
        let mut usage: Option<crate::agent::types::TokenUsage> = None;
        let mut turn_count = 0;
        let mut conversation_history = Vec::new();
        conversation_history.push(format!("USER: {}", payload.message));
        pub(crate) const MAX_TURNS: u32 = 10;

        while turn_count < MAX_TURNS {
            turn_count += 1;
            tracing::debug!(
                "🎯 [Intelligence] Start Turn {}/{} for agent {}",
                turn_count,
                MAX_TURNS,
                ctx.agent_id
            );

            self.state
                .yield_phase_transition(&ctx.agent_id, &format!("Execution: Turn {}", turn_count))
                .await;

            let swarm_tool = self.build_tools(ctx).await;

            // Build the current conversation history for the provider
            let current_prompt = conversation_history.join("\n\n");

            let provider_res = self
                .call_provider(
                    ctx,
                    &system_prompt,
                    &current_prompt,
                    Some(vec![swarm_tool.clone()]),
                )
                .await;

            let (mut turn_text, mut function_calls, turn_usage) = match provider_res {
                Ok(data) => data,
                Err(e) => return Err((e, usage)),
            };
            self.accumulate_usage(&mut usage, turn_usage);

            // 🛡️ [Sentinel Gate]
            // Intercept narrative leaks and force tactical autonomy.
            self.enforce_sentinel_gate(
                ctx,
                &system_prompt,
                &current_prompt,
                &mut turn_text,
                &mut function_calls,
                &mut usage,
            )
            .await
            .map_err(|e| (e, usage.clone()))?;

            // Accumulate conversational output (if any allowed)
            if !turn_text.is_empty() {
                if !output_text.is_empty() {
                    output_text.push_str("\n\n");
                }
                output_text.push_str(&turn_text);

                // Add to turn history for next turn's context
                conversation_history.push(format!("ASSISTANT: {}", turn_text));
            }

            // Financial Guardrail
            let cumulative_cost = crate::agent::rates::calculate_cost(
                &ctx.model_config.model_id,
                usage.as_ref().map(|u| u.input_tokens).unwrap_or(0),
                usage.as_ref().map(|u| u.output_tokens).unwrap_or(0),
            );
            if let Some(budget_msg) = self
                .check_budget(ctx, cumulative_cost, &output_text)
                .await
                .map_err(|e| (e, usage.clone()))?
            {
                return Ok(IntelligenceOutput {
                    text: budget_msg,
                    usage,
                });
            }

            if function_calls.is_empty() {
                tracing::debug!(
                    "🏁 [Intelligence] No tool calls for agent {}, breaking loop.",
                    ctx.agent_id
                );
                break;
            }

            // Tool Execution (Concurrent)
            let orbit_span = tracing::info_span!("ToolOrchestration", agent_id = %ctx.agent_id, count = function_calls.len());
            let _orbit_guard = orbit_span.enter();

            use futures::stream::{FuturesUnordered, StreamExt};
            let mut futures = FuturesUnordered::new();
            for fc in function_calls {
                let runner = self.clone();
                let ctx_clone = ctx.clone();
                let user_msg_clone = payload.message.clone(); // Original mission intent
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
                            &user_msg_clone,
                        )
                        .await;
                    (fc.name, result, local_text, local_usage)
                });
            }

            let mut observation_buffer = String::new();
            let mut mission_completed = false;
            let mut final_report = None;

            while let Some((name, result, local_text, local_usage)) = futures.next().await {
                self.accumulate_usage(&mut usage, local_usage);

                // Prioritize the tool output for the next turn's observation
                observation_buffer.push_str(&format!("\nTool {} Result: {}", name, local_text));

                if let Err(e) = result {
                    return Err((e, usage));
                }

                // If complete_mission was called, capture the formatted report
                if name == "complete_mission" {
                    mission_completed = true;
                    final_report = Some(local_text);
                } else if let Ok(Some(_)) = result {
                    mission_completed = true;
                }
            }
            drop(_orbit_guard);

            if !observation_buffer.is_empty() {
                conversation_history.push(format!("OBSERVATION: {}", observation_buffer));
            }

            if mission_completed {
                if let Some(report) = final_report {
                    if !output_text.is_empty() && !report.trim().is_empty() {
                        output_text.push_str("\n\n---\n## Final Report\n");
                    }
                    output_text.push_str(&report);
                }
                tracing::info!(
                    "🏁 [Intelligence] Agent {} signaled mission completion.",
                    ctx.agent_id
                );
                break;
            }
        }

        Ok(IntelligenceOutput {
            text: output_text,
            usage,
        })
    }

    /// Enforces the Sentinel Gate protocol: Specialist agents are forbidden from text-only turns.
    async fn enforce_sentinel_gate(
        &self,
        ctx: &RunContext,
        system_prompt: &str,
        user_directive: &str,
        output_text: &mut String,
        function_calls: &mut Vec<crate::agent::types::GeminiFunctionCall>,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> anyhow::Result<()> {
        let is_orchestrator = ctx.agent_id == "1" 
            || ctx.agent_id == "2" 
            || ctx.agent_id == "alpha"
            || ctx.role.to_lowercase().contains("ceo")
            || ctx.role.to_lowercase().contains("coo")
            || ctx.name.to_lowercase().contains("alpha");

        // If not an orchestrator, and no tools are being called, and mission isn't completed...
        // 🚨 OVERLORD BYPASS: If safe_mode is active, we allow specialists to be conversational.
        if !is_orchestrator
            && !ctx.safe_mode
            && function_calls.is_empty()
            && !output_text.contains("complete_mission")
        {
            tracing::warn!("🛡️ [Sentinel] Specialist {} attempted narrative leak. Enforcing tactical autonomy...", ctx.agent_id);

            let sentinel_directive = format!(
                "SYSTEM_SENTINEL: Your turn resulted in a narrative-only response. As an AGENT (Task Specialist), you are FORBIDDEN from text-only progress reports or roadblock apologies. \
                 You MUST execute tools or call 'complete_mission' with results. Directive: {}", 
                user_directive
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

            if let Ok((sent_text, sent_calls, sent_usage)) = sentinel_result {
                *output_text = sent_text;
                *function_calls = sent_calls;
                self.accumulate_usage(usage, sent_usage);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_test_runner() -> (AgentRunner, Arc<AppState>) {
        let state = Arc::new(AppState::default());
        let runner = AgentRunner::new(state.clone());
        (runner, state)
    }

    #[tokio::test]
    async fn test_enforce_sentinel_gate_specialist_narrative() {
        // Prepare environment for NullProvider
        std::env::set_var("TADPOLE_NULL_PROVIDERS", "true");

        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext {
            agent_id: "weather_api".to_string(), // Specialist ID
            ..RunContext::default()
        };

        let mut output_text = "I am thinking about the weather...".to_string();
        let mut calls = Vec::new();
        let mut usage = None;

        // The gate should trigger because it's a specialist (not 1, 2, or alpha)
        // and calls is empty and not complete_mission.
        let result = runner
            .enforce_sentinel_gate(
                &ctx,
                "System",
                "Report status",
                &mut output_text,
                &mut calls,
                &mut usage,
            )
            .await;

        assert!(result.is_ok());
        // NullProvider will return a [DEGRADED] message, which replaces the narrative text.
        assert!(output_text.contains("DEGRADED"));
    }

    #[tokio::test]
    async fn test_enforce_sentinel_gate_orchestrator_bypass() {
        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext {
            agent_id: "alpha".to_string(), // Orchestrator ID
            ..RunContext::default()
        };

        let mut output_text = "I am Alpha, I can be conversational.".to_string();
        let mut calls = Vec::new();
        let mut usage = None;

        let _ = runner
            .enforce_sentinel_gate(
                &ctx,
                "System",
                "Report status",
                &mut output_text,
                &mut calls,
                &mut usage,
            )
            .await;

        // Should NOT have changed the text because orchestrators are allowed narrative.
        assert_eq!(output_text, "I am Alpha, I can be conversational.");
    }
}
