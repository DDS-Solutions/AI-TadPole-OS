//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Intelligence Loop**: The heartbeat of an agent turn. Manages the
//! **Think->Act->Respond** cycle. Handles automatic hierarchy labeling
//! (CEO/COO/Alpha) and orchestrates concurrent tool execution using
//! FuturesUnordered. Enforces real-time **Financial Guardrails** (SEC-02)
//! by calculating neural costs per step.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Provider API timeout, tool execution panic, budget breach
//!   during a long chain, or tokenizer failure during prompt assembly.
//! - **Trace Scope**: `server-rs::agent::runner::intelligence`

use super::{AgentRunner, IntelligenceOutput, RunContext, service_traits};
use crate::agent::constants::*;
use crate::agent::types::TaskPayload;
use crate::error::AppError;
use sha2::{Digest, Sha256};
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

fn normalize_json(val: &serde_json::Value) -> String {
    normalize_json_inner(val, 0)
}

fn normalize_json_inner(val: &serde_json::Value, depth: usize) -> String {
    if depth > 32 {
        return "\"[DEPTH_EXCEEDED]\"".to_string();
    }
    match val {
        serde_json::Value::Object(map) => {
            let mut sorted_keys: Vec<_> = map.keys().collect();
            sorted_keys.sort();
            let mut parts = Vec::new();
            for key in sorted_keys {
                let normalized_val = normalize_json_inner(&map[key], depth + 1);
                parts.push(format!("\"{}\":{}", key, normalized_val));
            }
            format!("{{{}}}", parts.join(","))
        }
        serde_json::Value::Array(arr) => {
            let parts: Vec<_> = arr.iter().map(|v| normalize_json_inner(v, depth + 1)).collect();
            format!("[{}]", parts.join(","))
        }
        _ => val.to_string(),
    }
}

#[derive(Debug)]
pub struct DoomLoopDetector {
    signatures: std::collections::VecDeque<String>,
}

impl DoomLoopDetector {
    pub fn new() -> Self {
        Self {
            signatures: std::collections::VecDeque::new(),
        }
    }

    pub fn check(&mut self, tool_name: &str, arguments: &str, output: &str) -> bool {
        let normalized_args = match serde_json::from_str::<serde_json::Value>(arguments) {
            Ok(val) => normalize_json(&val),
            Err(_) => arguments.trim().to_string(),
        };

        let mut hasher = Sha256::new();
        hasher.update(output.as_bytes());
        let output_hash = hex::encode(hasher.finalize());

        let sig = format!("{}:{}:{}", tool_name, normalized_args, &output_hash[..16]);
        self.signatures.push_back(sig);

        if self.signatures.len() > 12 {
            self.signatures.pop_front();
        }

        self.has_loop()
    }

