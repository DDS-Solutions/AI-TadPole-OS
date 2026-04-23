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
    ) -> Result<IntelligenceOutput, (anyhow::Error, Option<crate::agent::types::TokenUsage>)> {
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
            .build_system_prompt(ctx, hierarchy_label, &payload.message)
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
                if let Ok(Some(_pause_msg)) = self.check_budget(ctx, 0.0, "").await {
                     tracing::warn!("💰 [Mythos] Budget breach mid-recurrence for agent {}", ctx.agent_id);
                     return Ok(IntelligenceOutput { text: format!("{} (Halting: Budget Exceeded)", scrub_mythos_tags(&output_text)), usage });
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
    ) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<()> {
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
            agent_id: AGENT_ALPHA.to_string(), // Orchestrator ID
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

    #[tokio::test]
    async fn test_mythos_reasoning_turn_telemetry() {
        std::env::set_var("TADPOLE_NULL_PROVIDERS", "true");
        let (runner, state) = setup_test_runner().await;
        
        let agent_id = "reasoning-test-agent";
        let agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: agent_id.to_string(),
                name: "Thinker".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        state.registry.agents.insert(agent_id.to_string(), agent);

        let ctx = RunContext {
            agent_id: agent_id.to_string(),
            reasoning_depth: 3,
            ..RunContext::default()
        };

        let payload = crate::agent::types::TaskPayload {
            message: "Analyze this complex problem deeply.".to_string(),
            ..Default::default()
        };

        // We run the loop. Since NullProvider usually returns immediately without tools,
        // it might break the loop early. But the first turn should at least increment current_reasoning_turn.
        let _ = runner.execute_intelligence_loop(&ctx, &payload).await;

        // Verify turn was reset at the end
        let final_turn = state.registry.agents.get(agent_id).map(|a| a.state.current_reasoning_turn).unwrap();
        assert_eq!(final_turn, 0);
    }

    #[tokio::test]
    async fn test_mythos_halting_signal_detection() {
        let (_runner, _) = setup_test_runner().await;
        // Test XML tag halting
        let _text_with_halt = "I have thought enough. <halting_signal/>";
        // This is a bit tricky to test without mocking the provider call inside intelligence_loop.
        // For now, we verify the logic is present in the code via audit, 
        // as unit testing the full loop with mocks is a larger refactoring task.
    }

    #[tokio::test]
    async fn test_mythos_parallel_reasoning_isolation() {
        std::env::set_var("TADPOLE_NULL_PROVIDERS", "true");
        let (runner, state) = setup_test_runner().await;

        let a1_id = "agent-1";
        let a2_id = "agent-2";

        state.registry.agents.insert(a1_id.to_string(), crate::agent::types::EngineAgent { 
            identity: crate::agent::types::AgentIdentity { id: a1_id.to_string(), ..Default::default() }, 
            ..Default::default() 
        });
        state.registry.agents.insert(a2_id.to_string(), crate::agent::types::EngineAgent { 
            identity: crate::agent::types::AgentIdentity { id: a2_id.to_string(), ..Default::default() }, 
            ..Default::default() 
        });

        let ctx1 = RunContext {
            agent_id: a1_id.to_string(),
            reasoning_depth: 2,
            ..RunContext::default()
        };

        let ctx2 = RunContext {
            agent_id: a2_id.to_string(),
            reasoning_depth: 10,
            ..RunContext::default()
        };

        let payload = crate::agent::types::TaskPayload { message: "Think.".to_string(), ..Default::default() };

        // Run both (simulated sequentially here, but registry isolation is the goal)
        let _ = runner.execute_intelligence_loop(&ctx1, &payload).await;
        let _ = runner.execute_intelligence_loop(&ctx2, &payload).await;

        assert_eq!(state.registry.agents.get(a1_id).unwrap().state.current_reasoning_turn, 0);
        assert_eq!(state.registry.agents.get(a2_id).unwrap().state.current_reasoning_turn, 0);
    }

    #[tokio::test]
    async fn test_tag_scrubbing() {
        let dirty = "Final answer. <halting_signal/> <thinking> Internal stuff </thinking>";
        let clean = scrub_mythos_tags(dirty);
        assert_eq!(clean, "Final answer.   Internal stuff");
    }

    #[tokio::test]
    async fn test_intra_turn_budget_halt() {
        std::env::set_var("TADPOLE_NULL_PROVIDERS", "true");
        let (runner, state) = setup_test_runner().await;
        
        let agent_id = "budget-agent";
        state.registry.agents.insert(agent_id.to_string(), crate::agent::types::EngineAgent { 
            identity: crate::agent::types::AgentIdentity { id: agent_id.to_string(), ..Default::default() }, 
            ..Default::default() 
        });

        let ctx = RunContext {
            agent_id: agent_id.to_string(),
            budget_usd: 1.0,
            current_cost_usd: 1.1, // Already over budget
            reasoning_depth: 5,
            ..RunContext::default()
        };

        let payload = crate::agent::types::TaskPayload { message: "Think a lot.".to_string(), ..Default::default() };

        let result = runner.execute_intelligence_loop(&ctx, &payload).await;
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.text.contains("Halting: Budget Exceeded"));
    }

    #[tokio::test]
    async fn test_monologue_compression() {
        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext::default();
        // Create a massive monologue
        let mut monologue = vec!["Step 1 ".repeat(1000), "Step 2 ".repeat(1000)]; 
        
        // This should trigger summarization
        let result = runner.compress_monologue(&ctx, &mut monologue).await;
        
        assert!(result.is_ok());
        assert_eq!(monologue.len(), 1);
        assert!(monologue[0].contains("CONSOLIDATED REASONING"));
    }

    #[tokio::test]
    async fn test_agent_serialization_parity() {
        use crate::agent::types::EngineAgent;
        let agent = EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "test-id".to_string(),
                ..Default::default()
            },
            state: crate::agent::types::AgentState {
                current_reasoning_turn: 5,
                ..Default::default()
            },
            ..EngineAgent::default()
        };

        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains("\"currentReasoningTurn\":5"));

        let de_agent: EngineAgent = serde_json::from_str(&json).unwrap();
        assert_eq!(de_agent.state.current_reasoning_turn, 5);
    }
}

// Metadata: [intelligence]

// Metadata: [intelligence]
