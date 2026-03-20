//! Metadata-driven Model Context Protocol (MCP) host for Tadpole OS.
//!
//! This module implements the MCP 1.0 specification, allowing agents 
//! to discover and execute tools. It also handles the "Hydra-RS" 
//! native tool layer for autonomous codebase exploration.

use crate::agent::capabilities::SkillDefinition;
use crate::utils::parser::SymbolExtractor;
use serde::{Deserialize, Serialize};
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
}

impl From<SkillDefinition> for McpToolHub {
    fn from(skill: SkillDefinition) -> Self {
        Self {
            name: skill.name,
            description: skill.description,
            input_schema: skill.schema,
            source: "legacy".to_string(),
            stats: McpToolStats::default(),
        }
    }
}

/// The primary host for managing tool registration and execution.
pub struct McpHost {
    extractor: Arc<Mutex<SymbolExtractor>>,
    pub stats: dashmap::DashMap<String, McpToolStats>,
    event_tx: broadcast::Sender<serde_json::Value>,
}

impl McpHost {
    pub fn new(event_tx: broadcast::Sender<serde_json::Value>) -> Self {
        Self {
            extractor: Arc::new(Mutex::new(SymbolExtractor::new())),
            stats: dashmap::DashMap::new(),
            event_tx,
        }
    }

