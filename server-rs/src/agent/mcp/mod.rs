//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `mcp::tests::*`, `mcp_tests::tests::*`

pub mod client;
pub mod registry;

#[allow(unused_imports)]
use self::registry::{McpRegistry, ToolHandler};
use crate::agent::script_skills::SkillDefinition;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use crate::security::permissions::{PermissionMode, PermissionPolicy, PermissionPrompter};
use crate::utils::parser::SymbolExtractor;
use serde::{Deserialize, Serialize};
use server_rs_macros::agent_tool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

pub const DEFAULT_EXTERNAL_TOOL_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_MCP_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(15);
pub const DEFAULT_SKILL_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_INTEGRITY_CHECK_TIMEOUT: Duration = Duration::from_secs(300);

/// Encodes an MCP server name and tool name into an unambiguous dispatch identifier.
pub fn encode_mcp_tool_name(server: &str, tool: &str) -> String {
    format!("mcp__{}__{}", server, tool)
}

/// Decodes an MCP tool name into `(server_name, actual_tool_name)`.
pub fn decode_mcp_tool_name(name: &str) -> Option<(&str, &str)> {
    if let Some(rest) = name.strip_prefix("mcp__") {
        if let Some((server, tool)) = rest.split_once("__") {
            return Some((server, tool));
        }
    }
    // Backward-compatibility fallback for single-underscore prefix
    if let Some(rest) = name.strip_prefix("mcp_") {
        if let Some((server, tool)) = rest.split_once('_') {
            return Some((server, tool));
        }
    }
    None
}

/// Returns whether an externally discovered MCP tool is present in an agent's
/// explicit capability declaration. Both the encoded runtime name and the
/// human-authored `server:tool` form are accepted. An empty declaration grants
/// no external MCP access.
pub fn is_mcp_tool_authorized(declarations: &[String], encoded_tool_name: &str) -> bool {
    let Some((server, tool)) = decode_mcp_tool_name(encoded_tool_name) else {
        return false;
    };
    let qualified = format!("{}:{}", server, tool);
    let server_wildcard = format!("{}:*", server);

    declarations.iter().any(|declaration| {
        declaration == encoded_tool_name
            || declaration == &qualified
            || declaration == &server_wildcard
    })
}

