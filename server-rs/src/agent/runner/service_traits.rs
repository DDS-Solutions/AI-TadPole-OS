//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[service_traits]` in tracing logs.

use crate::agent::types::RoleAuthorityLevel;
use std::collections::HashMap;

/// Interface for rendering system prompts with variable interpolation.
pub trait PromptRendererTrait: Send + Sync {
    /// Renders a template string using the provided variable map.
    fn render(&self, template: &str, variables: &HashMap<&str, String>) -> String;

    /// Returns the default system prompt template.
    fn default_system_template(&self) -> &'static str;
}

/// Interface for Access Control Lists (ACL) governance.
pub trait AclServiceTrait: Send + Sync {
    /// Checks if a tool is allowed for a specific agent/role/authority level.
    fn is_tool_allowed(
        &self,
        agent_id: &str,
        role: &str,
        authority: RoleAuthorityLevel,
        tool_name: &str,
    ) -> bool;

    /// Returns mandatory protocols for a given agent and role.
    fn get_role_protocols(
        &self,
        agent_id: &str,
        role: &str,
        authority: RoleAuthorityLevel,
    ) -> Vec<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    Planning,
    Execution,
    Default,
}

#[derive(Debug, Clone)]
pub struct SlotSelection {
    pub config: crate::agent::types::ModelConfig,
    pub kind: SlotKind,
    pub privacy_local_override: bool,
}

#[derive(Debug, Clone)]
pub struct ToolOrchestrationResult {
    pub observation_buffer: String,
    pub mission_completed: bool,
    pub final_report: Option<String>,
    pub active_slot_override: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AgentMissionState {
    Initial,
    SpecificationGeneration,
    AwaitingToolCalls,
    Reasoning,
    Execution,
    Finalizing,
    Halted,
}

impl AgentMissionState {
    /// Resolves the current mission phase from the spec content and mode flags.
    /// This drives the actual dispatch in execute_intelligence_loop (Fix 6).
    pub fn resolve(spec: &str, safe_mode: bool, is_fast_path: bool) -> Self {
        if safe_mode || is_fast_path {
            return AgentMissionState::Reasoning;
        }
        if !spec.contains("--- [ROOM: system::spec] ---") {
            AgentMissionState::SpecificationGeneration
        } else {
            AgentMissionState::Reasoning
        }
    }
}

pub trait ModelRouter: Send + Sync {
    fn select_model_slot(
        &self,
        state: &crate::state::AppState,
        models: &crate::agent::types::AgentModels,
        preferred: SlotKind,
    ) -> SlotSelection;
}

#[async_trait::async_trait]
pub trait PromptService: Send + Sync {
    async fn build_system_prompt(
        &self,
        runner: &super::AgentRunner,
        ctx: &super::RunContext,
        payload_message: &str,
    ) -> String;
}

#[async_trait::async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute_tool(
        &self,
        ctx: &super::RunContext,
        fc: &crate::agent::types::ToolCall,
        user_message: &str,
    ) -> Result<(String, Option<crate::agent::types::TokenUsage>), crate::error::AppError>;

    fn update_status(&self, agent_id: &str, mission_id: &str, status: &str, task: Option<&str>);

    fn handle_tool_failure_refinement(
        &self,
        ctx: &super::RunContext,
        fc: &crate::agent::types::ToolCall,
        local_text: &mut String,
    );

    fn accumulate_usage(
        &self,
        accumulated: &mut Option<crate::agent::types::TokenUsage>,
        new_usage: Option<crate::agent::types::TokenUsage>,
    );

    fn verify_mission_success(&self, observation_buffer: &str) -> bool;

    fn broadcast_agent(&self, ctx: &super::RunContext, msg: &str, level: &str);

    fn handle_tool_failure_slot_swap(&self, agent_id: &str) -> Option<String>;
}

