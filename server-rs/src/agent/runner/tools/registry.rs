//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Tool Registry**: Centralizes the management of all available agent 
//! capabilities. Uses the **Zero-Trust Tool Trait** to ensure that every 
//! tool is isolated and audited.

use super::trait_tool::Tool;
use std::collections::HashMap;
use std::sync::Arc;

pub const BUILTIN_TOOLS: &[&str] = &[
    "spawn_subagent", "issue_alpha_directive", "send_mission_directive",
    "request_peer_audit", "submit_peer_review", "archive_to_global_vault",
    "search_global_vault", "synthesize_micro_script", "refactor_synthesized_skill",
    "share_finding", "complete_mission", "propose_capability", "archive_to_vault", 
    "notify_discord", "fetch_url", "read_file", "write_file", "list_files", 
    "delete_file", "search_mission_knowledge", "read_codebase_file", 
    "list_file_symbols", "get_symbol_body", "update_working_memory",
    "recruit_specialist", "get_agent_metrics", "script_builder", "pin_mission"
];

pub struct ToolRegistry {
    pub handlers: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.handlers.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.handlers.get(name).cloned()
    }

    /// Returns a list of all tool metadata currently registered.
    pub fn list_tools(&self) -> Vec<super::trait_tool::ToolDefinitionData> {
        self.handlers.values().map(|t| t.metadata()).collect()
    }
}

// Metadata: [registry]
