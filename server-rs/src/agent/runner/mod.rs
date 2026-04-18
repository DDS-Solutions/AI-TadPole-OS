//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **The Heartbeat**: This is the core `run()` loop. Orchestrates the
//! full mission lifecycle: **Setup -> Initialization -> ContextResolution
//! -> IntelligenceLoop -> Finalization**. Manages the `RunContext`
//! (state bag) and hierarchical OTel tracing for real-time "God View"
//! visualization. Supports **Deterministic Workflows** and **Recursive Swarm**
//! recruitment.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Budget exhaustion, recursion depth limit (SEC-01),
//!   prompt injection detection, or API provider timeouts.
//! - **Trace Scope**: `server-rs::agent::runner` (Check for `AgentExecution` span)
//! - **Telemetry**: Emits `agent:status` and `agent:message` events to
//!   the global telemetry bus.

use crate::agent::backlog::MissionBacklog;
use crate::agent::types::{ModelConfig, TaskPayload};
use crate::state::AppState;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// ─────────────────────────────────────────────────────────
//  SUBMODULES
// ─────────────────────────────────────────────────────────
mod analysis;
mod context;
mod external_tools;
mod finalize;
mod fs_tools;
mod intelligence;
mod lifecycle;
mod metrics_tools;
mod mission_tools;
mod oversight;
mod provider;
mod swarm;
mod synthesis;
mod tools;
mod workflow;

// ─────────────────────────────────────────────────────────
//  CORE TYPES
// ─────────────────────────────────────────────────────────

/// Context bag for data resolved during the setup phase of a run.
#[derive(Clone)]
pub(crate) struct RunContext {
    pub agent_id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub description: String,
    pub model_config: ModelConfig,
    pub skills: Vec<String>,
    pub workflows: Vec<String>,
    #[allow(dead_code)]
    pub mcp_tools: Vec<String>,
    pub mission_id: String,
    pub user_id: Option<String>,
    pub depth: u32,
    pub lineage: Vec<String>,
    pub provider_name: String,
    pub workspace_root: std::path::PathBuf,
    pub fs_adapter: crate::adapter::filesystem::FilesystemAdapter,
    pub safe_mode: bool,
    pub analysis: bool,
    pub traceparent: Option<String>,
    pub last_accessed_files: std::sync::Arc<parking_lot::Mutex<Vec<String>>>,
    pub recent_findings: Option<String>,
    pub working_memory: serde_json::Value,
    pub base_dir: std::path::PathBuf,
    pub summarized_history: Option<String>,
    pub structured_output: bool,
    pub backlog: Option<Arc<parking_lot::Mutex<MissionBacklog>>>,
    pub primary_goal: Option<String>,
    pub budget_usd: f64,
    pub current_cost_usd: f64,
}

impl Default for RunContext {
    fn default() -> Self {
        Self {
            agent_id: "default-agent".to_string(),
            name: "Default".to_string(),
            role: "Specialist".to_string(),
            department: "Standard".to_string(),
            description: "Default test context".to_string(),
            model_config: ModelConfig::default(),
            skills: vec![],
            workflows: vec![],
            mcp_tools: vec![],
            mission_id: "default-mission".to_string(),
            user_id: None,
            depth: 0,
            lineage: vec![],
            provider_name: "mock".to_string(),
            workspace_root: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: serde_json::json!({}),
            base_dir: std::path::PathBuf::from("."),
            summarized_history: None,
            structured_output: false,
            backlog: None,
            primary_goal: None,
            budget_usd: 0.0,
            current_cost_usd: 0.0,
        }
    }
}

impl RunContext {
    #[allow(dead_code)]
    pub fn resolve_paths(&self) -> (String, String, String) {
        let cluster_name = self
            .workspace_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let agent_memory_dir = self
            .base_dir
            .join("data/workspaces")
            .join(&cluster_name)
            .join("agents")
            .join(&self.agent_id)
            .join("memory.lance")
            .to_string_lossy()
            .to_string();
        let mission_scope_dir = self
            .base_dir
            .join("data/workspaces")
            .join(&cluster_name)
            .join("missions")
            .join(&self.mission_id)
            .join("scope.lance")
            .to_string_lossy()
            .to_string();
        (cluster_name, agent_memory_dir, mission_scope_dir)
    }