#[async_trait::async_trait]
pub trait ToolOrchestrator: Send + Sync {
    async fn execute_tools(
        &self,
        executor: std::sync::Arc<dyn ToolExecutor>,
        active_ctx: &super::RunContext,
        function_calls: Vec<crate::agent::types::ToolCall>,
        user_message: &str,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<ToolOrchestrationResult, crate::error::AppError>;
}

#[async_trait::async_trait]
pub trait WorkflowCoordinator: Send + Sync {
    async fn execute_workflow(
        &self,
        runner: &super::AgentRunner,
        ctx: &super::RunContext,
        payload: &crate::agent::types::TaskPayload,
    ) -> Result<
        super::IntelligenceOutput,
        (
            crate::error::AppError,
            Option<crate::agent::types::TokenUsage>,
        ),
    >;
}

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait MissionStateManager: Send + Sync {
    async fn yield_phase_transition(
        &self,
        state: &crate::state::AppState,
        agent_id: &str,
        phase: &str,
    );
    fn update_status(
        &self,
        state: &crate::state::AppState,
        agent_id: &str,
        mission_id: &str,
        status: &str,
        task: Option<&str>,
    );
    async fn set_mission_spec(
        &self,
        state: &crate::state::AppState,
        mission_id: &str,
        agent_id: &str,
        spec_content: &str,
    ) -> Result<(), crate::error::AppError>;
}

pub struct DefaultModelRouter;

impl ModelRouter for DefaultModelRouter {
    fn select_model_slot(
        &self,
        state: &crate::state::AppState,
        models: &crate::agent::types::AgentModels,
        preferred: SlotKind,
    ) -> SlotSelection {
        let privacy_mode_active = state
            .governance
            .privacy_mode
            .load(std::sync::atomic::Ordering::Acquire);

        let preferred_config = match preferred {
            SlotKind::Planning => models.planning_slot.as_ref(),
            SlotKind::Execution => models.execution_slot.as_ref(),
            SlotKind::Default => Some(&models.model),
        };

        if privacy_mode_active {
            // 1. Preferred local slot
            if let Some(config) = preferred_config {
                if crate::agent::model_routing::is_local_model_config(config) {
                    return SlotSelection {
                        config: config.clone(),
                        kind: preferred,
                        privacy_local_override: true,
                    };
                }
            }

            // 2. Default local model
            if crate::agent::model_routing::is_local_model_config(&models.model) {
                return SlotSelection {
                    config: models.model.clone(),
                    kind: SlotKind::Default,
                    privacy_local_override: true,
                };
            }

            // 3. Alternate local slot
            let alternate = match preferred {
                SlotKind::Planning => Some((SlotKind::Execution, &models.execution_slot)),
                SlotKind::Execution => Some((SlotKind::Planning, &models.planning_slot)),
                SlotKind::Default => None,
            };
            if let Some((kind, Some(config))) = alternate {
                if crate::agent::model_routing::is_local_model_config(config) {
                    return SlotSelection {
                        config: config.clone(),
                        kind,
                        privacy_local_override: true,
                    };
                }
            }

            // 4. Synthesized Ollama fallback
            return SlotSelection {
                config: crate::agent::model_routing::privacy_fallback_config(),
                kind: SlotKind::Default,
                privacy_local_override: true,
            };
        }

        if let Some(config) = preferred_config {
            return SlotSelection {
                config: config.clone(),
                kind: preferred,
                privacy_local_override: false,
            };
        }

        SlotSelection {
            config: models.model.clone(),
            kind: SlotKind::Default,
            privacy_local_override: false,
        }
    }
}

pub struct DefaultPromptService;

#[async_trait::async_trait]
impl PromptService for DefaultPromptService {
    async fn build_system_prompt(
        &self,
        runner: &super::AgentRunner,
        ctx: &super::RunContext,
        payload_message: &str,
    ) -> String {
        runner.build_system_prompt(ctx, payload_message).await
    }
}

pub struct DefaultToolOrchestrator {
    loop_detectors: parking_lot::Mutex<
        std::collections::HashMap<
            String,
            (super::intelligence::DoomLoopDetector, std::time::Instant),
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

#[async_trait::async_trait]
impl ToolOrchestrator for DefaultToolOrchestrator {
    async fn execute_tools(
        &self,
        executor: std::sync::Arc<dyn ToolExecutor>,
        active_ctx: &super::RunContext,
        function_calls: Vec<crate::agent::types::ToolCall>,
        user_message: &str,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<ToolOrchestrationResult, crate::error::AppError> {
        use futures::stream::{FuturesUnordered, StreamExt};

        // TTL Eviction: Purge stale DoomLoopDetector entries (>30 min) to prevent memory leaks
        // from missions that end via budget breach, max turn exhaustion, or early return (Audit 1.4)
        {
            let mut guard = self.loop_detectors.lock();
            let cutoff = std::time::Instant::now() - std::time::Duration::from_secs(30 * 60);
            guard.retain(|_, (_, created_at)| *created_at > cutoff);
        }

        let mut futures = FuturesUnordered::new();
        for fc in function_calls {
            let executor_clone = executor.clone();
            let ctx_clone = active_ctx.clone();
            let user_msg_clone = user_message.to_string();
            let fc_name = fc.name.clone();
            let fc_args_str = serde_json::to_string(&fc.args).unwrap_or_default();
            futures.push(async move {
                executor_clone.update_status(
                    &ctx_clone.agent_id,
                    &ctx_clone.mission_id,
                    "working",
                    Some(&format!("Executing tool: {}...", fc.name)),
                );
                let (local_text, local_usage, success) = match executor_clone
                    .execute_tool(&ctx_clone, &fc, &user_msg_clone)
                    .await
                {
                    Ok((text, usage)) => (text, usage, true),
                    Err(e) => (format!("(TOOL FAILURE: {:?})", e), None, false),
                };

                let mut local_text_refined = local_text;
                // 🧬 [Evolution] Autonomous Refinement Hook
                executor_clone.handle_tool_failure_refinement(
                    &ctx_clone,
                    &fc,
                    &mut local_text_refined,
                );

                (
                    fc_name,
                    fc_args_str,
                    success,
                    local_text_refined,
                    local_usage,
                )
            });
        }

        let mut observation_buffer = String::new();
        let mut mission_completed = false;
        let mut final_report = None;
        let mut active_slot_override = None;

        while let Some((name, args_str, success, mut local_text, local_usage)) =
            futures.next().await
        {
            executor.accumulate_usage(usage, local_usage);

            let has_loop = {
                let mut guard = self.loop_detectors.lock();
                let (detector, _) =
                    guard
                        .entry(active_ctx.mission_id.clone())
                        .or_insert_with(|| {
                            (
                                super::intelligence::DoomLoopDetector::new(),
                                std::time::Instant::now(),
                            )
                        });
                detector.check(&name, &args_str, &local_text)
            };

            // Hashed Loop & Error Cycle Detection
            if has_loop {
                tracing::warn!(
                    "🛑 [DoomLoopDetector] Loop detected on tool {}! Halting agent.",
                    name
                );
                return Err(crate::error::AppError::Forbidden(format!(
                    "Execution halted: infinite tool cycle or repeating error loop detected on tool '{}'.",
                    name
                )));
            }

            // Preemptive large tool output truncation to optimize context window tokens
            // Uses floor_char_boundary to prevent UTF-8 panics on multi-byte characters (Audit 2.1)
            if local_text.len() > 4000 {
                let original_len = local_text.len();
                let mut boundary = local_text.floor_char_boundary(4000);
                // Don't split [REDACTED_*] markers at the truncation point (Audit 3.3)
                if let Some(marker_start) = local_text[..boundary].rfind("[REDACTED_") {
                    if local_text[marker_start..boundary].rfind(']').is_none() {
                        boundary = marker_start;
                    }
                }
                local_text = format!(
                    "{}... [Tool output truncated to optimize context window — original size: {} characters]",
                    &local_text[..boundary],
                    original_len
                );
            }

            // Builder-Debugger Slot Swap on Failure
            let is_failure = !success
                || local_text.contains("(TOOL FAILURE:")
                || local_text.contains("error:")
                || local_text.contains("compilation failed")
                || local_text.contains("test failure")
                || local_text.contains("FAILED");

            if is_failure {
                if let Some(new_slot) = executor.handle_tool_failure_slot_swap(&active_ctx.agent_id)
                {
                    active_slot_override = Some(new_slot);
                }
            }

            // Sandboxed contextual observation propagation
            if let Some(ref vt) = active_ctx.visible_transcript {
                let clean_obs = if local_text.len() > 300 {
                    format!("{}... [TRUNCATED]", &local_text[..300])
                } else {
                    local_text.clone()
                };
                vt.lock()
                    .push(format!("OBSERVATION (Tool {}): {}", name, clean_obs));
            }

            observation_buffer.push_str(&format!("\nTool {} Result: {}", name, local_text));

            if name == "complete_mission" {
                // --- ✅ [System 2] Success-by-Verification Sentinel ---
                if !executor.verify_mission_success(&observation_buffer) && !active_ctx.safe_mode {
                    executor.broadcast_agent(
                        active_ctx,
                        "🚨 Sentinel Gate: Finalization BLOCKED. No proof of verification found.",
                        "warning",
                    );
                    observation_buffer.push_str("\n[SENTINEL GATE]: Finalization BLOCKED. You must run a verification test (e.g. 'cargo test' or a reproduction script) and prove success before completing this mission. Your previous attempt lacked deterministic proof of correctness.");
                    mission_completed = false;
                } else {
                    mission_completed = true;
                    final_report = Some(local_text);
                    self.loop_detectors.lock().remove(&active_ctx.mission_id);
                }
            }
        }

        Ok(ToolOrchestrationResult {
            observation_buffer,
            mission_completed,
            final_report,
            active_slot_override,
        })
    }
}

pub struct DefaultMissionStateManager;

#[async_trait::async_trait]
impl MissionStateManager for DefaultMissionStateManager {
    async fn yield_phase_transition(
        &self,
        state: &crate::state::AppState,
        agent_id: &str,
        phase: &str,
    ) {
        state.yield_phase_transition(agent_id, phase).await;
    }

