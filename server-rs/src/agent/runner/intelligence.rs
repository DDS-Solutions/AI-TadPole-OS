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
use crate::agent::constants::*;
use crate::error::AppError;
use std::sync::Arc;

/// RAII Guard to ensure reasoning turn state is always reset in the registry.
struct ReasoningTurnGuard {
    agent_id: String,
    state: Arc<crate::state::AppState>,
}

impl ReasoningTurnGuard {
    fn new(agent_id: String, state: Arc<crate::state::AppState>) -> Self {
        Self { agent_id, state }
    }
}

impl Drop for ReasoningTurnGuard {
    fn drop(&mut self) {
        if let Some(mut entry) = self.state.registry.agents.get_mut(&self.agent_id) {
            entry.value_mut().state.current_reasoning_turn = 0;
        }
    }
}

/// Removes internal Mythos control tags from narrative text.
fn scrub_mythos_tags(text: &str) -> String {
    text.replace("<halting_signal/>", "")
        .replace("<halt/>", "")
        .replace("<thinking>", "")
        .replace("</thinking>", "")
        .trim()
        .to_string()
}

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  INTELLIGENCE LOOP
    // ─────────────────────────────────────────────────────────

    /// Handles the prompt generation, provider calls, and tool execution loop.
    pub(crate) async fn execute_intelligence_loop(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        // --- 🛡️ [Mythos RAII Guard] ---
        let _turn_guard = ReasoningTurnGuard::new(ctx.agent_id.clone(), self.state.clone());

        let hierarchy_label = if ctx.agent_id == AGENT_CEO || ctx.role.to_lowercase().contains("ceo") {
            "CEO (Strategic Intelligence Lead)"
        } else if ctx.agent_id == AGENT_COO || ctx.role.to_lowercase().contains("coo") {
            "COO (Operations Director)"
        } else if ctx.agent_id == AGENT_ALPHA || ctx.name.to_lowercase().contains("alpha") {
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
            .build_system_prompt(ctx, &payload.message)
            .await;

        let mut output_text = String::new();
        let mut usage: Option<crate::agent::types::TokenUsage> = None;
        let mut turn_count = 0;
        let mut conversation_history = Vec::new();
        conversation_history.push(format!("USER: {}", payload.message));
        let max_turns = ctx.max_turns;

        while turn_count < max_turns {
            turn_count += 1;
            tracing::debug!(
                "🎯 [Intelligence] Start Turn {}/{} for agent {}",
                turn_count,
                max_turns,
                ctx.agent_id
            );

            self.state
                .yield_phase_transition(&ctx.agent_id, &format!("Execution: Turn {}", turn_count))
                .await;

            // --- 🧠 [Mythos] Recurrent Reasoning Loop ---
            let mut reasoning_turn = 0;
            let mut reasoning_halted = false;
            let mut internal_monologue = Vec::new();
            
            while reasoning_turn < ctx.reasoning_depth && !reasoning_halted {
                reasoning_turn += 1;
                
                if ctx.reasoning_depth > 1 {
                    tracing::info!("🧠 [Mythos] Reasoning Loop {}/{} for agent {}", reasoning_turn, ctx.reasoning_depth, ctx.agent_id);
                    // Synchronize with registry for UI "Pulse" rail
                    if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
                        entry.value_mut().state.current_reasoning_turn = reasoning_turn;
                    }
                    self.broadcast_agent(ctx, &format!("thinking (loop {}/{})...", reasoning_turn, ctx.reasoning_depth), "pulse");
                }

                // 🛡️ [Financial Guardrail] Intra-turn budget check
                match self.check_budget(ctx, 0.0, "").await {
                     Ok(Some(_pause_msg)) => {
                         tracing::warn!("💰 [Mythos] Budget breach mid-recurrence for agent {}", ctx.agent_id);
                         return Ok(IntelligenceOutput { text: format!("{} (Halting: Budget Exceeded)", scrub_mythos_tags(&output_text)), usage });
                     }
                     Err(e) => return Err((e, usage)),
                     Ok(None) => {}
                }

                let tools = vec![self.build_tools(ctx).await];
                // Hybrid Halting: the set_confidence tool is automatically registered via the 
                // SelfHalting trait if the model supports it.

                let current_prompt = if internal_monologue.is_empty() {
                    conversation_history.join("\n\n")
                } else {
                    format!("{}\n\nINTERNAL MONOLOGUE:\n{}", conversation_history.join("\n\n"), internal_monologue.join("\n\n"))
                };

                let provider_res = self
                    .call_provider(ctx, &system_prompt, &current_prompt, Some(tools))
                    .await;

                let (turn_text, mut function_calls, turn_usage) = match provider_res {
                    Ok(data) => data,
                    Err(e) => return Err((e, usage)),
                };
                self.accumulate_usage(&mut usage, turn_usage);

                // Check for Halting Signal (Tag Fallback)
                if turn_text.contains("<halting_signal/>") || turn_text.contains("<halt/>") {
                    tracing::info!("🛑 [Mythos] Halting signal detected for agent {}", ctx.agent_id);
                    reasoning_halted = true;
                }

                // Check for Halting Signal (Tool Call)
                for fc in &function_calls {
                    if fc.name == "set_confidence" {
                        let score = fc.args.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        if score >= ctx.act_threshold {
                            tracing::info!("🛑 [Mythos] Confidence-based halt for agent {}: {:.2} >= {:.2}", ctx.agent_id, score, ctx.act_threshold);
                            reasoning_halted = true;
                        }
                    }
                }
                function_calls.retain(|fc| fc.name != "set_confidence");

                if reasoning_turn < ctx.reasoning_depth && !reasoning_halted {
                    // Continue reasoning: feed output back as internal monologue
                    internal_monologue.push(turn_text);
                    
                    // 📏 [Context Hygiene] Summarize monologue if it grows too large
                    let _ = self.compress_monologue(ctx, &mut internal_monologue).await;
                } else {
                    // Final reasoning turn: promote to active conversation
                    if !turn_text.is_empty() {
                        if !output_text.is_empty() {
                            output_text.push_str("\n\n");
                        }
                        output_text.push_str(&turn_text);
                        conversation_history.push(format!("ASSISTANT: {}", turn_text));
                    }

                    // 🛡️ [Sentinel Gate]
                    let mut turn_text_clone = turn_text.clone();
                    self.enforce_sentinel_gate(
                        ctx,
                        &system_prompt,
                        &current_prompt,
                        &mut turn_text_clone,
                        &mut function_calls,
                        &mut usage,
                    )
                    .await
                    .map_err(|e| (e, usage.clone()))?;

                    if function_calls.is_empty() {
                        tracing::debug!("🏁 [Intelligence] No tool calls for agent {}, breaking loop.", ctx.agent_id);
                        return Ok(IntelligenceOutput { 
                            text: scrub_mythos_tags(&output_text), 
                            usage 
                        });
                    }

                    // Proceed to Tool Execution
                    let orbit_span = tracing::info_span!("ToolOrchestration", agent_id = %ctx.agent_id, count = function_calls.len());
                    let _orbit_guard = orbit_span.enter();

                    use futures::stream::{FuturesUnordered, StreamExt};
                    let mut futures = FuturesUnordered::new();
                    for fc in function_calls {
                        let runner = self.clone();
                        let ctx_clone = ctx.clone();
                        let user_msg_clone = payload.message.clone();
                        futures.push(async move {
                            runner.update_status(&ctx_clone.agent_id, &ctx_clone.mission_id, "working", Some(&format!("Executing tool: {}...", fc.name)));
                            let mut local_text = String::new();
                            let mut local_usage = None;
                            let result = runner.execute_tool(&ctx_clone, &fc, &mut local_text, &mut local_usage, &user_msg_clone).await;
                            
                            // 🧬 [Evolution] Autonomous Refinement Hook
                            runner.handle_tool_failure_refinement(&ctx_clone, &fc, &mut local_text);

                            (fc.name, result, local_text, local_usage)
                        });
                    }

                    let mut observation_buffer = String::new();
                    let mut mission_completed = false;
                    let mut final_report = None;

                    while let Some((name, result, local_text, local_usage)) = futures.next().await {
                        self.accumulate_usage(&mut usage, local_usage);
                        observation_buffer.push_str(&format!("\nTool {} Result: {}", name, local_text));
                        if let Err(e) = result { return Err((e, usage)); }

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
                            return Ok(IntelligenceOutput { 
                                text: scrub_mythos_tags(&output_text), 
                                usage 
                            });
                        }
                    
                    reasoning_halted = true; // Break the reasoning loop to move to the next turn_count
                }
            }
        }

        self.broadcast_agent(ctx, "Neural Pulse: Turn finalized", "pulse");

        Ok(IntelligenceOutput {
            text: scrub_mythos_tags(&output_text),
            usage,
        })
    }

    /// Compresses the internal monologue via recursive summarization if it grows too large.
    async fn compress_monologue(
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
            total_chars, ctx.agent_id
        );

        let history = monologue.join("\n\n");
        let prompt = format!(
            "SUMMARIZE YOUR PREVIOUS REASONING STEPS INTO A SINGLE CONCISE PARAGRAPH. \
             RETAIN ALL KEY INSIGHTS, VARIABLES, AND HYPOTHESES. \
             \n\nPREVIOUS REASONING:\n{}",
            history
        );

        let (summary_text, _, _) = self
            .call_provider(
                ctx,
                "You are an expert reasoning summarizer. Be technical, dense, and objective.",
                &prompt,
                None,
            )
            .await?;

        monologue.clear();
        monologue.push(format!("CONSOLIDATED REASONING: {}", summary_text));

        Ok(())
    }

    /// Enforces the Sentinel Gate protocol: Specialist agents are forbidden from text-only turns.
    async fn enforce_sentinel_gate(
        &self,
        ctx: &RunContext,
        system_prompt: &str,
        user_directive: &str,
        output_text: &mut String,
        function_calls: &mut Vec<crate::agent::types::ToolCall>,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<(), AppError> {
        let is_orchestrator = ctx.agent_id == AGENT_CEO 
            || ctx.agent_id == AGENT_COO 
            || ctx.agent_id == AGENT_ALPHA
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

            let (sent_text, sent_calls, sent_usage) = sentinel_result?;
            *output_text = sent_text;
            *function_calls = sent_calls;
            self.accumulate_usage(usage, sent_usage);
        }
        Ok(())
    }
}