    pub fn derive_subtask_payload(&self, message: String) -> TaskPayload {
        TaskPayload {
            message,
            cluster_id: Some(self.mission_id.clone()),
            provider: Some(self.model_config.provider),
            model_id: Some(self.model_config.model_id.clone()),
            api_key: self.model_config.api_key.clone(),
            base_url: self.model_config.base_url.clone(),
            rpm: self.model_config.rpm,
            tpm: self.model_config.tpm,
            rpd: self.model_config.rpd,
            tpd: self.model_config.tpd,
            swarm_depth: Some(self.depth + 1),
            swarm_lineage: Some({
                let mut l = self.lineage.clone();
                l.push(self.agent_id.clone());
                l
            }),
            external_id: self.model_config.external_id.clone(),
            safe_mode: Some(self.safe_mode),
            traceparent: self.traceparent.clone(),
            user_id: self.user_id.clone(),
            context_files: Some(self.last_accessed_files.lock().clone()),
            recent_findings: self.recent_findings.clone(),
            structured_output: Some(false),
            primary_goal: self.primary_goal.clone(),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct AgentRunner {
    pub state: Arc<AppState>,
}

pub(crate) struct IntelligenceOutput {
    pub text: String,
    pub usage: Option<crate::agent::types::TokenUsage>,
}

struct ActiveAgentGuard<'a> {
    counter: &'a AtomicU32,
}

impl<'a> ActiveAgentGuard<'a> {
    fn acquire(counter: &'a AtomicU32) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self { counter }
    }
}

impl Drop for ActiveAgentGuard<'_> {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

impl AgentRunner {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Emits a diagnostic event to the global system terminal.
    pub(crate) fn broadcast_sys(&self, msg: &str, level: &str, mission_id: Option<String>) {
        self.state.broadcast_sys(msg, level, mission_id);
    }

    /// Emits an agent-specific personality event to the dashboard.
    pub(crate) fn broadcast_agent(&self, ctx: &RunContext, msg: &str, level: &str) {
        self.state.broadcast_agent(
            msg,
            level,
            Some(ctx.mission_id.clone()),
            &ctx.agent_id,
            &ctx.name,
        );
    }

    /// Updates the heartbeat timestamp in the registry and persistence layer.
    /// Used by the `reap_stale_agents` safety valve to detect hung missions.
    pub(crate) async fn record_heartbeat(&self, agent_id: &str) {
        let now = chrono::Utc::now();
        if let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) {
            entry.value_mut().heartbeat_at = Some(now);
        }
        let _ =
            crate::agent::persistence::update_agent_heartbeat(&self.state.resources.pool, agent_id)
                .await;
    }

    /// ### 🔄 Processing Pipeline: The Intelligence Heartbeat
    /// Orchestrates the full autonomous mission lifecycle for an agent identity.
    ///
    /// ### 🧬 Mission Phases
    /// 1. **Setup**: Resolves the static `RunContext` (agent name, role, department) 
    ///    and validates the incoming `TaskPayload` for circular recursion.
    /// 2. **Initialization**: Creates a persistent record in the `mission_history` 
    ///    table and clears the agent's short-term working memory.
    /// 3. **ContextResolution**: Loads the workspace path, canonicalizes the 
    ///    filesystem adapter, and injects semantic history (if enabled). Checks 
    ///    for active **Deterministic Workflows** (SOPs).
    /// 4. **IntelligenceLoop**: The primary cognitive cycle. Alternates between 
    ///    LLM inference (Reasoning) and Tool Execution (Interaction) until 
    ///    the goal is met or the budget/recursion limit is hit.
    /// 5. **Finalization**: Records total mission cost, logs the final completion 
    ///    text, and releases the `ActiveAgentGuard`.
    #[tracing::instrument(
        name = "AgentExecution",
        skip(self, payload),
        fields(
            agent_id = %agent_id,
            mission_id = %payload.cluster_id.as_deref().unwrap_or("unknown"),
            status = "running",
            swarm_depth = payload.swarm_depth.unwrap_or(0),
            trace_id = tracing::field::Empty
        )
    )]
    pub async fn run(&self, agent_id: String, mut payload: TaskPayload) -> anyhow::Result<String> {
        if payload.primary_goal.is_none() {
            payload.primary_goal = Some(payload.message.clone());
        }

        let _mission_id = payload
            .cluster_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        self.state.yield_phase_transition(&agent_id, "Setup").await;
        self.setup_and_validate(&agent_id, &payload)?;

        self.state
            .yield_phase_transition(&agent_id, "Initialization")
            .await;
        let _active_agent_guard = ActiveAgentGuard::acquire(&self.state.governance.active_agents);
        let mission = self.initialize_mission_state(&agent_id, &payload).await?;
        let mission_id = mission.id.clone();

        self.state
            .yield_phase_transition(&agent_id, "ContextResolution")
            .await;
        let agent_data = self
            .state
            .registry
            .agents
            .get(&agent_id)
            .map(|a| a.value().clone())
            .ok_or_else(|| anyhow::anyhow!("Agent not found for context resolution"))?;

        if let Some(workflow_name) = agent_data.workflows.first() {
            let msg_lower = payload.message.to_lowercase();
            let workflow_requested = msg_lower
                .contains(&workflow_name.to_lowercase().replace("_", " "))
                || msg_lower.contains("workflow")
                || msg_lower.contains("sop");
            let is_sme = agent_data.department.to_lowercase().contains("sme")
                || agent_data.role.to_lowercase().contains("specialist");

            if workflow_requested || is_sme {
                if let Ok(mut state) = crate::agent::workflows::load_workflow(
                    self.state.base_dir.as_path(),
                    workflow_name,
                )
                .await
                {
                    return self
                        .run_deterministic_workflow(&agent_id, payload, &mut state)
                        .await;
                }
            }
        }

        let depth = payload.swarm_depth.unwrap_or(0);
        let lineage = payload.swarm_lineage.clone().unwrap_or_default();
        let ctx = self
            .prepare_run_context(&agent_id, &payload, &mission_id, depth, &lineage)
            .await?;

        self.state
            .yield_phase_transition(&agent_id, "IntelligenceLoop")
            .await;
        self.record_heartbeat(&ctx.agent_id).await;
        let output_res = self.execute_intelligence_loop(&ctx, &payload).await;

        match output_res {
            Ok(output) => {
                self.state
                    .yield_phase_transition(&agent_id, "Finalization")
                    .await;
                self.record_heartbeat(&ctx.agent_id).await;
                self.finalize_run(&ctx, &output.text, &output.usage).await
            }
            Err((e, usage)) => {
                self.fail_mission(&ctx, &e, &usage).await?;
                Err(e)
            }
        }
    }

    /// Safely truncates a string to a byte limit without breaking UTF-8 boundaries.
    pub(crate) fn safe_truncate(&self, s: &str, limit: usize) -> String {
        if s.len() <= limit {
            return s.to_string();
        }
        let mut end = limit;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}... [TRUNCATED]", &s[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_payload(msg: &str) -> TaskPayload {
        TaskPayload {
            message: msg.to_string(),
            primary_goal: Some(msg.to_string()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_finalize_run_fallback_on_empty_output() {
        let state = Arc::new(crate::state::AppState::new().await.unwrap());
        let runner = AgentRunner::new(state.clone());
        let agent_id = "test-fin".to_string();
        let mission_id = "mission-fin".to_string();

        // Must insert into DB first because finalize_run updates/logs to mission tables
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Test Runner', 'tester', 'QA', 'desc', 'idle', '{}')")
            .bind(&agent_id)
            .execute(&state.resources.pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Test Mission', 'active')")
            .bind(&mission_id)
            .bind(&agent_id)
            .execute(&state.resources.pool)
            .await
            .unwrap();

        let ctx = RunContext {
            agent_id,
            mission_id,
            ..RunContext::default()
        };
        let res = runner.finalize_run(&ctx, "  ", &None).await.unwrap();
        assert!(res.contains("completed its actions"));
    }

    #[tokio::test]
    async fn validate_input_detects_circular_recursion() {
        let (runner, _) = (
            AgentRunner::new(Arc::new(crate::state::AppState::new().await.unwrap())),
            (),
        );
        let mut payload = make_payload("test");
        payload.swarm_lineage = Some(vec!["agent-1".to_string(), "agent-2".to_string()]);
        let res = runner.validate_input("agent-1", &payload);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("CIRCULAR"));
    }

    #[tokio::test]
    async fn build_system_prompt_includes_lineage_when_present() {
        let state = Arc::new(crate::state::AppState::new().await.unwrap());
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            agent_id: "test-3".to_string(),
            lineage: vec!["agent-1".to_string(), "agent-2".to_string()],
            ..RunContext::default()
        };
        let prompt = runner.build_system_prompt(&ctx, "root", "hi").await;
        assert!(prompt.contains("agent-1 -> agent-2"));
    }
}
