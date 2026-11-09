//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / turn
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Intelligence]`
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.
//! - Use the active execution context for reasoning depth, including fast-path overrides.

use super::fast_path::is_fast_path_query;
use super::sentinel::scrub_mythos_tags;
use super::step_telemetry::TurnInferenceResult;
use super::TurnOutcome;
use crate::agent::runner::{AgentRunner, IntelligenceOutput, RunContext};
use crate::agent::types::TaskPayload;
use crate::error::AppError;

pub(crate) struct TurnState<'a> {
    pub active_ctx: &'a mut RunContext,
    pub conversation_history: &'a mut Vec<String>,
    pub output_text: &'a mut String,
    pub usage: &'a mut Option<crate::agent::types::TokenUsage>,
    pub parent_id: &'a mut Option<String>,
}

pub(crate) struct TurnParams<'a> {
    pub active_slot_kind: super::super::service_traits::SlotKind,
    pub system_prompt: &'a str,
    pub payload: &'a TaskPayload,
    pub turn_count: u32,
    pub is_fast_path: bool,
}

struct CycleContext<'a> {
    base_prompt: &'a str,
    internal_monologue: &'a mut Vec<String>,
    reasoning_turn: u32,
}

fn check_confidence_halt(
    function_calls: &mut Vec<crate::agent::types::ToolCall>,
    agent_id: &str,
    act_threshold: f32,
) -> bool {
    let mut halted = false;
    for fc in function_calls.iter() {
        if fc.name == "set_confidence" {
            let score = fc.args.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            if score >= act_threshold {
                tracing::info!(
                    "🛑 [Mythos] Confidence halt for agent {}: {:.2} >= {:.2}",
                    agent_id,
                    score,
                    act_threshold
                );
                halted = true;
            }
        }
    }
    function_calls.retain(|fc| fc.name != "set_confidence");
    halted
}

fn format_budget_halt_output(agent_id: &str, mut out: IntelligenceOutput) -> IntelligenceOutput {
    tracing::warn!(
        "💰 [Intelligence] Budget limit halted reasoning for {}",
        agent_id
    );
    if !out.text.contains("Budget Exceeded") {
        out.text = format!(
            "{}\n\n> ⚠️ [Budget Exceeded] Execution halted due to budget cap.",
            out.text
        );
    }
    out
}

fn is_transient_retry(e: &AppError, retries: &mut u32) -> Option<std::time::Duration> {
    if e.error_class() == crate::error::ErrorClass::Transient && *retries < 2 {
        *retries += 1;
        Some(std::time::Duration::from_millis(500 * (1 << *retries)))
    } else {
        None
    }
}

fn append_final_report(output_text: &mut String, report: Option<&str>) {
    let Some(report) = report else {
        return;
    };
    if !output_text.is_empty() && !report.trim().is_empty() {
        output_text.push_str("\n\n---\n## Final Report\n");
    }
    output_text.push_str(report);
}

fn evaluate_halt_signals(
    turn_text: &str,
    function_calls: &mut Vec<crate::agent::types::ToolCall>,
    agent_id: &str,
    act_threshold: f32,
) -> bool {
    let has_halt_tag = turn_text.contains("<halting_signal/>") || turn_text.contains("<halt/>");
    let has_conf_halt = check_confidence_halt(function_calls, agent_id, act_threshold);
    let halted = has_halt_tag || has_conf_halt;
    if halted {
        tracing::info!("🛑 [Mythos] Halting signal detected for agent {}", agent_id);
    }
    halted
}

fn format_cycle_prompt(base_prompt: &str, monologue: &[String]) -> String {
    if monologue.is_empty() {
        base_prompt.to_string()
    } else {
        format!(
            "{}\n\nINTERNAL MONOLOGUE:\n{}",
            base_prompt,
            monologue.join("\n\n")
        )
    }
}

impl AgentRunner {
    pub(crate) fn is_fast_path_query(message: &str) -> bool {
        is_fast_path_query(message)
    }

    pub(crate) fn apply_action_mask(
        ctx: &RunContext,
        tool_def: &mut crate::agent::types::ToolDefinition,
    ) -> Vec<String> {
        super::loop_detector::apply_action_mask(ctx, tool_def)
    }

    fn init_reasoning_conversation(active_ctx: &RunContext, message: &str) -> Vec<String> {
        let mut conversation_history = Vec::new();
        if let Some(ref vt) = active_ctx.visible_transcript {
            let guard = vt.lock();
            for entry in guard.iter() {
                conversation_history.push(entry.clone());
            }
        }
        crate::agent::runner::context::close_unresolved_tool_calls(&mut conversation_history);
        conversation_history.push(format!("USER: {}", message));
        conversation_history
    }

    async fn try_final_conversational_recovery(
        &self,
        ctx: &RunContext,
        state: &mut TurnState<'_>,
        system_prompt: &str,
    ) {
        if !state.output_text.is_empty() {
            return;
        }
        let ended_on_obs = state
            .conversation_history
            .last()
            .is_some_and(|s| s.starts_with("OBSERVATION:"));
        if !ended_on_obs {
            return;
        }

        tracing::info!(
            "🔄 [Intelligence] Reasoning loop terminated on observation. Final recovery turn."
        );
        let current_prompt = state.conversation_history.join("\n\n");
        match self
            .call_provider(state.active_ctx, system_prompt, &current_prompt, None)
            .await
        {
            Ok((turn_text, _, turn_usage)) => {
                self.accumulate_usage(state.usage, turn_usage);
                *state.output_text = scrub_mythos_tags(&turn_text);
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Intelligence] Recovery turn failed for agent {}: {:?}",
                    ctx.identity.agent_id,
                    e
                );
            }
        }
    }

    pub(crate) async fn handle_reasoning_phase(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        let mut active_ctx = ctx.clone();
        let is_fast_path = is_fast_path_query(&payload.message);
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
        let mut usage = ctx
            .planning_usage
            .clone()
            .or_else(|| Some(crate::agent::types::TokenUsage::default()));
        let mut conversation_history =
            Self::init_reasoning_conversation(&active_ctx, &payload.message);
        let mut parent_id: Option<String> = None;
        let mut state = TurnState {
            active_ctx: &mut active_ctx,
            conversation_history: &mut conversation_history,
            output_text: &mut output_text,
            usage: &mut usage,
            parent_id: &mut parent_id,
        };

        let max_turns = if is_fast_path { 2 } else { ctx.max_turns };
        for turn_count in 1..=max_turns {
            let params = TurnParams {
                active_slot_kind,
                system_prompt: &system_prompt,
                payload,
                turn_count,
                is_fast_path,
            };
            match self.run_agent_turn(ctx, &mut state, &params).await? {
                TurnOutcome::Continue => continue,
                TurnOutcome::Completed(out) => return Ok(out),
                TurnOutcome::BudgetExceeded(out) => {
                    return Ok(format_budget_halt_output(&ctx.identity.agent_id, out))
                }
            }
        }

        self.try_final_conversational_recovery(ctx, &mut state, &system_prompt)
            .await;
        self.broadcast_agent(ctx, "Neural Pulse: Turn finalized", "pulse");

        Ok(IntelligenceOutput {
            text: scrub_mythos_tags(state.output_text),
            usage,
        })
    }

    fn notify_reasoning_loop(&self, ctx: &RunContext, depth: u32, turn: u32) {
        if depth <= 1 {
            return;
        }
        tracing::info!(
            "🧠 [Mythos] Reasoning Loop {}/{} for agent {}",
            turn,
            depth,
            ctx.identity.agent_id
        );
        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.identity.agent_id) {
            entry.value_mut().state.current_reasoning_turn = turn;
        }
        self.broadcast_agent(
            ctx,
            &format!("thinking (loop {}/{})...", turn, depth),
            "pulse",
        );
    }

    async fn check_intra_turn_budget(
        &self,
        ctx: &RunContext,
        output_text: &str,
        usage: &Option<crate::agent::types::TokenUsage>,
    ) -> Result<Option<IntelligenceOutput>, (AppError, Option<crate::agent::types::TokenUsage>)>
    {
        match self.check_budget(ctx, 0.0, "").await {
            Ok(Some(_)) => {
                tracing::warn!(
                    "💰 [Mythos] Budget breach mid-recurrence for agent {}",
                    ctx.identity.agent_id
                );
                Ok(Some(IntelligenceOutput {
                    text: format!(
                        "{} (Halting: Budget Exceeded)",
                        scrub_mythos_tags(output_text)
                    ),
                    usage: usage.clone(),
                }))
            }
            Err(e) => Err((e, usage.clone())),
            Ok(None) => Ok(None),
        }
    }

    async fn build_active_tools(
        &self,
        ctx: &RunContext,
        active_ctx: &mut RunContext,
    ) -> Vec<crate::agent::types::ToolDefinition> {
        let mut tool_def = self.build_tools(active_ctx).await;
        let excluded = Self::apply_action_mask(active_ctx, &mut tool_def);
        if !excluded.is_empty() {
            tracing::info!(
                "🛡️ [AttractorBreak] Mask active for {}: excluded {:?}",
                ctx.identity.agent_id,
                excluded
            );
            self.broadcast_agent(
                ctx,
                &format!(
                    "Attractor Break: Action-space mask active (excluded: {:?})",
                    excluded
                ),
                "pulse",
            );
        }
        vec![tool_def]
    }

    async fn execute_turn_inference(
        &self,
        active_ctx: &RunContext,
        system_prompt: &str,
        current_prompt: &str,
        tools: Vec<crate::agent::types::ToolDefinition>,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<
        (
            String,
            Vec<crate::agent::types::ToolCall>,
            Option<crate::agent::types::TokenUsage>,
            u64,
        ),
        (AppError, Option<crate::agent::types::TokenUsage>),
    > {
        let mut retries = 0;
        let turn_start = std::time::Instant::now();
        let (turn_text, function_calls, turn_usage) = loop {
            let res = self
                .call_provider(
                    active_ctx,
                    system_prompt,
                    current_prompt,
                    Some(tools.clone()),
                )
                .await;
            if let Ok(data) = res {
                break data;
            }
            let Err(e) = res else { unreachable!() };
            let Some(delay) = is_transient_retry(&e, &mut retries) else {
                return Err((e, usage.clone()));
            };
            tracing::warn!(
                "⚡ [Intelligence] Transient error, retrying in {:?}:",
                delay
            );
            tokio::time::sleep(delay).await;
        };
        let latency_ms = turn_start.elapsed().as_millis() as u64;
        self.accumulate_usage(usage, turn_usage.clone());
        Ok((turn_text, function_calls, turn_usage, latency_ms))
    }

    fn record_turn_assistant_output(
        &self,
        turn_text: &str,
        output_text: &mut String,
        conversation_history: &mut Vec<String>,
        active_ctx: &RunContext,
    ) {
        if turn_text.is_empty() {
            return;
        }
        if !output_text.is_empty() {
            output_text.push_str("\n\n");
        }
        output_text.push_str(turn_text);
        conversation_history.push(format!("ASSISTANT: {}", turn_text));

        let clean = scrub_mythos_tags(turn_text);
        if let Some(ref vt) = active_ctx.visible_transcript {
            vt.lock().push(format!("ASSISTANT: {}", clean));
        }
    }

    async fn handle_tool_execution(
        &self,
        ctx: &RunContext,
        state: &mut TurnState<'_>,
        function_calls: Vec<crate::agent::types::ToolCall>,
        message: &str,
    ) -> Result<Option<IntelligenceOutput>, (AppError, Option<crate::agent::types::TokenUsage>)>
    {
        let orbit_span = tracing::info_span!("ToolOrchestration", agent_id = %ctx.identity.agent_id, count = function_calls.len());
        let _orbit_guard = orbit_span.enter();

        let orch_res = self
            .tool_orchestrator
            .execute_tools(
                std::sync::Arc::new(self.clone()),
                state.active_ctx,
                function_calls,
                message,
                state.usage,
            )
            .await
            .map_err(|e| (e, state.usage.clone()))?;

        drop(_orbit_guard);

        if let Some(ref slot) = orch_res.active_slot_override {
            self.apply_slot_override(state.active_ctx, slot);
        }

        if !orch_res.observation_buffer.is_empty() {
            let obs = &orch_res.observation_buffer;
            let root = &ctx.env.workspace_root;
            let (compacted, _) =
                crate::agent::runner::turn_compactor::compact_and_offload_observation(
                    "tool", obs, root,
                )
                .await;
            state.conversation_history.push(compacted);
        }

        if orch_res.mission_completed {
            append_final_report(state.output_text, orch_res.final_report.as_deref());
            return Ok(Some(IntelligenceOutput {
                text: scrub_mythos_tags(state.output_text),
                usage: state.usage.clone(),
            }));
        }

        Ok(None)
    }

    async fn execute_reasoning_step(
        &self,
        ctx: &RunContext,
        state: &mut TurnState<'_>,
        params: &TurnParams<'_>,
        cycle: &mut CycleContext<'_>,
    ) -> Result<(Option<TurnOutcome>, bool), (AppError, Option<crate::agent::types::TokenUsage>)>
    {
        self.notify_reasoning_loop(ctx, state.active_ctx.reasoning_depth, cycle.reasoning_turn);

        if let Some(out) = self
            .check_intra_turn_budget(ctx, state.output_text, state.usage)
            .await?
        {
            return Ok((Some(TurnOutcome::BudgetExceeded(out)), true));
        }

        let tools = self.build_active_tools(ctx, state.active_ctx).await;
        let current_prompt = format_cycle_prompt(cycle.base_prompt, cycle.internal_monologue);

        let (turn_text, mut function_calls, turn_usage, latency_ms) = self
            .execute_turn_inference(
                state.active_ctx,
                params.system_prompt,
                &current_prompt,
                tools,
                state.usage,
            )
            .await?;

        self.emit_turn_telemetry(
            ctx,
            state.active_ctx,
            params.active_slot_kind,
            state.parent_id,
            TurnInferenceResult {
                turn_text: &turn_text,
                function_calls: &function_calls,
                turn_usage: &turn_usage,
                latency_ms,
                reasoning_turn: cycle.reasoning_turn as usize,
            },
        );

        let mut reasoning_halted = evaluate_halt_signals(
            &turn_text,
            &mut function_calls,
            &ctx.identity.agent_id,
            ctx.act_threshold,
        );

        if cycle.reasoning_turn < state.active_ctx.reasoning_depth && !reasoning_halted {
            cycle.internal_monologue.push(turn_text);
            Self::prune_circular_monologue(cycle.internal_monologue);
            let _ = self
                .compress_monologue(state.active_ctx, cycle.internal_monologue)
                .await;
            return Ok((None, false));
        }

        self.record_turn_assistant_output(
            &turn_text,
            state.output_text,
            state.conversation_history,
            state.active_ctx,
        );

        let mut text_clone = turn_text.clone();
        if !params.is_fast_path {
            self.enforce_sentinel_gate(
                state.active_ctx,
                params.system_prompt,
                &current_prompt,
                &mut text_clone,
                &mut function_calls,
                state.usage,
            )
            .await
            .map_err(|e| (e, state.usage.clone()))?;
        }

        if function_calls.is_empty() {
            let out = IntelligenceOutput {
                text: scrub_mythos_tags(state.output_text),
                usage: state.usage.clone(),
            };
            return Ok((Some(TurnOutcome::Completed(out)), true));
        }

        if let Some(completed) = self
            .handle_tool_execution(ctx, state, function_calls, &params.payload.message)
            .await?
        {
            return Ok((Some(TurnOutcome::Completed(completed)), true));
        }

        reasoning_halted = true;
        Ok((None, reasoning_halted))
    }

    pub(crate) async fn run_agent_turn(
        &self,
        ctx: &RunContext,
        state: &mut TurnState<'_>,
        params: &TurnParams<'_>,
    ) -> Result<TurnOutcome, (AppError, Option<crate::agent::types::TokenUsage>)> {
        tracing::debug!(
            "🎯 [Intelligence] Start Turn {}/{} for agent {}",
            params.turn_count,
            ctx.max_turns,
            ctx.identity.agent_id
        );
        self.state
            .yield_phase_transition(
                &ctx.identity.agent_id,
                &format!("Execution: Turn {}", params.turn_count),
            )
            .await;

        let mut reasoning_turn: u32 = 0;
        let mut reasoning_halted = false;
        let mut internal_monologue = Vec::new();
        let base_prompt = state.conversation_history.join("\n\n");

        while reasoning_turn < state.active_ctx.reasoning_depth && !reasoning_halted {
            reasoning_turn += 1;
            let mut cycle = CycleContext {
                base_prompt: &base_prompt,
                internal_monologue: &mut internal_monologue,
                reasoning_turn,
            };
            let (maybe_outcome, halted) = self
                .execute_reasoning_step(ctx, state, params, &mut cycle)
                .await?;
            if let Some(outcome) = maybe_outcome {
                return Ok(outcome);
            }
            reasoning_halted = halted;
        }
        Ok(TurnOutcome::Continue)
    }
}