/// Returns whether an agent declaration grants any access to an MCP server.
/// This check is used before discovery so an undeclared external process is
/// never started merely to build the agent's tool list.
pub fn is_mcp_server_authorized(declarations: &[String], server_name: &str) -> bool {
    declarations.iter().any(|declaration| {
        decode_mcp_tool_name(declaration).is_some_and(|(server, _)| server == server_name)
            || declaration
                .split_once(':')
                .is_some_and(|(server, _)| server == server_name)
    })
}

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

    pub async fn evict_client(&self, server_name: &str) {
        let mut clients = self.clients.lock().await;
        if let Some(removed) = clients.remove(server_name) {
            info!("[MCP] Evicted poisoned MCP client '{}'", server_name);
            tokio::spawn(async move {
                let mut guard = removed.lock().await;
                let _ = guard.shutdown().await;
            });
        }
    }

    pub async fn list_tools(
        &self,
        agent_skills: &[String],
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> Vec<McpToolHub> {
        self.list_tools_scoped(agent_skills, all_skills, None).await
    }

    /// Lists tools for a single agent while limiting external discovery to
    /// servers named by that agent's explicit MCP capability declarations.
    pub async fn list_tools_for_agent(
        &self,
        agent_skills: &[String],
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
        mcp_declarations: &[String],
    ) -> Vec<McpToolHub> {
        self.list_tools_scoped(agent_skills, all_skills, Some(mcp_declarations))
            .await
    }

    async fn list_tools_scoped(
        &self,
        agent_skills: &[String],
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
        mcp_declarations: Option<&[String]>,
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
            let authorized_base =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            match crate::utils::security::validate_path(&authorized_base, &path.to_string_lossy()) {
                Ok(safe_path) => match tokio::fs::read_to_string(safe_path).await {
                    Ok(content) => match serde_json::from_str::<McpConfig>(&content) {
                        Ok(config) => {
                            let mut server_names: Vec<String> = config
                                .mcp_servers
                                .into_keys()
                                .filter(|server_name| {
                                    mcp_declarations.is_none_or(|declarations| {
                                        is_mcp_server_authorized(declarations, server_name)
                                    })
                                })
                                .collect();
                            server_names.sort();
                            let discoveries = server_names.iter().map(|server_name| async move {
                                (
                                    server_name,
                                    tokio::time::timeout(
                                        DEFAULT_MCP_DISCOVERY_TIMEOUT,
                                        self.discover_server_tools(server_name),
                                    )
                                    .await,
                                )
                            });
                            for (server_name, discovery) in
                                futures::future::join_all(discoveries).await
                            {
                                match discovery {
                                    Ok(Ok(mut discovered)) => tools.append(&mut discovered),
                                    Ok(Err(error)) => warn!(
                                        "⚠️ [MCP] Tool discovery failed for server '{}': {}",
                                        server_name, error
                                    ),
                                    Err(_) => warn!(
                                        "⚠️ [MCP] Tool discovery timed out for server '{}' after {:?}",
                                        server_name, DEFAULT_MCP_DISCOVERY_TIMEOUT
                                    ),
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "⚠️ [MCP] Failed to parse MCP config JSON from {:?}: {}",
                                path, e
                            );
                        }
                    },
                    Err(e) => {
                        warn!("⚠️ [MCP] Failed to read MCP config file {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    warn!("⚠️ [MCP] Invalid MCP config path {:?}: {}", path, e);
                }
            }
        }

        tools
    }

    async fn discover_server_tools(&self, server_name: &str) -> Result<Vec<McpToolHub>, AppError> {
        let client = self.get_or_spawn_client(server_name).await?;
        let mut client = client.lock().await;
        let definitions = client.list_tools().await?;
        let mut tools = Vec::with_capacity(definitions.len());

        for definition in definitions {
            let Some(tool_name) = definition.get("name").and_then(|value| value.as_str()) else {
                warn!(
                    "⚠️ [MCP] Server '{}' returned a tool without a valid name",
                    server_name
                );
                continue;
            };
            let encoded_name = encode_mcp_tool_name(server_name, tool_name);
            let mut stats = McpToolStats::default();
            if let Some(existing) = self.stats.get(&encoded_name) {
                stats = existing.clone();
            }
            tools.push(McpToolHub {
                name: encoded_name,
                description: definition
                    .get("description")
                    .and_then(|value| value.as_str())
                    .unwrap_or("External MCP tool")
                    .to_string(),
                input_schema: definition
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({"type": "object"})),
                source: format!("mcp:{}", server_name),
                stats,
                category: "external".to_string(),
            });
        }

        tools.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(tools)
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> Result<McpResult, AppError> {
        let start_time = std::time::Instant::now();

        // 1. Verify tool existence before invoking permission gates
        let tool_exists = {
            let registry = self.registry.lock().await;
            registry.get(tool_name).is_some()
        } || all_skills.contains_key(tool_name)
            || decode_mcp_tool_name(tool_name).is_some();

        if !tool_exists {
            return Err(AppError::NotFound(format!(
                "Tool '{}' not found",
                tool_name
            )));
        }

        let mode = self.policy.get_mode(None, None, tool_name).await;
        match mode {
            PermissionMode::Deny => {
                return Err(AppError::Forbidden(format!(
                    "Permission denied: Tool {} is explicitly blocked by policy.",
                    tool_name
                )))
            }
            PermissionMode::Prompt => {
                if let Some(ref prompter) = self.prompter {
                    let arg_summary = {
                        let full = arguments.to_string();
                        if full.len() > 500 {
                            format!("{}... [truncated]", &full[..500])
                        } else {
                            full
                        }
                    };
                    let decision = prompter
                        .prompt_user(tool_name, &arg_summary)
                        .await
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
                    if decision != PermissionMode::Allow {
                        return Err(AppError::Forbidden(
                            "User rejected tool execution".to_string(),
                        ));
                    }
                } else {
                    // Fail-Closed Security: refuse execution when no prompter is present
                    return Err(AppError::Forbidden(format!(
                        "Permission denied: Tool '{}' requires human confirmation via Prompt policy, but no prompter is configured.",
                        tool_name
                    )));
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

        if let Some((server_name, actual_tool_name)) = decode_mcp_tool_name(tool_name) {
            let client = self.get_or_spawn_client(server_name).await?;
            let mut client_lock = client.lock().await;

            let exec_fut = client_lock.call_tool(actual_tool_name, arguments);
            let result_res = tokio::time::timeout(DEFAULT_EXTERNAL_TOOL_TIMEOUT, exec_fut).await;

            let result = match result_res {
                Ok(Ok(val)) => val,
                Ok(Err(e)) => {
                    self.evict_client(server_name).await;
                    return Err(AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "MCP server '{}' tool execution failed: {}",
                            server_name, e
                        ),
                        help_link: None,
                    });
                }
                Err(_) => {
                    self.evict_client(server_name).await;
                    return Err(AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::Timeout,
                        detail: format!(
                            "MCP server '{}' tool '{}' timed out after {:?}",
                            server_name, actual_tool_name, DEFAULT_EXTERNAL_TOOL_TIMEOUT
                        ),
                        help_link: None,
                    });
                }
            };

            if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                let mut output = String::new();
                for item in content {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        output.push_str(text);
                    }
                }
                return Ok(McpResult::Raw(output));
            }

            return Ok(McpResult::Raw(
                serde_json::to_string_pretty(&result)
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            ));
        }

        Err(AppError::NotFound(format!(
            "Tool '{}' not found",
            tool_name
        )))
    }

    async fn get_or_spawn_client(
        &self,
        server_name: &str,
    ) -> Result<Arc<Mutex<client::McpClient>>, AppError> {
        // Fast-path: Check existing client under read lock
        {
            let clients = self.clients.lock().await;
            if let Some(client) = clients.get(server_name) {
                return Ok(client.clone());
            }
        }

        let config_path = self
            .mcp_config_path
            .as_ref()
            .ok_or_else(|| AppError::InternalServerError("MCP config path not set".to_string()))?;

        let authorized_base =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let safe_path =
            crate::utils::security::validate_path(&authorized_base, &config_path.to_string_lossy())
                .map_err(|e| AppError::Forbidden(e.to_string()))?;

        let content = tokio::fs::read_to_string(safe_path)
            .await
            .map_err(AppError::Io)?;
        let config: McpConfig =
            serde_json::from_str(&content).map_err(|e| AppError::BadRequest(e.to_string()))?;

        let server_config = config.mcp_servers.get(server_name).ok_or_else(|| {
            AppError::NotFound(format!("MCP server '{}' not found in config", server_name))
        })?;

        let cmd_to_test = if server_config.args.is_empty() {
            server_config.command.clone()
        } else {
            format!("{} {}", server_config.command, server_config.args.join(" "))
        };
        crate::utils::security::validate_shell_command(&cmd_to_test).map_err(|e| {
            AppError::Forbidden(format!(
                "Security boundary: Refusing to spawn untrusted MCP server '{}' with hazardous command '{}': {}",
                server_name, cmd_to_test, e
            ))
        })?;

        // Spawn and initialize without holding the global clients lock
        let resolved_env = resolve_mcp_environment(server_config.env.as_ref())?;
        let mut client = client::McpClient::spawn(
            server_name,
            &server_config.command,
            &server_config.args,
            if resolved_env.is_empty() {
                None
            } else {
                Some(&resolved_env)
            },
        )
        .await
        .map_err(|e| AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::Other,
            detail: format!("Failed to spawn MCP server '{}': {}", server_name, e),
            help_link: None,
        })?;

        client
            .initialize()
            .await
            .map_err(|e| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!("Failed to initialize MCP client '{}': {}", server_name, e),
                help_link: None,
            })?;

        let client_arc = Arc::new(Mutex::new(client));

        // Re-acquire lock to insert
        let mut clients = self.clients.lock().await;
        if let Some(existing) = clients.get(server_name) {
            return Ok(existing.clone());
        }
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
            let prev_count = entry.invocations.saturating_sub(1) as u128;
            let current_latency = latency as u128;
            let avg = entry.avg_latency_ms as u128;
            let new_avg = ((avg * prev_count) + current_latency) / (entry.invocations as u128);
            entry.avg_latency_ms = new_avg as u64;
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

        // Hardening: Verify program is on the whitelist of approved interpreters/utilities
        let allowed_binaries = [
            "python",
            "python3",
            "node",
            "sh",
            "bash",
            "cmd",
            "powershell",
            "pwsh",
            "echo",
            "ls",
        ];
        let program_lower = program.to_lowercase();
        let clean_program = program_lower.trim_end_matches(".exe");
        if !allowed_binaries.contains(&clean_program) {
            return Err(AppError::Forbidden(format!(
                "Command execution blocked: '{}' is not an approved binary. Allowed: {:?}",
                program, allowed_binaries
            )));
        }

        let mut cmd = tokio::process::Command::new(program);
        for arg in parts {
            cmd.arg(arg);
        }
        cmd.env("TADPOLE_SKILL_ARGS", &args_json);
        cmd.current_dir(workspace_root);
        let output = tokio::time::timeout(DEFAULT_SKILL_TIMEOUT, cmd.output())
            .await
            .map_err(|_| AppError::InfrastructureError {
                provider_id: ProviderId::System,
                kind: InfrastructureErrorKind::Timeout,
                detail: format!(
                    "Skill execution timed out after {:?}",
                    DEFAULT_SKILL_TIMEOUT
                ),
                help_link: None,
            })?
            .map_err(AppError::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if output.status.success() {
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(AppError::InfrastructureError {
                provider_id: ProviderId::System,
                kind: InfrastructureErrorKind::ApiError,
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
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
}

pub fn validate_mcp_server_config(
    server_name: &str,
    config: &McpServerConfig,
) -> Result<(), AppError> {
    if server_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "MCP server name cannot be empty".to_string(),
        ));
    }
    if config.command.trim().is_empty() {
        return Err(AppError::BadRequest(format!(
            "MCP server '{}' has an empty command",
            server_name
        )));
    }
    let command_line = if config.args.is_empty() {
        config.command.clone()
    } else {
        format!("{} {}", config.command, config.args.join(" "))
    };
    crate::utils::security::validate_shell_command(&command_line).map_err(|error| {
        AppError::Forbidden(format!(
            "MCP server '{}' has an unsafe launcher: {}",
            server_name, error
        ))
    })?;

    if let Some(environment) = &config.env {
        for (name, value) in environment {
            if !is_valid_environment_name(name) {
                return Err(AppError::BadRequest(format!(
                    "MCP server '{}' has invalid environment variable name '{}'",
                    server_name, name
                )));
            }
            if value.starts_with("${") && value.ends_with('}') {
                let placeholder = &value[2..value.len() - 1];
                if !is_valid_environment_name(placeholder) {
                    return Err(AppError::BadRequest(format!(
                        "MCP server '{}' has invalid environment placeholder '{}'",
                        server_name, value
                    )));
                }
            }
        }
    }

    Ok(())
}

