//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Stdio Transport
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Bounded buffer execution, explicit process lifecycle, no unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `client::stdio::tests::*`

use super::jsonrpc::{
    make_meta, make_tool_call_meta, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    JSONRPC_METHOD_NOT_FOUND,
};
use super::{
    DEFAULT_MCP_CALL_TIMEOUT, MAX_CONSECUTIVE_SKIPPED_LINES, MAX_LINE_LENGTH_BYTES,
    MCP_PROTOCOL_2024_11_05, MCP_PROTOCOL_2026_07_28,
};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::{debug, info, warn};

pub struct McpStdioClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    pub server_name: String,
    pub protocol_version: Option<String>,
    pub capabilities: Option<Value>,
    pub last_raw_error: Option<JsonRpcError>,
    listed_tools: HashSet<String>,
    operation_bindings: HashMap<String, String>,
    retryable_operations: HashSet<String>,
}

impl std::fmt::Debug for McpStdioClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpStdioClient")
            .field("server_name", &self.server_name)
            .field("protocol_version", &self.protocol_version)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl McpStdioClient {
    pub async fn spawn(
        server_name: &str,
        program: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
    ) -> Result<Self, AppError> {
        Self::spawn_with_cwd(server_name, program, args, env, None).await
    }

    pub async fn spawn_with_cwd(
        server_name: &str,
        program: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
        cwd: Option<&str>,
    ) -> Result<Self, AppError> {
        info!(
            "🚀 [client] [MCP] Spawning server '{}': {} with args: {:?} (cwd={:?})",
            server_name, program, args, cwd
        );

        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        if let Some(env_vars) = env {
            cmd.envs(env_vars);
        }

        let mut child = cmd.spawn().map_err(|e| AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::NetworkError,
            detail: format!("Failed to spawn MCP child process '{}': {}", program, e),
            help_link: None,
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!("Failed to open stdin pipe for MCP server '{}'", server_name),
                help_link: None,
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!(
                    "Failed to open stdout pipe for MCP server '{}'",
                    server_name
                ),
                help_link: None,
            })?;

        // Pipe stderr and relay via tracing to avoid corrupting console or swallowing diagnostic output
        if let Some(stderr) = child.stderr.take() {
            let sname = server_name.to_string();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if !line.trim().is_empty() {
                        warn!(target: "mcp_stderr", server = %sname, "{}", line);
                    }
                }
            });
        }

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            server_name: server_name.to_string(),
            protocol_version: None,
            capabilities: None,
            last_raw_error: None,
            listed_tools: HashSet::new(),
            operation_bindings: HashMap::new(),
            retryable_operations: HashSet::new(),
        })
    }

    /// Explicit graceful shutdown: closes stdin, signals termination, and awaits exit
    pub async fn shutdown(&mut self) -> Result<(), AppError> {
        info!(
            "[MCP] Gracefully shutting down MCP server '{}'",
            self.server_name
        );
        let _ = self.stdin.shutdown().await;

        let wait_res = tokio::time::timeout(Duration::from_secs(5), self.child.wait()).await;
        match wait_res {
            Ok(Ok(status)) => {
                debug!(
                    "[MCP] Server '{}' exited with status {:?}",
                    self.server_name, status
                );
                Ok(())
            }
            Ok(Err(e)) => Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!("Error waiting on server exit: {}", e),
                help_link: None,
            }),
            Err(_) => {
                warn!(
                    "[MCP] Server '{}' did not exit in time; killing process",
                    self.server_name
                );
                let _ = self.child.kill().await;
                Ok(())
            }
        }
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, AppError> {
        let fut = self.call_internal(method, params);
        tokio::time::timeout(DEFAULT_MCP_CALL_TIMEOUT, fut)
            .await
            .map_err(|_| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::Timeout,
                detail: format!(
                    "MCP server '{}' timed out after {:?} awaiting response to method '{}'",
                    self.server_name, DEFAULT_MCP_CALL_TIMEOUT, method
                ),
                help_link: None,
            })?
    }

    async fn call_internal(&mut self, method: &str, params: Value) -> Result<Value, AppError> {
        let id = self.next_id;
        self.next_id += 1;
        self.last_raw_error = None;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: method.to_string(),
            params,
        };

        let request_str = serde_json::to_string(&request)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            + "\n";
        debug!(
            ">> [MCP] [{}] Sending method '{}'",
            self.server_name, method
        );

        self.stdin
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!(
                    "Failed to write to MCP stdin for '{}': {}",
                    self.server_name, e
                ),
                help_link: None,
            })?;

        self.stdin
            .flush()
            .await
            .map_err(|e| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!(
                    "Failed to flush MCP stdin for '{}': {}",
                    self.server_name, e
                ),
                help_link: None,
            })?;

        let target_id = json!(id);
        let mut skipped_count = 0;

        loop {
            let mut response_line = String::new();
            let bytes_read = self
                .stdout
                .read_line(&mut response_line)
                .await
                .map_err(|e| AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::NetworkError,
                    detail: format!(
                        "Failed to read stdout from MCP server '{}': {}",
                        self.server_name, e
                    ),
                    help_link: None,
                })?;

            if bytes_read == 0 || response_line.is_empty() {
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::NetworkError,
                    detail: format!(
                        "MCP server '{}' closed connection unexpectedly",
                        self.server_name
                    ),
                    help_link: None,
                });
            }

            // Guard against unbounded message buffers
            if response_line.len() > MAX_LINE_LENGTH_BYTES {
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!(
                        "MCP server '{}' exceeded maximum allowed message size ({} MB)",
                        self.server_name,
                        MAX_LINE_LENGTH_BYTES / 1024 / 1024
                    ),
                    help_link: None,
                });
            }

            // Harmless whitespace-only lines do not count against skip budget
            if response_line.trim().is_empty() {
                continue;
            }

            debug!("<< [MCP] [{}] Received JSON response", self.server_name);

            if let Ok(raw_val) = serde_json::from_str::<Value>(&response_line) {
                // Check if this is a server-to-client request (both method and id present)
                if raw_val.get("method").is_some() && raw_val.get("id").is_some() {
                    let req_id = raw_val.get("id").cloned().unwrap_or(Value::Null);
                    let req_method = raw_val
                        .get("method")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown");
                    debug!(
                        "<< [MCP] Server-to-client request '{}' with id={:?}; replying MethodNotFound",
                        req_method, req_id
                    );
                    let reply = json!({
                        "jsonrpc": "2.0",
                        "id": req_id,
                        "error": {
                            "code": JSONRPC_METHOD_NOT_FOUND,
                            "message": format!("Method '{}' not supported by client", req_method)
                        }
                    });
                    if let Ok(reply_str) = serde_json::to_string(&reply) {
                        let _ = self.stdin.write_all((reply_str + "\n").as_bytes()).await;
                        let _ = self.stdin.flush().await;
                    }
                    skipped_count += 1;
                    continue;
                }

                // If it is a notification (contains method and no id), skip it.
                if raw_val.get("method").is_some() && raw_val.get("id").is_none() {
                    debug!("<< [MCP] Skipping notification while awaiting response");
                    skipped_count += 1;
                    if skipped_count > MAX_CONSECUTIVE_SKIPPED_LINES {
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::Mcp,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: format!(
                                "Exceeded {} skipped notification lines while awaiting response id={}",
                                MAX_CONSECUTIVE_SKIPPED_LINES, id
                            ),
                            help_link: None,
                        });
                    }
                    continue;
                }

                if let Some(resp_id) = raw_val.get("id") {
                    if resp_id == &target_id {
                        let response: JsonRpcResponse = serde_json::from_value(raw_val)
                            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
                        response.validate_for(&target_id).map_err(|detail| {
                            AppError::InfrastructureError {
                                provider_id: ProviderId::Mcp,
                                kind: InfrastructureErrorKind::ApiError,
                                detail: format!(
                                    "Invalid JSON-RPC response from '{}': {}",
                                    self.server_name, detail
                                ),
                                help_link: None,
                            }
                        })?;

                        if let Some(error) = response.error {
                            self.last_raw_error = Some(error.clone());
                            return Err(AppError::InfrastructureError {
                                provider_id: ProviderId::Mcp,
                                kind: InfrastructureErrorKind::ApiError,
                                detail: format!("MCP Error from '{}': {}", self.server_name, error),
                                help_link: None,
                            });
                        }

                        return response
                            .result
                            .ok_or_else(|| AppError::InfrastructureError {
                                provider_id: ProviderId::Mcp,
                                kind: InfrastructureErrorKind::ApiError,
                                detail: format!(
                                    "Missing result in MCP response from '{}'",
                                    self.server_name
                                ),
                                help_link: None,
                            });
                    } else {
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::Mcp,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: format!(
                                "MCP server '{}' returned a mismatched JSON-RPC response id",
                                self.server_name
                            ),
                            help_link: None,
                        });
                    }
                }
            } else {
                debug!(
                    "<< [MCP] Malformed JSON-RPC message from '{}'",
                    self.server_name
                );
                skipped_count += 1;
            }

            if skipped_count > MAX_CONSECUTIVE_SKIPPED_LINES {
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!(
                        "Received {} consecutive malformed or non-matching lines from '{}' while awaiting response id={}",
                        skipped_count, self.server_name, id
                    ),
                    help_link: None,
                });
            }
        }
    }

    /// Spec-conformant stdio handshake:
    /// Per the 2026-07-28 backward-compatibility rules:
    /// 1. A client supporting both modern and legacy protocols sends `server/discover` first.
    /// 2. If `server/discover` succeeds, the server is modern (stateless / per-request `_meta`).
    /// 3. If `server/discover` returns MethodNotFound (-32601), fall back to legacy `initialize` handshake.
    pub async fn initialize(&mut self) -> Result<(), AppError> {
        let meta = make_meta(MCP_PROTOCOL_2026_07_28);
        let discover_params = json!({ "_meta": meta });

        match self.call("server/discover", discover_params).await {
            Ok(result) => {
                if result.get("resultType").and_then(Value::as_str) != Some("complete")
                    || !result
                        .get("supportedVersions")
                        .and_then(Value::as_array)
                        .is_some_and(|versions| {
                            versions
                                .iter()
                                .any(|version| version.as_str() == Some(MCP_PROTOCOL_2026_07_28))
                        })
                    || !result.get("capabilities").is_some_and(Value::is_object)
                {
                    return Err(AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "MCP stdio server '{}' returned invalid modern discovery",
                            self.server_name
                        ),
                        help_link: None,
                    });
                }
                self.protocol_version = Some(MCP_PROTOCOL_2026_07_28.to_string());
                self.capabilities = result.get("capabilities").cloned();
                debug!(
                    "[MCP] Modern 2026-07-28 stdio server '{}' discovered with protocol version '{}'",
                    self.server_name, MCP_PROTOCOL_2026_07_28
                );
                Ok(())
            }
            Err(e) => {
                if self.last_raw_error.as_ref().map(|error| error.code)
                    != Some(JSONRPC_METHOD_NOT_FOUND)
                {
                    return Err(e);
                }
                debug!(
                    "[MCP] Server '{}' returned MethodNotFound for server/discover; using legacy stdio initialize",
                    self.server_name
                );

                // Legacy fallback: Send initialize @ 2024-11-05
                let init_params = json!({
                    "protocolVersion": MCP_PROTOCOL_2024_11_05,
                    "capabilities": {},
                    "clientInfo": {
                        "name": super::jsonrpc::MCP_CLIENT_NAME,
                        "version": super::jsonrpc::MCP_CLIENT_VERSION
                    }
                });

                let result = self.call("initialize", init_params).await?;
                let negotiated = result
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "Legacy MCP server '{}' omitted protocolVersion",
                            self.server_name
                        ),
                        help_link: None,
                    })?;
                if negotiated != MCP_PROTOCOL_2024_11_05 {
                    return Err(AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "Legacy MCP server '{}' selected unsupported protocol '{}'",
                            self.server_name, negotiated
                        ),
                        help_link: None,
                    });
                }

                self.protocol_version = Some(negotiated.to_string());
                if let Some(caps) = result.get("capabilities") {
                    self.capabilities = Some(caps.clone());
                }

                // Send initialized notification per 2024-11-05 handshake requirement
                let notification = json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/initialized"
                });
                let notif_str = serde_json::to_string(&notification)
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?
                    + "\n";

                self.stdin
                    .write_all(notif_str.as_bytes())
                    .await
                    .map_err(|e| AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::NetworkError,
                        detail: format!(
                            "Failed to send initialized notification to '{}': {}",
                            self.server_name, e
                        ),
                        help_link: None,
                    })?;

                self.stdin
                    .flush()
                    .await
                    .map_err(|e| AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::NetworkError,
                        detail: format!(
                            "Failed to flush initialized notification for '{}': {}",
                            self.server_name, e
                        ),
                        help_link: None,
                    })?;

                Ok(())
            }
        }
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Value>, AppError> {
        let is_modern = self.protocol_version.as_deref() == Some(MCP_PROTOCOL_2026_07_28);

        let params = if is_modern {
            let proto = self
                .protocol_version
                .as_deref()
                .unwrap_or(MCP_PROTOCOL_2026_07_28);
            json!({ "_meta": make_meta(proto) })
        } else {
            json!({})
        };

        let result = self.call("tools/list", params).await?;
        if is_modern && result.get("resultType").and_then(Value::as_str) != Some("complete") {
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP stdio server '{}' returned invalid tools/list resultType",
                    self.server_name
                ),
                help_link: None,
            });
        }
        let tools = result
            .get("tools")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP stdio server '{}' omitted tools array",
                    self.server_name
                ),
                help_link: None,
            })?;
        self.listed_tools = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .map(str::to_string)
            .collect();
        Ok(tools)
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, AppError> {
        let operation_id = uuid::Uuid::new_v4().to_string();
        self.call_tool_with_operation_id(name, arguments, &operation_id)
            .await
    }

    pub async fn call_tool_with_operation_id(
        &mut self,
        name: &str,
        arguments: Value,
        operation_id: &str,
    ) -> Result<Value, AppError> {
        if !self.listed_tools.contains(name) {
            return Err(AppError::Forbidden(format!(
                "MCP tool '{}' is absent from the last successful tools/list catalog",
                name
            )));
        }
        uuid::Uuid::parse_str(operation_id)
            .map_err(|_| AppError::BadRequest("MCP operation_id must be a UUID".to_string()))?;
        let binding = format!("{}:{}", name, arguments);
        if let Some(existing) = self.operation_bindings.get(operation_id) {
            if existing != &binding {
                return Err(AppError::Conflict(
                    "MCP operation_id cannot be reused with different tool arguments".to_string(),
                ));
            }
            if !self.retryable_operations.remove(operation_id) {
                return Err(AppError::Conflict(
                    "MCP operation_id may be retried only after an explicit retryable outcome"
                        .to_string(),
                ));
            }
        } else {
            self.operation_bindings
                .insert(operation_id.to_string(), binding);
        }

        let is_modern = self.protocol_version.as_deref() == Some(MCP_PROTOCOL_2026_07_28);

        let mut params = json!({
            "name": name,
            "arguments": arguments
        });

        let proto = if is_modern {
            MCP_PROTOCOL_2026_07_28
        } else {
            MCP_PROTOCOL_2024_11_05
        };
        params["_meta"] = make_tool_call_meta(proto, Some(operation_id));

        let call_result = self.call("tools/call", params).await;
        if self
            .last_raw_error
            .as_ref()
            .and_then(|error| error.data.as_ref())
            .and_then(|data| data.get("retryable"))
            .and_then(Value::as_bool)
            == Some(true)
        {
            self.retryable_operations.insert(operation_id.to_string());
        }
        let result = call_result?;
        if result
            .pointer("/_meta/execution/retryable")
            .and_then(Value::as_bool)
            == Some(true)
        {
            self.retryable_operations.insert(operation_id.to_string());
        }
        Ok(result)
    }
}