    /// Lists only the tools allowed for a specific agent based on its skill IDs.
    pub fn list_tools(
        &self,
        agent_skills: &[String],
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> Vec<McpToolHub> {
        let mut tools: Vec<McpToolHub> = agent_skills
            .iter()
            .filter_map(|skill_name| all_skills.get(skill_name)) // Skill list in agent refers to names
            .map(|skill| {
                let mut hub = McpToolHub::from(skill.clone());
                if let Some(s) = self.stats.get(&hub.name) {
                    hub.stats = s.clone();
                }
                hub
            })
            .collect();

        // Always include relevant system tools
        for mut t in self.get_system_tools() {
            if let Some(s) = self.stats.get(&t.name) {
                t.stats = s.clone();
            }
            tools.push(t);
        }

        tools
    }

    fn get_system_tools(&self) -> Vec<McpToolHub> {
        vec![
            McpToolHub {
                name: "recruit_specialist".to_string(),
                description: "Standardized A2A recruitment. Spawns a sub-agent to handle a specific sub-task.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "The ID or role of the agent to recruit (e.g., 'researcher', 'coder')." },
                        "task_description": { "type": "string", "description": "Detailed instructions for the recruited agent." },
                        "safe_mode": { "type": "boolean", "description": "If true, enforces strict oversight even for 'safe' skills." }
                    },
                    "required": ["agent_id", "task_description"]
                }),
                source: "system".to_string(),
                stats: McpToolStats::default(),
            },
            McpToolHub {
                name: "list_file_symbols".to_string(),
                description: "Natively lists all symbols (functions, classes, etc.) in a file with their signatures. Use this for navigating a file without reading its full body.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative path to the file." }
                    },
                    "required": ["path"]
                }),
                source: "system".to_string(),
                stats: McpToolStats::default(),
            },
            McpToolHub {
                name: "get_symbol_body".to_string(),
                description: "Retrieves the full implementation body of a specific symbol in a file. Use this for precision retrieval of exactly what you need.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative path to the file." },
                        "symbol_name": { "type": "string", "description": "The exact name of the symbol to retrieve." }
                    },
                    "required": ["path", "symbol_name"]
                }),
                source: "system".to_string(),
                stats: McpToolStats::default(),
            }
        ]
    }

    /// Dispatches tool calls to specific MCP servers or internal legacy handlers.
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> anyhow::Result<McpResult> {
        let start_time = std::time::Instant::now();

        let result = self
            .execute_tool_internal(tool_name, arguments, workspace_root, all_skills)
            .await;

        let latency = start_time.elapsed().as_millis() as u64;
        let is_success = result.is_ok();

        // Update Stats
        {
            let mut entry = self
                .stats
                .entry(tool_name.to_string())
                .or_default();
            entry.invocations += 1;
            if is_success {
                entry.success_count += 1;
            } else {
                entry.failure_count += 1;
            }

            // Simple moving average for latency
            if entry.avg_latency_ms == 0 {
                entry.avg_latency_ms = latency;
            } else {
                entry.avg_latency_ms = (entry.avg_latency_ms + latency) / 2;
            }
        }

        // Emit Pulse Event
        let pulse = serde_json::json!({
            "type": "engine:mcp_pulse",
            "tool": tool_name,
            "status": if is_success { "success" } else { "error" },
            "latency": latency
        });
        let _ = self.event_tx.send(pulse);

        result
    }

    async fn execute_tool_internal(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> anyhow::Result<McpResult> {
        // 1. System Tools (Special internal routing)
        if tool_name == "recruit_specialist" {
            return Ok(McpResult::SystemDelegate(tool_name.to_string(), arguments));
        }

        // 2. Hydra-RS Native Tools
        if tool_name == "list_file_symbols" {
            let path_str = arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;
            let full_path = workspace_root.join(path_str);
            let content = tokio::fs::read_to_string(&full_path).await?;

            let mut extractor = self.extractor.lock().await;
            let symbols = extractor.extract_symbols(&full_path, &content);

            let outline: Vec<String> = symbols
                .iter()
                .map(|s| format!("{} {} -> {}", s.kind, s.name, s.signature))
                .collect();
            return Ok(McpResult::Raw(outline.join("\n")));
        }

        if tool_name == "get_symbol_body" {
            let path_str = arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;
            let symbol_name = arguments
                .get("symbol_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'symbol_name'"))?;
            let full_path = workspace_root.join(path_str);
            let content = tokio::fs::read_to_string(&full_path).await?;

            let mut extractor = self.extractor.lock().await;
            let symbols = extractor.extract_symbols(&full_path, &content);

            if let Some(symbol) = symbols.into_iter().find(|s| s.name == symbol_name) {
                return Ok(McpResult::Raw(symbol.body));
            } else {
                return Err(anyhow::anyhow!(
                    "Symbol '{}' not found in {}",
                    symbol_name,
                    path_str
                ));
            }
        }

        // 3. Look up the skill in the dynamic registry (Legacy/Dynamic Skills)
        if let Some(skill) = all_skills.get(tool_name) {
            let output = self
                .execute_legacy_skill(skill.value(), arguments, workspace_root)
                .await?;
            return Ok(McpResult::Raw(output));
        }

        // 4. Future: Check registered external MCP servers

        Err(anyhow::anyhow!(
            "Tool '{}' not found in any registered MCP server or legacy handler",
            tool_name
        ))
    }

    async fn execute_legacy_skill(
        &self,
        skill: &SkillDefinition,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
    ) -> anyhow::Result<String> {
        let args_json = serde_json::to_string(&arguments).unwrap_or_else(|_| "{}".to_string());

        let mut parts = skill.execution_command.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty execution command"))?;

        let mut cmd = tokio::process::Command::new(program);
        for arg in parts {
            cmd.arg(arg);
        }

        cmd.env("TADPOLE_SKILL_ARGS", &args_json);
        cmd.current_dir(workspace_root);

        let output_res =
            tokio::time::timeout(std::time::Duration::from_secs(60), cmd.output()).await;

        match output_res {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let mut combined = stdout;
                if !stderr.is_empty() {
                    combined.push_str("\n(STDERR): ");
                    combined.push_str(&stderr);
                }

                if output.status.success() {
                    Ok(combined)
                } else {
                    Err(anyhow::anyhow!(
                        "Skill '{}' failed with status {}: {}",
                        skill.name,
                        output.status,
                        combined
                    ))
                }
            }
            Ok(Err(e)) => Err(anyhow::anyhow!(
                "Failed to start subprocess for skill '{}': {}",
                skill.name,
                e
            )),
            Err(_) => Err(anyhow::anyhow!(
                "Skill '{}' timed out after 60s",
                skill.name
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum McpResult {
    Raw(String),
    SystemDelegate(String, serde_json::Value),
}
