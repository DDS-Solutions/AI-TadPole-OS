//! @docs ARCHITECTURE:Orchestrator
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits / orchestrator
//! - **Architecture**: `@docs ARCHITECTURE:Orchestrator`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural] [Concurrency]` All async tool futures MUST be instrumented with `.instrument(tool_span)` to
//!   prevent tracing context leak across `.await` points.
//! - `[Structural] [DoomLoop]` `DoomLoopDetector` check MUST be evaluated on RAW output — before any refinement mutation.
//! - `[Structural] [Token Drain]` On doom-loop halt, ALL in-flight futures MUST be drained to preserve token accounting.
//! - `[Structural] [Limit]` At most 16 parallel tool calls per turn. Exceeding this returns `BadRequest`.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `Execution halted: infinite tool cycle or repeating error loop`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

const LOOP_DETECTOR_TTL_SECS: u64 = 1800; // 30 minutes
const TOOL_INPUT_SUMMARY_MAX_LEN: usize = 120;

pub struct DefaultToolOrchestrator {
    loop_detectors: parking_lot::Mutex<
        std::collections::HashMap<
            String,
            (
                super::super::intelligence::DoomLoopDetector,
                std::time::Instant,
            ),
        >,
    >,
}

impl Default for DefaultToolOrchestrator {
    fn default() -> Self {
        Self {
            loop_detectors: parking_lot::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl DefaultToolOrchestrator {
    pub async fn execute_tools(
        &self,
        executor: std::sync::Arc<dyn super::ports::ToolExecutor>,
        active_ctx: &super::super::RunContext,
        function_calls: Vec<crate::agent::types::ToolCall>,
        user_message: &str,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<super::ports::ToolOrchestrationResult, crate::error::AppError> {
        <Self as super::ports::ToolOrchestrator>::execute_tools(
            self,
            executor,
            active_ctx,
            function_calls,
            user_message,
            usage,
        )
        .await
    }
}

#[async_trait::async_trait]
impl super::ports::ToolOrchestrator for DefaultToolOrchestrator {
    async fn execute_tools(
        &self,
        executor: std::sync::Arc<dyn super::ports::ToolExecutor>,
        active_ctx: &super::super::RunContext,
        function_calls: Vec<crate::agent::types::ToolCall>,
        user_message: &str,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<super::ports::ToolOrchestrationResult, crate::error::AppError> {
        use futures::stream::{FuturesUnordered, StreamExt};
        use tracing::Instrument;

        const MAX_PARALLEL_TOOL_CALLS: usize = 16;
        if function_calls.len() > MAX_PARALLEL_TOOL_CALLS {
            return Err(crate::error::AppError::BadRequest(format!(
                "A model turn may request at most {MAX_PARALLEL_TOOL_CALLS} tools; received {}",
                function_calls.len()
            )));
        }

        // TTL Eviction: Purge stale DoomLoopDetector entries (>30 min) to prevent memory leaks
        // from missions that end via budget breach, max turn exhaustion, or early return (Audit 1.4)
        {
            let mut guard = self.loop_detectors.lock();
            let cutoff =
                std::time::Instant::now() - std::time::Duration::from_secs(LOOP_DETECTOR_TTL_SECS);
            guard.retain(|_, (_, created_at)| *created_at > cutoff);
        }

        let mut cumulative_turn_chars = 0usize;
        let mut futures = FuturesUnordered::new();
        for fc in function_calls {
            let executor_clone = executor.clone();
            let ctx_clone = active_ctx.clone();
            let user_msg_clone = user_message.to_string();
            let fc_clone = fc.clone();
            // Gap 5: Sanitized input summary — only well-known safe keys, 120-char truncated
            let fc_input_summary = fc
                .args
                .get("path")
                .or_else(|| fc.args.get("query"))
                .or_else(|| fc.args.get("command"))
                .or_else(|| fc.args.get("url"))
                .and_then(|v| v.as_str())
                .map(|s| {
                    s.chars()
                        .take(TOOL_INPUT_SUMMARY_MAX_LEN)
                        .collect::<String>()
                })
                .unwrap_or_default();
            // Gap 5: Individual tool span — child of ToolOrchestration
            let tool_span = tracing::info_span!(
                "tool_execution",
                tool_name = %fc.name,
                agent_id = %ctx_clone.agent_id,
                mission_id = %ctx_clone.mission_id,
                input_summary = %fc_input_summary,
                success = tracing::field::Empty,
                output_bytes = tracing::field::Empty,
                error = tracing::field::Empty,
            );

            futures.push(
                async move {
                    executor_clone.update_status(
                        &ctx_clone.agent_id,
                        &ctx_clone.mission_id,
                        "working",
                        Some(&format!("Executing tool: {}...", fc_clone.name)),
                    );
                    let timeout_secs = executor_clone.get_tool_timeout_secs();
                    let tool_timeout = std::time::Duration::from_secs(timeout_secs);
                    let (local_text, local_usage, exec_success) = match tokio::time::timeout(
                        tool_timeout,
                        executor_clone.execute_tool(&ctx_clone, &fc_clone, &user_msg_clone),
                    )
                    .await
                    {
                        Ok(Ok((text, usage))) => {
                            let is_err =
                                super::observation::classify_failure(&fc_clone.name, true, &text);
                            (text, usage, !is_err)
                        }
                        Ok(Err(e)) => (format!("(TOOL FAILURE: {:?})", e), None, false),
                        Err(_) => (
                            format!(
                                "(TOOL TIMEOUT: Tool execution timed out after {} seconds)",
                                tool_timeout.as_secs()
                            ),
                            None,
                            false,
                        ),
                    };

                    // Gap 5: Record outcome on the span before it closes
                    let current_span = tracing::Span::current();
                    current_span.record("success", exec_success);
                    current_span.record("output_bytes", local_text.len() as u64);
                    if !exec_success {
                        current_span
                            .record("error", format!("tool {} failed", fc_clone.name).as_str());
                    }

                    ctx_clone.execution_metrics.record_tool_attempt();
                    if !exec_success {
                        ctx_clone.execution_metrics.record_tool_failure();
                    }

                    (fc_clone, exec_success, local_text, local_usage)
                }
                .instrument(tool_span),
            );
        }

        let mut observation_buffer = String::new();
        let mut mission_completed = false;
        let mut final_report = None;
        let mut active_slot_override = None;

        while let Some((tool_call, success, raw_local_text, local_usage)) = futures.next().await {
            executor.accumulate_usage(usage, local_usage);

            // 1. Deterministic Doom Loop Detection (Evaluated on RAW output before refinement mutations)
            let args_str = tool_call.args.to_string();
            let has_loop = {
                let mut guard = self.loop_detectors.lock();
                let (detector, _) =
                    guard
                        .entry(active_ctx.mission_id.clone())
                        .or_insert_with(|| {
                            (
                                super::super::intelligence::DoomLoopDetector::new(),
                                std::time::Instant::now(),
                            )
                        });
                detector.check(
                    &active_ctx.agent_id,
                    &tool_call.name,
                    &args_str,
                    &raw_local_text,
                )
            };

            // Hashed Loop & Error Cycle Detection
            if has_loop {
                tracing::warn!(
                    "🛑 [DoomLoopDetector] Loop detected on tool {}! Draining in-flight futures and halting agent.",
                    tool_call.name
                );
                // Drain remaining futures so token usage and child tasks are not lost/leaked
                while let Some((_, _, _, remaining_usage)) = futures.next().await {
                    executor.accumulate_usage(usage, remaining_usage);
                }
                return Err(crate::error::AppError::Forbidden(format!(
                    "Execution halted: infinite tool cycle or repeating error loop detected on tool '{}'.",
                    tool_call.name
                )));
            }

            // 2. Autonomous Refinement Hook
            let mut local_text = raw_local_text;
            executor.handle_tool_failure_refinement(active_ctx, &tool_call, &mut local_text);

            let is_failure =
                super::observation::classify_failure(&tool_call.name, success, &local_text);
            let effective_success = !is_failure;

            // 3. Preemptive large tool output offload / truncation (Token Defense)
            local_text = super::observation::offload_large_tool_response(
                &tool_call.name,
                &local_text,
                is_failure,
                cumulative_turn_chars,
                Some(&active_ctx.workspace_root),
            );
            cumulative_turn_chars += local_text.len();

            // 4. Builder-Debugger Slot Swap on Failure
            if is_failure {
                if let Some(new_slot) = executor.handle_tool_failure_slot_swap(&active_ctx.agent_id)
                {
                    active_slot_override = Some(new_slot);
                }
            }

            // 5. Sandboxed contextual observation propagation
            if let Some(ref vt) = active_ctx.visible_transcript {
                let clean_obs = if local_text.len() > 300 && !local_text.contains("tool_overflow") {
                    format!(
                        "{}... [TRUNCATED TOOL OUTPUT: {} chars total]",
                        super::super::safe_truncate_str(&local_text, 300),
                        local_text.len()
                    )
                } else {
                    local_text.clone()
                };
                vt.lock().push(format!(
                    "OBSERVATION (Tool {}): {}",
                    tool_call.name, clean_obs
                ));
            }

            // 6. Unambiguous fenced observation appending
            observation_buffer.push_str(&super::observation::format_fenced_observation(
                &tool_call.name,
                effective_success,
                &local_text,
            ));

            // 7. Complete Mission / Sentinel Gate
            if tool_call.name == "complete_mission" {
                let req_verif = {
                    let mods = active_ctx.modified_files.lock();
                    let cmds = active_ctx.commands_run.lock();
                    super::mission_state::requires_verification(&mods, &cmds, active_ctx.safe_mode)
                };

                let last_mutation_ms = active_ctx
                    .last_mutation_timestamp_ms
                    .load(std::sync::atomic::Ordering::SeqCst);
                let records = active_ctx.execution_records.lock().clone();
                let has_process_witness =
                    executor.verify_execution_witnesses(&records, last_mutation_ms);
                let has_text_proof = executor.verify_mission_success(&observation_buffer);

                if req_verif && !has_process_witness && !has_text_proof {
                    executor.broadcast_agent(
                        active_ctx,
                        "🚨 Sentinel Gate: Finalization BLOCKED. No proof of verification found.",
                        "warning",
                    );
                    observation_buffer.push_str("\n[SENTINEL GATE]: Finalization BLOCKED. You must run a verification test (e.g. 'cargo test' or a reproduction script) and prove success before completing this mission. Your previous attempt lacked deterministic proof of correctness.\n");
                    mission_completed = false;
                } else {
                    mission_completed = true;
                    final_report = Some(local_text);
                    self.loop_detectors.lock().remove(&active_ctx.mission_id);
                }
            }
        }

        Ok(super::ports::ToolOrchestrationResult {
            observation_buffer,
            mission_completed,
            final_report,
            active_slot_override,
        })
    }
}
