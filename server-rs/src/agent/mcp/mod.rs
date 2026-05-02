//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Model Context Protocol (MCP) Host**: Orchestrates the sovereign
//! tool execution and environment interaction for the Tadpole OS
//! engine. Features **Sovereign Permission Isolation**: every
//! `call_tool` request passes through a `PermissionPolicy` gateway,
//! preventing unauthorized access to filesystem or network resources.
//! Implements **Dynamic Tool Registration**: supports both native Rust
//! handlers (e.g., `RecruitSpecialist`) and external MCP servers
//! defined in `mcp_config.json`. AI agents must check the `stats`
//! telemetry (TPM/Latency) before dispatching high-frequency tool
//! calls to avoid system throttling (MCP-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Permission denied due to `Deny` policy mode,
//!   timeout during external MCP server execution, or malformed
//!   JSON-RPC tool arguments causing parse failures.
//! - **Telemetry Link**: Search for `[MCP]` or `engine:mcp_pulse` in
//!   `tracing` logs for real-time tool performance tracking.
//! - **Trace Scope**: `server-rs::agent::mcp`

pub mod registry;
pub mod client;

#[allow(unused_imports)]
use self::registry::{McpRegistry, ToolHandler};
use crate::agent::script_skills::SkillDefinition;
use crate::security::permissions::{PermissionMode, PermissionPolicy, PermissionPrompter};
use crate::utils::parser::SymbolExtractor;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use server_rs_macros::agent_tool;
use std::collections::HashMap;
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
    pub stats: Arc<dashmap::DashMap<String, McpToolStats>>,
    event_tx: broadcast::Sender<serde_json::Value>,
    mcp_config_path: Option<PathBuf>,
    pub policy: Arc<PermissionPolicy>,
    pub prompter: Option<Arc<dyn PermissionPrompter>>,
    pub clients: Arc<Mutex<HashMap<String, Arc<Mutex<client::McpClient>>>>>,
}

impl McpHost {
    pub fn new(
        event_tx: broadcast::Sender<serde_json::Value>,
        mcp_config_path: Option<PathBuf>,
        policy: Arc<PermissionPolicy>,
    ) -> Self {
        let mut registry = McpRegistry::new();
        let stats = Arc::new(dashmap::DashMap::new());
        let clients = Arc::new(Mutex::new(HashMap::new()));

        // Register native Hydra-RS tools
        registry.register(Arc::new(RecruitSpecialistHandler));
        registry.register(Arc::new(ListFileSymbolsHandler));
        registry.register(Arc::new(GetSymbolBodyHandler));
        registry.register(Arc::new(RunIntegrityCheckHandler));
        registry.register(Arc::new(InspectEngineHealthHandler {
            stats: stats.clone(),
        }));

        Self {
            registry: Arc::new(Mutex::new(registry)),
            stats,
            event_tx,
            mcp_config_path,
            policy,
            prompter: None,
            clients,
        }
    }

    pub fn _set_prompter(&mut self, prompter: Arc<dyn PermissionPrompter>) {
        self.prompter = Some(prompter);
    }

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

        {
            let registry = self.registry.lock().await;
            for mut t in registry.list_all() {
                if let Some(s) = self.stats.get(&t.name) {
                    t.stats = s.clone();
                }
                tools.push(t);
            }
        }