    fn update_status(
        &self,
        state: &crate::state::AppState,
        agent_id: &str,
        mission_id: &str,
        status: &str,
        task: Option<&str>,
    ) {
        if let Some(mut entry) = state.registry.agents.get_mut(agent_id) {
            let agent = entry.value_mut();
            agent.health.status = status.to_string();
            agent.state.current_task = task.map(|t| t.to_string());

            // Sync active mission for high-speed pulse telemetry
            if status == "idle" {
                agent.state.active_mission = None;
            } else if agent.state.active_mission.is_none() {
                agent.state.active_mission = Some(serde_json::json!({ "id": mission_id }));
            }

            // Sync status to the database (F-10)
            let pool = state.resources.pool.clone();
            let mut agent_clone = agent.clone();
            tokio::spawn(async move {
                if let Err(e) = crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await {
                    tracing::error!("❌ Failed to sync agent status update to database: {}", e);
                }
            });
        }

        let task_data = state
            .registry
            .agents
            .get(agent_id)
            .and_then(|a| a.state.current_task.clone());

        let _ = state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:status",
            "agent_id": agent_id,
            "mission_id": mission_id,
            "status": status,
            "current_task": task_data
        }));
    }

    async fn set_mission_spec(
        &self,
        state: &crate::state::AppState,
        mission_id: &str,
        agent_id: &str,
        spec_content: &str,
    ) -> Result<(), crate::error::AppError> {
        crate::agent::mission::set_mission_spec(
            &state.resources.pool,
            mission_id,
            agent_id,
            spec_content,
        )
        .await
    }
}

