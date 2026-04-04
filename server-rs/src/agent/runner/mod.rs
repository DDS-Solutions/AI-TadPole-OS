//! Agent Runner — Mission execution and task loop
//!
//! This module implements the core runner responsible for taking an agent mission, 
//! resolving its context, and executing it via a unified LLM interface.
//!
//! @docs ARCHITECTURE:Runner
//! @docs OPERATIONS_MANUAL:SwarmOrchestration
//!
//! @state MissionStatus: (Pending | Active | Completed | Failed | Paused)
//! @state AgentHealth: (Healthy | Degraded | SelfHealCooldown)
//!
//! ### AI Assist Note
//! **The Heartbeat**: This is the core `run()` loop. It manages the 
//! `RunContext` (state bag for execution) and hierarchical OTel tracing. 
//! Every mission transitions through: **Setup -> Initialization -> ContextResolution -> IntelligenceLoop -> Finalization**.
//! Use `RecordHeartbeat` to maintain liveness in the dashboard.
//!
//! ### Module Layout
//! - `mod.rs` — Core structs, `run()` entry point, `record_heartbeat()`
//! - `lifecycle.rs` — Setup, validation, and mission initialization
//! - `context.rs` — Agent identity resolution, model config, RAG injection
//! - `intelligence.rs` — LLM provider calls, tool dispatch, budget enforcement
//! - `finalize.rs` — Result persistence, memory archival, failure handling
//! - `oversight.rs` — Human-in-the-loop approval and telemetry helpers
//! - `workflow.rs` — Deterministic SOP-driven step execution
//! - `synthesis.rs` — System prompt generation and context assembly
//! - `provider.rs` — LLM provider abstraction and call routing
//! - `tools.rs` — Tool definition and dispatch logic
//! - `swarm.rs` — Sub-agent recruitment and parallel execution
//! - `analysis.rs` — Post-mission analysis spawning
//! - `external_tools.rs` — External MCP tool integration
//! - `fs_tools.rs` — Filesystem tool implementations
//! - `mission_tools.rs` — Mission-specific tool implementations
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Budget exhaustion, recursion depth limit, prompt injection detection, or API provider timeouts.
//! - **Trace Scope**: `server-rs::agent::runner` (Check for `AgentExecution` span)
//! - **Budget Gate**: Refer to `crate::security::metering` for quota enforcement.
use crate::agent::backlog::MissionBacklog;
use crate::agent::types::{ModelConfig, TaskPayload};
use crate::state::AppState;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;

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
mod mission_tools;
mod oversight;
mod provider;
mod swarm;
mod synthesis;
mod tools;
mod workflow;

// Note: spawn_post_mission_analysis is imported directly in finalize.rs

// ─────────────────────────────────────────────────────────
//  CORE TYPES
// ─────────────────────────────────────────────────────────

/// Context bag for data resolved during the setup phase of a run.
///
/// This struct aggregates all state required for an agent execution session,
/// including identity, governance limits, workspace paths, and telemetry markers.
/// Avoids the "argument explosion" anti-pattern in internal runner methods.
#[derive(Clone)]
pub(crate) struct RunContext {
    /// Unique identifier for the agent instance.
    pub agent_id: String,
    /// Friendly name for display and logs.
    pub name: String,
    /// The agent's functional role (e.g., "Developer").
    pub role: String,
    /// The organizational unit (e.g., "Standard-Core").
    pub department: String,
    /// High-level personality and utility instructions.
    pub description: String,
    /// Config for the specific model assigned to this run.
    pub model_config: ModelConfig,
    /// List of enabled skill IDs (tools).
    pub skills: Vec<String>,
    /// List of enabled workflow IDs (procedures).
    pub workflows: Vec<String>,
    /// List of enabled MCP tool IDs.
    #[allow(dead_code)]
    pub mcp_tools: Vec<String>,
    /// The unique ID of the mission cluster.
    pub mission_id: String,
    /// Optional ID of the user who initiated the request.
    pub user_id: Option<String>,
    /// Current recursion depth in the swarm.
    pub depth: u32,
    /// Ancestry path of nodes that led to this one.
    pub lineage: Vec<String>,
    /// Normalized name of the LLM provider (e.g., "openai").
    pub provider_name: String,
    /// Physical path to the cluster's data sandbox.
    pub workspace_root: std::path::PathBuf,
    /// Secure filesystem bridge for the workspace.
    pub fs_adapter: crate::adapter::filesystem::FilesystemAdapter,
    /// If true, mutation and recruitment tools are disabled.
    pub safe_mode: bool,
    /// If true, performs additional iterative refinement.
    pub analysis: bool,
    /// OTel traceparent for cross-service tracing.
    pub traceparent: Option<String>,
    /// Tracks the last few files accessed for context propagation.
    pub last_accessed_files: std::sync::Arc<parking_lot::Mutex<Vec<String>>>,
    /// Summary of recent findings from previous turns or parents.
    pub recent_findings: Option<String>,
    /// Structured reasoning scratchpad.
    pub working_memory: serde_json::Value,
    /// Global workspace root directory for data resolution.
    pub base_dir: std::path::PathBuf,
    /// Compressed semantic summary of mission history.
    pub summarized_history: Option<String>,
    /// If true, enforces JSON mode for the provider response.
    pub structured_output: bool,
    /// Shared mission-level task backlog.
    pub backlog: Option<Arc<parking_lot::Mutex<MissionBacklog>>>,
}

