//! Model Context Protocol (MCP) Host - Sovereign Tool Execution
//!
//! Implements the MCP 1.0 specification for tool discovery and execution. 
//! Acts as the primary security gateway for all agent-led environment interactions.
//!
//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Sovereign Permission Check**: Every `call_tool` request passes 
//! through the `PermissionPolicy` gateway. If `Prompt` is active, 
//! execution halts until a user provides a UI-level signal.

pub mod registry;

#[allow(unused_imports)]
use self::registry::{McpRegistry, ToolHandler};
use crate::agent::script_skills::SkillDefinition;
use crate::security::permissions::{PermissionMode, PermissionPolicy, PermissionPrompter};
use crate::utils::parser::SymbolExtractor;
use serde::{Deserialize, Serialize};
use server_rs_macros::agent_tool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// Operational statistics for a specific tool.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpToolStats {
    pub invocations: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_latency_ms: u64,
}

/// A structured tool definition registered within the MCP ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolHub {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub source: String,
    pub stats: McpToolStats,
    pub category: String,
}

impl From<SkillDefinition> for McpToolHub {
    fn from(skill: SkillDefinition) -> Self {
        Self {
            name: skill.name,
            description: skill.description,
            input_schema: skill.schema,
            source: "legacy".to_string(),
            stats: McpToolStats::default(),
            category: skill.category,
        }
    }
}

/// The primary host for managing tool registration and execution.
pub struct McpHost {
    pub registry: Arc<Mutex<McpRegistry>>,
    pub stats: dashmap::DashMap<String, McpToolStats>,
    event_tx: broadcast::Sender<serde_json::Value>,
    mcp_config_path: Option<PathBuf>,
    pub policy: PermissionPolicy,
    pub prompter: Option<Arc<dyn PermissionPrompter>>,
}

impl McpHost {
    pub fn new(
        event_tx: broadcast::Sender<serde_json::Value>,
        mcp_config_path: Option<PathBuf>,
    ) -> Self {
        let mut registry = McpRegistry::new();

        // Register native Hydra-RS tools
        registry.register(Arc::new(RecruitSpecialistHandler));
        registry.register(Arc::new(ListFileSymbolsHandler));
        registry.register(Arc::new(GetSymbolBodyHandler));

        Self {
            registry: Arc::new(Mutex::new(registry)),
            stats: dashmap::DashMap::new(),
            event_tx,
            mcp_config_path,
            policy: PermissionPolicy::default(),
            prompter: None,
        }
    }

    /// Set a custom permission prompter (e.g. from the Tauri frontend).
    #[allow(dead_code)]
    pub fn set_prompter(&mut self, prompter: Arc<dyn PermissionPrompter>) {
        self.prompter = Some(prompter);
    }

    /// Lists only the tools allowed for a specific agent based on its skill IDs.
    pub async fn list_tools(
        &self,
        agent_skills: &[String],
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> Vec<McpToolHub> {
        let mut tools: Vec<McpToolHub> = agent_skills
            .iter()
            .filter_map(|skill_name| all_skills.get(skill_name))
            .map(|skill| {
                let mut hub = McpToolHub::from(skill.clone());
                if let Some(s) = self.stats.get(&hub.name) {
                    hub.stats = s.clone();
                }
                hub
            })
            .collect();

        // Include registry-managed tools
        {
            let registry = self.registry.lock().await;
            for mut t in registry.list_all() {
                if let Some(s) = self.stats.get(&t.name) {
                    t.stats = s.clone();
                }
                tools.push(t);
            }
        }

        // Include external MCP tools from config (Legacy path)
        if let Some(ref path) = self.mcp_config_path {
            let _path_str = path.to_string_lossy();
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str::<McpConfig>(&content) {
                    for (server_name, _) in config.mcp_servers {
                        tools.push(McpToolHub {
                            name: format!("{}:server", server_name),
                            description: format!("External MCP Server: {}", server_name),
                            input_schema: serde_json::json!({}),
                            source: "mcp".to_string(),
                            stats: McpToolStats::default(),
                            category: "agent".to_string(),
                        });
                    }
                }
            }
        }

        tools
    }

    /// Dispatches tool calls to specific MCP servers or internal modular handlers.
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> anyhow::Result<McpResult> {
        let start_time = std::time::Instant::now();

        // --- Sovereign Permission Check ---
        let mode = self.policy.get_mode(tool_name);
        match mode {
            PermissionMode::Deny => return Err(anyhow::anyhow!("Permission denied")),
            PermissionMode::Prompt => {
                if let Some(ref prompter) = self.prompter {
                    let decision = prompter.prompt_user(tool_name, &arguments.to_string())?;
                    if decision != PermissionMode::Allow {
                        return Err(anyhow::anyhow!("User rejected tool execution"));
                    }
                }
            }
            PermissionMode::Allow => {}
        }

        let result = self
            .execute_tool_internal(tool_name, arguments, workspace_root, all_skills)
            .await;

        let latency = start_time.elapsed().as_millis() as u64;
        self.update_stats(tool_name, result.is_ok(), latency);
        self.emit_pulse(tool_name, result.is_ok(), latency);

        result
    }

