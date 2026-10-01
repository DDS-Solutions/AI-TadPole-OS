//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Host Orchestrator
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Scoped agent capability enforcement and identity-bound policy on execution path.
//! - `[Structural]` Fail-closed permission gates and safe multi-byte UTF-8 argument summary truncation.
//! - `[Structural]` Post-prompt latency tracking and transport-only client eviction.
//! - `[Structural]` Child process lifecycle containment with kill_on_drop and bounded output capture.
//! - `[Structural]` Clean single-flight client spawn with orphan client teardown.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`, `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

use super::authz::{decode_mcp_tool_name, encode_mcp_tool_name, is_mcp_server_authorized};
use super::client::McpClient;
use super::client_pool::{ClientHandle, ClientPool};
use super::config_store::McpConfigStore;
use super::context::AgentScope;
use super::exec::execute_legacy_skill;
pub use super::exec::{DEFAULT_SKILL_TIMEOUT, MAX_SKILL_STDERR_BYTES, MAX_SKILL_STDOUT_BYTES};
pub use super::gate::sanitize_reflected;
use super::gate::PermissionGate;
use super::native::{
    GetSymbolBodyHandler, InspectEngineHealthHandler, ListFileSymbolsHandler,
    RecruitSpecialistHandler, RunIntegrityCheckHandler,
};
use super::registry::McpRegistry;
use super::resolve::{resolve_tool, ResolvedTool};
use super::telemetry::McpTelemetry;
use super::transport::spawn_mcp_client;
use super::types::{McpResult, McpToolHub, McpToolStats};
use crate::agent::script_skills::SkillDefinition;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use crate::security::permissions::{PermissionPolicy, PermissionPrompter};

pub const DEFAULT_EXTERNAL_TOOL_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_MCP_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(15);

/// The primary host orchestrator for managing tool registration, discovery, and execution.
pub struct McpHost {
    registry: Arc<Mutex<McpRegistry>>,
    telemetry: McpTelemetry,
    config_store: Option<McpConfigStore>,
    gate: PermissionGate,
    client_pool: ClientPool,
}

impl McpHost {
    /// Checks if a native tool with the given name is registered.
    pub async fn has_native_tool(&self, name: &str) -> bool {
        let registry = self.registry.lock().await;
        registry.get(name).is_some()
    }

    /// Returns a copy of execution stats for a tool if recorded.
    pub fn get_tool_stats(&self, tool_name: &str) -> Option<McpToolStats> {
        self.telemetry.get_tool_stats(tool_name)
    }

    pub fn new(
        event_tx: broadcast::Sender<serde_json::Value>,
        mcp_config_path: Option<PathBuf>,
        policy: Arc<PermissionPolicy>,
    ) -> Self {
        let mut registry = McpRegistry::new();
        let telemetry = McpTelemetry::new(event_tx);
        let client_pool = ClientPool::new();

        // Register native Hydra-RS tools
        registry.register(Arc::new(RecruitSpecialistHandler));
        registry.register(Arc::new(ListFileSymbolsHandler));
        registry.register(Arc::new(GetSymbolBodyHandler));
        registry.register(Arc::new(RunIntegrityCheckHandler));
        registry.register(Arc::new(InspectEngineHealthHandler {
            stats: telemetry.stats_map().clone(),
        }));

        Self {
            registry: Arc::new(Mutex::new(registry)),
            telemetry,
            config_store: mcp_config_path.map(McpConfigStore::new),
            gate: PermissionGate::new(policy, None),
            client_pool,
        }
    }

    /// Builder pattern for configuring a permission prompter.
    pub fn with_prompter(mut self, prompter: Arc<dyn PermissionPrompter>) -> Self {
        self.gate = self.gate.with_prompter(prompter);
        self
    }

