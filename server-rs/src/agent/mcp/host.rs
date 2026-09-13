//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Host Orchestrator
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Scoped agent capability enforcement on the execution path.
//! - `[Structural]` Fail-closed permission gates and safe multi-byte UTF-8 argument summary truncation.
//! - `[Structural]` Post-prompt latency tracking and transport-only client eviction.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`, `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `host::tests::*`

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

use super::authz::{
    decode_mcp_tool_name, encode_mcp_tool_name, is_mcp_server_authorized, is_mcp_tool_authorized,
};
use super::client;
use super::config::{
    resolve_mcp_environment, resolve_mcp_headers, validate_mcp_server_config, McpConfig,
};
use super::native::{
    GetSymbolBodyHandler, InspectEngineHealthHandler, ListFileSymbolsHandler,
    RecruitSpecialistHandler, RunIntegrityCheckHandler,
};
use super::registry::McpRegistry;
use super::types::{McpResult, McpToolHub, McpToolStats};
use crate::agent::script_skills::SkillDefinition;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use crate::security::permissions::{PermissionMode, PermissionPolicy, PermissionPrompter};

pub const DEFAULT_EXTERNAL_TOOL_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_MCP_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(15);
pub const DEFAULT_SKILL_TIMEOUT: Duration = Duration::from_secs(60);