/// Undo operations for state transactions (Fix 2: Formalized Rollback).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UndoOp {
    RestoreAgentRegistry {
        agent_id: String,
        status: String,
        task: Option<String>,
        active_mission: Option<serde_json::Value>,
    },
    DeleteMissionSpec {
        mission_id: String,
        agent_id: String,
    },
    RestoreMissionStatus {
        mission_id: String,
        status: crate::agent::types::MissionStatus,
    },
}

impl UndoOp {
    pub async fn execute(
        &self,
        state: &crate::state::AppState,
    ) -> Result<(), crate::error::AppError> {
        match self {
            UndoOp::RestoreAgentRegistry {
                agent_id,
                status,
                task,
                active_mission,
            } => {
                if let Some(mut entry) = state.registry.agents.get_mut(agent_id) {
                    let agent = entry.value_mut();
                    agent.health.status = status.clone();
                    agent.state.current_task = task.clone();
                    agent.state.active_mission = active_mission.clone();
                }
                let _ = state.comms.telemetry_tx.send(serde_json::json!({
                    "type": "agent:status",
                    "agent_id": agent_id,
                    "mission_id": "",
                    "status": status,
                    "current_task": task
                }));
                Ok(())
            }
            UndoOp::DeleteMissionSpec {
                mission_id,
                agent_id,
            } => {
                sqlx::query::<sqlx::Sqlite>(
                    "DELETE FROM swarm_context WHERE mission_id = ?1 AND agent_id = ?2 AND topic = 'system::spec'"
                )
                .bind(mission_id)
                .bind(agent_id)
                .execute(&state.resources.pool)
                .await?;
                Ok(())
            }
            UndoOp::RestoreMissionStatus { mission_id, status } => {
                let status_str = match status {
                    crate::agent::types::MissionStatus::Pending => "pending",
                    crate::agent::types::MissionStatus::SpecReview => "spec_review",
                    crate::agent::types::MissionStatus::Active => "active",
                    crate::agent::types::MissionStatus::Completed => "completed",
                    crate::agent::types::MissionStatus::Failed => "failed",
                    crate::agent::types::MissionStatus::Paused => "paused",
                };
                sqlx::query::<sqlx::Sqlite>(
                    "UPDATE mission_history SET status = ?1, updated_at = ?2 WHERE id = ?3",
                )
                .bind(status_str)
                .bind(chrono::Utc::now())
                .bind(mission_id)
                .execute(&state.resources.pool)
                .await?;
                Ok(())
            }
        }
    }
}

