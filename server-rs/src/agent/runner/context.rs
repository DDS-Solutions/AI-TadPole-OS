//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / context
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[OpenToolCallCloser]`
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

use crate::agent::types::{EngineAgent, ModelConfig};
use crate::error::AppError;

#[cfg(test)]
use super::AgentRunner;
use super::{AgentIdentity, Environment, MissionState, RunContext};
#[cfg(test)]
use crate::agent::types::TaskPayload;

mod memory;
mod preparation;
mod resolution;

/// Conversion factor between database micro-dollars and USD.
pub const MICRO_DOLLARS_PER_USD: f64 = 1_000_000.0;
/// Log entry retrieval limit when preparing mission history.
pub const MISSION_LOG_RETRIEVAL_LIMIT: i64 = 250;
/// Token count threshold above which mission history summarization is triggered.
pub const TOKEN_SUMMARIZATION_THRESHOLD: usize = 4_000;
/// Cosine similarity threshold for RAG semantic deduplication.
pub const SEMANTIC_DEDUP_SIMILARITY: f32 = 0.95;
/// Maximum entries retained in the active mission scope vector window.
pub const MAX_SCOPE_ENTRIES: usize = 25;

/// Validates and resolves a workspace root directory, strictly preventing path traversal.
pub fn validate_and_resolve_workspace_dir(
    base_dir: &std::path::Path,
    cluster_id: Option<&str>,
    default_name: &str,
) -> Result<std::path::PathBuf, AppError> {
    let workspace_id = cluster_id.unwrap_or(default_name);
    let trimmed = workspace_id.trim();

    // Strict validation: Reject rather than strip to prevent collision squatting and traversal
    if trimmed.is_empty()
        || trimmed.len() > 64
        || !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        || trimmed.starts_with('.')
        || trimmed.starts_with('-')
        || trimmed.starts_with('_')
        || trimmed.contains("..")
    {
        return Err(AppError::BadRequest(format!(
            "Invalid workspace ID '{}'. Must contain only alphanumeric, dash, or underscore characters (max 64 chars) and start with an alphanumeric character.",
            workspace_id
        )));
    }

    let mut workspace_root = base_dir.join("data/workspaces");
    workspace_root.push(trimmed);

    if !workspace_root.exists() {
        std::fs::create_dir_all(&workspace_root).map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to create workspace directory: {}. Error: {}",
                workspace_root.display(),
                e
            ))
        })?;
    }

    Ok(workspace_root)
}

impl RunContext {
    /// Factory for initializing the canonical base state of RunContext, ensuring
    /// identical defaults and avoiding duplication across runner and provider resolvers.
    pub fn new_base(
        identity: AgentIdentity,
        mission_state: MissionState,
        env: Environment,
        model_config: ModelConfig,
        agent_models: crate::agent::types::AgentModels,
        skills: Vec<String>,
        workflows: Vec<String>,
        mcp_tools: Vec<String>,
        working_memory: serde_json::Value,
    ) -> Self {
        let reasoning_depth = model_config.reasoning_depth.unwrap_or(1);
        let act_threshold = model_config.act_threshold.unwrap_or(0.9);
        let max_turns = model_config.max_turns.unwrap_or(20);
        let provider_name = model_config.provider.to_string().to_lowercase();

        RunContext {
            identity,
            mission_state,
            env,
            model_config,
            skills,
            workflows,
            agent_models,
            mcp_tools,
            provider_name,
            safe_mode: false,
            analysis: false,
            traceparent: None,
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            modified_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            commands_run: std::sync::Arc::new(parking_lot::Mutex::new(
                std::collections::HashSet::new(),
            )),
            execution_records: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            last_mutation_timestamp_ms: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            current_dir: std::sync::Arc::new(parking_lot::Mutex::new(None)),
            allowed_files: None,
            recent_findings: None,
            working_memory,
            summarized_history: None,
            structured_output: false,
            backlog: None,
            visible_transcript: None,
            conductor_plan: None,
            planning_usage: None,
            reasoning_depth,
            act_threshold,
            max_turns,
            resource_weights: {
                #[cfg(feature = "vector-memory")]
                {
                    crate::utils::data_weighting::DataWeighting::default_weights()
                }
                #[cfg(not(feature = "vector-memory"))]
                {
                    std::collections::HashMap::new()
                }
            },
            graph_context: None,
            swarm_partial_failure: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            swarm_review_approved: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            resolved_model_id: std::sync::Arc::new(parking_lot::Mutex::new(None)),
            dispatch_accounted: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            execution_generation: None,
            verification_passed: false,
            execution_metrics: std::sync::Arc::new(
                crate::agent::runner::execution_metrics::ExecutionMetrics::new(),
            ),
        }
    }

    /// Cleans and filters raw conversation history (e.g., removing thought blocks
    /// and truncating/summarizing large tool outputs) to produce a sandboxed visible transcript.
    /// (PD-002: Delegated to `turn_compactor` module).
    #[allow(dead_code)]
    pub fn build_sandboxed_transcript(&self, raw_history: &[String]) -> Vec<String> {
        super::turn_compactor::build_sandboxed_transcript(&self.identity.role, raw_history)
    }
}

/// Closes any unresolved tool calls in conversation history.
///
/// If an ASSISTANT message invoked a tool call but execution was interrupted before
/// an OBSERVATION could be recorded, provider APIs (OpenAI, Anthropic) will reject
/// subsequent calls with HTTP 400 Bad Request.
///
/// This processor scans the history and injects a synthetic error observation to resolve
/// the dangling invocation, allowing the conversation turn to proceed seamlessly.
pub fn close_unresolved_tool_calls(history: &mut Vec<String>) -> usize {
    if history.is_empty() {
        return 0;
    }

    // Find the last ASSISTANT message
    let last_assistant_idx = history
        .iter()
        .rposition(|msg| msg.starts_with("ASSISTANT:"));
    let Some(idx) = last_assistant_idx else {
        return 0;
    };

    let assistant_msg = &history[idx];

    // Check if this assistant message invoked a tool
    let has_tool_invocation = assistant_msg.contains("\"call\":")
        || assistant_msg.contains("\"name\":")
        || assistant_msg.contains("<tool_call>")
        || assistant_msg.contains("Tool Call:");

    if !has_tool_invocation {
        return 0;
    }

    // Check if there is an OBSERVATION after this assistant message
    let has_subsequent_observation = history[idx + 1..]
        .iter()
        .any(|msg| msg.starts_with("OBSERVATION:") || msg.starts_with("--- [TOOL OBSERVATION:"));

    if !has_subsequent_observation {
        tracing::warn!(
            "🛠️ [OpenToolCallCloser] Detected unclosed tool call in turn at index {}. Injecting synthetic recovery observation.",
            idx
        );
        history.push(
            "OBSERVATION: {\"error\": \"Tool call was not executed due to turn interruption. Please retry this tool call.\"}".to_string()
        );
        1
    } else {
        0
    }
}

impl EngineAgent {
    #[allow(dead_code)]
    pub fn resolve_provider_context(
        &self,
        base_dir: std::path::PathBuf,
        cluster_id: Option<&str>,
    ) -> Result<RunContext, AppError> {
        let workspace_root = validate_and_resolve_workspace_dir(&base_dir, cluster_id, "default")?;
        let fs_adapter = crate::adapter::filesystem::FilesystemAdapter::new(workspace_root.clone());
        let authority_level =
            crate::agent::types::RoleAuthorityLevel::from_role(&self.identity.role);

        let identity = AgentIdentity {
            agent_id: self.identity.id.clone(),
            name: self.identity.name.clone(),
            role: self.identity.role.clone(),
            department: self.identity.department.clone(),
            description: self.identity.description.clone(),
            authority_level,
        };

        let mission_state = MissionState {
            mission_id: "system-internal".to_string(),
            cluster_id: cluster_id.map(|s| s.to_string()),
            user_id: None,
            depth: 0,
            lineage: vec![],
            primary_goal: None,
            budget_usd: self.economics.budget_usd,
            current_cost_usd: self.economics.cost_usd,
            sub_budget_usd: None,
        };

        let env = Environment {
            workspace_root,
            fs_adapter,
            base_dir,
        };

        let ctx = RunContext::new_base(
            identity,
            mission_state,
            env,
            self.models.model.clone(),
            self.models.clone(),
            self.capabilities.skills.clone(),
            self.capabilities.workflows.clone(),
            self.capabilities.mcp_tools.clone(),
            self.state.working_memory.clone(),
        );

        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_mock_runner() -> AgentRunner {
        let state = Arc::new(AppState::new_minimal_mock().await);
        AgentRunner::new(state)
    }

    #[tokio::test]
    async fn test_workspace_path_sanitization() {
        let runner = setup_mock_runner().await;

        // 1. Valid ID
        let path = runner.resolve_workspace_paths(Some("cluster-123")).unwrap();
        assert!(path.to_string_lossy().contains("cluster-123"));

        // 2. Path Traversal & Invalid Characters Rejection
        assert!(runner
            .resolve_workspace_paths(Some("../../../etc/passwd"))
            .is_err());
        assert!(runner.resolve_workspace_paths(Some("cluster.123")).is_err());
        assert!(runner
            .resolve_workspace_paths(Some("-leading-dash"))
            .is_err());
        assert!(runner.resolve_workspace_paths(Some("")).is_err());

        // 3. Provider Context Traversal Rejection
        let agent = EngineAgent::default();
        let traversal_result = agent
            .resolve_provider_context(runner.state.base_dir.clone(), Some("../../../evil_cluster"));
        assert!(traversal_result.is_err());

        let valid_provider_result =
            agent.resolve_provider_context(runner.state.base_dir.clone(), Some("valid-cluster-99"));
        assert!(valid_provider_result.is_ok());
    }

    #[tokio::test]
    async fn test_security_gates_safe_mode() {
        let runner = setup_mock_runner().await;

        // Register custom_skill in the mock registry so it passes safety checks
        let skill_def = crate::agent::script_skills::SkillDefinition {
            id: None,
            name: "custom_skill".to_string(),
            description: "Test skill".to_string(),
            execution_command: "echo test".to_string(),
            schema: serde_json::json!({}),
            oversight_required: false,
            doc_url: None,
            tags: None,
            full_instructions: None,
            negative_constraints: None,
            verification_script: None,
            category: "user".to_string(),
            security_score: None,
            security_severity: None,
            security_report: None,
        };
        runner
            .state
            .registry
            .skills
            .skills
            .insert("custom_skill".to_string(), skill_def);

        let mut skills = vec![
            "read_file".to_string(),
            "write_file".to_string(),
            "custom_skill".to_string(),
        ];
        let mut workflows = vec!["deploy_flow".to_string()];
        let mut mcp_tools = vec!["fetch_remote".to_string()];

        // 1. Safe Mode OFF
        runner.apply_security_gates(false, &mut skills, &mut workflows, &mut mcp_tools);
        assert_eq!(skills.len(), 3);
        assert_eq!(workflows.len(), 1);
        assert_eq!(mcp_tools.len(), 1);

        // 2. Safe Mode ON
        runner.apply_security_gates(true, &mut skills, &mut workflows, &mut mcp_tools);
        assert!(skills.contains(&"read_file".to_string()));
        assert!(!skills.contains(&"write_file".to_string()));
        assert!(skills.contains(&"custom_skill".to_string()));
        assert_eq!(workflows.len(), 0); // Workflows cleared in safe mode
        assert_eq!(mcp_tools.len(), 0); // MCP tools cleared in safe mode
    }

    #[tokio::test]
    async fn test_payload_overrides() {
        let runner = setup_mock_runner().await;
        let mut base_config = ModelConfig {
            model_id: "base-model".to_string(),
            api_key: Some("base-key".to_string()),
            external_id: Some("base-external-id".to_string()),
            ..Default::default()
        };
        let payload = TaskPayload {
            model_id: Some("override-model".to_string()),
            api_key: Some("override-key".to_string()),
            base_url: Some("https://evil.proxy".to_string()),
            external_id: Some("evil-tenant-id".to_string()),
            ..TaskPayload::default()
        };

        // Root execution allows override
        runner.merge_payload_overrides(&mut base_config, &payload, false);
        assert_eq!(base_config.model_id, "override-model");
        assert_eq!(base_config.api_key, Some("override-key".to_string()));
        assert_eq!(base_config.base_url, Some("https://evil.proxy".to_string()));
        assert_eq!(base_config.external_id, Some("evil-tenant-id".to_string()));

        // Swarm children retain their own model and provider connection settings.
        let mut child_config = ModelConfig {
            model_id: "base-model".to_string(),
            api_key: Some("base-key".to_string()),
            base_url: Some("https://official.api".to_string()),
            external_id: Some("legit-tenant-id".to_string()),
            ..Default::default()
        };
        runner.merge_payload_overrides(&mut child_config, &payload, true);
        assert_eq!(child_config.model_id, "base-model");
        assert_eq!(child_config.api_key, Some("base-key".to_string()));
        assert_eq!(
            child_config.base_url,
            Some("https://official.api".to_string())
        );
        assert_eq!(
            child_config.external_id,
            Some("legit-tenant-id".to_string())
        );
    }

    #[test]
    fn cloned_context_retains_grouped_scope_and_shared_execution_evidence() {
        let mut context = RunContext::default();
        context.identity.agent_id = "context-owner".into();
        context.mission_state.lineage = vec!["parent".into()];
        context.mission_state.sub_budget_usd = Some(0.25);
        context.env.workspace_root = std::path::PathBuf::from("scoped-workspace");
        let cloned = context.clone();
        assert_eq!(cloned.identity.agent_id, "context-owner");
        assert_eq!(cloned.mission_state.lineage, vec!["parent"]);
        assert_eq!(cloned.mission_state.sub_budget_usd, Some(0.25));
        assert_eq!(cloned.env.workspace_root, context.env.workspace_root);
        cloned.modified_files.lock().push("evidence.rs".into());
        assert_eq!(context.modified_files.lock().len(), 1);
        assert!(std::sync::Arc::ptr_eq(
            &context.execution_metrics,
            &cloned.execution_metrics
        ));
        assert!(std::sync::Arc::ptr_eq(
            &context.dispatch_accounted,
            &cloned.dispatch_accounted
        ));
    }

    #[test]
    fn test_close_unresolved_tool_calls() {
        // Case 1: Unclosed tool call gets synthetic observation
        let mut history = vec![
            "USER: Please read a file".to_string(),
            "ASSISTANT: I will read the file. {\"call\": \"read_file\", \"path\": \"Cargo.toml\"}"
                .to_string(),
        ];
        let closed = close_unresolved_tool_calls(&mut history);
        assert_eq!(closed, 1);
        assert_eq!(history.len(), 3);
        assert!(history[2].starts_with("OBSERVATION:"));
        assert!(history[2].contains("Tool call was not executed"));

        // Case 2: Already closed tool call does not get another observation
        let closed_again = close_unresolved_tool_calls(&mut history);
        assert_eq!(closed_again, 0);
        assert_eq!(history.len(), 3);

        // Case 3: Conversational assistant message without tool invocation is untouched
        let mut convo_history = vec![
            "USER: Hello".to_string(),
            "ASSISTANT: Hi! How can I help you today?".to_string(),
        ];
        let closed_convo = close_unresolved_tool_calls(&mut convo_history);
        assert_eq!(closed_convo, 0);
        assert_eq!(convo_history.len(), 2);

        // Case 4: Conversational message mentioning "OBSERVATION" is not mistaken for a tool call
        let mut convo_with_obs = vec![
            "USER: What did you notice?".to_string(),
            "ASSISTANT: Based on the OBSERVATION in our previous turn, everything is operational."
                .to_string(),
        ];
        let closed_obs = close_unresolved_tool_calls(&mut convo_with_obs);
        assert_eq!(closed_obs, 0);
        assert_eq!(convo_with_obs.len(), 2);
    }
}
