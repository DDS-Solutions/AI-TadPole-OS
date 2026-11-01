//! @docs ARCHITECTURE:Orchestrator
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits / orchestrator
//! - **Architecture**: `@docs ARCHITECTURE:Orchestrator`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `Execution halted: infinite tool cycle or repeating error loop`
//! - **Telemetry Targets**: none declared

use crate::agent::sanitizer::Sanitizer;

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
        if function_calls.len() > 1
            && function_calls
                .iter()
                .any(|call| call.name == "complete_mission")
        {
            return Err(crate::error::AppError::BadRequest(
                "complete_mission must run alone after other tools finish".into(),
            ));
        }
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
                agent_id = %ctx_clone.identity.agent_id,
                mission_id = %ctx_clone.mission_state.mission_id,
                input_summary = %fc_input_summary,
                success = tracing::field::Empty,
                output_bytes = tracing::field::Empty,
                error = tracing::field::Empty,
            );

            futures.push(
                async move {
                    executor_clone.update_status(
                        &ctx_clone.identity.agent_id,
                        &ctx_clone.mission_state.mission_id,
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
            // Tool responses can include sensitive file or connector data. Redact
            // before loop tracking, failure refinement, transcript propagation,
            // or large-response offload can retain or forward the content.
            let safe_local_text = Sanitizer::sanitize_for_prompt(&raw_local_text);

            // 1. Deterministic Doom Loop Detection (Evaluated before refinement mutations)
            let args_str = tool_call.args.to_string();
            let loop_status = {
                let mut guard = self.loop_detectors.lock();
                let detector_key = format!(
                    "{}:{}",
                    active_ctx.mission_state.mission_id, active_ctx.identity.agent_id
                );
                let (detector, _) = guard.entry(detector_key).or_insert_with(|| {
                    (
                        super::super::intelligence::DoomLoopDetector::new(),
                        std::time::Instant::now(),
                    )
                });
                detector.check(
                    &active_ctx.identity.agent_id,
                    &tool_call.name,
                    &args_str,
                    &safe_local_text,
                )
            };

            // Hashed Loop & Error Cycle Detection
            if loop_status.is_terminal() {
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
            let completion_report =
                (tool_call.name == "complete_mission").then(|| safe_local_text.clone());
            let mut local_text = safe_local_text;
            executor.handle_tool_failure_refinement(active_ctx, &tool_call, &mut local_text);

            if loop_status.is_warning() {
                tracing::warn!(
                    "⚠️ [DoomLoopDetector] Repetition warning on tool {}! Injecting steering advisory.",
                    tool_call.name
                );
                local_text.push_str(&format!(
                    "\n\n⚠️ [CRITICAL REPETITION WARNING]: You have executed tool '{}' in a repeating cycle with identical arguments and received the same result. DO NOT execute this identical call or sequence again. Another identical repetition will trigger an immediate hard termination. Proceed now to synthesize your results, inspect different paths, or use alternative tools.",
                    tool_call.name
                ));
            }

            let is_failure =
                super::observation::classify_failure(&tool_call.name, success, &local_text);
            let effective_success = !is_failure;

            // 3. Preemptive large tool output offload / truncation (Token Defense)
            local_text = super::observation::offload_large_tool_response(
                &tool_call.name,
                &local_text,
                is_failure,
                cumulative_turn_chars,
                Some(&active_ctx.env.workspace_root),
            );
            cumulative_turn_chars += local_text.len();

            // 4. Builder-Debugger Slot Swap on Failure
            if is_failure {
                if let Some(new_slot) =
                    executor.handle_tool_failure_slot_swap(&active_ctx.identity.agent_id)
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
                if !effective_success || (req_verif && !has_process_witness) {
                    active_ctx
                        .swarm_partial_failure
                        .store(true, std::sync::atomic::Ordering::Release);
                    active_ctx
                        .swarm_review_approved
                        .store(false, std::sync::atomic::Ordering::Release);
                    executor.broadcast_agent(
                        active_ctx,
                        "🚨 Sentinel Gate: Finalization BLOCKED. No proof of verification found.",
                        "warning",
                    );
                    observation_buffer.push_str("\n[SENTINEL GATE]: Finalization BLOCKED. You must run a verification test (e.g. 'cargo test' or a reproduction script) and prove success before completing this mission. Your previous attempt lacked deterministic proof of correctness.\n");
                    // End with the report requiring review; finalization pauses
                    // the ledger without another model call for recovery.
                    mission_completed = true;
                    let report = completion_report.unwrap_or_default();
                    final_report = Some(if report.starts_with("MISSION_REVIEW_REQUIRED") {
                        report
                    } else {
                        format!("MISSION_REVIEW_REQUIRED: Completion was not verified.\n\n{report}")
                    });
                } else {
                    mission_completed = true;
                    final_report = completion_report;
                    self.loop_detectors
                        .lock()
                        .remove(&active_ctx.mission_state.mission_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{
        runner::{ProcessExecutionRecord, RunContext},
        types::{TokenUsage, ToolCall},
    };
    use crate::error::AppError;
    use std::sync::Arc;

    struct CompletionExecutor {
        report: String,
        fail: bool,
    }

    #[async_trait::async_trait]
    impl super::super::ports::ToolExecutor for CompletionExecutor {
        async fn execute_tool(
            &self,
            _: &RunContext,
            _: &ToolCall,
            _: &str,
        ) -> Result<(String, Option<TokenUsage>), AppError> {
            if self.fail {
                Err(AppError::BadRequest("completion rejected".into()))
            } else {
                Ok((self.report.clone(), None))
            }
        }
        fn update_status(&self, _: &str, _: &str, _: &str, _: Option<&str>) {}
        fn get_provider_timeout_secs(&self) -> u64 {
            30
        }
        fn handle_tool_failure_refinement(&self, _: &RunContext, _: &ToolCall, _: &mut String) {}
        fn accumulate_usage(&self, _: &mut Option<TokenUsage>, _: Option<TokenUsage>) {}
        fn verify_mission_success(&self, _: &str) -> bool {
            true
        } // Spoofed text must never authorize completion.
        fn verify_execution_witnesses(
            &self,
            records: &[ProcessExecutionRecord],
            last_mutation_ms: u64,
        ) -> bool {
            crate::agent::runner::oversight::verify_execution_witnesses(records, last_mutation_ms)
        }
        fn broadcast_agent(&self, _: &RunContext, _: &str, _: &str) {}
        fn handle_tool_failure_slot_swap(&self, _: &str) -> Option<String> {
            None
        }
    }

    fn completion() -> ToolCall {
        ToolCall {
            name: "complete_mission".into(),
            args: serde_json::json!({"final_report":"report"}),
        }
    }

    #[tokio::test]
    async fn unverified_completion_preserves_full_report_without_text_proof_bypass() {
        let folder = tempfile::tempdir().unwrap();
        let mut ctx = RunContext::default();
        ctx.env.workspace_root = folder.path().to_path_buf();
        let report = format!("test result: ok. Candidate: {}", "根拠".repeat(10_000));
        let result = DefaultToolOrchestrator::default()
            .execute_tools(
                Arc::new(CompletionExecutor {
                    report: report.clone(),
                    fail: false,
                }),
                &ctx,
                vec![completion()],
                "audit",
                &mut None,
            )
            .await
            .unwrap();
        assert!(result.mission_completed); // Terminal delivery; ledger finalization requires review.
        let delivered = result.final_report.unwrap();
        assert!(delivered.starts_with("MISSION_REVIEW_REQUIRED"));
        assert!(delivered.ends_with(&report));
        assert!(ctx
            .swarm_partial_failure
            .load(std::sync::atomic::Ordering::Acquire));
    }

    #[tokio::test]
    async fn failed_completion_is_never_successful_even_with_a_passing_witness() {
        for fail in [true, false] {
            let ctx = RunContext::default();
            ctx.execution_records.lock().push(ProcessExecutionRecord {
                command: "cargo test".into(),
                exit_code: 0,
                timestamp_ms: 1,
            });
            let result = DefaultToolOrchestrator::default()
                .execute_tools(
                    Arc::new(CompletionExecutor {
                        report: "Preserved report".into(),
                        fail,
                    }),
                    &ctx,
                    vec![completion()],
                    "audit",
                    &mut None,
                )
                .await
                .unwrap();
            let report = result.final_report.unwrap();
            assert_eq!(report.starts_with("MISSION_REVIEW_REQUIRED"), fail);
            assert_eq!(
                ctx.swarm_partial_failure
                    .load(std::sync::atomic::Ordering::Acquire),
                fail
            );
            if !fail {
                assert_eq!(report, "Preserved report");
            }
        }
    }

    #[tokio::test]
    async fn completion_cannot_race_another_tool_in_the_same_batch() {
        let result = DefaultToolOrchestrator::default()
            .execute_tools(
                Arc::new(CompletionExecutor {
                    report: "unused".into(),
                    fail: false,
                }),
                &RunContext::default(),
                vec![
                    completion(),
                    ToolCall {
                        name: "write_file".into(),
                        args: serde_json::json!({}),
                    },
                ],
                "audit",
                &mut None,
            )
            .await;
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }
}