fn resolve_mcp_environment(
    configured: Option<&HashMap<String, String>>,
) -> Result<HashMap<String, String>, AppError> {
    let mut resolved = HashMap::new();
    let Some(configured) = configured else {
        return Ok(resolved);
    };

    for (key, value) in configured {
        if !is_valid_environment_name(key) {
            return Err(AppError::BadRequest(format!(
                "MCP environment variable name '{}' is invalid",
                key
            )));
        }

        let resolved_value = if value.starts_with("${") && value.ends_with('}') {
            let variable_name = &value[2..value.len() - 1];
            if !is_valid_environment_name(variable_name) {
                return Err(AppError::BadRequest(format!(
                    "MCP environment placeholder '{}' is invalid",
                    value
                )));
            }
            std::env::var(variable_name).map_err(|_| {
                AppError::BadRequest(format!(
                    "MCP environment variable '{}' is required but is not configured",
                    variable_name
                ))
            })?
        } else if value == "CONFIGURE_LOCALLY" {
            return Err(AppError::BadRequest(format!(
                "MCP environment variable '{}' must be configured locally before the server can start",
                key
            )));
        } else {
            value.clone()
        };

        resolved.insert(key.clone(), resolved_value);
    }

    Ok(resolved)
}

fn is_valid_environment_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

