//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Turn**: Core reasoning turn runner, planning phase dispatcher, and telemetry broadcaster.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Model token budget breach, connection timeouts, or telemetry send failures.
//!

use crate::error::AppError;
use crate::agent::runner::{AgentRunner, IntelligenceOutput, RunContext};
use crate::agent::types::TaskPayload;
use super::sentinel::scrub_mythos_tags;
use super::TurnOutcome;

impl AgentRunner {
    pub(crate) async fn handle_specification_generation_phase(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
        state_tx: &mut crate::agent::runner::service_traits::StateTransaction,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        let mut active_ctx = ctx.clone();

        // --- [System 2] Planning Slot Activation (Fix 9: shared helper) ---
        // CHAT-DEFAULT: When the payload explicitly pins the "default" slot (e.g.
        // direct UI chat requests), skip the automatic Planning slot switch so the
        // agent uses its primary modelConfig instead of any configured cloud slot.
        let _slot_kind = if payload.active_model_slot.as_deref() == Some("default") {
            super::super::service_traits::SlotKind::Default
        } else {
            self.activate_model_slot(
                ctx,
                &mut active_ctx,
                super::super::service_traits::SlotKind::Planning,
                "Planning",
            )
        };

        let system_prompt = self
            .prompt_service
            .build_system_prompt(self, &active_ctx, &payload.message)
            .await;

        self.broadcast_agent(ctx, "Phase: Specification (System 2 Thinking)", "info");
        self.update_status(
            &ctx.agent_id,
            &ctx.mission_id,
            "thinking",
            Some("Generating Unified Technical Specification (UTS)..."),
        );

        let spec_prompt = format!(
            "Generate a Unified Technical Specification (UTS) for this mission based on the provided Codebase Map.\n\n\
             ### MISSION OBJECTIVE:\n\
             {}\n\n\
             ### TASK:\n\
             1. Review the Codebase Map in your context.\n\
             2. Outline planned changes, architectural impact, and verification plan.\n\
             3. Provide the spec in clean Markdown format.\n\
             Do NOT write code yet. Respond with the UTS only.",
            payload.message
        );

        let mut usage: Option<crate::agent::types::TokenUsage> = None;
        let (spec_content, _, spec_usage) = self
            .call_provider(&active_ctx, &system_prompt, &spec_prompt, None)
            .await
            .map_err(|e| (e, usage.clone()))?;
        self.accumulate_usage(&mut usage, spec_usage);

        crate::agent::mission::set_mission_spec(
            &self.state.resources.pool,
            &ctx.mission_id,
            &ctx.agent_id,
            &spec_content,
        )
        .await
        .map_err(|e| (e, usage.clone()))?;

        state_tx.record_mission_spec_change(&ctx.mission_id, &ctx.agent_id);

        Ok(IntelligenceOutput {
            text: format!("## Unified Technical Specification (UTS) Generated\n\n{}\n\n> [!IMPORTANT]\n> Mission status set to **Spec Review**. Please approve the spec before execution proceeds.", spec_content),
            usage,
        })
    }

    pub(crate) fn is_fast_path_query(message: &str) -> bool {
        let lower = message.to_lowercase();
        let complex_triggers = [
            "implement",
            "build",
            "refactor",
            "debug",
            "compile",
            "run tests",
            "migrate",
            "deploy",
            "stress test",
            "benchmark",
            "integrate",
            "solve",
        ];
        for trigger in &complex_triggers {
            if lower.contains(trigger) {
                return false;
            }
        }
        let simple_triggers = [
            "what is",
            "get status",
            "read file",
            "list files",
            "check budget",
            "show metrics",
            "get metrics",
            "health",
            "system details",
            "who are you",
        ];
        let is_short = message.len() < 150;
        is_short
            || simple_triggers
                .iter()
                .any(|trigger| lower.contains(trigger))
    }