impl RunContext {
    /// Resolves common workspace and data paths for the current mission.
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

    /// Derives a TaskPayload for a recursive sub-agent recruitment.
    /// Safely propagates traceparents and parent context.
    pub fn derive_subtask_payload(&self, message: String) -> TaskPayload {
        TaskPayload {
            message,
            cluster_id: Some(self.mission_id.clone()),
            department: None,
            provider: Some(self.model_config.provider.clone()),
            model_id: Some(self.model_config.model_id.clone()),
            api_key: self.model_config.api_key.clone(),
            base_url: self.model_config.base_url.clone(),
            rpm: self.model_config.rpm,
            tpm: self.model_config.tpm,
            budget_usd: None,
            swarm_depth: Some(self.depth + 1),
            swarm_lineage: Some({
                let mut l = self.lineage.clone();
                l.push(self.agent_id.clone());
                l
            }),
            external_id: self.model_config.external_id.clone(),
            safe_mode: Some(self.safe_mode),
            analysis: None,
            traceparent: self.traceparent.clone(),
            user_id: self.user_id.clone(),
            context_files: Some(self.last_accessed_files.lock().clone()),
            recent_findings: self.recent_findings.clone(),
            structured_output: Some(false),
        }
    }
}

/// Internal result type for the intelligence loop.
pub(crate) struct IntelligenceOutput {
    pub text: String,
    pub usage: Option<crate::agent::types::TokenUsage>,
}

// ─────────────────────────────────────────────────────────
//  AGENT RUNNER
// ─────────────────────────────────────────────────────────

/// The primary entry point for executing autonomous agent tasks.
///
/// The `AgentRunner` wraps the shared `AppState` and provides methods for
/// orchestrating the full lifecycle of a mission, from prompt synthesis to
/// final result persistence.
#[derive(Clone)]
pub struct AgentRunner {
    /// Reference to the global application state.
    pub state: Arc<AppState>,
}