    async fn execute_tool_internal(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> anyhow::Result<McpResult> {
        // 1. Check Modular Registry (Hydra-RS & System Tools)
        let handler = {
            let registry = self.registry.lock().await;
            registry.get(tool_name)
        };

        if let Some(h) = handler {
            return h.execute(arguments, workspace_root).await;
        }

        // 2. Legacy/Dynamic Skills fallback
        if let Some(skill) = all_skills.get(tool_name) {
            let output = self
                .execute_legacy_skill(skill.value(), arguments, workspace_root)
                .await?;
            return Ok(McpResult::Raw(output));
        }

        Err(anyhow::anyhow!("Tool '{}' not found", tool_name))
    }

    fn update_stats(&self, tool_name: &str, is_success: bool, latency: u64) {
        let mut entry = self.stats.entry(tool_name.to_string()).or_default();
        entry.invocations += 1;
        if is_success {
            entry.success_count += 1;
        } else {
            entry.failure_count += 1;
        }
        if entry.avg_latency_ms == 0 {
            entry.avg_latency_ms = latency;
        } else {
            entry.avg_latency_ms = (entry.avg_latency_ms + latency) / 2;
        }
    }

    fn emit_pulse(&self, tool_name: &str, is_success: bool, latency: u64) {
        let pulse = serde_json::json!({
            "type": "engine:mcp_pulse",
            "tool": tool_name,
            "status": if is_success { "success" } else { "error" },
            "latency": latency
        });
        let _ = self.event_tx.send(pulse);
    }

    async fn execute_legacy_skill(
        &self,
        skill: &SkillDefinition,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
    ) -> anyhow::Result<String> {
        let args_json = serde_json::to_string(&arguments).unwrap_or_default();
        let mut parts = skill.execution_command.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty command"))?;
        let mut cmd = tokio::process::Command::new(program);
        for arg in parts {
            cmd.arg(arg);
        }
        cmd.env("TADPOLE_SKILL_ARGS", &args_json);
        cmd.current_dir(workspace_root);
        let output =
            tokio::time::timeout(std::time::Duration::from_secs(60), cmd.output()).await??;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(anyhow::anyhow!("Skill failed"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: std::collections::HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub enum McpResult {
    Raw(String),
    SystemDelegate(String, serde_json::Value),
}

// --- Native Tool Handlers (Pillar 4) ---

/// Standardized A2A recruitment. Spawns a sub-agent to handle a specific sub-task.
#[agent_tool]
pub async fn recruit_specialist(
    args: serde_json::Value,
    _workspace_root: std::path::PathBuf,
) -> anyhow::Result<McpResult> {
    Ok(McpResult::SystemDelegate(
        "recruit_specialist".to_string(),
        args,
    ))
}

/// Natively lists all symbols (functions, classes, etc.) in a file with their signatures.
#[agent_tool]
pub async fn list_file_symbols(
    args: serde_json::Value,
    workspace_root: std::path::PathBuf,
) -> anyhow::Result<McpResult> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;
    let full_path = crate::utils::security::validate_path(&workspace_root, path_str)?;
    let content = tokio::fs::read_to_string(&full_path).await?;
    let mut extractor = SymbolExtractor::new();
    let symbols = extractor.extract_symbols(&full_path, &content);
    let outline: Vec<String> = symbols
        .iter()
        .map(|s| format!("{} {} -> {}", s.kind, s.name, s.signature))
        .collect();
    Ok(McpResult::Raw(outline.join("\n")))
}

/// Retrieves the full implementation body of a specific symbol in a file.
#[agent_tool]
pub async fn get_symbol_body(
    args: serde_json::Value,
    workspace_root: std::path::PathBuf,
) -> anyhow::Result<McpResult> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;
    let symbol_name = args
        .get("symbol_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'symbol_name'"))?;
    let full_path = crate::utils::security::validate_path(&workspace_root, path_str)?;
    let content = tokio::fs::read_to_string(&full_path).await?;
    let mut extractor = SymbolExtractor::new();
    let symbols = extractor.extract_symbols(&full_path, &content);
    if let Some(symbol) = symbols.into_iter().find(|s| s.name == symbol_name) {
        Ok(McpResult::Raw(symbol.body))
    } else {
        Err(anyhow::anyhow!("Symbol '{}' not found", symbol_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A test tool for macro verification.
    ///
    /// Detailed description for testing.
    #[agent_tool]
    async fn test_macro_tool(
        args: serde_json::Value,
        _workspace_root: std::path::PathBuf,
    ) -> anyhow::Result<McpResult> {
        let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("world");
        Ok(McpResult::Raw(format!("hello {}", name)))
    }

    #[tokio::test]
    async fn test_agent_tool_metadata_generation() {
        let handler = TestMacroToolHandler;
        let meta = handler.metadata();

        assert_eq!(meta.name, "test_macro_tool");
        assert!(meta
            .description
            .contains("A test tool for macro verification"));
        assert!(meta
            .description
            .contains("Detailed description for testing"));
        // Default category should be "ai"
        assert_eq!(meta.category, "ai");
    }

    #[tokio::test]
    async fn test_agent_tool_execution() {
        let handler = TestMacroToolHandler;
        let args = json!({ "name": "tadpole" });
        let result = handler.execute(args, PathBuf::from(".")).await.unwrap();

        match result {
            McpResult::Raw(s) => assert_eq!(s, "hello tadpole"),
            _ => panic!("Expected Raw result"),
        }
    }
}