/// The primary host orchestrator for managing tool registration, discovery, and execution.
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
        self.call_tool_scoped(tool_name, arguments, workspace_root, all_skills, None)
            .await
    }

    /// Calls a tool while strictly enforcing agent capability declarations if scoped.
    pub async fn call_tool_scoped(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
        mcp_declarations: Option<&[String]>,
    ) -> Result<McpResult, AppError> {
        // H2 Fix: Enforce agent capability declarations on the execution path
        if let Some(declarations) = mcp_declarations {
            if decode_mcp_tool_name(tool_name).is_some()
                && !is_mcp_tool_authorized(declarations, tool_name)
            {
                return Err(AppError::Forbidden(format!(
                    "Permission denied: MCP tool '{}' is not authorized by agent declarations",
                    tool_name
                )));
            }
        }

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
                    // H1 Fix: Character-safe truncation preventing panic on multi-byte UTF-8 boundaries
                    let arg_summary = {
                        let full = arguments.to_string();
                        if full.chars().count() > 500 {
                            let truncated: String = full.chars().take(500).collect();
                            format!("{}... [truncated]", truncated)
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

        // Latency measurement moved here to exclude human prompt response wait time
        let start_time = std::time::Instant::now();

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
                    // Evict client only on transport-level failures (network/timeout), not application errors
                    if matches!(
                        e,
                        AppError::InfrastructureError {
                            kind: InfrastructureErrorKind::NetworkError
                                | InfrastructureErrorKind::Timeout,
                            ..
                        }
                    ) {
                        self.evict_client(server_name).await;
                    }
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

            // Preserve the complete governed outcome. A tool-level isError response is
            // complete, not a transport failure, and retains retry/audit metadata.
            return Ok(McpResult::Structured(result));
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

        validate_mcp_server_config(server_name, server_config)?;

        let mode = server_config.effective_mode();
        let http_opt = server_config.resolved_http_config();
        let stdio_opt = server_config.resolved_stdio_config();

        let mut client = match mode {
            crate::agent::mcp::config::McpMode::PreferHttp => {
                let http_cfg = http_opt.ok_or_else(|| {
                    AppError::BadRequest(format!(
                        "MCP server '{}' in prefer_http mode must have http configuration",
                        server_name
                    ))
                })?;
                let mut resolved_http = http_cfg.clone();
                let resolved_headers = resolve_mcp_headers(resolved_http.headers.as_ref())?;
                resolved_http.headers = if resolved_headers.is_empty() {
                    None
                } else {
                    Some(resolved_headers)
                };

                let resolved_stdio = if let Some(stdio_cfg) = stdio_opt {
                    let cmd_to_test = if stdio_cfg.args.is_empty() {
                        stdio_cfg.command.clone()
                    } else {
                        format!("{} {}", stdio_cfg.command, stdio_cfg.args.join(" "))
                    };
                    crate::utils::security::validate_shell_command(&cmd_to_test).map_err(|e| {
                        AppError::Forbidden(format!(
                            "Security boundary: Refusing to spawn untrusted MCP server '{}' with hazardous command '{}': {}",
                            server_name, cmd_to_test, e
                        ))
                    })?;
                    let resolved_env = resolve_mcp_environment(stdio_cfg.env.as_ref())?;
                    let mut s = stdio_cfg.clone();
                    s.env = if resolved_env.is_empty() {
                        None
                    } else {
                        Some(resolved_env)
                    };
                    Some(s)
                } else {
                    None
                };

                client::McpClient::connect_adaptive(server_name, resolved_http, resolved_stdio)?
            }
            crate::agent::mcp::config::McpMode::Http => {
                let http_cfg = http_opt.ok_or_else(|| {
                    AppError::BadRequest(format!(
                        "MCP server '{}' in http mode must have url configuration",
                        server_name
                    ))
                })?;
                let resolved_headers = resolve_mcp_headers(http_cfg.headers.as_ref())?;
                client::McpClient::connect_http(
                    server_name,
                    &http_cfg.url,
                    if resolved_headers.is_empty() {
                        None
                    } else {
                        Some(&resolved_headers)
                    },
                    http_cfg.protocol_versions.first().map(|s| s.as_str()),
                )?
            }
            crate::agent::mcp::config::McpMode::Stdio
            | crate::agent::mcp::config::McpMode::Auto => {
                let stdio_cfg = stdio_opt.ok_or_else(|| {
                    AppError::BadRequest(format!(
                        "MCP server '{}' in stdio mode must have command configuration",
                        server_name
                    ))
                })?;
                let cmd_to_test = if stdio_cfg.args.is_empty() {
                    stdio_cfg.command.clone()
                } else {
                    format!("{} {}", stdio_cfg.command, stdio_cfg.args.join(" "))
                };
                crate::utils::security::validate_shell_command(&cmd_to_test).map_err(|e| {
                    AppError::Forbidden(format!(
                        "Security boundary: Refusing to spawn untrusted MCP server '{}' with hazardous command '{}': {}",
                        server_name, cmd_to_test, e
                    ))
                })?;

                let resolved_env = resolve_mcp_environment(stdio_cfg.env.as_ref())?;
                client::McpClient::spawn_stdio_with_cwd(
                    server_name,
                    &stdio_cfg.command,
                    &stdio_cfg.args,
                    if resolved_env.is_empty() {
                        None
                    } else {
                        Some(&resolved_env)
                    },
                    stdio_cfg.cwd.as_deref(),
                )
                .await
                .map_err(|e| AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::Other,
                    detail: format!("Failed to spawn MCP server '{}': {}", server_name, e),
                    help_link: None,
                })?
            }
        };

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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_call_tool_scoped_blocks_unauthorized_mcp_tool() {
        let (tx, _) = broadcast::channel(16);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let policy = PermissionPolicy::new(pool);
        let host = McpHost::new(tx, None, Arc::new(policy));
        let skills = dashmap::DashMap::new();

        // Agent has only brave search declared, but tries to invoke github tool
        let declarations = vec!["brave-search:*".to_string()];
        let tool_name = encode_mcp_tool_name("github", "create_issue");

        let res = host
            .call_tool_scoped(
                &tool_name,
                serde_json::json!({}),
                PathBuf::from("."),
                &skills,
                Some(&declarations),
            )
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_remote_structured_result_survives_host_boundary() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            for index in 0..3 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                let mut chunk = [0u8; 2048];
                loop {
                    let read = socket.read(&mut chunk).await.unwrap();
                    request.extend_from_slice(&chunk[..read]);
                    if read == 0 || request.windows(4).any(|window| window == b"\r\n\r\n") {
                        let text = String::from_utf8_lossy(&request);
                        let content_length = text
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .and_then(|value| value.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        let header_end = request
                            .windows(4)
                            .position(|window| window == b"\r\n\r\n")
                            .map(|position| position + 4)
                            .unwrap_or(request.len());
                        if request.len().saturating_sub(header_end) >= content_length {
                            break;
                        }
                    }
                }
                let body_start = request
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|position| position + 4)
                    .unwrap();
                let request_json: serde_json::Value =
                    serde_json::from_slice(&request[body_start..]).unwrap();
                let id = request_json["id"].clone();
                let result = match index {
                    0 => serde_json::json!({
                        "resultType": "complete",
                        "supportedVersions": ["2026-07-28"],
                        "capabilities": {}
                    }),
                    1 => serde_json::json!({
                        "resultType": "complete",
                        "tools": [{
                            "name": "get_budget",
                            "description": "fixture",
                            "inputSchema": {"type": "object"}
                        }]
                    }),
                    _ => serde_json::json!({
                        "resultType": "complete",
                        "content": [{"type": "text", "text": "Budget 1000"}],
                        "structuredContent": {"budget": 1000},
                        "isError": true,
                        "_meta": {"execution": {"retryable": false, "auditId": "audit-1", "reason": "budget_denied"}}
                    }),
                };
                let body =
                    serde_json::json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(), body
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });

        let mut client = client::McpClient::connect_http(
            "structured",
            &format!("http://127.0.0.1:{}/mcp", port),
            None,
            Some("2026-07-28"),
        )
        .unwrap();
        client.initialize().await.unwrap();
        assert_eq!(client.list_tools().await.unwrap().len(), 1);

        let (tx, _) = broadcast::channel(16);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let host = McpHost::new(tx, None, Arc::new(PermissionPolicy::new(pool)));
        host.clients
            .lock()
            .await
            .insert("structured".to_string(), Arc::new(Mutex::new(client)));
        let result = host
            .execute_tool_internal(
                &encode_mcp_tool_name("structured", "get_budget"),
                serde_json::json!({}),
                PathBuf::from("."),
                &dashmap::DashMap::new(),
            )
            .await
            .unwrap();
        let McpResult::Structured(value) = result else {
            panic!("expected structured MCP result");
        };
        assert_eq!(value["structuredContent"]["budget"], 1000);
        assert_eq!(value["content"][0]["text"], "Budget 1000");
        assert_eq!(value["isError"], true);
        assert_eq!(value["_meta"]["execution"]["auditId"], "audit-1");
        assert_eq!(value["_meta"]["execution"]["reason"], "budget_denied");
    }
}