        if let Some(ref path) = self.mcp_config_path {
            let authorized_base = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            if let Ok(safe_path) = crate::utils::security::validate_path(&authorized_base, &path.to_string_lossy()) {
                if let Ok(content) = std::fs::read_to_string(safe_path) {
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
        }

        tools
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> Result<McpResult, AppError> {
        let start_time = std::time::Instant::now();

        let mode = self.policy.get_mode(tool_name).await;
        match mode {
            PermissionMode::Deny => {
                return Err(AppError::Forbidden(format!(
                    "Permission denied: Tool {} is explicitly blocked by policy.",
                    tool_name
                )))
            }
            PermissionMode::Prompt => {
                if let Some(ref prompter) = self.prompter {
                    let decision = prompter.prompt_user(tool_name, &arguments.to_string())
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
                    if decision != PermissionMode::Allow {
                        return Err(AppError::Forbidden("User rejected tool execution".to_string()));
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
    ) -> Result<McpResult, AppError> {
        let handler = {
            let registry = self.registry.lock().await;
            registry.get(tool_name)
        };

        if let Some(h) = handler {
            return h.execute(arguments, workspace_root).await;
        }

        if let Some(skill) = all_skills.get(tool_name) {
            let output = self
                .execute_legacy_skill(skill.value(), arguments, workspace_root)
                .await?;
            return Ok(McpResult::Raw(output));
        }

        if tool_name.starts_with("mcp_") {
            let parts: Vec<&str> = tool_name.splitn(3, '_').collect();
            if parts.len() >= 3 {
                let server_name = parts[1];
                let actual_tool_name = parts[2];
                
                let client = self.get_or_spawn_client(server_name).await?;
                let mut client_lock = client.lock().await;
                
                let result = client_lock.call_tool(actual_tool_name, arguments).await
                    .map_err(|e| AppError::InfrastructureError {
                        provider_id: format!("mcp:{}", server_name),
                        detail: e.to_string(),
                        help_link: None,
                    })?;
                
                if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                    let mut output = String::new();
                    for item in content {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            output.push_str(text);
                        }
                    }
                    return Ok(McpResult::Raw(output));
                }
                
                return Ok(McpResult::Raw(serde_json::to_string_pretty(&result)
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?));
            }
        }

        Err(AppError::NotFound(format!("Tool '{}' not found", tool_name)))
    }

    async fn get_or_spawn_client(&self, server_name: &str) -> Result<Arc<Mutex<client::McpClient>>, AppError> {
        let mut clients = self.clients.lock().await;
        
        if let Some(client) = clients.get(server_name) {
            return Ok(client.clone());
        }
        
        let config_path = self.mcp_config_path.as_ref()
            .ok_or_else(|| AppError::InternalServerError("MCP config path not set".to_string()))?;
        
        let authorized_base = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let safe_path = crate::utils::security::validate_path(&authorized_base, &config_path.to_string_lossy())
            .map_err(|e| AppError::Forbidden(e.to_string()))?;
        
        let content = std::fs::read_to_string(safe_path).map_err(AppError::Io)?;
        let config: McpConfig = serde_json::from_str(&content).map_err(|e| AppError::BadRequest(e.to_string()))?;
        
        let server_config = config.mcp_servers.get(server_name)
            .ok_or_else(|| AppError::NotFound(format!("MCP server '{}' not found in config", server_name)))?;
        
        let full_command = if server_config.args.is_empty() {
            server_config.command.clone()
        } else {
            format!("{} {}", server_config.command, server_config.args.join(" "))
        };
        
        let mut client = client::McpClient::spawn(&full_command).await
            .map_err(|e| AppError::InfrastructureError {
                provider_id: format!("mcp:{}", server_name),
                detail: format!("Failed to spawn MCP server: {}", e),
                help_link: None,
            })?;
        client.initialize().await.map_err(|e| AppError::InfrastructureError {
            provider_id: format!("mcp:{}", server_name),
            detail: format!("Failed to initialize MCP client: {}", e),
            help_link: None,
        })?;
        
        let client_arc = Arc::new(Mutex::new(client));
        clients.insert(server_name.to_string(), client_arc.clone());
        
        Ok(client_arc)
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
    ) -> Result<String, AppError> {
        let args_json = serde_json::to_string(&arguments).unwrap_or_default();
        let mut parts = skill.execution_command.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| AppError::BadRequest("Empty command".to_string()))?;
        let mut cmd = tokio::process::Command::new(program);
        for arg in parts {
            cmd.arg(arg);
        }
        cmd.env("TADPOLE_SKILL_ARGS", &args_json);
        cmd.current_dir(workspace_root);
        let output =
            tokio::time::timeout(std::time::Duration::from_secs(60), cmd.output()).await
            .map_err(|_| AppError::InfrastructureError {
                provider_id: "legacy_skill".to_string(),
                detail: "Skill execution timed out after 60s".to_string(),
                help_link: None,
            })?
            .map_err(AppError::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if output.status.success() {
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(AppError::InfrastructureError {
                provider_id: "legacy_skill".to_string(),
                detail: format!("Skill failed with status {}: {}", output.status, stderr),
                help_link: None,
            })
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

// --- Native Tool Handlers ---

#[agent_tool]
pub async fn recruit_specialist(
    args: serde_json::Value,
    _workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    Ok(McpResult::SystemDelegate(
        "recruit_specialist".to_string(),
        args,
    ))
}

#[agent_tool]
pub async fn list_file_symbols(
    args: serde_json::Value,
    workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing 'path'".to_string()))?;
    let full_path = crate::utils::security::validate_path(&workspace_root, path_str)
        .map_err(|e| AppError::Forbidden(e.to_string()))?;
    let content = tokio::fs::read_to_string(&full_path).await.map_err(AppError::Io)?;
    let mut extractor = SymbolExtractor::new();
    let symbols = extractor.extract_symbols(&full_path, &content);
    let outline: Vec<String> = symbols
        .iter()
        .map(|s| format!("{} {} -> {}", s.kind, s.name, s.signature))
        .collect();
    Ok(McpResult::Raw(outline.join("\n")))
}

#[agent_tool]
pub async fn get_symbol_body(
    args: serde_json::Value,
    workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing 'path'".to_string()))?;
    let symbol_name = args
        .get("symbol_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing 'symbol_name'".to_string()))?;
    let full_path = crate::utils::security::validate_path(&workspace_root, path_str)
        .map_err(|e| AppError::Forbidden(e.to_string()))?;
    let content = tokio::fs::read_to_string(&full_path).await.map_err(AppError::Io)?;
    let mut extractor = SymbolExtractor::new();
    let symbols = extractor.extract_symbols(&full_path, &content);
    if let Some(symbol) = symbols.into_iter().find(|s| s.name == symbol_name) {
        Ok(McpResult::Raw(symbol.body))
    } else {
        Err(AppError::NotFound(format!("Symbol '{}' not found", symbol_name)))
    }
}

#[agent_tool]
pub async fn run_integrity_check(
    _args: serde_json::Value,
    _workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let mut cmd = tokio::process::Command::new("python");
    cmd.arg("execution/self_audit_tool.py");

    let output = tokio::time::timeout(std::time::Duration::from_secs(300), cmd.output()).await
        .map_err(|_| AppError::InfrastructureError {
            provider_id: "integrity_check".to_string(),
            detail: "Integrity check timed out after 300s".to_string(),
            help_link: None,
        })?
        .map_err(AppError::Io)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if output.status.success() {
        Ok(McpResult::Raw(stdout))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::InfrastructureError {
            provider_id: "integrity_check".to_string(),
            detail: format!("Integrity Audit Failed: {}\n{}", stdout, stderr),
            help_link: None,
        })
    }
}

pub struct InspectEngineHealthHandler {
    pub stats: Arc<dashmap::DashMap<String, McpToolStats>>,
}

#[async_trait::async_trait]
impl ToolHandler for InspectEngineHealthHandler {
    async fn execute(
        &self,
        _args: serde_json::Value,
        _workspace_root: std::path::PathBuf,
    ) -> Result<McpResult, AppError> {
        let stats_vec: Vec<serde_json::Value> = self.stats
            .iter()
            .map(|kv| {
                let name = kv.key();
                let stats = kv.value();
                serde_json::json!({
                    "tool": name,
                    "invocations": stats.invocations,
                    "success_rate": if stats.invocations > 0 { stats.success_count as f64 / stats.invocations as f64 } else { 0.0 },
                    "avg_latency_ms": stats.avg_latency_ms
                })
            })
            .collect();

        Ok(McpResult::Raw(serde_json::to_string_pretty(&stats_vec)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?))
    }

    fn metadata(&self) -> McpToolHub {
        McpToolHub {
            name: "inspect_engine_health".to_string(),
            description: "Retrieves real-time execution statistics for all registered MCP tools."
                .to_string(),
            input_schema: serde_json::json!({}),
            source: "native".to_string(),
            stats: McpToolStats::default(),
            category: "introspection".to_string(),
        }
    }
}

// Metadata: [mod]

// Metadata: [mod]
