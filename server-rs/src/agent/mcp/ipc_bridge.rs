//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / MCP / IPC Bridge
//! - **Primary Entrypoints**: `IpcBridge`, `JsonRpcRequest`, `JsonRpcResponse`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Zero external dependencies — uses only tokio + serde_json.
//! - `[Structural]` Named pipe path is deterministic and scoped to the workspace.
//! - `[Structural]` Framed JSON-RPC 2.0 over newline-delimited JSON (NDJSON).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `IPC_BRIDGE_001` (bind failure), `IPC_BRIDGE_002` (frame parse error)
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_jsonrpc_request_parsing`, `test_dispatch_ping`

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Notify;

/// JSON-RPC 2.0 Request (subset).
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 Response.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// IPC Bridge server — exposes tool registry to local Python scripts via Named Pipe (Windows)
/// or Unix Domain Socket (Linux/macOS) using newline-delimited JSON-RPC 2.0.
pub struct IpcBridge {
    pipe_path: PathBuf,
    tool_registry: Arc<crate::agent::runner::tools::registry::ToolRegistry>,
    shutdown: Arc<Notify>,
}

impl IpcBridge {
    /// Deterministic pipe path: `\\.\pipe\tadpoleos-ipc-{workspace_hash}` (Windows)
    /// or `/tmp/tadpoleos-ipc-{workspace_hash}.sock` (Unix).
    pub fn pipe_path_for(workspace_root: &std::path::Path) -> PathBuf {
        let hash = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            workspace_root.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        };

        #[cfg(windows)]
        {
            PathBuf::from(format!(r"\\.\pipe\tadpoleos-ipc-{}", hash))
        }

        #[cfg(not(windows))]
        {
            PathBuf::from(format!("/tmp/tadpoleos-ipc-{}.sock", hash))
        }
    }

    pub fn new(
        workspace_root: &std::path::Path,
        tool_registry: Arc<crate::agent::runner::tools::registry::ToolRegistry>,
    ) -> Self {
        Self {
            pipe_path: Self::pipe_path_for(workspace_root),
            tool_registry,
            shutdown: Arc::new(Notify::new()),
        }
    }

    /// Returns the pipe path for external callers to connect to.
    pub fn path(&self) -> &std::path::Path {
        &self.pipe_path
    }

    /// Starts the IPC listener. Returns a JoinHandle for the accept loop.
    /// The caller should hold the `IpcBridge` instance and call `shutdown()` to stop.
    pub fn start(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let bridge = Arc::clone(self);
        tokio::spawn(async move {
            if let Err(e) = bridge.run_accept_loop().await {
                tracing::error!(target: "ipc_bridge", "[IPC_BRIDGE_001] Accept loop error: {}", e);
            }
        })
    }

    /// Signal the accept loop to stop.
    pub fn shutdown(&self) {
        self.shutdown.notify_waiters();
    }

    #[cfg(windows)]
    async fn run_accept_loop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::net::windows::named_pipe::{PipeMode, ServerOptions};

        tracing::info!(target: "ipc_bridge", "🔌 IPC Bridge listening on: {}", self.pipe_path.display());

        loop {
            // Create a new pipe instance for each client
            let pipe = ServerOptions::new()
                .first_pipe_instance(false)
                .pipe_mode(PipeMode::Byte)
                .create(&self.pipe_path)?;

            // Wait for a client or shutdown signal
            tokio::select! {
                result = pipe.connect() => {
                    match result {
                        Ok(()) => {
                            let registry = Arc::clone(&self.tool_registry);
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(pipe, registry).await {
                                    tracing::warn!(target: "ipc_bridge", "Client connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!(target: "ipc_bridge", "Pipe connect error: {}", e);
                        }
                    }
                }
                _ = self.shutdown.notified() => {
                    tracing::info!(target: "ipc_bridge", "IPC Bridge shutting down");
                    break;
                }
            }
        }
        Ok(())
    }

    #[cfg(not(windows))]
    async fn run_accept_loop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::net::UnixListener;

        // Clean up stale socket file
        let _ = std::fs::remove_file(&self.pipe_path);

        let listener = UnixListener::bind(&self.pipe_path)?;
        tracing::info!(target: "ipc_bridge", "🔌 IPC Bridge listening on: {}", self.pipe_path.display());

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let registry = Arc::clone(&self.tool_registry);
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(stream, registry).await {
                                    tracing::warn!(target: "ipc_bridge", "Client connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!(target: "ipc_bridge", "Accept error: {}", e);
                        }
                    }
                }
                _ = self.shutdown.notified() => {
                    tracing::info!(target: "ipc_bridge", "IPC Bridge shutting down");
                    let _ = std::fs::remove_file(&self.pipe_path);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Handle a single client connection — reads NDJSON lines, dispatches, writes responses.
    async fn handle_connection<S>(
        stream: S,
        registry: Arc<crate::agent::runner::tools::registry::ToolRegistry>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let (reader, mut writer) = tokio::io::split(stream);
        let mut lines = BufReader::new(reader).lines();

        while let Some(line) = lines.next_line().await? {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(req) => Self::dispatch(&req, &registry).await,
                Err(e) => {
                    tracing::debug!(target: "ipc_bridge", "[IPC_BRIDGE_002] Parse error: {}", e);
                    JsonRpcResponse::error(
                        serde_json::Value::Null,
                        -32700,
                        format!("Parse error: {}", e),
                    )
                }
            };

            let mut out = serde_json::to_string(&response)?;
            out.push('\n');
            writer.write_all(out.as_bytes()).await?;
            writer.flush().await?;
        }

        Ok(())
    }

    /// Dispatch a JSON-RPC request to the appropriate handler.
    async fn dispatch(
        req: &JsonRpcRequest,
        registry: &crate::agent::runner::tools::registry::ToolRegistry,
    ) -> JsonRpcResponse {
        match req.method.as_str() {
            "list_tools" => {
                let tools: Vec<serde_json::Value> = registry
                    .list_tools()
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "description": t.description,
                            "is_mutating": t.is_mutating,
                            "is_dangerous": t.is_dangerous,
                            "is_cacheable": t.is_cacheable,
                        })
                    })
                    .collect();

                JsonRpcResponse::success(req.id.clone(), serde_json::json!(tools))
            }

            "get_tool_schema" => {
                let tool_name = req
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                match registry.get(tool_name) {
                    Some(tool) => {
                        let meta = tool.metadata();
                        JsonRpcResponse::success(
                            req.id.clone(),
                            serde_json::json!({
                                "name": meta.name,
                                "description": meta.description,
                                "parameters": meta.parameters,
                                "is_mutating": meta.is_mutating,
                                "is_dangerous": meta.is_dangerous,
                            }),
                        )
                    }
                    None => JsonRpcResponse::error(
                        req.id.clone(),
                        -32601,
                        format!("Tool '{}' not found", tool_name),
                    ),
                }
            }

            "ping" => JsonRpcResponse::success(req.id.clone(), serde_json::json!("pong")),

            _ => JsonRpcResponse::error(
                req.id.clone(),
                -32601,
                format!(
                    "Method '{}' not found. Available: list_tools, get_tool_schema, ping",
                    req.method
                ),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_path_deterministic() {
        let p1 = IpcBridge::pipe_path_for(std::path::Path::new("/workspace/project"));
        let p2 = IpcBridge::pipe_path_for(std::path::Path::new("/workspace/project"));
        assert_eq!(p1, p2, "Same workspace root should produce same pipe path");
    }

    #[test]
    fn test_pipe_path_different_workspaces() {
        let p1 = IpcBridge::pipe_path_for(std::path::Path::new("/workspace/project-a"));
        let p2 = IpcBridge::pipe_path_for(std::path::Path::new("/workspace/project-b"));
        assert_ne!(
            p1, p2,
            "Different workspace roots should produce different pipe paths"
        );
    }

    #[test]
    fn test_jsonrpc_response_success_serialization() {
        let resp = JsonRpcResponse::success(
            serde_json::json!(1),
            serde_json::json!({"tools": ["read_file"]}),
        );
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_jsonrpc_response_error_serialization() {
        let resp =
            JsonRpcResponse::error(serde_json::json!(2), -32601, "Method not found".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
        assert!(!json.contains("\"result\""));
    }

    #[test]
    fn test_jsonrpc_request_parsing() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}"#;
        let req: JsonRpcRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.method, "list_tools");
        assert_eq!(req.id, serde_json::json!(1));
    }

    #[test]
    fn test_jsonrpc_request_no_params() {
        let raw = r#"{"jsonrpc":"2.0","id":"abc","method":"ping"}"#;
        let req: JsonRpcRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.method, "ping");
        assert!(req.params.is_null());
    }

    #[tokio::test]
    async fn test_dispatch_ping() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(42),
            method: "ping".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap(), serde_json::json!("pong"));
    }

    #[tokio::test]
    async fn test_dispatch_list_tools_empty_registry() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "list_tools".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry).await;
        assert!(resp.error.is_none());
        let tools = resp.result.unwrap();
        assert!(tools.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_dispatch_unknown_method() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(3),
            method: "nonexistent_method".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_dispatch_get_tool_schema_not_found() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(4),
            method: "get_tool_schema".to_string(),
            params: serde_json::json!({"name": "nonexistent_tool"}),
        };
        let resp = IpcBridge::dispatch(&req, &registry).await;
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().message.contains("not found"));
    }
}
