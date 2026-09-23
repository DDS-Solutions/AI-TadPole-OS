//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / oversight
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_verify_mission_success_sentinel`, `test_execution_witness_sentinel`, `test_update_status`, `test_broadcast_agent_message`, `test_oversight_cleanup_guard`, `test_e2e_oversight_approval_loop`

use super::AgentRunner;

/// RAII Guard ensuring in-memory resolvers and queue items are cleaned up on all exit paths,
/// including timeouts, channel drops, cancellation, or panics.
struct OversightCleanupGuard<'a> {
    comms: &'a crate::state::hubs::comm::CommunicationHub,
    id: String,
}

impl Drop for OversightCleanupGuard<'_> {
    fn drop(&mut self) {
        self.comms.oversight_resolvers.remove(&self.id);
        self.comms.oversight_queue.remove(&self.id);
    }
}

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  OVERSIGHT (HUMAN-IN-THE-LOOP)
    // ─────────────────────────────────────────────────────────

    pub async fn submit_oversight_resolution(
        &self,
        mut tool_call: crate::agent::types::ToolCallAudit,
        mission_id: Option<String>,
    ) -> Result<crate::agent::types::OversightResolution, crate::error::AppError> {
        let entry_id = uuid::Uuid::new_v4().to_string();

        tool_call.mission_id = mission_id.clone();

        let entry = crate::agent::types::OversightEntry {
            id: entry_id.clone(),
            mission_id: mission_id.clone(),
            tool_call: Some(tool_call.clone()),
            skill_proposal: None,
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // 1. [Persistence FIRST] Record action attempt in SQLite for audit history
        // If SQL fails, nothing is registered in memory yet, preventing ghost leaks.
        let payload_json = serde_json::to_string(&tool_call).ok();
        let params_json = serde_json::to_string(&tool_call.params).map_err(|e| {
            crate::error::AppError::InternalServerError(format!(
                "Failed to serialize oversight params: {}",
                e
            ))
        })?;

        sqlx::query(
            "INSERT INTO oversight_log (id, mission_id, agent_id, entry_type, skill, params, status, payload) VALUES (?, ?, ?, 'tool_call', ?, ?, 'pending', ?)"
        )
        .bind(&entry_id)
        .bind(&mission_id)
        .bind(&tool_call.agent_id)
        .bind(&tool_call.skill)
        .bind(params_json)
        .bind(payload_json)
        .execute(&self.state.resources.pool)
        .await?;

        // 2. Register in memory and guard with RAII cleanup against cancellation / errors
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.state
            .comms
            .oversight_resolvers
            .insert(entry_id.clone(), tx);
        self.state
            .comms
            .oversight_queue
            .insert(entry_id.clone(), entry.clone());

        let _guard = OversightCleanupGuard {
            comms: &self.state.comms,
            id: entry_id.clone(),
        };

        // 3. Record proposal in Merkle audit trail
        let audit_params = serde_json::json!({
            "entry_id": &entry_id,
            "skill": &tool_call.skill,
            "params": &tool_call.params,
        })
        .to_string();
        if let Err(e) = self
            .state
            .security
            .audit_trail
            .record(
                &tool_call.agent_id,
                mission_id.as_deref(),
                None,
                "[OVERSIGHT_PROPOSAL]",
                &audit_params,
            )
            .await
        {
            tracing::error!(
                "❌ [Oversight] Failed to record proposal in audit trail: {:?}",
                e
            );
        }

        // 4. Notify the UI
        self.state.emit_event(serde_json::json!({
            "type": "oversight:new",
            "entry": entry
        }));

        // 5. Await user decision with safety timeout, grace window for in-flight approvals, and CAS resolution
        let timeout_dur = std::time::Duration::from_secs(300);
        let mut rx = rx;
        let decision = tokio::select! {
            res = &mut rx => res.ok(),
            _ = tokio::time::sleep(timeout_dur) => {
                // Claim the resolver atomically; if it's already gone, a decider
                // holds the sender — give the in-flight send a short grace window.
                if self.state.comms.oversight_resolvers.remove(&entry_id).is_none() {
                    tokio::time::timeout(std::time::Duration::from_millis(250), &mut rx)
                        .await
                        .ok()
                        .and_then(|r| r.ok())
                } else {
                    None
                }
            }
        };

        if let Some(resolution) = decision {
            return Ok(resolution);
        }

        // If decision is None (timeout expired or channel closed without decision), execute atomic CAS transition
        tracing::warn!(
            "⚠️ [Oversight] Timeout or channel closed waiting for oversight decision on entry {}. Rejecting by default.",
            entry_id
        );

        let now_rfc3339 = chrono::Utc::now().to_rfc3339();
        let rows = sqlx::query(
            "UPDATE oversight_log SET status = 'rejected', decision = 'timed_out', decided_at = ?, decided_by = 'system' WHERE id = ? AND status = 'pending'"
        )
        .bind(&now_rfc3339)
        .bind(&entry_id)
        .execute(&self.state.resources.pool)
        .await;

        match rows {
            Ok(r) if r.rows_affected() == 0 => {
                tracing::warn!(
                    "⚠️ [Oversight] Entry {} was already resolved; timeout CAS update was a no-op.",
                    entry_id
                );
            }
            Err(e) => {
                tracing::error!(
                    "❌ [Oversight] Failed to update timeout status in DB for {}: {:?}",
                    entry_id,
                    e
                );
            }
            _ => {}
        }

        let timeout_params = serde_json::json!({
            "entry_id": &entry_id,
            "timeout_secs": timeout_dur.as_secs(),
            "status": "rejected",
        })
        .to_string();

        if let Err(e) = self
            .state
            .security
            .audit_trail
            .record(
                "system",
                mission_id.as_deref(),
                None,
                "[OVERSIGHT_TIMEOUT]",
                &timeout_params,
            )
            .await
        {
            tracing::error!(
                "❌ [Oversight] Failed to record timeout in audit trail: {:?}",
                e
            );
        }

        self.state.emit_event(serde_json::json!({
            "type": "oversight:expired",
            "entry_id": entry_id
        }));

        Ok(crate::agent::types::OversightResolution {
            approved: false,
            override_slot: None,
            user_answer: None,
        })
    }

    /// Submits a tool call for manual user approval.
    /// Returns true if approved, false if rejected.
    pub async fn submit_oversight(
        &self,
        tool_call: crate::agent::types::ToolCallAudit,
        mission_id: Option<String>,
    ) -> Result<bool, crate::error::AppError> {
        let res = self
            .submit_oversight_resolution(tool_call, mission_id)
            .await?;
        Ok(res.approved)
    }

    // ─────────────────────────────────────────────────────────
    //  TELEMETRY HELPERS
    // ─────────────────────────────────────────────────────────

    pub(crate) fn broadcast_agent_status(&self, agent_id: &str, mission_id: &str, status: &str) {
        let agent_info = self.state.registry.agents.get(agent_id).map(|a| {
            let task = a.state.current_task.clone();
            let tokens = a.economics.tokens_used;
            let elapsed_ms = a.state.active_mission.as_ref().and_then(|m| {
                m.get("started_at")
                    .and_then(|v| v.as_u64())
                    .map(|started_at| {
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        now_ms.saturating_sub(started_at)
                    })
            });
            (task, tokens, elapsed_ms)
        });

        let (task, tokens_used, elapsed_ms) = match agent_info {
            Some((t, tok, el)) => (t, tok, el),
            None => {
                tracing::warn!(
                    "⚠️ [Telemetry] Attempted to broadcast status for unregistered agent '{}'",
                    agent_id
                );
                (None, 0, None)
            }
        };

        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:status",
            "agent_id": agent_id,
            "mission_id": mission_id,
            "status": status,
            "current_task": task,
            "tokens_used_so_far": tokens_used,
            "elapsed_ms": elapsed_ms,
        }));
    }

    /// Centralized status and task update that syncs registry AND broadcasts telemetry.
    pub(crate) fn update_status(
        &self,
        agent_id: &str,
        mission_id: &str,
        status: &str,
        task: Option<&str>,
    ) {
        if let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) {
            let agent = entry.value_mut();
            agent.health.status = status.to_string();
            agent.state.current_task = task.map(|t| t.to_string());

            // Sync active mission for high-speed pulse telemetry
            if status == "idle" {
                agent.state.active_mission = None;
            } else {
                let should_reset = match &agent.state.active_mission {
                    Some(m) => m.get("id").and_then(|v| v.as_str()) != Some(mission_id),
                    None => true,
                };
                if should_reset {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    agent.state.active_mission = Some(serde_json::json!({
                        "id": mission_id,
                        "started_at": now_ms
                    }));
                }
            }
        }
        self.broadcast_agent_status(agent_id, mission_id, status);
    }

    pub(crate) fn broadcast_agent_message(
        &self,
        agent_id: &str,
        mission_id: &str,
        text: &str,
        role: &str,
        turn_index: usize,
    ) {
        let agent_name = self
            .state
            .registry
            .agents
            .get(agent_id)
            .map(|a| a.identity.name.clone())
            .unwrap_or_else(|| {
                if agent_id == "1" {
                    "Agent of Nine".to_string()
                } else {
                    format!("Agent {}", agent_id)
                }
            });

        // Approximate token count without a tokenizer (whitespace split)
        let token_count = text.split_whitespace().count();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let event_payload = serde_json::json!({
            "type": "agent:message",
            "agent_id": agent_id,
            "agent_name": agent_name,
            "mission_id": mission_id,
            "message_id": uuid::Uuid::new_v4().to_string(),
            "role": role,
            "content": text,
            "text": text,
            "turn_index": turn_index,
            "token_count": token_count,
            "timestamp": timestamp,
        });

        // 1. Primary Engine WebSocket Event channel (comms.event_tx)
        self.state.emit_event(event_payload.clone());

        // 2. High-Speed Telemetry channel (comms.telemetry_tx)
        let _ = self.state.comms.telemetry_tx.send(event_payload);
    }

    /// ### ✅ [System 2] Success Sentinel
    /// Checks the current observation buffer for deterministic evidence of a successful verification test run.
    /// This is the "Sentinel Gate" that prevents agents from reporting success
    /// without deterministic proof (e.g. an explicit passing test runner summary).
    pub(crate) fn verify_mission_success(&self, observation: &str) -> bool {
        // 1. Sanitize and normalize lines, stripping blockquotes and sentinel feedback lines
        // so sentinel rejection messages cannot self-satisfy keywords on retry turns.
        let lines: Vec<String> = observation
            .lines()
            .map(|l| {
                let trimmed = l.trim().trim_start_matches('>').trim();
                trimmed.to_lowercase()
            })
            .filter(|l| {
                !l.contains("[sentinel gate]") && !l.contains("you must run a verification test")
            })
            .collect();

        // 2. VETO: An explicit failure verdict from a test runner or assertion beats any success indicator.
        let has_failure = lines.iter().any(|l| {
            l.contains("test result: failed")
                || l.contains("panicked at")
                || l.contains("assertionerror")
                || l.contains("failures:")
                || l.contains("failed:")
                || (l.contains(" failed") && !l.contains("0 failed") && !l.contains("failed: 0"))
                || (l.contains("failed;") && !l.contains("0 failed;"))
                || l.contains("error[e")
                || l.contains("syntaxerror:")
        });

        if has_failure {
            return false;
        }

        // 3. Verdict must be a structured runner summary or explicit passing test run verdict,
        // rather than loose agent prose.
        let has_pass = lines.iter().any(|l| {
            // Cargo test: "test result: ok. 12 passed; 0 failed" or "test result: ok"
            l.contains("test result: ok")
                // Pytest: "=== 12 passed in 1.24s ===" or "12 passed, 0 failed"
                || (l.contains("passed") && (l.contains("0 failed") || l.contains("failed: 0")))
                || (l.contains("==") && l.contains(" passed") && !l.contains("0 passed"))
                // Vitest / Jest: "Tests: 12 passed, 12 total"
                || (l.contains("tests:") && l.contains("passed") && !l.contains("failed"))
                // Explicit tool execution confirmation e.g. "cargo test passed"
                || l.contains("cargo test passed")
                || l.contains("pytest passed")
        });

        has_pass
    }

    /// Verifies if any recorded OS command execution proves deterministic verification
    /// by exiting with status 0 at or after the last file modification timestamp.
    pub(crate) fn verify_execution_witnesses(
        &self,
        records: &[super::ProcessExecutionRecord],
        last_mutation_ms: u64,
    ) -> bool {
        verify_execution_witnesses(records, last_mutation_ms)
    }
}

/// Validates process-level execution witnesses: verifies if any test execution command exited with OS code 0
/// at or after the last recorded file mutation.
pub fn verify_execution_witnesses(
    records: &[super::ProcessExecutionRecord],
    last_mutation_ms: u64,
) -> bool {
    records.iter().any(|rec| {
        rec.exit_code == 0 && rec.timestamp_ms >= last_mutation_ms && is_test_command(&rec.command)
    })
}

/// Determines if a shell command corresponds to a test runner or test script execution.
pub fn is_test_command(cmd: &str) -> bool {
    let lower = cmd.trim().to_lowercase();
    let norm = lower.replace('\\', "/");

    // Standard CLI test commands & package managers
    if norm.contains("cargo test")
        || norm.contains("pytest")
        || norm.contains("npm test")
        || norm.contains("npm run test")
        || norm.contains("pnpm test")
        || norm.contains("yarn test")
        || norm.contains("bun test")
        || norm.contains("vitest")
        || norm.contains("jest")
        || norm.contains("go test")
        || norm.contains("ctest")
        || norm.contains("python -m unittest")
        || norm.contains("python -m pytest")
        || norm.contains("python3 -m unittest")
        || norm.contains("python3 -m pytest")
    {
        return true;
    }

    // Direct invocation of test files or scripts
    let tokens: Vec<&str> = norm.split_whitespace().collect();
    for token in &tokens {
        let clean = token.trim_matches(|c| c == '\u{22}' || c == '\'' || c == '`');
        let filename = clean.split('/').next_back().unwrap_or(clean);
        if filename == "test.py"
            || filename == "tests.py"
            || filename == "test.sh"
            || filename == "tests.sh"
            || filename == "test.js"
            || filename == "test.ts"
            || filename.starts_with("test_")
            || filename.ends_with("_test.py")
            || filename.ends_with("_test.go")
            || filename.ends_with("_test.rs")
            || filename.ends_with(".test.js")
            || filename.ends_with(".test.ts")
            || filename.ends_with(".test.jsx")
            || filename.ends_with(".test.tsx")
            || filename.ends_with(".spec.js")
            || filename.ends_with(".spec.ts")
        {
            return true;
        }
    }

    false
}