/// Ensures the active agent counter is decremented on every exit path.
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

    pub(crate) fn broadcast_sys(&self, msg: &str, level: &str, mission_id: Option<String>) {
        self.state.broadcast_sys(msg, level, mission_id);
    }

    /// Updates the heartbeat for the agent in both memory and database.
    pub(crate) async fn record_heartbeat(&self, agent_id: &str) {
        let now = chrono::Utc::now();

        // 1. Update in-memory registry
        if let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) {
            entry.value_mut().heartbeat_at = Some(now);
        }

        // 2. Persist to DB
        let _ =
            crate::agent::persistence::update_agent_heartbeat(&self.state.resources.pool, agent_id)
                .await;
    }

    // ─────────────────────────────────────────────────────────
    //  MAIN ENTRY POINT
    // ─────────────────────────────────────────────────────────

    /// The core execution loop for a mission.
    ///
    /// This is the primary entry point for all agent actions. It orchestrates:
    /// 1. **Validation**: Checks task length, budget, and recursion depth.
    /// 2. **Context**: Gathers long-term memory, system architecture, and swarm findings.
    /// 3. **Intelligence**: Calls the LLM provider and dispatches tools.
    /// 4. **Persistence**: Saves results to SQL and Merkle-signed audit trails.
    /// Runs the agent mission with hierarchical OpenTelemetry (OTel) instrumentation.
    ///
    /// This method wraps major execution phases in `tracing::info_span!` blocks,
    /// which are automatically captured by the `TelemetryLayer` and broadcast
    /// to the frontend for real-time visualization in the "God View" graph.
    ///
    /// Attributes follow OTel conventions:
    /// - `agent_id`: The resource identifier
    /// - `mission_id`: The trace identifier
    /// - `status`: Span outcome (running, ok, error)
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
    pub async fn run(&self, agent_id: String, payload: TaskPayload) -> anyhow::Result<String> {
        let _span = tracing::Span::current();
        let mission_id = payload
            .cluster_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        // 1. Setup tracing, validation and security
        let setup_span =
            tracing::info_span!("Setup", agent_id = %agent_id, mission_id = %mission_id);
        let _setup_guard = setup_span.enter();
        self.state.yield_phase_transition(&agent_id, "Setup").await;
        self.setup_and_validate(&agent_id, &payload)?;
        drop(_setup_guard);

        // 2. Initialize mission and monitoring state
        let init_span =
            tracing::info_span!("Initialization", agent_id = %agent_id, mission_id = %mission_id);
        let _init_guard = init_span.enter();
        self.state
            .yield_phase_transition(&agent_id, "Initialization")
            .await;
        let _active_agent_guard = ActiveAgentGuard::acquire(&self.state.governance.active_agents);
        let mission = self.initialize_mission_state(&agent_id, &payload).await?;
        let mission_id = mission.id.clone();
        drop(_init_guard);

        // 3. Resolve context and prepare runtime environment (memory/RAG)
        let context_span = tracing::info_span!("ContextResolution", agent_id = %agent_id, mission_id = %mission_id);
        let _context_guard = context_span.enter();
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

        // Phase 3: Check for active workflow
        if let Some(workflow_name) = agent_data
            .workflows
            .first()
            .filter(|_| !agent_data.workflows.is_empty())
        {
            // We'll treat the first active workflow as the deterministic pilot
            tracing::info!(
                "📜 [Runner] Deterministic Workflow Active: {}",
                workflow_name
            );
            match crate::agent::workflows::load_workflow(
                self.state.base_dir.as_path(),
                workflow_name,
            )
            .await
            {
                Ok(mut state) => {
                    return self
                        .run_deterministic_workflow(&agent_id, payload, &mut state)
                        .await;
                }
                Err(e) => {
                    tracing::error!(
                        "❌ [Runner] Failed to load workflow {}: {}",
                        workflow_name,
                        e
                    );
                }
            }
        }

        let depth = payload.swarm_depth.unwrap_or(0);
        let lineage = payload.swarm_lineage.clone().unwrap_or_default();
        let ctx = self
            .prepare_run_context(&agent_id, &payload, &mission_id, depth, &lineage)
            .await?;
        drop(_context_guard);

        // 4. Intelligence Loop: Prompt -> Provider -> Tool Execution
        let intel_span =
            tracing::info_span!("IntelligenceLoop", agent_id = %agent_id, mission_id = %mission_id);
        let _intel_guard = intel_span.enter();
        self.state
            .yield_phase_transition(&agent_id, "IntelligenceLoop")
            .await;
        self.record_heartbeat(&ctx.agent_id).await;
        let output_res = self.execute_intelligence_loop(&ctx, &payload).await;
        drop(_intel_guard);

        match output_res {
            Ok(output) => {
                let final_span = tracing::info_span!("Finalization", agent_id = %agent_id, mission_id = %mission_id);
                let _final_guard = final_span.enter();
                self.state
                    .yield_phase_transition(&agent_id, "Finalization")
                    .await;
                // 5. Finalize results and cleanup
                self.record_heartbeat(&ctx.agent_id).await;
                self.finalize_run(&ctx, &output.text, &output.usage).await
            }
            Err((e, usage)) => {
                self.fail_mission(&ctx, &e, &usage).await?;
                Err(e)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────
//  TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::TaskPayload;

    fn make_payload(msg: &str) -> TaskPayload {
        TaskPayload {
            message: msg.to_string(),
            cluster_id: None,
            department: None,
            provider: None,
            model_id: None,
            api_key: None,
            base_url: None,
            rpm: None,
            tpm: None,
            budget_usd: None,
            swarm_depth: None,
            swarm_lineage: None,
            external_id: None,
            safe_mode: None,
            analysis: None,
            traceparent: None,
            user_id: None,
            context_files: None,
            recent_findings: None,
            structured_output: None,
        }
    }

    #[tokio::test]
    async fn test_finalize_run_fallback_on_empty_output() {
        let state = Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for runner tests"));
        let runner = AgentRunner::new(state.clone());

        let test_uuid = uuid::Uuid::new_v4().to_string();
        let agent_id = format!("agent-test-{}", test_uuid);
        let mission_id = format!("mission-test-{}", test_uuid);

        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Test Runner', 'tester', 'QA', 'desc', 'idle', '{}')").bind(&agent_id).execute(&state.resources.pool).await.unwrap();
        sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Test Mission', 'active')").bind(&mission_id).bind(&agent_id).execute(&state.resources.pool).await.unwrap();

        let ctx = RunContext {
            agent_id: agent_id.clone(),
            name: "Test Runner".to_string(),
            role: "tester".to_string(),
            department: "QA".to_string(),
            description: "desc".to_string(),
            mission_id: mission_id.clone(),
            model_config: crate::agent::types::ModelConfig {
                provider: "google".to_string(),
                model_id: "gemini-pro-latest".to_string(),
                api_key: None,
                base_url: None,
                system_prompt: None,
                temperature: None,
                max_tokens: None,
                external_id: None,
                rpm: None,
                rpd: None,
                tpm: None,
                tpd: None,
                skills: None,
                workflows: None,
                mcp_tools: None,
                connector_configs: None,
                extra_parameters: None,
            },
            provider_name: "mock".to_string(),
            skills: vec![],
            workflows: vec![],
            mcp_tools: vec![],
            depth: 0,
            lineage: vec![],
            workspace_root: std::path::PathBuf::from("."),
            base_dir: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: serde_json::json!({}),
            summarized_history: None,
            backlog: None,
            structured_output: false,
        };

        let result_empty = runner
            .finalize_run(&ctx, "   \n  \t ", &None)
            .await
            .unwrap();
        assert_eq!(
            result_empty,
            "(Agent completed its actions without a final conversational response.)"
        );

        let result_normal = runner
            .finalize_run(&ctx, "  Hello Context!  ", &None)
            .await
            .unwrap();
        assert_eq!(result_normal, "Hello Context!");
    }

    #[tokio::test]
    async fn validate_input_accepts_normal_message() {
        let state = Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for runner tests"));
        state.registry.agents.clear(); // Ensure clean state for runner tests
        let runner = AgentRunner::new(state.clone());
        let payload = make_payload("Hello, agent!");
        let result = runner.validate_input("agent-1", &payload);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_input_rejects_oversized_message() {
        let state = Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for runner tests"));
        state
            .governance
            .max_task_length
            .store(1000, std::sync::atomic::Ordering::Relaxed); // Lower limit for test
        state.registry.agents.clear();
        let runner = AgentRunner::new(state.clone());
        let long_msg = "x".repeat(2000);
        let payload = make_payload(&long_msg);
        let result = runner.validate_input("agent-1", &payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[tokio::test]
    async fn validate_input_detects_circular_recursion() {
        let state = Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for runner tests"));
        state.registry.agents.clear(); // Ensure clean state for runner tests
        let runner = AgentRunner::new(state.clone());
        let mut payload = make_payload("test");
        payload.swarm_lineage = Some(vec!["agent-1".to_string(), "agent-2".to_string()]);
        payload.swarm_depth = Some(2);

        let result = runner.validate_input("agent-1", &payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CIRCULAR"));
    }

    #[tokio::test]
    async fn validate_input_allows_non_circular_lineage() {
        let state = Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for runner tests"));
        state.registry.agents.clear(); // Ensure clean state for runner tests
        let runner = AgentRunner::new(state.clone());
        let mut payload = make_payload("test");
        payload.swarm_lineage = Some(vec!["agent-1".to_string(), "agent-2".to_string()]);
        payload.swarm_depth = Some(2);

        let result = runner.validate_input("agent-3", &payload);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_input_enforces_depth_limit() {
        let state = Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for runner tests"));
        state
            .governance
            .max_swarm_depth
            .store(3, std::sync::atomic::Ordering::Relaxed); // Lower limit for test
        state.registry.agents.clear();
        let runner = AgentRunner::new(state.clone());
        let mut payload = make_payload("test");
        payload.swarm_depth = Some(3); // Should trigger limit if depth >= max

        let result = runner.validate_input("agent-1", &payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth limit"));
    }

    #[tokio::test]
    async fn build_system_prompt_includes_role_and_department() {
        let state = Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for runner tests"));
        let runner = AgentRunner::new(state);

        let ctx = RunContext {
            agent_id: "test-1".to_string(),
            name: "Test Agent".to_string(),
            role: "Researcher".to_string(),
            department: "Intelligence".to_string(),
            description: "A test agent".to_string(),
            model_config: crate::agent::types::ModelConfig {
                provider: "mock".to_string(),
                model_id: "mock".to_string(),
                api_key: None,
                base_url: None,
                system_prompt: None,
                temperature: None,
                max_tokens: None,
                external_id: None,
                rpm: None,
                rpd: None,
                tpm: None,
                tpd: None,
                skills: None,
                workflows: None,
                mcp_tools: None,
                connector_configs: None,
                extra_parameters: None,
            },
            provider_name: "mock".to_string(),
            skills: vec![],
            workflows: vec![],
            mcp_tools: vec![],
            depth: 0,
            lineage: vec![],
            mission_id: "m1".to_string(),
            workspace_root: std::path::PathBuf::from("."),
            base_dir: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: serde_json::json!({}),
            summarized_history: None,
            backlog: None,
            structured_output: false,
        };

        let prompt = runner
            .build_system_prompt(&ctx, "OVERLORD", "Analyze the system.")
            .await;
        assert!(prompt.contains("Test Agent"));
        assert!(prompt.contains("Researcher"));
        assert!(prompt.contains("Intelligence"));
    }

    #[tokio::test]
    async fn build_system_prompt_includes_lineage_when_present() {
        let state = Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for runner tests"));
        let runner = AgentRunner::new(state);

        let ctx = RunContext {
            agent_id: "test-3".to_string(),
            name: "Specialist".to_string(),
            role: "Analyst".to_string(),
            department: "Research".to_string(),
            description: "A test agent".to_string(),
            model_config: crate::agent::types::ModelConfig {
                provider: "mock".to_string(),
                model_id: "mock".to_string(),
                api_key: None,
                base_url: None,
                system_prompt: None,
                temperature: None,
                max_tokens: None,
                external_id: None,
                rpm: None,
                rpd: None,
                tpm: None,
                tpd: None,
                skills: None,
                workflows: None,
                mcp_tools: None,
                connector_configs: None,
                extra_parameters: None,
            },
            provider_name: "mock".to_string(),
            skills: vec![],
            workflows: vec![],
            mcp_tools: vec![],
            depth: 2,
            lineage: vec!["agent-1".to_string(), "agent-2".to_string()],
            mission_id: "m2".to_string(),
            workspace_root: std::path::PathBuf::from("."),
            base_dir: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: serde_json::json!({}),
            summarized_history: None,
            backlog: None,
            structured_output: false,
        };

        let prompt_small = runner.build_system_prompt(&ctx, "root", "hi").await;
        assert!(prompt_small.contains("agent-1 -> agent-2"));
    }

    #[tokio::test]
    async fn telemetry_emits_hierarchical_spans() {
        use crate::telemetry::{TelemetryLayer, TELEMETRY_TX};
        use tracing_subscriber::prelude::*;

        let mut rx = TELEMETRY_TX.subscribe();

        // Initialize a local subscriber for this test
        let subscriber = tracing_subscriber::registry().with(TelemetryLayer::new());
        let _guard = tracing::subscriber::set_default(subscriber);

        // Simulate a span hierarchy
        {
            let _span = tracing::info_span!("test_parent", mission_id = "test_m").entered();
            {
                let _child = tracing::info_span!("test_child", agent_id = "test_a").entered();
            }
        }

        // Check for child span event in the broadcast channel
        let mut found_child = false;
        for _ in 0..50 {
            if let Ok(msg) = rx.try_recv() {
                if msg["span"]["name"].as_str() == Some("test_child") {
                    found_child = true;
                    break;
                }
            }
            tokio::task::yield_now().await;
        }
        assert!(
            found_child,
            "Should have received child span via telemetry channel"
        );
    }
}