    pub(crate) async fn handle_reasoning_phase(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        let mut active_ctx = ctx.clone();

        let is_fast_path = Self::is_fast_path_query(&payload.message);
        if is_fast_path {
            tracing::info!(
                "⚡ [Fast-Path] Simple/read query detected. Engaging System 1 fast execution."
            );
            active_ctx.reasoning_depth = 1;
        }

        let active_slot_kind = self.activate_model_slot(
            ctx,
            &mut active_ctx,
            super::super::service_traits::SlotKind::Execution,
            "Tactical Execution",
        );

        let system_prompt = self
            .prompt_service
            .build_system_prompt(self, &active_ctx, &payload.message)
            .await;

        let mut output_text = String::new();
        // Fix 11: Initialize with zero-value struct to eliminate None checks in callers
        let mut usage: Option<crate::agent::types::TokenUsage> =
            Some(crate::agent::types::TokenUsage::default());

        let mut turn_count = 0;
        let mut conversation_history = Vec::new();
        if let Some(ref vt) = active_ctx.visible_transcript {
            let guard = vt.lock();
            for entry in guard.iter() {
                conversation_history.push(entry.clone());
            }
        }
        conversation_history.push(format!("USER: {}", payload.message));
        let max_turns = if is_fast_path { 2 } else { ctx.max_turns };
        let mut parent_id: Option<String> = None;

        while turn_count < max_turns {
            turn_count += 1;
            match self
                .run_agent_turn(
                    ctx,
                    &mut active_ctx,
                    active_slot_kind,
                    &system_prompt,
                    payload,
                    turn_count,
                    &mut conversation_history,
                    &mut output_text,
                    &mut usage,
                    &mut parent_id,
                    is_fast_path,
                )
                .await?
            {
                TurnOutcome::Continue => continue,
                TurnOutcome::Completed(out) | TurnOutcome::BudgetExceeded(out) => return Ok(out),
            }
        }

        // --- 🤖 [Final Conversational Recovery Turn] ---
        if output_text.is_empty()
            && conversation_history
                .last()
                .map(|s| s.starts_with("OBSERVATION:"))
                .unwrap_or(false)
        {
            tracing::info!("🔄 [Intelligence] Reasoning loop terminated on tool observation. Executing final conversational recovery turn.");
            let current_prompt = conversation_history.join("\n\n");

            // Execute provider call with no tools to force a conversational completion text response
            if let Ok((turn_text, _, turn_usage)) = self
                .call_provider(&active_ctx, &system_prompt, &current_prompt, None)
                .await
            {
                self.accumulate_usage(&mut usage, turn_usage);
                output_text = scrub_mythos_tags(&turn_text);
            }
        }

        self.broadcast_agent(ctx, "Neural Pulse: Turn finalized", "pulse");

        Ok(IntelligenceOutput {
            text: scrub_mythos_tags(&output_text),
            usage,
        })
    }

    /// Executes a single Think -> Act -> Observe cycle (Fix 4: Core Refactoring Mandate).
    pub(crate) async fn run_agent_turn(
        &self,
        ctx: &RunContext,
        active_ctx: &mut RunContext,
        active_slot_kind: super::super::service_traits::SlotKind,
        system_prompt: &str,
        payload: &TaskPayload,
        turn_count: u32,
        conversation_history: &mut Vec<String>,
        output_text: &mut String,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        parent_id: &mut Option<String>,
        is_fast_path: bool,
    ) -> Result<TurnOutcome, (AppError, Option<crate::agent::types::TokenUsage>)> {
        tracing::debug!(
            "🎯 [Intelligence] Start Turn {}/{} for agent {}",
            turn_count,
            ctx.max_turns,
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
                tracing::info!(
                    "🧠 [Mythos] Reasoning Loop {}/{} for agent {}",
                    reasoning_turn,
                    ctx.reasoning_depth,
                    ctx.agent_id
                );
                // Synchronize with registry for UI "Pulse" rail
                if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
                    entry.value_mut().state.current_reasoning_turn = reasoning_turn;
                }
                self.broadcast_agent(
                    ctx,
                    &format!(
                        "thinking (loop {}/{})...",
                        reasoning_turn, ctx.reasoning_depth
                    ),
                    "pulse",
                );
            }

