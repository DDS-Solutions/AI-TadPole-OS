//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / client
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::{debug, info, warn};

pub const DEFAULT_MCP_CALL_TIMEOUT: Duration = Duration::from_secs(30);
pub const MAX_CONSECUTIVE_SKIPPED_LINES: usize = 50;

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

pub struct McpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    pub server_name: String,
    pub protocol_version: Option<String>,
    pub capabilities: Option<Value>,
}

impl McpClient {
    pub async fn spawn(
        server_name: &str,
        program: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
    ) -> Result<Self, AppError> {
        info!(
            "🚀 [client] [MCP] Spawning server '{}': {} with args: {:?}",
            server_name, program, args
        );

        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

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
            ">> [MCP] [{}] Sending: {}",
            self.server_name,
            request_str.trim()
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

            debug!(
                "<< [MCP] [{}] Received: {}",
                self.server_name,
                response_line.trim()
            );

            if let Ok(raw_val) = serde_json::from_str::<Value>(&response_line) {
                // If it is a notification (contains method and no id), skip it.
                if raw_val.get("method").is_some() && raw_val.get("id").is_none() {
                    debug!("<< [MCP] Skipping notification: {}", response_line.trim());
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

                        if let Some(error) = response.error {
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
                        debug!(
                            "<< [MCP] Mismatched JSON-RPC response ID (expected {:?}, got {:?}): {}",
                            target_id, resp_id, response_line.trim()
                        );
                        skipped_count += 1;
                    }
                }
            } else {
                debug!(
                    "<< [MCP] Malformed JSON-RPC message from '{}': {}",
                    self.server_name,
                    response_line.trim()
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

    pub async fn initialize(&mut self) -> Result<(), AppError> {
        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "tadpole-os-engine",
                "version": "1.1.0"
            }
        });

        let result = self.call("initialize", params).await?;

        // Inspect and store negotiated protocolVersion and capabilities
        if let Some(pv) = result.get("protocolVersion").and_then(|v| v.as_str()) {
            self.protocol_version = Some(pv.to_string());
            debug!(
                "[MCP] Negotiated protocol version with '{}': {}",
                self.server_name, pv
            );
        }
        if let Some(caps) = result.get("capabilities") {
            self.capabilities = Some(caps.clone());
        }

        // Notify initialized
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

    pub async fn list_tools(&mut self) -> Result<Vec<Value>, AppError> {
        let result = self.call("tools/list", json!({})).await?;
        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(tools)
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, AppError> {
        let params = json!({
            "name": name,
            "arguments": arguments
        });
        self.call("tools/call", params).await
    }
}