pub struct StateTransaction {
    #[allow(dead_code)]
    manager: std::sync::Arc<dyn MissionStateManager>,
    state: std::sync::Arc<crate::state::AppState>,
    agent_id: String,
    mission_id: String,
    committed: bool,
    undo_ops: Vec<UndoOp>,
}

impl StateTransaction {
    pub fn new(
        manager: std::sync::Arc<dyn MissionStateManager>,
        state: std::sync::Arc<crate::state::AppState>,
        agent_id: &str,
        mission_id: &str,
    ) -> Self {
        let (original_status, original_task, original_active_mission) =
            if let Some(entry) = state.registry.agents.get(agent_id) {
                let agent = entry.value();
                (
                    agent.health.status.clone(),
                    agent.state.current_task.clone(),
                    agent.state.active_mission.clone(),
                )
            } else {
                ("idle".to_string(), None, None)
            };

        let initial_agent_undo = UndoOp::RestoreAgentRegistry {
            agent_id: agent_id.to_string(),
            status: original_status,
            task: original_task,
            active_mission: original_active_mission,
        };

        Self {
            manager,
            state,
            agent_id: agent_id.to_string(),
            mission_id: mission_id.to_string(),
            committed: false,
            undo_ops: vec![initial_agent_undo],
        }
    }

    #[allow(dead_code)]
    pub fn record_agent_status_change(
        &mut self,
        _agent_id: &str,
        _status: &str,
        _task: Option<&str>,
    ) {
        // Registry status modifications are captured by the primary registry undo operation
    }

    pub fn record_mission_spec_change(&mut self, mission_id: &str, agent_id: &str) {
        self.undo_ops.push(UndoOp::DeleteMissionSpec {
            mission_id: mission_id.to_string(),
            agent_id: agent_id.to_string(),
        });
    }

    #[allow(dead_code)]
    pub fn record_mission_status_change(
        &mut self,
        mission_id: &str,
        status: crate::agent::types::MissionStatus,
    ) {
        self.undo_ops.push(UndoOp::RestoreMissionStatus {
            mission_id: mission_id.to_string(),
            status,
        });
    }

    pub fn commit(mut self) {
        self.committed = true;
    }