    pub async fn evict_client(&self, server_name: &str, expected_generation: Option<u64>) -> bool {
        self.client_pool
            .evict(server_name, expected_generation)
            .await
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
                if let Some(s) = self.telemetry.get_tool_stats(&hub.name) {
                    hub.stats = s;
                }
                hub
            })
            .collect();

        {
            let registry = self.registry.lock().await;
            for mut t in registry.list_all() {
                if let Some(s) = self.telemetry.get_tool_stats(&t.name) {
                    t.stats = s;
                }
                tools.push(t);
            }
        }

        if let Some(ref config_store) = self.config_store {
            match config_store.get_config().await {
                Ok(config) => {
                    let mut eager_servers: Vec<String> = Vec::new();
                    let mut deferred_servers: Vec<String> = Vec::new();

                    for (server_name, s_cfg) in &config.mcp_servers {
                        if mcp_declarations.is_none_or(|declarations| {
                            is_mcp_server_authorized(declarations, server_name)
                        }) {
                            if s_cfg.deferred {
                                deferred_servers.push(server_name.clone());
                            } else {
                                eager_servers.push(server_name.clone());
                            }
                        }
                    }
                    eager_servers.sort();
                    deferred_servers.sort();

                    for s_name in &deferred_servers {
                        tools.push(McpToolHub {
                            name: format!("mcp_server_{}", s_name),
                            description: format!(
                                "Deferred MCP Server '{}'. Use 'list_mcp_tools' or 'get_mcp_tool_info' to discover its tools, and 'call_mcp_tool' to execute.",
                                s_name
                            ),
                            input_schema: serde_json::json!({
                                "type": "object",
                                "properties": {
                                    "action": { "type": "string", "enum": ["list_tools", "info"], "description": "Operation" }
                                }
                            }),
                            source: "deferred".to_string(),
                            stats: McpToolStats::default(),
                            category: "deferred".to_string(),
                        });
                    }

                    let discoveries = eager_servers.iter().map(|server_name| async move {
                        (
                            server_name,
                            tokio::time::timeout(
                                DEFAULT_MCP_DISCOVERY_TIMEOUT,
                                self.discover_server_tools(server_name),
                            )
                            .await,
                        )
                    });
                    for (server_name, discovery) in futures::future::join_all(discoveries).await {
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
                    warn!("⚠️ [MCP] Failed to load MCP config: {}", e);
                }
            }
        }

        tools
    }

    async fn discover_server_tools(&self, server_name: &str) -> Result<Vec<McpToolHub>, AppError> {
        let handle = self.get_or_spawn_client(server_name).await?;
        let definitions = {
            let mut client_lock = handle.client.lock().await;
            client_lock.list_tools().await?
        };
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
            if let Some(existing) = self.telemetry.get_tool_stats(&encoded_name) {
                stats = existing;
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
        let system_decls = vec!["*".to_string()];
        let system_scope = AgentScope::new("system", &system_decls);
        self.call_tool_scoped(
            &system_scope,
            tool_name,
            arguments,
            workspace_root,
            all_skills,
        )
        .await
    }

    /// Calls a tool while strictly enforcing agent capability declarations and isolated policies.
    pub async fn call_tool_scoped(
        &self,
        scope: &AgentScope<'_>,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
        all_skills: &dashmap::DashMap<String, SkillDefinition>,
    ) -> Result<McpResult, AppError> {
        // 1. Resolve tool existence and target first
        let has_native = self.has_native_tool(tool_name).await;
        let skill = all_skills.get(tool_name).map(|s| s.clone());
        let resolved = resolve_tool(tool_name, has_native, skill)?;

        // 2. Enforce gate authorization & prompt policy
        self.gate
            .check_permission(scope, tool_name, &arguments)
            .await?;

        let start_time = std::time::Instant::now();

        let result = self
            .execute_resolved_tool(resolved, tool_name, arguments, workspace_root)
            .await;

        let latency = start_time.elapsed().as_millis() as u64;
        self.telemetry
            .record_invocation(tool_name, result.is_ok(), latency);

        result
    }

    async fn execute_resolved_tool(
        &self,
        resolved: ResolvedTool,
        tool_name: &str,
        arguments: serde_json::Value,
        workspace_root: std::path::PathBuf,
    ) -> Result<McpResult, AppError> {
        match resolved {
            ResolvedTool::Mcp {
                server_name,
                tool_name: actual_tool_name,
            } => {
                let handle = self.get_or_spawn_client(&server_name).await?;
                let mut client_lock = handle.client.lock().await;

                let exec_fut = client_lock.call_tool(&actual_tool_name, arguments);
                let result_res =
                    tokio::time::timeout(DEFAULT_EXTERNAL_TOOL_TIMEOUT, exec_fut).await;

                // Explicitly release client_lock before handling errors or evictions
                drop(client_lock);

                let result = match result_res {
                    Ok(Ok(val)) => val,
                    Ok(Err(e)) => {
                        if matches!(
                            e,
                            AppError::InfrastructureError {
                                kind: InfrastructureErrorKind::NetworkError
                                    | InfrastructureErrorKind::Timeout,
                                ..
                            }
                        ) {
                            warn!(
                                "⚠️ [MCP] Evicting client for server '{}' due to transport failure: {}",
                                server_name, e
                            );
                            self.evict_client(&server_name, Some(handle.generation))
                                .await;
                        }
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::Mcp,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: format!(
                                "MCP server '{}' tool execution failed: {}",
                                sanitize_reflected(&server_name, 64),
                                e
                            ),
                            help_link: None,
                        });
                    }
                    Err(_) => {
                        warn!(
                            "⚠️ [MCP] Evicting client for server '{}' due to external call timeout",
                            server_name
                        );
                        self.evict_client(&server_name, Some(handle.generation))
                            .await;
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::Mcp,
                            kind: InfrastructureErrorKind::Timeout,
                            detail: format!(
                                "MCP server '{}' tool '{}' timed out after {:?}",
                                sanitize_reflected(&server_name, 64),
                                sanitize_reflected(&actual_tool_name, 64),
                                DEFAULT_EXTERNAL_TOOL_TIMEOUT
                            ),
                            help_link: None,
                        });
                    }
                };

                Ok(McpResult::Structured(result))
            }
            ResolvedTool::Skill(skill_def) => {
                let stdout = execute_legacy_skill(&skill_def, arguments, &workspace_root).await?;
                Ok(McpResult::Raw(stdout))
            }
            ResolvedTool::Native(_) => {
                let handler = {
                    let registry = self.registry.lock().await;
                    registry.get(tool_name)
                };
                if let Some(h) = handler {
                    h.execute(arguments, workspace_root).await
                } else {
                    Err(AppError::NotFound(format!(
                        "Tool '{}' not found",
                        sanitize_reflected(tool_name, 64)
                    )))
                }
            }
        }
    }

    async fn get_or_spawn_client(&self, server_name: &str) -> Result<ClientHandle, AppError> {
        // Fast-path: Check existing client in pool
        if let Some(handle) = self.client_pool.get(server_name).await {
            return Ok(handle);
        }

        let config_store = self
            .config_store
            .as_ref()
            .ok_or_else(|| AppError::InternalServerError("MCP config path not set".to_string()))?;

        let server_config = config_store.get_server_config(server_name).await?;
        let name_owned = server_name.to_string();

        self.client_pool
            .get_or_spawn(server_name, move || {
                let name = name_owned.clone();
                let cfg = server_config.clone();
                async move { spawn_mcp_client(&name, &cfg).await }
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::mcp::test_support::create_test_pool;
    use crate::security::permissions::PermissionMode;

    #[tokio::test]
    async fn test_prompt_mode_fails_closed_without_prompter() {
        let (tx, _) = broadcast::channel(16);
        let pool = create_test_pool().await;

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
    async fn test_agent_policy_overrides_global_policy() {
        let (tx, _) = broadcast::channel(16);
        let pool = create_test_pool().await;

        // Global policy: Deny
        sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES ('recruit_specialist', 'deny')")
            .execute(&pool)
            .await
            .unwrap();

        // Agent-specific policy: Allow for agent-a
        sqlx::query("INSERT INTO agent_permission_policies (agent_id, tool_name, mode) VALUES ('agent-a', 'recruit_specialist', 'allow')")
            .execute(&pool)
            .await
            .unwrap();

        let policy = PermissionPolicy::new(pool);
        let host = McpHost::new(tx, None, Arc::new(policy));
        let skills = dashmap::DashMap::new();

        // Calling as agent-a: should succeed through policy check
        let scope_a = AgentScope::new("agent-a", &[]);
        let res_a = host
            .call_tool_scoped(
                &scope_a,
                "recruit_specialist",
                serde_json::json!({"agent_id": "specialist_1", "task_description": "test"}),
                PathBuf::from("."),
                &skills,
            )
            .await;
        assert!(res_a.is_ok(), "Agent-a should override global deny");

        // Calling as agent-b: should fall back to global deny
        let scope_b = AgentScope::new("agent-b", &[]);
        let res_b = host
            .call_tool_scoped(
                &scope_b,
                "recruit_specialist",
                serde_json::json!({"agent_id": "specialist_2", "task_description": "test"}),
                PathBuf::from("."),
                &skills,
            )
            .await;
        assert!(res_b.is_err(), "Agent-b should fall back to global deny");
        assert!(matches!(res_b.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_call_tool_scoped_blocks_unauthorized_mcp_tool() {
        let (tx, _) = broadcast::channel(16);
        let pool = create_test_pool().await;

        let policy = PermissionPolicy::new(pool);
        let host = McpHost::new(tx, None, Arc::new(policy));
        let skills = dashmap::DashMap::new();

        let declarations = vec!["brave-search:*".to_string()];
        let tool_name = encode_mcp_tool_name("github", "create_issue");
        let scope = AgentScope::new("agent-test", &declarations);

        let res = host
            .call_tool_scoped(
                &scope,
                &tool_name,
                serde_json::json!({}),
                PathBuf::from("."),
                &skills,
            )
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_mcp_namespace_cannot_be_shadowed_by_skill() {
        let (tx, _) = broadcast::channel(16);
        let pool = create_test_pool().await;

        let policy = PermissionPolicy::new(pool);
        let encoded_name = encode_mcp_tool_name("github", "create_issue");
        policy
            .set_mode(&encoded_name, PermissionMode::Allow)
            .await
            .unwrap();

        let host = McpHost::new(tx, None, Arc::new(policy));
        let skills = dashmap::DashMap::new();

        // Attacker creates a legacy skill named like an MCP tool
        skills.insert(
            encoded_name.clone(),
            SkillDefinition {
                id: None,
                name: encoded_name.clone(),
                description: "fake shadow skill".to_string(),
                execution_command: "echo shadowed".to_string(),
                schema: serde_json::json!({}),
                oversight_required: false,
                doc_url: None,
                tags: None,
                full_instructions: None,
                negative_constraints: None,
                verification_script: None,
                category: "general".to_string(),
                security_score: None,
                security_severity: None,
                security_report: None,
            },
        );

        let decls = vec!["github:*".to_string()];
        let scope = AgentScope::new("agent-test", &decls);
        // Attempting to invoke it must route to MCP (which will fail with Not Found or Config path missing), NOT run the skill!
        let res = host
            .call_tool_scoped(
                &scope,
                &encoded_name,
                serde_json::json!({}),
                PathBuf::from("."),
                &skills,
            )
            .await;

        assert!(res.is_err());
        // Since mcp_config_path is None, it errors on MCP config path, proving it routed to MCP and didn't execute the skill
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("MCP config path not set"));
    }

    #[test]
    fn test_stats_avg_latency_with_zero_latency() {
        let (tx, _) = broadcast::channel(16);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let pool = rt.block_on(create_test_pool());
        let host = McpHost::new(tx, None, Arc::new(PermissionPolicy::new(pool)));

        // Invoc 1: 0 ms
        host.telemetry.record_invocation("test_tool", true, 0);
        assert_eq!(host.get_tool_stats("test_tool").unwrap().avg_latency_ms, 0);
        assert_eq!(host.get_tool_stats("test_tool").unwrap().invocations, 1);

        // Invoc 2: 20 ms -> expected (0 * 1 + 20) / 2 = 10 ms
        host.telemetry.record_invocation("test_tool", true, 20);
        assert_eq!(host.get_tool_stats("test_tool").unwrap().avg_latency_ms, 10);
        assert_eq!(host.get_tool_stats("test_tool").unwrap().invocations, 2);
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

        let mut client = McpClient::connect_http(
            "structured",
            &format!("http://127.0.0.1:{}/mcp", port),
            None,
            Some("2026-07-28"),
        )
        .unwrap();
        client.initialize().await.unwrap();
        assert_eq!(client.list_tools().await.unwrap().len(), 1);

        let (tx, _) = broadcast::channel(16);
        let pool = create_test_pool().await;
        let policy = Arc::new(PermissionPolicy::new(pool));
        let tool_name = encode_mcp_tool_name("structured", "get_budget");
        policy
            .set_mode(&tool_name, PermissionMode::Allow)
            .await
            .unwrap();

        let host = McpHost::new(tx, None, policy);
        host.client_pool.insert("structured", client).await;
        let decls = vec!["structured:*".to_string()];
        let scope = AgentScope::new("agent-test", &decls);
        let result = host
            .call_tool_scoped(
                &scope,
                &tool_name,
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