    fn has_loop(&self) -> bool {
        let n = self.signatures.len();
        for period in 1..=4 {
            let reps = if period == 1 { 3 } else { 2 };
            let needed = period * reps;
            if n < needed {
                continue;
            }

            let start_idx = n - needed;
            let mut is_loop = true;
            for i in 0..needed {
                if self.signatures[start_idx + i] != self.signatures[start_idx + (i % period)] {
                    is_loop = false;
                    break;
                }
            }
            if is_loop {
                return true;
            }
        }
        false
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

/// Result type for a single agent turn.
#[derive(Debug, Clone)]
pub enum TurnOutcome {
    /// Turn completed, continue to next turn
    Continue,
    /// Mission completed with final output
    Completed(IntelligenceOutput),
    /// Budget exceeded, halt with partial output
    BudgetExceeded(IntelligenceOutput),
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
        self.workflow_coordinator.execute_workflow(self, ctx, payload).await
    }
}

pub struct MissionWorkflowCoordinator {
    pub state: std::sync::Arc<crate::state::AppState>,
    #[allow(dead_code)]
    pub prompt_service: std::sync::Arc<dyn service_traits::PromptService>,
    pub mission_state_manager: std::sync::Arc<dyn service_traits::MissionStateManager>,
}

#[async_trait::async_trait]
impl service_traits::WorkflowCoordinator for MissionWorkflowCoordinator {
    async fn execute_workflow(
        &self,
        runner: &AgentRunner,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        // --- 🛡️ [Anti-Injection Escaping] ---
        let sanitized_message = crate::agent::sanitizer::Sanitizer::sanitize_for_prompt(&payload.message);
        let mut sanitized_payload = payload.clone();
        sanitized_payload.message = sanitized_message;

        // --- 🛡️ [StateTransaction Boundary] ---
        let mut state_tx = crate::agent::runner::service_traits::StateTransaction::new(
            self.mission_state_manager.clone(),
            self.state.clone(),
            &ctx.agent_id,
            &ctx.mission_id,
        );

        // NOTE: All state-touching operations below this point are inside the
        // StateTransaction boundary. If commit() is never reached, the
        // status/task will be rolled back automatically via Drop. (Fix 10)

        // --- [Mythos RAII Guard] ---
        let _turn_guard = ReasoningTurnGuard::new(ctx.agent_id.clone(), self.state.clone());

        let hierarchy_label = resolve_hierarchy_label(&ctx.agent_id, &ctx.role);

        runner.broadcast_agent(
            ctx,
            &format!("starting task ({})...", hierarchy_label),
            "info",
        );
        runner.update_status(
            &ctx.agent_id,
            &ctx.mission_id,
            "thinking",
            Some("Consulting intelligence model..."),
        );

        // --- 📑 [System 2] Specification Check (Fix 6: AgentMissionState dispatch) ---
        let spec =
            crate::agent::mission::get_mission_context(&self.state.resources.pool, &ctx.mission_id)
                .await
                .map_err(|e| (e, None))?;
        
        let mission_state = super::service_traits::AgentMissionState::resolve(&spec, ctx.safe_mode);
        let output = match mission_state {
            super::service_traits::AgentMissionState::SpecificationGeneration => {
                runner.handle_specification_generation_phase(ctx, &sanitized_payload, &mut state_tx).await?
            }
            _ => {
                runner.handle_reasoning_phase(ctx, &sanitized_payload).await?
            }
        };

        state_tx.commit();
        Ok(output)
    }
}

impl AgentRunner {
    async fn handle_specification_generation_phase(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
        state_tx: &mut crate::agent::runner::service_traits::StateTransaction,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        let mut active_ctx = ctx.clone();
        
        // --- [System 2] Planning Slot Activation (Fix 9: shared helper) ---
        let _slot_kind = self.activate_model_slot(
            ctx, &mut active_ctx,
            super::service_traits::SlotKind::Planning,
            "Planning",
        );

        let system_prompt = self.prompt_service.build_system_prompt(self, &active_ctx, &payload.message).await;

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

    async fn handle_reasoning_phase(
        &self,
        ctx: &RunContext,
        payload: &TaskPayload,
    ) -> Result<IntelligenceOutput, (AppError, Option<crate::agent::types::TokenUsage>)> {
        let mut active_ctx = ctx.clone();
        let active_slot_kind = self.activate_model_slot(
            ctx, &mut active_ctx,
            super::service_traits::SlotKind::Execution,
            "Tactical Execution",
        );

        let system_prompt = self.prompt_service.build_system_prompt(self, &active_ctx, &payload.message).await;

        let mut output_text = String::new();
        // Fix 11: Initialize with zero-value struct to eliminate None checks in callers
        let mut usage: Option<crate::agent::types::TokenUsage> = Some(crate::agent::types::TokenUsage::default());

        let mut turn_count = 0;
        let mut conversation_history = Vec::new();
        conversation_history.push(format!("USER: {}", payload.message));
        let max_turns = ctx.max_turns;
        let mut parent_id: Option<String> = None;

        while turn_count < max_turns {
            turn_count += 1;
            match self
                .run_agent_turn(
                    ctx,
                    &active_ctx,
                    active_slot_kind,
                    &system_prompt,
                    payload,
                    turn_count,
                    &mut conversation_history,
                    &mut output_text,
                    &mut usage,
                    &mut parent_id,
                )
                .await?
            {
                TurnOutcome::Continue => continue,
                TurnOutcome::Completed(out) | TurnOutcome::BudgetExceeded(out) => return Ok(out),
            }
        }

        self.broadcast_agent(ctx, "Neural Pulse: Turn finalized", "pulse");

        Ok(IntelligenceOutput {
            text: scrub_mythos_tags(&output_text),
            usage,
        })
    }

    /// Executes a single Think -> Act -> Observe cycle (Fix 4: Core Refactoring Mandate).
    async fn run_agent_turn(
        &self,
        ctx: &RunContext,
        active_ctx: &RunContext,
        active_slot_kind: super::service_traits::SlotKind,
        system_prompt: &str,
        payload: &TaskPayload,
        turn_count: u32,
        conversation_history: &mut Vec<String>,
        output_text: &mut String,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        parent_id: &mut Option<String>,
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
            let (turn_text, mut function_calls, turn_usage) = loop {
                let provider_res = self
                    .call_provider(active_ctx, system_prompt, &current_prompt, Some(tools.clone()))
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
            self.accumulate_usage(usage, turn_usage);

            // --- 📊 [System 2] Telemetry: Reasoning Step ---
            let current_step_id = uuid::Uuid::new_v4().to_string();
            let slot = match active_slot_kind {
                super::service_traits::SlotKind::Planning => "planning",
                super::service_traits::SlotKind::Execution => "execution",
                super::service_traits::SlotKind::Default => "default",
            };

            self.broadcast_reasoning_step(
                ctx,
                &current_step_id,
                parent_id.clone(),
                &turn_text,
                &active_ctx.model_config.model_id,
                slot,
            );
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
                    let score =
                        fc.args.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
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
                        ctx.agent_id, e
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
                }

                // 🛡️ [Sentinel Gate]
                let mut turn_text_clone = turn_text.clone();
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

                let orch_res = self.tool_orchestrator.execute_tools(
                    std::sync::Arc::new(self.clone()),
                    active_ctx,
                    function_calls,
                    &payload.message,
                    usage,
                )
                .await
                .map_err(|e| (e, usage.clone()))?;

                drop(_orbit_guard);

                if !orch_res.observation_buffer.is_empty() {
                    conversation_history.push(format!("OBSERVATION: {}", orch_res.observation_buffer));
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

    /// Truncates embedded markdown code blocks in a string if they exceed 2,000 characters.
    fn truncate_embedded_tool_logs(content: &str) -> String {
        let mut result = String::new();
        let mut current_pos = 0;
        while let Some(start_idx) = content[current_pos..].find("```") {
            let abs_start = current_pos + start_idx;
            result.push_str(&content[current_pos..abs_start]);

            let rest = &content[abs_start + 3..];
            if let Some(end_idx) = rest.find("```") {
                let abs_end = abs_start + 3 + end_idx;
                let code_block_content = &content[abs_start + 3..abs_end];

                let newline_pos = code_block_content.find('\n').unwrap_or(0);
                let header = &code_block_content[..newline_pos];
                let body = &code_block_content[newline_pos..];

                if body.len() > 2000 {
                    result.push_str(&format!(
                        "```{}\n[Raw tool result evicted to save context — was {} bytes]\n```",
                        header,
                        body.len()
                    ));
                } else {
                    result.push_str(&content[abs_start..abs_end + 3]);
                }
                current_pos = abs_end + 3;
            } else {
                result.push_str(&content[abs_start..]);
                current_pos = content.len();
                break;
            }
        }
        if current_pos < content.len() {
            result.push_str(&content[current_pos..]);
        }
        result
    }

    /// Fallback summarizer that runs deterministically without calling LLMs.
    /// Keeps header structures and short lines, removing code blocks entirely.
    fn deterministic_fallback_summarize(history: &str) -> String {
        let mut lines = Vec::new();
        let mut in_code_block = false;
        for line in history.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                lines.push("[Code block header omitted]".to_string());
                continue;
            }
            if in_code_block {
                continue;
            }
            if trimmed.len() > 150 {
                lines.push(format!("{}... [truncated]", &trimmed[..150]));
            } else if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }
        format!("DETERMINISTIC FALLBACK SUMMARY:\n{}", lines.join("\n"))
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
            total_chars,
            ctx.agent_id
        );

        let tail_count = 4;
        let monologue_len = monologue.len();

        let (older_turns, tail_turns) = if monologue_len > tail_count {
            let split_idx = monologue_len - tail_count;
            (
                monologue[..split_idx].to_vec(),
                monologue[split_idx..].to_vec(),
            )
        } else {
            (Vec::new(), monologue.to_vec())
        };

        if older_turns.is_empty() {
            return Ok(());
        }

        let mut processed_older_turns = Vec::new();
        for turn in older_turns {
            processed_older_turns.push(Self::truncate_embedded_tool_logs(&turn));
        }

        let history = processed_older_turns.join("\n\n");
        let prompt = format!(
            "SUMMARIZE YOUR PREVIOUS REASONING STEPS INTO A SINGLE CONCISE PARAGRAPH. \
             RETAIN ALL KEY INSIGHTS, VARIABLES, AND HYPOTHESES. \
             \n\nPREVIOUS REASONING:\n{}",
            history
        );

        let summary_text = match self
            .call_provider(
                ctx,
                "You are an expert reasoning summarizer. Be technical, dense, and objective.",
                &prompt,
                None,
            )
            .await
        {
            Ok((text, _, _)) => format!("CONSOLIDATED REASONING: {}", text),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Mythos] Summarizer call failed ({:?}). Falling back to deterministic summarization.",
                    e
                );
                Self::deterministic_fallback_summarize(&history)
            }
        };

        monologue.clear();
        monologue.push(summary_text);
        monologue.extend(tail_turns);

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
        let is_orchestrator = crate::agent::runner::service_traits::IdentityService::is_orchestrator(&ctx.agent_id);

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
                &user_directive.chars().take(200).collect::<String>()
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

    pub(crate) fn broadcast_reasoning_step(
        &self,
        ctx: &RunContext,
        step_id: &str,
        parent_id: Option<String>,
        content: &str,
        model: &str,
        slot: &str,
    ) {
        let redacted_content = crate::agent::sanitizer::Sanitizer::redact_sensitive_data(content);
        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:reasoning_step",
            "agent_id": ctx.agent_id,
            "mission_id": ctx.mission_id,
            "step": {
                "id": step_id,
                "parent_id": parent_id,
                "content": redacted_content,
                "model": model,
                "slot": slot
            }
        }));
    }

    /// Shared helper for model slot activation (Fix 9: DRY).
    /// Used by both handle_specification_generation_phase and handle_reasoning_phase.
    fn activate_model_slot(
        &self,
        ctx: &RunContext,
        active_ctx: &mut RunContext,
        preferred_kind: super::service_traits::SlotKind,
        phase_label: &str,
    ) -> super::service_traits::SlotKind {
        let selection = self.model_router.select_model_slot(&self.state, &ctx.agent_models, preferred_kind);
        if selection.kind != super::service_traits::SlotKind::Default || selection.privacy_local_override {
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

/// Pure function for hierarchy label resolution (Fix 14).
/// Extracted from execute_intelligence_loop for isolated testability.
pub(crate) fn resolve_hierarchy_label(agent_id: &str, role: &str) -> &'static str {
    if crate::agent::runner::service_traits::IdentityService::is_orchestrator(agent_id) {
        if agent_id == AGENT_CEO || role.to_lowercase().contains("ceo") {
            "CEO (Strategic Intelligence Lead)"
        } else if agent_id == AGENT_COO || role.to_lowercase().contains("coo") {
            "COO (Operations Director)"
        } else {
            "ALPHA NODE (Swarm Mission Commander)"
        }
    } else {
        "AGENT (Task Specialist)"
    }
}

// Metadata: [intelligence]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::state::AppState;
    use crate::agent::types::{ModelProvider, ModelConfig};
    use crate::agent::runner::service_traits::SlotKind;
    use std::sync::Arc;

    #[test]
    fn test_doom_loop_detector() {
        let mut detector = DoomLoopDetector::new();
        // Period 1 loop (A -> A -> A)
        assert!(!detector.check("ls", "{}", "file1.txt"));
        assert!(!detector.check("ls", "{}", "file1.txt"));
        assert!(detector.check("ls", "{}", "file1.txt"));

        // Reset and test Period 2 loop (A -> B -> A -> B)
        let mut detector = DoomLoopDetector::new();
        assert!(!detector.check("ls", "{}", "file1.txt"));
        assert!(!detector.check("cat", "{}", "content"));
        assert!(!detector.check("ls", "{}", "file1.txt"));
        assert!(detector.check("cat", "{}", "content"));
    }

    #[test]
    fn test_scrub_mythos_tags() {
        let input =
            "Thinking... <thinking>I should search</thinking> <halting_signal/> Done. <halt/>";
        let expected = "Thinking... I should search  Done.";
        assert_eq!(scrub_mythos_tags(input), expected);

        let clean = "No tags here";
        assert_eq!(scrub_mythos_tags(clean), "No tags here");
    }

    #[test]
    fn test_hierarchy_labeling() {
        // Fix 14: Now tests the extracted pure function directly
        assert_eq!(
            resolve_hierarchy_label(AGENT_CEO, "Chief Executive Officer"),
            "CEO (Strategic Intelligence Lead)"
        );
        assert_eq!(
            resolve_hierarchy_label(AGENT_COO, "Chief Operations"),
            "COO (Operations Director)"
        );
        assert_eq!(
            resolve_hierarchy_label(AGENT_ALPHA, "Commander"),
            "ALPHA NODE (Swarm Mission Commander)"
        );
        assert_eq!(
            resolve_hierarchy_label("specialist-42", "Engineer"),
            "AGENT (Task Specialist)"
        );
        // Role-based fallback for non-standard agent IDs
        assert_eq!(
            resolve_hierarchy_label("custom-agent", "Chief COO Officer"),
            "AGENT (Task Specialist)"
        );
    }

    #[test]
    fn test_normalize_json_depth_limit() {
        // Build a deeply nested JSON object (depth > 32)
        let mut val = serde_json::json!("leaf");
        for _ in 0..40 {
            val = serde_json::json!({"nested": val});
        }
        let result = normalize_json(&val);
        assert!(result.contains("[DEPTH_EXCEEDED]"));
    }

    #[test]
    fn test_resolve_mission_state() {
        use crate::agent::runner::service_traits::AgentMissionState;

        // No spec marker -> SpecificationGeneration
        let state = AgentMissionState::resolve("empty context", false);
        assert_eq!(state, AgentMissionState::SpecificationGeneration);

        // Has spec marker -> Reasoning
        let state = AgentMissionState::resolve("--- [ROOM: system::spec] --- some spec", false);
        assert_eq!(state, AgentMissionState::Reasoning);

        // Safe mode always -> Reasoning
        let state = AgentMissionState::resolve("empty context", true);
        assert_eq!(state, AgentMissionState::Reasoning);
    }

    #[tokio::test]
    async fn test_privacy_mode_prefers_local_default_for_planning_slot() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "llama3-local".to_string(),
            ..Default::default()
        };
        agent.models.planning_slot = Some(ModelConfig {
            provider: ModelProvider::Anthropic,
            model_id: "Claude Opus 4.5".to_string(),
            ..Default::default()
        });

        let selection = runner.model_router.select_model_slot(&runner.state, &agent.models, SlotKind::Planning);

        assert_eq!(selection.kind, SlotKind::Default);
        assert!(selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Ollama);
        assert_eq!(selection.config.model_id, "llama3-local");
    }

    #[tokio::test]
    async fn test_privacy_mode_keeps_local_preferred_slot() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "llama3-default".to_string(),
            ..Default::default()
        };
        agent.models.execution_slot = Some(ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "qwen2.5-coder-local".to_string(),
            base_url: Some("http://127.0.0.1:1234/v1".to_string()),
            ..Default::default()
        });

        let selection = runner.model_router.select_model_slot(&runner.state, &agent.models, SlotKind::Execution);

        assert_eq!(selection.kind, SlotKind::Execution);
        assert!(selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Openai);
        assert_eq!(selection.config.model_id, "qwen2.5-coder-local");
    }

    #[tokio::test]
    async fn test_hybrid_mode_uses_preferred_cloud_slot() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "llama3-default".to_string(),
            ..Default::default()
        };
        agent.models.planning_slot = Some(ModelConfig {
            provider: ModelProvider::Anthropic,
            model_id: "Claude Opus 4.5".to_string(),
            ..Default::default()
        });

        let selection = runner.model_router.select_model_slot(&runner.state, &agent.models, SlotKind::Planning);

        assert_eq!(selection.kind, SlotKind::Planning);
        assert!(!selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Anthropic);
        assert_eq!(selection.config.model_id, "Claude Opus 4.5");
    }

    #[tokio::test]
    async fn test_enforce_sentinel_gate_orchestrator() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());

        let mut ctx = RunContext::default();
        ctx.agent_id = AGENT_CEO.to_string(); // Orchestrator

        let mut output_text = "I am the CEO".to_string();
        let mut function_calls = vec![];
        let mut usage = None;

        // Should NOT enforce (do nothing) because it's an orchestrator
        let result = runner
            .enforce_sentinel_gate(
                &ctx,
                "system",
                "user",
                &mut output_text,
                &mut function_calls,
                &mut usage,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(output_text, "I am the CEO");
        assert!(function_calls.is_empty());
    }

    #[tokio::test]
    async fn test_enforce_sentinel_gate_specialist_narrative_leak() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());

        let mut ctx = RunContext::default();
        ctx.agent_id = "specialist-1".to_string();
        ctx.role = "Specialist".to_string();
        ctx.safe_mode = false;

        let mut output_text = "I am just talking".to_string();
        let mut function_calls = vec![];
        let mut usage = None;

        // This will try to call_provider, which will fail because there are no providers configured in mock.
        // But we can verify that it *attempts* to enforce by checking the error or using a more robust mock.
        // Actually, call_provider will likely return an error because the mock state has no providers.

        let result = runner
            .enforce_sentinel_gate(
                &ctx,
                "system",
                "user",
                &mut output_text,
                &mut function_calls,
                &mut usage,
            )
            .await;

        // In a minimal mock, call_provider returns Ok with a DEGRADED message from NullProvider.
        // This proves the sentinel gate was triggered and successfully re-called the provider.
        assert!(result.is_ok());
        assert!(output_text.contains("DEGRADED"));
    }

    #[tokio::test]
    async fn test_compress_monologue() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let ctx = RunContext::default();

        // 1. If total chars < 8192, compress_monologue should do nothing
        let mut monologue = vec!["Turn 1".to_string(), "Turn 2".to_string()];
        let res = runner.compress_monologue(&ctx, &mut monologue).await;
        assert!(res.is_ok());
        assert_eq!(monologue.len(), 2);
        assert_eq!(monologue[0], "Turn 1");

        // 2. Test tool log truncation helper function directly
        let input_with_large_log =
            "Some reasoning\n```json\n".to_string() + &"a".repeat(2500) + "\n```";
        let truncated = AgentRunner::truncate_embedded_tool_logs(&input_with_large_log);
        assert!(truncated.contains("[Raw tool result evicted to save context"));
        assert!(!truncated.contains(&"a".repeat(2500)));

        // 3. Test deterministic fallback summary directly
        let history = "Some long paragraph that will be kept.\n```\nbody of code block\n```\nAnother short line.";
        let summary = AgentRunner::deterministic_fallback_summarize(history);
        assert!(summary.contains("DETERMINISTIC FALLBACK SUMMARY"));
        assert!(summary.contains("[Code block header omitted]"));
        assert!(!summary.contains("body of code block"));

        // 4. Test compress_monologue with > 8192 chars and > 4 turns
        // We will make 6 turns, where first two have huge data, and last 4 are small.
        let mut monologue = vec![
            "Huge turn 1: ".to_string() + &"a".repeat(5000) + "\n```\ncode block\n```",
            "Huge turn 2: ".to_string() + &"b".repeat(4000),
            "Tail 1".to_string(),
            "Tail 2".to_string(),
            "Tail 3".to_string(),
            "Tail 4".to_string(),
        ];
        // Since we use mock config, calling provider will use NullProvider or fallback.
        // In either case, the monologue should be reduced to 5 turns: 1 summary turn + 4 tail turns.
        let res = runner.compress_monologue(&ctx, &mut monologue).await;
        assert!(res.is_ok());
        assert_eq!(monologue.len(), 5);
        assert_eq!(monologue[1], "Tail 1");
        assert_eq!(monologue[2], "Tail 2");
        assert_eq!(monologue[3], "Tail 3");
        assert_eq!(monologue[4], "Tail 4");
    }

    #[tokio::test]
    async fn test_privacy_mode_all_cloud_returns_synthesized_ollama_fallback() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "gpt-4-cloud-default".to_string(),
            ..Default::default()
        };
        agent.models.planning_slot = Some(ModelConfig {
            provider: ModelProvider::Anthropic,
            model_id: "Claude-cloud-planning".to_string(),
            ..Default::default()
        });

        let selection = runner.model_router.select_model_slot(&runner.state, &agent.models, SlotKind::Planning);

        assert_eq!(selection.kind, SlotKind::Default);
        assert!(selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Ollama);
        assert_eq!(selection.config.model_id, "gemma4:e4b");
    }
}

// Metadata: [intelligence]