// Metadata: [oversight]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::EngineAgent;
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_status() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let agent_id = "test-agent";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        state.registry.agents.insert(agent_id.to_string(), agent);

        // Mission 1: Busy
        runner.update_status(agent_id, "mission-1", "busy", Some("Thinking..."));
        let agent = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(agent.health.status, "busy");
        assert_eq!(agent.state.current_task.as_deref(), Some("Thinking..."));
        let m1_active = agent.state.active_mission.clone().unwrap();
        assert_eq!(m1_active["id"], "mission-1");
        let m1_started = m1_active["started_at"].as_u64().unwrap();
        drop(agent);

        // Mission 2: Switching directly from mission-1 to mission-2 resets active_mission
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        runner.update_status(agent_id, "mission-2", "busy", Some("Executing..."));
        let agent = state.registry.agents.get(agent_id).unwrap();
        let m2_active = agent.state.active_mission.clone().unwrap();
        assert_eq!(m2_active["id"], "mission-2");
        assert!(m2_active["started_at"].as_u64().unwrap() >= m1_started);
        drop(agent);

        // Idle state clears active_mission
        runner.update_status(agent_id, "mission-2", "idle", None);
        let agent = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(agent.health.status, "idle");
        assert!(agent.state.active_mission.is_none());
    }

    #[tokio::test]
    async fn test_verify_mission_success_sentinel() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());

        // Happy path: passing test runner output
        assert!(runner.verify_mission_success("Running cargo test... test result: ok (3 passed)"));
        assert!(runner.verify_mission_success("pytest output: 12 passed, 0 failed"));
        assert!(runner.verify_mission_success("=== 12 passed in 1.24s ==="));
        assert!(runner.verify_mission_success("Tests: 5 passed, 5 total"));
        assert!(runner.verify_mission_success("CARGO TEST PASSED"));
        assert!(runner.verify_mission_success("pytest passed"));

        // Critical Finding #1 Regression Tests: Partial / failing test runs MUST NOT pass
        assert!(
            !runner.verify_mission_success("test result: FAILED. 12 passed; 3 failed; 0 ignored")
        );
        assert!(!runner.verify_mission_success("=== 3 failed, 9 passed in 1.24s ==="));
        assert!(!runner.verify_mission_success("test result: FAILED. 0 passed; 1 failed"));
        assert!(!runner.verify_mission_success("Running cargo test... test result: failed"));
        assert!(!runner.verify_mission_success("failures: test_foo panicked at 'assertion failed'"));
        assert!(
            !runner.verify_mission_success("error[E0425]: cannot find value `foo` in this scope")
        );

        // Critical Finding #2 Regression Tests: Prose, substring collisions, and filter bypasses MUST NOT pass
        assert!(!runner.verify_mission_success("The latest build was unsuccessful"));
        assert!(!runner.verify_mission_success("I ran the test and the operation was a success."));
        assert!(!runner.verify_mission_success("Test status: success")); // Prose without runner proof
        assert!(
            !runner.verify_mission_success("All operations completed successfully without errors.")
        );

        // Sentinel feedback echo (blockquoted, lowercased, etc.) MUST NOT self-satisfy
        assert!(!runner.verify_mission_success("\n[SENTINEL GATE]: Finalization BLOCKED. You must run a verification test (e.g. 'cargo test' or a reproduction script) and prove success before completing this mission. Your previous attempt lacked deterministic proof of correctness.\n"));
        assert!(!runner.verify_mission_success(
            "> [SENTINEL GATE]: Finalization BLOCKED. You must run a verification test"
        ));
        assert!(!runner.verify_mission_success("> [sentinel gate]: finalization blocked. you must run a verification test and prove success"));
    }

    #[test]
    fn test_execution_witness_sentinel() {
        use super::super::ProcessExecutionRecord;

        // Mutation happened at t = 1000
        let mutation_time = 1000;

        // 1. Passing test executed after mutation -> PASS
        let records_pass = vec![ProcessExecutionRecord {
            command: "cargo test --bin server-rs".to_string(),
            exit_code: 0,
            timestamp_ms: 1050,
        }];
        assert!(verify_execution_witnesses(&records_pass, mutation_time));

        // 2. Passing test executed before mutation (stale test) -> FAIL
        let records_stale = vec![ProcessExecutionRecord {
            command: "cargo test".to_string(),
            exit_code: 0,
            timestamp_ms: 950,
        }];
        assert!(!verify_execution_witnesses(&records_stale, mutation_time));

        // 3. Failing test executed after mutation -> FAIL
        let records_failed = vec![ProcessExecutionRecord {
            command: "pytest tests/".to_string(),
            exit_code: 1,
            timestamp_ms: 1100,
        }];
        assert!(!verify_execution_witnesses(&records_failed, mutation_time));

        // 4. Non-test command exiting 0 -> FAIL
        let records_non_test = vec![ProcessExecutionRecord {
            command: "cargo build".to_string(),
            exit_code: 0,
            timestamp_ms: 1200,
        }];
        assert!(!verify_execution_witnesses(
            &records_non_test,
            mutation_time
        ));

        // 5. Custom test script exiting 0 after mutation -> PASS
        let records_script = vec![ProcessExecutionRecord {
            command: "python execution/test_verification.py".to_string(),
            exit_code: 0,
            timestamp_ms: 1300,
        }];
        assert!(verify_execution_witnesses(&records_script, mutation_time));

        // 6. npm test / vitest exiting 0 -> PASS
        let records_npm = vec![ProcessExecutionRecord {
            command: "npm test".to_string(),
            exit_code: 0,
            timestamp_ms: 1400,
        }];
        assert!(verify_execution_witnesses(&records_npm, mutation_time));
    }

    #[tokio::test]
    async fn test_broadcast_agent_message() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut rx = state.comms.telemetry_tx.subscribe();

        runner.broadcast_agent_message("agent-1", "mission-1", "hello world", "assistant", 0);

        let msg = rx.try_recv().unwrap();
        assert_eq!(msg["type"], "agent:message");
        assert_eq!(msg["agent_id"], "agent-1");
        assert_eq!(msg["content"], "hello world");
        assert_eq!(msg["role"], "assistant");
        assert_eq!(msg["turn_index"], 0);
        assert!(msg["timestamp"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_oversight_cleanup_guard() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let entry_id = "test-cleanup-entry".to_string();

        let (tx, _rx) = tokio::sync::oneshot::channel();
        state.comms.oversight_resolvers.insert(entry_id.clone(), tx);
        state.comms.oversight_queue.insert(
            entry_id.clone(),
            crate::agent::types::OversightEntry {
                id: entry_id.clone(),
                mission_id: None,
                tool_call: None,
                skill_proposal: None,
                status: "pending".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        );

        assert!(state.comms.oversight_resolvers.contains_key(&entry_id));
        assert!(state.comms.oversight_queue.contains_key(&entry_id));

        {
            let _guard = OversightCleanupGuard {
                comms: &state.comms,
                id: entry_id.clone(),
            };
        }

        // After guard drops, both maps must be cleaned up
        assert!(!state.comms.oversight_resolvers.contains_key(&entry_id));
        assert!(!state.comms.oversight_queue.contains_key(&entry_id));
    }
}