#[derive(Debug, Clone)]
pub enum McpResult {
    Raw(String),
    SystemDelegate(String, serde_json::Value),
}

// --- Native Tool Handlers ---

#[agent_tool]
pub async fn recruit_specialist(
    agent_id: String,
    task_description: String,
    _workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    Ok(McpResult::SystemDelegate(
        "recruit_specialist".to_string(),
        serde_json::json!({
            "agent_id": agent_id,
            "task_description": task_description
        }),
    ))
}

#[agent_tool]
pub async fn list_file_symbols(
    path: String,
    workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let full_path = crate::utils::security::validate_path(&workspace_root, &path)
        .map_err(|e| AppError::Forbidden(e.to_string()))?;
    if full_path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Path '{}' is a directory, not a file. list_file_symbols only accepts file paths.",
            path
        )));
    }
    let content = tokio::fs::read_to_string(&full_path)
        .await
        .map_err(AppError::Io)?;
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
    path: String,
    symbol_name: String,
    workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let full_path = crate::utils::security::validate_path(&workspace_root, &path)
        .map_err(|e| AppError::Forbidden(e.to_string()))?;
    if full_path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Path '{}' is a directory, not a file. get_symbol_body only accepts file paths.",
            path
        )));
    }
    let content = tokio::fs::read_to_string(&full_path)
        .await
        .map_err(AppError::Io)?;
    let mut extractor = SymbolExtractor::new();
    let symbols = extractor.extract_symbols(&full_path, &content);
    if let Some(symbol) = symbols.into_iter().find(|s| s.name == symbol_name) {
        Ok(McpResult::Raw(symbol.body))
    } else {
        Err(AppError::NotFound(format!(
            "Symbol '{}' not found",
            symbol_name
        )))
    }
}