            // 🛡️ [Financial Guardrail] Intra-turn budget check
            match self.check_budget(ctx, 0.0, "").await {
                Ok(Some(_pause_msg)) => {
                    tracing::warn!(
                        "💰 [Mythos] Budget breach mid-recurrence for agent {}",
                        ctx.agent_id
                    );
                    return Ok(TurnOutcome::BudgetExceeded(IntelligenceOutput {
                        text: format!(
                            "{} (Halting: Budget Exceeded)",
                            scrub_mythos_tags(output_text)
                        ),
                        usage: usage.clone(),
                    }));
                }
                Err(e) => return Err((e, usage.clone())),
                Ok(None) => {}
            }

            let tools = vec![self.build_tools(active_ctx).await];

            let current_prompt = if internal_monologue.is_empty() {
                conversation_history.join("\n\n")
            } else {
                format!(
                    "{}\n\nINTERNAL MONOLOGUE:\n{}",
                    conversation_history.join("\n\n"),
                    internal_monologue.join("\n\n")
                )
            };

            // Fix 7: Retry on Transient errors with exponential backoff
            let mut retries = 0;
            let turn_start = std::time::Instant::now();
            let (turn_text, mut function_calls, turn_usage) = loop {
                let provider_res = self
                    .call_provider(
                        active_ctx,
                        system_prompt,
                        &current_prompt,
                        Some(tools.clone()),
                    )
                    .await;
                match provider_res {
                    Ok(data) => break data,
                    Err(e) => {
                        if e.error_class() == crate::error::ErrorClass::Transient && retries < 2 {
                            retries += 1;
                            let delay = std::time::Duration::from_millis(500 * (1 << retries));
                            tracing::warn!(
                                "⚡ [Intelligence] Transient error, retrying in {:?} (retry {}/2): {:?}",
                                delay, retries, e
                            );
                            tokio::time::sleep(delay).await;
                        } else {
                            return Err((e, usage.clone()));
                        }
                    }
                }
            };
            let turn_latency_ms = turn_start.elapsed().as_millis() as u64;
            self.accumulate_usage(usage, turn_usage.clone());

            // --- 📊 [System 2] Telemetry: Reasoning Step ---
            let current_step_id = uuid::Uuid::new_v4().to_string();
            let slot = match active_slot_kind {
                super::super::service_traits::SlotKind::Planning => "planning",
                super::super::service_traits::SlotKind::Execution => "execution",
                super::super::service_traits::SlotKind::Default => "default",
            };

            // Gap 1: derive finish_reason and pass enriched context
            let finish_reason = if turn_text.contains("<halting_signal/>") || turn_text.contains("<halt/>") {
                "halted"
            } else if function_calls.is_empty() {
                "stop"
            } else {
                "tool_use"
            };

            self.broadcast_reasoning_step(
                ctx,
                &current_step_id,
                parent_id.clone(),
                &turn_text,
                &active_ctx.model_config.model_id,
                slot,
                reasoning_turn as usize,
                turn_latency_ms,
                &turn_usage,
                finish_reason,
            );

            // Gap 1: Emit a dedicated tool-step event after tools are identified
            if !function_calls.is_empty() {
                let tool_names: Vec<String> = function_calls.iter().map(|fc| fc.name.clone()).collect();
                self.broadcast_tool_step(ctx, parent_id.clone(), tool_names);
            }

            *parent_id = Some(current_step_id);

            // Check for Halting Signal (Tag Fallback)
            if turn_text.contains("<halting_signal/>") || turn_text.contains("<halt/>") {
                tracing::info!(
                    "🛑 [Mythos] Halting signal detected for agent {}",
                    ctx.agent_id
                );
                reasoning_halted = true;
            }