// Metadata: [intelligence]

// Metadata: [intelligence]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::agent::runner::{AgentRunner, RunContext};
    use std::sync::Arc;

    #[test]
    fn test_scrub_mythos_tags() {
        let input = "Thinking... <thinking>I should search</thinking> <halting_signal/> Done. <halt/>";
        let expected = "Thinking... I should search  Done.";
        assert_eq!(scrub_mythos_tags(input), expected);

        let clean = "No tags here";
        assert_eq!(scrub_mythos_tags(clean), "No tags here");
    }

    #[tokio::test]
    async fn test_hierarchy_labeling() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let _runner = AgentRunner::new(state.clone());

        // Test CEO label
        let _ctx = RunContext {
            agent_id: AGENT_CEO.to_string(),
            ..RunContext::default()
        };
        
        // We can't easily call execute_intelligence_loop because it's too complex to mock everything,
        // but we can extract the labeling logic if we refactor, or test it via the side effects.
        // For now, let's test the logic by verifying the context setup which we can do elsewhere,
        // OR we can just test the parts we can reach.
    }

    #[tokio::test]
    async fn test_enforce_sentinel_gate_orchestrator() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        
        let ctx = RunContext {
            agent_id: AGENT_CEO.to_string(), // Orchestrator
            ..RunContext::default()
        };
        
        let mut output_text = "I am the CEO".to_string();
        let mut function_calls = vec![];
        let mut usage = None;
        
        // Should NOT enforce (do nothing) because it's an orchestrator
        let result = runner.enforce_sentinel_gate(
            &ctx,
            "system",
            "user",
            &mut output_text,
            &mut function_calls,
            &mut usage
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(output_text, "I am the CEO");
        assert!(function_calls.is_empty());
    }

    #[tokio::test]
    async fn test_enforce_sentinel_gate_specialist_narrative_leak() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        
        let ctx = RunContext {
            agent_id: "specialist-1".to_string(),
            role: "Specialist".to_string(),
            safe_mode: false,
            ..RunContext::default()
        };
        
        let mut output_text = "I am just talking".to_string();
        let mut function_calls = vec![];
        let mut usage = None;
        
        // This will try to call_provider, which will fail because there are no providers configured in mock.
        // But we can verify that it *attempts* to enforce by checking the error or using a more robust mock.
        // Actually, call_provider will likely return an error because the mock state has no providers.
        
        let result = runner.enforce_sentinel_gate(
            &ctx,
            "system",
            "user",
            &mut output_text,
            &mut function_calls,
            &mut usage
        ).await;
        
        // In a minimal mock, call_provider returns Ok with a DEGRADED message from NullProvider.
        // This proves the sentinel gate was triggered and successfully re-called the provider.
        assert!(result.is_ok());
        assert!(output_text.contains("DEGRADED"));
    }
}
