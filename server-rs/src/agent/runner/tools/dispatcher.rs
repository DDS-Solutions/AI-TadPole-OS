//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **Zero-Trust Dispatcher**: Orchestrates tool registration and execution
//! via categorical handlers. Enforces the **Tool Context Isolation** pattern.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing tool manifest entry or handler registration mismatch.
//! - **Telemetry Link**: Search `[dispatcher]` in tracing logs.
//! - **Trace Scope**: `server-rs::agent::runner::tools::dispatcher`

use super::error::ToolExecutionError;
use super::registry::ToolRegistry;
use super::trait_tool::{Tool, ToolContext};
//use crate::agent::runner::AgentRunner;
//use crate::agent::runner::RunContext;
use crate::agent::types::TokenUsage;
use std::sync::Arc;

use super::manifest::load_core_tool_manifest;

pub struct Dispatcher {
    pub registry: ToolRegistry,
}

impl Dispatcher {
    pub fn new() -> Self {
        let mut registry = ToolRegistry::new();
        let manifest = load_core_tool_manifest();
        tracing::info!("🔧 [dispatcher] Initializing Tool Dispatcher with {} tools", manifest.len());

        // --- Register Categorical Handlers ---

        // 1. Mission Tools
        let mission_handler = Arc::new(MissionHandler);
        let mission_tools = &[
            "share_finding",
            "complete_mission",
            "pin_mission",
            "search_mission_knowledge",
            "read_codebase_file",
            "propose_capability",
            "list_file_symbols",
            "get_symbol_body",
            "send_mission_directive",
            "request_peer_audit",
            "submit_peer_review",
            "archive_to_global_vault",
            "search_global_vault",
            "update_working_memory",
            "query_financial_logs",
        ];

        // 2. Filesystem Tools
        let fs_handler = Arc::new(FsHandler);
        let fs_tools = &[
            "read_file",
            "write_file",
            "list_files",
            "delete_file",
            "grep_search",
            "archive_to_vault",
        ];

        // 3. Swarm Tools
        let swarm_handler = Arc::new(SwarmHandler);
        let swarm_tools = &[
            "spawn_subagent",
            "issue_alpha_directive",
            "recruit_specialist",
        ];

        // 4. Metrics & External
        let aux_handler = Arc::new(AuxHandler);
        let aux_tools = &[
            "get_agent_metrics",
            "notify_discord",
            "fetch_url",
            "script_builder",
            "search_web",
            "execute_shell",
        ];

        // 5. Evolution Tools
        let evolution_handler = Arc::new(EvolutionHandler);
        let evolution_tools = &["synthesize_micro_script", "refactor_synthesized_skill"];

        for meta in manifest {
            let handler: Arc<dyn CategoricalHandler> =
                if mission_tools.contains(&meta.name.as_str()) {
                    mission_handler.clone()
                } else if fs_tools.contains(&meta.name.as_str()) {
                    fs_handler.clone()
                } else if swarm_tools.contains(&meta.name.as_str()) {
                    swarm_handler.clone()
                } else if aux_tools.contains(&meta.name.as_str()) {
                    aux_handler.clone()
                } else if evolution_tools.contains(&meta.name.as_str()) {
                    evolution_handler.clone()
                } else {
                    continue; // Skip unknown tools in manifest
                };

            registry.register(Arc::new(Wrapper {
                metadata: meta,
                handler,
            }));
        }

        Self { registry }
    }
}

/// A wrapper to map specific tool names to categorical handlers.
struct Wrapper {
    metadata: crate::agent::runner::tools::trait_tool::ToolDefinitionData,
    handler: Arc<dyn CategoricalHandler>,
}

#[async_trait::async_trait]
impl Tool for Wrapper {
    fn metadata(&self) -> crate::agent::runner::tools::trait_tool::ToolDefinitionData {
        self.metadata.clone()
    }
    fn name(&self) -> &str {
        &self.metadata.name
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        args: serde_json::Value,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        self.handler
            .handle(&self.metadata.name, ctx, args, usage)
            .await
    }
}

#[async_trait::async_trait]
pub trait CategoricalHandler: Send + Sync {
    async fn handle(
        &self,
        name: &str,
        ctx: &ToolContext,
        args: serde_json::Value,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError>;
}

// --- Categorical Handler Implementations ---

crate::define_categorical_handler!(MissionHandler, {
    "share_finding" => handle_share_finding (),
    "complete_mission" => handle_complete_mission (),
    "pin_mission" => handle_pin_mission (usage),
    "search_mission_knowledge" => handle_search_mission_knowledge (),
    "read_codebase_file" => handle_read_codebase_file (),
    "propose_capability" => handle_propose_capability (),
    "list_file_symbols" => handle_list_file_symbols (),
    "get_symbol_body" => handle_get_symbol_body (),
    "send_mission_directive" => handle_send_mission_directive (),
    "request_peer_audit" => handle_request_peer_audit (),
    "submit_peer_review" => handle_submit_peer_review (),
    "archive_to_global_vault" => handle_archive_to_global_vault (),
    "search_global_vault" => handle_search_global_vault (),
    "update_working_memory" => handle_update_working_memory (output),
    "query_financial_logs" => handle_query_financial_logs (usage),
});

crate::define_categorical_handler!(FsHandler, {
    "read_file" => handle_read_file (usage),
    "get_file_contents" => handle_read_file (usage),
    "write_file" => handle_write_file (),
    "list_files" => handle_list_files (usage),
    "delete_file" => handle_delete_file (),
    "grep_search" => handle_grep_search (usage),
    "archive_to_vault" => handle_archive_to_vault (),
});

crate::define_categorical_handler!(SwarmHandler, {
    "spawn_subagent" => handle_spawn_subagent (usage),
    "issue_alpha_directive" => handle_alpha_directive (),
    "recruit_specialist" => handle_spawn_subagent (usage),
});

crate::define_categorical_handler!(AuxHandler, {
    "get_agent_metrics" => handle_get_agent_metrics (usage),
    "notify_discord" => handle_notify_discord (),
    "fetch_url" => handle_fetch_url (usage),
    "script_builder" => handle_script_builder (output, usage, ""),
    "search_web" => handle_search_web (usage),
    "execute_shell" => handle_execute_shell (output),
});

crate::define_categorical_handler!(EvolutionHandler, {
    "synthesize_micro_script" => handle_synthesize_micro_script (),
    "refactor_synthesized_skill" => handle_refactor_synthesized_skill (),
});

// Metadata: [dispatcher]

// Metadata: [dispatcher]
