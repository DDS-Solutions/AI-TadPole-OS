//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / trait_tool
//!
//! ### AI Assist Note
//! Native handlers retain the authorized run context and its shared execution state.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::error::ToolExecutionError;
use crate::agent::types::TokenUsage;
use std::sync::Arc;

use serde_json::Value;

/// Data-driven tool definition for deduplication and discovery.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinitionData {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    #[serde(default)]
    pub is_mutating: bool,
    #[serde(default)]
    pub is_dangerous: bool,
    #[serde(default)]
    pub is_cacheable: bool,
}

/// Engine tool context retained for native adapters.
///
/// This includes global application state and is not a company-scoped context.
/// Plane 2 runs must be rejected before constructing this value.
pub struct ToolContext {
    pub mission_id: String,
    pub agent_id: String,
    pub workspace_root: std::path::PathBuf,
    pub fs_adapter: crate::adapter::filesystem::FilesystemAdapter,
    pub state: Arc<crate::state::AppState>,
    pub(crate) run_context: crate::agent::runner::RunContext,
}

impl ToolContext {
    pub(crate) fn from_run_context(
        ctx: &crate::agent::runner::RunContext,
        state: Arc<crate::state::AppState>,
    ) -> Self {
        Self {
            mission_id: ctx.mission_state.mission_id.clone(),
            agent_id: ctx.identity.agent_id.clone(),
            workspace_root: ctx.env.workspace_root.clone(),
            fs_adapter: ctx.env.fs_adapter.clone(),
            state,
            run_context: ctx.clone(),
        }
    }
}

#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Returns the metadata for this tool (name, description, parameters).
    fn metadata(&self) -> ToolDefinitionData;

    /// The unique identifier for the tool (e.g., "read_file").
    fn name(&self) -> &str;

    /// Checks if the tool outputs are cacheable.
    fn is_cacheable(&self) -> bool {
        false
    }

    /// Checks if the tool mutates state or files in the workspace.
    fn is_mutating(&self) -> bool {
        false
    }

    /// Checks if the tool performs dangerous operations (e.g. command execution, writes, external networks).
    fn is_dangerous(&self) -> bool {
        false
    }

    /// Executes the tool with the provided engine context and arguments.
    async fn execute(
        &self,
        ctx: &ToolContext,
        args: Value,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_native_tool_context_preserves_mission_constraints_and_shared_state() {
        use crate::agent::runner::{tools::dispatcher::CategoricalHandler, RunContext};
        use std::sync::atomic::Ordering;

        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "Alpha".into();
        ctx.identity.role = "Code Audit Specialist".into();
        ctx.mission_state.mission_id = "audit-mission".into();
        ctx.mission_state.cluster_id = Some("audit-cluster".into());
        ctx.mission_state.lineage = vec!["1".into(), "2".into()];
        ctx.mission_state.depth = 2;
        ctx.mission_state.budget_usd = 5.0;
        ctx.mission_state.sub_budget_usd = Some(1.0);
        ctx.safe_mode = true;
        ctx.execution_generation = Some(7);
        ctx.allowed_files = Some(vec!["server-rs/src/agent/runner/swarm/mod.rs".into()]);
        ctx.env.base_dir = state.base_dir.clone();

        let tool_ctx = ToolContext::from_run_context(&ctx, state.clone());
        let restored = RunContext::from_tool_ctx(&tool_ctx);
        let child = restored.derive_subtask_payload("read source".into());
        assert_eq!(child.safe_mode, Some(true));
        assert_eq!(child.swarm_depth, Some(3));
        assert_eq!(
            child.swarm_lineage,
            Some(vec!["1".into(), "2".into(), "Alpha".into()])
        );
        assert_eq!(child.cluster_id.as_deref(), Some("audit-cluster"));
        assert_eq!(child.sub_budget_usd, Some(0.5));
        assert_eq!(child.allowed_files, ctx.allowed_files);
        assert_eq!(restored.mission_state.budget_usd, 5.0);
        assert_eq!(restored.execution_generation, Some(7));
        assert_eq!(restored.env.base_dir, state.base_dir);
        assert_eq!(restored.identity.role, ctx.identity.role);
        assert!(Arc::ptr_eq(
            &restored.execution_metrics,
            &ctx.execution_metrics
        ));
        restored
            .swarm_partial_failure
            .store(true, Ordering::Release);
        assert!(ctx.swarm_partial_failure.load(Ordering::Acquire));
        restored.modified_files.lock().push("changed.rs".into());
        assert_eq!(*ctx.modified_files.lock(), vec!["changed.rs".to_string()]);

        // Exercise the real categorical adapter: a worker must not recruit its COO.
        let outcome = crate::agent::runner::tools::dispatcher::SwarmHandler
            .handle(
                "issue_alpha_directive",
                &tool_ctx,
                serde_json::json!({"directive": "delegate back to supervisor"}),
                &mut None,
            )
            .await
            .unwrap();
        assert!(outcome.contains("CIRCULAR_RECRUITMENT"), "{outcome}");
    }

    #[test]
    fn test_tool_definition_data_defaults() {
        let json_str = r#"{
            "name": "example_tool",
            "description": "An example tool",
            "parameters": {}
        }"#;

        let def: ToolDefinitionData = serde_json::from_str(json_str).unwrap();
        assert_eq!(def.name, "example_tool");
        assert!(!def.is_mutating);
        assert!(!def.is_dangerous);
        assert!(!def.is_cacheable);
    }
}
