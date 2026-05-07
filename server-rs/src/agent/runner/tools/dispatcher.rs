//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Zero-Trust Dispatcher**: Orchestrates tool registration and execution 
//! via categorical handlers. Enforces the **Tool Context Isolation** pattern.

use crate::agent::runner::RunContext;
use crate::agent::types::TokenUsage;
use crate::agent::runner::AgentRunner;
use super::error::ToolExecutionError;
use super::registry::ToolRegistry;
use super::trait_tool::{Tool, ToolContext};
use std::sync::Arc;

use super::manifest::load_core_tool_manifest;

pub struct Dispatcher {
    pub registry: ToolRegistry,
}

impl Dispatcher {
    pub fn new() -> Self {
        let mut registry = ToolRegistry::new();
        let manifest = load_core_tool_manifest();
        
        // --- Register Categorical Handlers ---
        
        // 1. Mission Tools
        let mission_handler = Arc::new(MissionHandler);
        let mission_tools = &["share_finding", "complete_mission", "pin_mission", "search_mission_knowledge", "read_codebase_file", "propose_capability", "list_file_symbols", "get_symbol_body", "send_mission_directive", "request_peer_audit", "submit_peer_review", "archive_to_global_vault", "search_global_vault", "update_working_memory", "query_financial_logs"];
        
        // 2. Filesystem Tools
        let fs_handler = Arc::new(FsHandler);
        let fs_tools = &["read_file", "write_file", "list_files", "delete_file", "grep_search", "archive_to_vault"];

        // 3. Swarm Tools
        let swarm_handler = Arc::new(SwarmHandler);
        let swarm_tools = &["spawn_subagent", "issue_alpha_directive", "recruit_specialist"];

        // 4. Metrics & External
        let aux_handler = Arc::new(AuxHandler);
        let aux_tools = &["get_agent_metrics", "notify_discord", "fetch_url", "script_builder", "search_web", "execute_shell"];

        // 5. Evolution Tools
        let evolution_handler = Arc::new(EvolutionHandler);
        let evolution_tools = &["synthesize_micro_script", "refactor_synthesized_skill"];

        for meta in manifest {
            let handler: Arc<dyn CategoricalHandler> = if mission_tools.contains(&meta.name.as_str()) {
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
                handler 
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
    fn metadata(&self) -> crate::agent::runner::tools::trait_tool::ToolDefinitionData { self.metadata.clone() }
    fn name(&self) -> &str { &self.metadata.name }
    
    async fn execute(&self, ctx: &ToolContext, args: serde_json::Value, usage: &mut Option<TokenUsage>) -> Result<String, ToolExecutionError> {
        self.handler.handle(&self.metadata.name, ctx, args, usage).await
    }
}

#[async_trait::async_trait]
pub trait CategoricalHandler: Send + Sync {
    async fn handle(&self, name: &str, ctx: &ToolContext, args: serde_json::Value, usage: &mut Option<TokenUsage>) -> Result<String, ToolExecutionError>;
}

// --- Categorical Handler Implementations ---

struct MissionHandler;
#[async_trait::async_trait]
impl CategoricalHandler for MissionHandler {
    async fn handle(&self, name: &str, ctx: &ToolContext, args: serde_json::Value, usage: &mut Option<TokenUsage>) -> Result<String, ToolExecutionError> {
        // Here we'd bridge to the AgentRunner methods.
        // For the Zero-Trust refactor, we pass the ToolContext which has everything needed.
        let runner = AgentRunner::new(ctx.state.clone()); // Lightweight wrapper or use a different bridge
        let run_ctx = RunContext::from_tool_ctx(ctx); // Helper needed
        let fc = crate::agent::types::ToolCall { name: name.to_string(), args };
        
        match name {
            "share_finding" => runner.handle_share_finding(&run_ctx, &fc).await,
            "complete_mission" => runner.handle_complete_mission(&run_ctx, &fc).await,
            "pin_mission" => runner.handle_pin_mission(&run_ctx, &fc, usage).await,
            "search_mission_knowledge" => runner.handle_search_mission_knowledge(&run_ctx, &fc).await,
            "read_codebase_file" => runner.handle_read_codebase_file(&run_ctx, &fc).await,
            "propose_capability" => runner.handle_propose_capability(&run_ctx, &fc).await,
            "list_file_symbols" => runner.handle_list_file_symbols(&run_ctx, &fc).await,
            "get_symbol_body" => runner.handle_get_symbol_body(&run_ctx, &fc).await,
            "send_mission_directive" => runner.handle_send_mission_directive(&run_ctx, &fc).await,
            "request_peer_audit" => runner.handle_request_peer_audit(&run_ctx, &fc).await,
            "submit_peer_review" => runner.handle_submit_peer_review(&run_ctx, &fc).await,
            "archive_to_global_vault" => runner.handle_archive_to_global_vault(&run_ctx, &fc).await,
            "search_global_vault" => runner.handle_search_global_vault(&run_ctx, &fc).await,
            "update_working_memory" => {
                let mut output = String::new();
                runner.handle_update_working_memory(&run_ctx, &fc, &mut output).await?;
                Ok(output)
            }
            "query_financial_logs" => runner.handle_query_financial_logs(&run_ctx, &fc, usage).await,
            _ => Err(ToolExecutionError::ExecutionFailed(format!("MissionHandler cannot handle '{}'", name)))
        }
    }
}

struct FsHandler;
#[async_trait::async_trait]
impl CategoricalHandler for FsHandler {
    async fn handle(&self, name: &str, ctx: &ToolContext, args: serde_json::Value, usage: &mut Option<TokenUsage>) -> Result<String, ToolExecutionError> {
        let runner = AgentRunner::new(ctx.state.clone());
        let run_ctx = RunContext::from_tool_ctx(ctx);
        let fc = crate::agent::types::ToolCall { name: name.to_string(), args };

        match name {
            "read_file" | "get_file_contents" => runner.handle_read_file(&run_ctx, &fc, usage).await,
            "write_file" => runner.handle_write_file(&run_ctx, &fc).await,
            "list_files" => runner.handle_list_files(&run_ctx, &fc, usage).await,
            "delete_file" => runner.handle_delete_file(&run_ctx, &fc).await,
            "grep_search" => runner.handle_grep_search(&run_ctx, &fc, usage).await,
            "archive_to_vault" => runner.handle_archive_to_vault(&run_ctx, &fc).await,
            _ => Err(ToolExecutionError::ExecutionFailed(format!("FsHandler cannot handle '{}'", name)))
        }
    }
}

struct SwarmHandler;
#[async_trait::async_trait]
impl CategoricalHandler for SwarmHandler {
    async fn handle(&self, name: &str, ctx: &ToolContext, args: serde_json::Value, usage: &mut Option<TokenUsage>) -> Result<String, ToolExecutionError> {
        let runner = AgentRunner::new(ctx.state.clone());
        let run_ctx = RunContext::from_tool_ctx(ctx);
        let fc = crate::agent::types::ToolCall { name: name.to_string(), args };

        match name {
            "spawn_subagent" => runner.handle_spawn_subagent(&run_ctx, &fc, usage).await,
            "issue_alpha_directive" => runner.handle_alpha_directive(&run_ctx, &fc).await.map_err(ToolExecutionError::from),
            "recruit_specialist" => {
                // In the current architecture, recruit_specialist is handled by the MCP/SystemDelegate logic in mod.rs,
                // but we bridge it here for completeness if invoked directly.
                runner.handle_spawn_subagent(&run_ctx, &fc, usage).await
            }
            _ => Err(ToolExecutionError::ExecutionFailed(format!("SwarmHandler cannot handle '{}'", name)))
        }
    }
}

struct AuxHandler;
#[async_trait::async_trait]
impl CategoricalHandler for AuxHandler {
    async fn handle(&self, name: &str, ctx: &ToolContext, args: serde_json::Value, usage: &mut Option<TokenUsage>) -> Result<String, ToolExecutionError> {
        let runner = AgentRunner::new(ctx.state.clone());
        let run_ctx = RunContext::from_tool_ctx(ctx);
        let fc = crate::agent::types::ToolCall { name: name.to_string(), args };

        match name {
            "get_agent_metrics" => runner.handle_get_agent_metrics(&run_ctx, &fc, usage).await,
            "notify_discord" => runner.handle_notify_discord(&run_ctx, &fc).await,
            "fetch_url" => runner.handle_fetch_url(&run_ctx, &fc, usage).await,
            "script_builder" => {
                let mut output = String::new();
                runner.handle_script_builder(&run_ctx, &fc, &mut output, usage, "").await?;
                Ok(output)
            }
            "search_web" => runner.handle_search_web(&run_ctx, &fc, usage).await,
            "execute_shell" => {
                let mut output = String::new();
                runner.handle_execute_shell(&run_ctx, &fc, &mut output).await?;
                Ok(output)
            }
            _ => Err(ToolExecutionError::ExecutionFailed(format!("AuxHandler cannot handle '{}'", name)))
        }
    }
}

struct EvolutionHandler;
#[async_trait::async_trait]
impl CategoricalHandler for EvolutionHandler {
    async fn handle(&self, name: &str, ctx: &ToolContext, args: serde_json::Value, _usage: &mut Option<TokenUsage>) -> Result<String, ToolExecutionError> {
        let runner = AgentRunner::new(ctx.state.clone());
        let run_ctx = RunContext::from_tool_ctx(ctx);
        let fc = crate::agent::types::ToolCall { name: name.to_string(), args };

        match name {
            "synthesize_micro_script" => runner.handle_synthesize_micro_script(&run_ctx, &fc).await,
            "refactor_synthesized_skill" => runner.handle_refactor_synthesized_skill(&run_ctx, &fc).await,
            _ => Err(ToolExecutionError::ExecutionFailed(format!("EvolutionHandler cannot handle '{}'", name)))
        }
    }
}

// Metadata: [dispatcher]