#[agent_tool]
pub async fn run_integrity_check(
    workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let mut cmd = tokio::process::Command::new("python");
    cmd.arg("execution/self_audit_tool.py");
    cmd.current_dir(workspace_root);

    let output = tokio::time::timeout(DEFAULT_INTEGRITY_CHECK_TIMEOUT, cmd.output())
        .await
        .map_err(|_| AppError::InfrastructureError {
            provider_id: ProviderId::System,
            kind: InfrastructureErrorKind::Timeout,
            detail: format!(
                "Integrity check timed out after {:?}",
                DEFAULT_INTEGRITY_CHECK_TIMEOUT
            ),
            help_link: None,
        })?
        .map_err(AppError::Io)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if output.status.success() {
        Ok(McpResult::Raw(stdout))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::InfrastructureError {
            provider_id: ProviderId::System,
            kind: InfrastructureErrorKind::ApiError,
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

        Ok(McpResult::Raw(
            serde_json::to_string_pretty(&stats_vec)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_name_encoding_and_decoding() {
        let encoded = encode_mcp_tool_name("github_tools", "create_issue");
        assert_eq!(encoded, "mcp__github_tools__create_issue");

        let decoded = decode_mcp_tool_name(&encoded).unwrap();
        assert_eq!(decoded.0, "github_tools");
        assert_eq!(decoded.1, "create_issue");

        // Legacy fallback
        let legacy_decoded = decode_mcp_tool_name("mcp_sqlite_query").unwrap();
        assert_eq!(legacy_decoded.0, "sqlite");
        assert_eq!(legacy_decoded.1, "query");
    }

    #[test]
    fn mcp_authorization_accepts_exact_and_qualified_names() {
        let encoded = encode_mcp_tool_name("github", "create_issue");

        assert!(is_mcp_tool_authorized(
            std::slice::from_ref(&encoded),
            &encoded
        ));
        assert!(is_mcp_tool_authorized(
            &["github:create_issue".to_string()],
            &encoded
        ));
        assert!(is_mcp_tool_authorized(&["github:*".to_string()], &encoded));
        assert!(!is_mcp_tool_authorized(&[], &encoded));
        assert!(!is_mcp_tool_authorized(
            &["github:delete_issue".to_string()],
            &encoded
        ));
    }

    #[test]
    fn mcp_server_authorization_accepts_only_declared_servers() {
        assert!(is_mcp_server_authorized(
            &["github:create_issue".to_string()],
            "github"
        ));
        assert!(is_mcp_server_authorized(
            &["github:*".to_string()],
            "github"
        ));
        assert!(is_mcp_server_authorized(
            &[encode_mcp_tool_name("github", "create_issue")],
            "github"
        ));
        assert!(!is_mcp_server_authorized(
            &["gitlab:create_issue".to_string()],
            "github"
        ));
        assert!(!is_mcp_server_authorized(&[], "github"));
    }

    #[test]
    fn resolves_environment_placeholders_without_exposing_secret_values() {
        let expected_path = std::env::var_os("PATH")
            .expect("PATH must exist in the test environment")
            .into_string()
            .expect("PATH must be valid Unicode in the test environment");
        let configured = HashMap::from([
            ("CHILD_PATH".to_string(), "${PATH}".to_string()),
            ("LITERAL_SETTING".to_string(), "enabled".to_string()),
        ]);

        let resolved = resolve_mcp_environment(Some(&configured)).unwrap();

        assert_eq!(resolved.get("CHILD_PATH"), Some(&expected_path));
        assert_eq!(
            resolved.get("LITERAL_SETTING").map(String::as_str),
            Some("enabled")
        );
    }

    #[test]
    fn rejects_unconfigured_environment_placeholders() {
        let configured =
            HashMap::from([("API_TOKEN".to_string(), "CONFIGURE_LOCALLY".to_string())]);

        let error = resolve_mcp_environment(Some(&configured)).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
        assert!(!error.to_string().contains("API_TOKEN="));
    }

    #[test]
    fn rejects_invalid_environment_names() {
        let configured = HashMap::from([("BAD-NAME".to_string(), "value".to_string())]);

        let error = resolve_mcp_environment(Some(&configured)).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn validates_safe_mcp_server_configuration() {
        let config = McpServerConfig {
            command: "python".to_string(),
            args: vec!["execution/server.py".to_string()],
            env: Some(HashMap::from([(
                "API_TOKEN".to_string(),
                "${LOCAL_API_TOKEN}".to_string(),
            )])),
        };

        validate_mcp_server_config("example", &config).unwrap();
    }

    #[test]
    fn rejects_mcp_server_with_invalid_placeholder() {
        let config = McpServerConfig {
            command: "python".to_string(),
            args: vec!["execution/server.py".to_string()],
            env: Some(HashMap::from([(
                "API_TOKEN".to_string(),
                "${BAD-NAME}".to_string(),
            )])),
        };

        let error = validate_mcp_server_config("example", &config).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[tokio::test]
    async fn test_prompt_mode_fails_closed_without_prompter() {
        let (tx, _) = broadcast::channel(16);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE permission_policies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tool_name TEXT NOT NULL UNIQUE,
                mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE agent_permission_policies (
                agent_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                mode TEXT NOT NULL,
                PRIMARY KEY (agent_id, tool_name)
            );
            CREATE TABLE role_permission_policies (
                role TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                mode TEXT NOT NULL,
                PRIMARY KEY (role, tool_name)
            );
            CREATE TABLE capability_policies (
                capability_class TEXT NOT NULL,
                resource_pattern TEXT NOT NULL,
                mode TEXT NOT NULL,
                PRIMARY KEY (capability_class, resource_pattern)
            );
            CREATE TABLE agent_capability_policies (
                agent_id TEXT NOT NULL,
                capability_class TEXT NOT NULL,
                resource_pattern TEXT NOT NULL,
                mode TEXT NOT NULL,
                PRIMARY KEY (agent_id, capability_class, resource_pattern)
            );
            CREATE TABLE role_capability_policies (
                role TEXT NOT NULL,
                capability_class TEXT NOT NULL,
                resource_pattern TEXT NOT NULL,
                mode TEXT NOT NULL,
                PRIMARY KEY (role, capability_class, resource_pattern)
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        let policy = PermissionPolicy::new(pool);
        policy
            .set_mode("recruit_specialist", PermissionMode::Prompt)
            .await
            .unwrap();

        let host = McpHost::new(tx, None, Arc::new(policy));
        let skills = dashmap::DashMap::new();

        let res = host
            .call_tool(
                "recruit_specialist",
                serde_json::json!({}),
                PathBuf::from("."),
                &skills,
            )
            .await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));
    }
}