            // Check for Halting Signal (Tool Call)
            for fc in &function_calls {
                if fc.name == "set_confidence" {
                    let score = fc.args.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    if score >= ctx.act_threshold {
                        tracing::info!(
                            "🛑 [Mythos] Confidence-based halt for agent {}: {:.2} >= {:.2}",
                            ctx.agent_id,
                            score,
                            ctx.act_threshold
                        );
                        reasoning_halted = true;
                    }
                }
            }
            function_calls.retain(|fc| fc.name != "set_confidence");

            if reasoning_turn < ctx.reasoning_depth && !reasoning_halted {
                // Continue reasoning: feed output back as internal monologue
                internal_monologue.push(turn_text);

                // [Context Hygiene] Summarize monologue if it grows too large
                // Fix 13: Log compression failures instead of silently swallowing
                if let Err(e) = self.compress_monologue(ctx, &mut internal_monologue).await {
                    tracing::warn!(
                        "[Mythos] Monologue compression failed for agent {}: {:?}",
                        ctx.agent_id,
                        e
                    );
                }
            } else {
                // Final reasoning turn: promote to active conversation
                if !turn_text.is_empty() {
                    if !output_text.is_empty() {
                        output_text.push_str("\n\n");
                    }
                    output_text.push_str(&turn_text);
                    conversation_history.push(format!("ASSISTANT: {}", turn_text));

                    let clean = scrub_mythos_tags(&turn_text);
                    if let Some(ref vt) = active_ctx.visible_transcript {
                        vt.lock().push(format!("ASSISTANT: {}", clean));
                    }
                }

                // 🛡️ [Sentinel Gate]
                let mut turn_text_clone = turn_text.clone();
                if !is_fast_path {
                    self.enforce_sentinel_gate(
                        active_ctx,
                        system_prompt,
                        &current_prompt,
                        &mut turn_text_clone,
                        &mut function_calls,
                        usage,
                    )
                    .await
                    .map_err(|e| (e, usage.clone()))?;
                }

                if function_calls.is_empty() {
                    tracing::debug!(
                        "🏁 [Intelligence] No tool calls for agent {}, breaking loop.",
                        ctx.agent_id
                    );
                    return Ok(TurnOutcome::Completed(IntelligenceOutput {
                        text: scrub_mythos_tags(output_text),
                        usage: usage.clone(),
                    }));
                }

                // Proceed to Tool Execution
                let orbit_span = tracing::info_span!("ToolOrchestration", agent_id = %ctx.agent_id, count = function_calls.len());
                let _orbit_guard = orbit_span.enter();

                let orch_res = self
                    .tool_orchestrator
                    .execute_tools(
                        std::sync::Arc::new(self.clone()),
                        active_ctx,
                        function_calls,
                        &payload.message,
                        usage,
                    )
                    .await
                    .map_err(|e| (e, usage.clone()))?;

                drop(_orbit_guard);

                if let Some(ref slot) = orch_res.active_slot_override {
                    if let Some(agent_entry) = self.state.registry.agents.get(&active_ctx.agent_id)
                    {
                        let a = agent_entry.value();
                        let slot_cfg = match slot.as_str() {
                            "planning" => a.models.planning_slot.as_ref(),
                            "execution" => a.models.execution_slot.as_ref(),
                            _ => Some(&a.models.model),
                        };
                        if let Some(cfg) = slot_cfg {
                            active_ctx.model_config = cfg.clone();
                            active_ctx.provider_name = cfg.provider.to_string().to_lowercase();
                            tracing::info!("🔄 [Builder-Debugger Swap] Active context model config updated to slot '{}' (model: {})", slot, cfg.model_id);
                        }
                    }
                }

                if !orch_res.observation_buffer.is_empty() {
                    conversation_history
                        .push(format!("OBSERVATION: {}", orch_res.observation_buffer));
                }

                if orch_res.mission_completed {
                    if let Some(report) = orch_res.final_report {
                        if !output_text.is_empty() && !report.trim().is_empty() {
                            output_text.push_str("\n\n---\n## Final Report\n");
                        }
                        output_text.push_str(&report);
                    }
                    return Ok(TurnOutcome::Completed(IntelligenceOutput {
                        text: scrub_mythos_tags(output_text),
                        usage: usage.clone(),
                    }));
                }

                reasoning_halted = true; // Break the reasoning loop to move to the next turn_count
            }
        }
        Ok(TurnOutcome::Continue)
    }

    pub(crate) fn broadcast_reasoning_step(
        &self,
        ctx: &RunContext,
        step_id: &str,
        parent_id: Option<String>,
        content: &str,
        model: &str,
        slot: &str,
        turn_index: usize,
        latency_ms: u64,
        usage: &Option<crate::agent::types::TokenUsage>,
        finish_reason: &str,
    ) {
        let redacted_content = crate::agent::sanitizer::Sanitizer::redact_sensitive_data(content);
        let access_list = ctx.conductor_plan.as_ref().map(|plan| {
            plan.steps
                .iter()
                .filter(|s| s.target_agent == ctx.agent_id)
                .flat_map(|s| s.access_list.clone())
                .collect::<Vec<u32>>()
        });

        let input_tokens = usage.as_ref().map(|u| u.input_tokens).unwrap_or(0);
        let output_tokens = usage.as_ref().map(|u| u.output_tokens).unwrap_or(0);
        let cost_usd = crate::agent::rates::calculate_cost(&ctx.model_config.model_id, input_tokens, output_tokens);

        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:reasoning_step",
            "agent_id": ctx.agent_id,
            "mission_id": ctx.mission_id,
            "step": {
                "id": step_id,
                "parent_id": parent_id,
                "content": redacted_content,
                "model": model,
                "provider": ctx.provider_name,
                "slot": slot,
                "lineage": ctx.lineage,
                "access_list": access_list,
                "turn_index": turn_index,
                "latency_ms": latency_ms,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "cost_usd": cost_usd,
                "finish_reason": finish_reason,
            }
        }));
    }

    pub(crate) fn broadcast_tool_step(
        &self,
        ctx: &RunContext,
        parent_id: Option<String>,
        tool_names: Vec<String>,
    ) {
        let step_id = uuid::Uuid::new_v4().to_string();
        let tool_count = tool_names.len();
        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:reasoning_step",
            "agent_id": ctx.agent_id,
            "mission_id": ctx.mission_id,
            "step": {
                "id": step_id,
                "parent_id": parent_id,
                "content": "",
                "model": "tool_orchestrator",
                "slot": "tool",
                "tool_names": tool_names,
                "tool_count": tool_count,
                "lineage": ctx.lineage,
            }
        }));
    }

    fn activate_model_slot(
        &self,
        ctx: &RunContext,
        active_ctx: &mut RunContext,
        preferred_kind: super::super::service_traits::SlotKind,
        phase_label: &str,
    ) -> super::super::service_traits::SlotKind {
        let selection =
            self.model_router
                .select_model_slot(&self.state, &ctx.agent_models, preferred_kind, Some(&ctx.mission_id));
        if selection.kind != super::super::service_traits::SlotKind::Default
            || selection.privacy_local_override
        {
            active_ctx.model_config = selection.config.clone();
            active_ctx.provider_name = selection.config.provider.to_string().to_lowercase();
            let msg = if selection.privacy_local_override {
                format!(
                    "Orchestrator: Privacy Shield active; using Local Slot ({}) for {}",
                    selection.config.model_id, phase_label
                )
            } else {
                format!(
                    "Orchestrator: Switching slot ({}) for {}",
                    selection.config.model_id, phase_label
                )
            };
            self.broadcast_agent(ctx, &msg, "pulse");
        }
        selection.kind
    }
}