    pub async fn rollback(&mut self) -> Result<(), crate::error::AppError> {
        if !self.committed {
            self.committed = true;
            let ops = std::mem::take(&mut self.undo_ops);
            tracing::info!(
                "🔄 [StateTransaction] Explicit rollback triggered for agent {} on mission {}",
                self.agent_id,
                self.mission_id
            );
            for op in ops.into_iter().rev() {
                op.execute(&self.state).await?;
            }
        }
        Ok(())
    }
}

impl Drop for StateTransaction {
    fn drop(&mut self) {
        if !self.committed {
            let state = self.state.clone();
            let agent_id = self.agent_id.clone();
            let mission_id = self.mission_id.clone();
            let ops = std::mem::take(&mut self.undo_ops);

            tokio::spawn(async move {
                tracing::warn!(
                    "🚨 [StateTransaction] IMPLICIT ROLLBACK TRIGGERED INSIDE DROP for agent {} on mission {}. Use explicit .rollback() to ensure deterministic sequencing.",
                    agent_id,
                    mission_id
                );
                for op in ops.into_iter().rev() {
                    if let Err(e) = op.execute(&state).await {
                        tracing::error!(
                            "❌ [StateTransaction] Implicit rollback operation failed: {:?}",
                            e
                        );
                    }
                }
            });
        }
    }
}

pub struct IdentityService;

impl IdentityService {
    /// Validates if an agent ID maps to a known orchestrator.
    /// Non-mutable, hardcoded check to secure trust boundaries (ID-01).
    pub fn is_orchestrator(agent_id: &str) -> bool {
        matches!(
            agent_id,
            crate::agent::constants::AGENT_CEO
                | crate::agent::constants::AGENT_COO
                | crate::agent::constants::AGENT_ALPHA
        )
    }
}

// Metadata: [service_traits]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::EngineAgent;
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_identity_service() {
        assert!(IdentityService::is_orchestrator(
            crate::agent::constants::AGENT_CEO
        ));
        assert!(IdentityService::is_orchestrator(
            crate::agent::constants::AGENT_COO
        ));
        assert!(IdentityService::is_orchestrator(
            crate::agent::constants::AGENT_ALPHA
        ));
        assert!(!IdentityService::is_orchestrator("agent-specialist"));
        assert!(!IdentityService::is_orchestrator("another-agent"));
    }

    #[tokio::test]
    async fn test_state_transaction_rollback() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let agent_id = "test-agent";
        let mission_id = "mission-1";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        agent.health.status = "idle".to_string();
        agent.state.current_task = None;
        state.registry.agents.insert(agent_id.to_string(), agent);

        let manager = Arc::new(DefaultMissionStateManager);

        {
            let _tx = StateTransaction::new(manager.clone(), state.clone(), agent_id, mission_id);
            // Simulate status update during execution
            manager.update_status(&state, agent_id, mission_id, "busy", Some("Running..."));

            let entry = state.registry.agents.get(agent_id).unwrap();
            assert_eq!(entry.value().health.status, "busy");
            assert_eq!(
                entry.value().state.current_task.as_deref(),
                Some("Running...")
            );
            // Drop without committing triggers rollback
        }

        // Wait a brief moment for the background rollback to execute
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Verify status rolled back
        let entry = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(entry.value().health.status, "idle");
        assert_eq!(entry.value().state.current_task, None);
    }

    #[tokio::test]
    async fn test_state_transaction_commit() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let agent_id = "test-agent";
        let mission_id = "mission-1";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        agent.health.status = "idle".to_string();
        agent.state.current_task = None;
        state.registry.agents.insert(agent_id.to_string(), agent);

        let manager = Arc::new(DefaultMissionStateManager);

        {
            let tx = StateTransaction::new(manager.clone(), state.clone(), agent_id, mission_id);
            manager.update_status(&state, agent_id, mission_id, "busy", Some("Running..."));
            tx.commit();
        }

        // Verify status did NOT roll back since it was committed
        let entry = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(entry.value().health.status, "busy");
        assert_eq!(
            entry.value().state.current_task.as_deref(),
            Some("Running...")
        );
    }
}
