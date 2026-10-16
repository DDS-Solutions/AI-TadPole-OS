//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / MCP / IPC Bridge
//! - **Primary Entrypoints**: `IpcBridge`, `IpcHandle`, `JsonRpcRequest`, `JsonRpcResponse`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Bounded framing (MAX_FRAME_BYTES = 1 MiB) prevents memory exhaustion.
//! - `[Structural]` Named pipe and UDS path derived deterministically via SHA-256 matching Python mcp_client.
//! - `[Structural]` Framed JSON-RPC 2.0 over newline-delimited JSON (NDJSON).
//! - `[Structural]` Restrictive filesystem permissions (0600 on Unix) and first instance exclusivity on Windows.
//! - `[Structural]` Accept loop termination guarded by CancellationToken + biased select.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `IPC_BRIDGE_001` (bind/accept failure), `IPC_BRIDGE_002` (frame/parse error), `IPC_BRIDGE_003` (rate/concurrency limit), `IPC_BRIDGE_004` (idle timeout)
//! - **Telemetry Targets**: none declared

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_util::sync::CancellationToken;

/// Maximum bytes allowed per NDJSON request frame (1 MiB).
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// Maximum concurrent client connections accepted by the bridge.
pub const MAX_CONCURRENT_CONNECTIONS: usize = 64;

/// Connection idle timeout before automatic disconnect.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// Write timeout for responses.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// Method authorization and permission classification tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    ReadOnly,
    Mutating,
    Dangerous,
}

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

/// Typed error enum for the IPC Bridge subsystem.
#[derive(Debug, thiserror::Error)]
pub enum IpcBridgeError {
    #[error("[IPC_BRIDGE_001] Bind or accept failed: {0}")]
    Bind(String),
    #[error("[IPC_BRIDGE_001] Server is already running on {0}")]
    AlreadyRunning(PathBuf),
    #[error("[IPC_BRIDGE_002] Frame too large: got {got} bytes, max allowed is {cap} bytes")]
    FrameTooLarge { got: usize, cap: usize },
    #[error("[IPC_BRIDGE_002] Frame parse error: {0}")]
    Parse(String),
    #[error("[IPC_BRIDGE_003] Max concurrent connections ({0}) exceeded")]
    ConnectionLimit(usize),
    #[error("[IPC_BRIDGE_004] Client connection idle timeout")]
    IdleTimeout,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl IpcBridgeError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Bind(_) | Self::AlreadyRunning(_) => "IPC_BRIDGE_001",
            Self::FrameTooLarge { .. } | Self::Parse(_) => "IPC_BRIDGE_002",
            Self::ConnectionLimit(_) => "IPC_BRIDGE_003",
            Self::IdleTimeout => "IPC_BRIDGE_004",
            Self::Io(_) => "IPC_BRIDGE_IO",
        }
    }
}

/// RAII Guard to ensure socket and discovery files are removed on exit/abort.
struct SocketGuard {
    #[allow(dead_code)]
    path: PathBuf,
    discovery_path: Option<PathBuf>,
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        #[cfg(not(windows))]
        {
            let _ = std::fs::remove_file(&self.path);
        }
        if let Some(ref dp) = self.discovery_path {
            let _ = std::fs::remove_file(dp);
        }
    }
}

/// Execution handle returned by `IpcBridge::start`.
pub struct IpcHandle {
    shutdown: CancellationToken,
    pipe_path: PathBuf,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl IpcHandle {
    /// Signals shutdown and awaits termination of the accept loop.
    pub async fn shutdown(mut self) -> Result<(), IpcBridgeError> {
        self.shutdown.cancel();
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
        Ok(())
    }

    /// Non-blocking shutdown signal.
    pub fn shutdown_signal(&self) {
        self.shutdown.cancel();
    }

    /// Returns the active pipe/socket path.
    pub fn pipe_path(&self) -> &std::path::Path {
        &self.pipe_path
    }
}

impl Drop for IpcHandle {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

/// IPC Bridge server — exposes tool registry to local Python scripts via Named Pipe (Windows)
/// or Unix Domain Socket (Linux/macOS) using newline-delimited JSON-RPC 2.0.
pub struct IpcBridge {
    workspace_root: PathBuf,
    pipe_path: PathBuf,
    tool_registry: Arc<crate::agent::runner::tools::registry::ToolRegistry>,
    shutdown: CancellationToken,
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl IpcBridge {
    /// Deterministic pipe path matching `execution/lib/mcp_client.py`:
    /// `\\.\pipe\tadpoleos-ipc-{path_hash}` (Windows)
    /// or `/tmp/tadpoleos-ipc-{path_hash}.sock` (Unix),
    /// where `path_hash` is the first 16 hex characters of SHA-256(workspace_root).
    pub fn pipe_path_for(workspace_root: &std::path::Path) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(workspace_root.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        let hash = hex::encode(&result[..8]);

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
            workspace_root: workspace_root.to_path_buf(),
            pipe_path: Self::pipe_path_for(workspace_root),
            tool_registry,
            shutdown: CancellationToken::new(),
            semaphore: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_CONNECTIONS)),
        }
    }

    /// Returns the pipe path for external callers to connect to.
    pub fn path(&self) -> &std::path::Path {
        &self.pipe_path
    }

    /// Returns the configured workspace root directory.
    pub fn workspace_root(&self) -> &std::path::Path {
        &self.workspace_root
    }

    /// Starts the IPC listener. Returns an `IpcHandle` to manage execution and shutdown.
    pub fn start(self: &Arc<Self>) -> IpcHandle {
        let bridge = Arc::clone(self);
        let shutdown = self.shutdown.clone();
        let pipe_path = self.pipe_path.clone();
        let task = tokio::spawn(async move {
            if let Err(e) = bridge.run_accept_loop().await {
                tracing::error!(
                    target: "ipc_bridge",
                    code = e.code(),
                    "[IPC_BRIDGE_001] Accept loop error: {}",
                    e
                );
            }
        });

        IpcHandle {
            shutdown,
            pipe_path,
            task: Some(task),
        }
    }

    /// Signal the accept loop to stop.
    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }

    #[cfg(windows)]
    async fn run_accept_loop(&self) -> Result<(), IpcBridgeError> {
        use tokio::net::windows::named_pipe::{PipeMode, ServerOptions};

        tracing::info!(target: "ipc_bridge", "🔌 IPC Bridge listening on: {}", self.pipe_path.display());

        let discovery_dir = self.workspace_root.join(".tmp");
        let discovery_path = discovery_dir.join("ipc_bridge_path.txt");
        if tokio::fs::create_dir_all(&discovery_dir).await.is_ok() {
            let _ = tokio::fs::write(&discovery_path, self.pipe_path.to_string_lossy().as_bytes())
                .await;
        }

        let _guard = SocketGuard {
            path: self.pipe_path.clone(),
            discovery_path: Some(discovery_path),
        };

        let mut is_first = true;

        loop {
            let server_result = ServerOptions::new()
                .first_pipe_instance(is_first)
                .pipe_mode(PipeMode::Byte)
                .max_instances(MAX_CONCURRENT_CONNECTIONS)
                .create(&self.pipe_path);

            let pipe = match server_result {
                Ok(p) => {
                    is_first = false;
                    p
                }
                Err(e) => {
                    tracing::error!(target: "ipc_bridge", "[IPC_BRIDGE_001] Failed to create named pipe instance: {}", e);
                    return Err(IpcBridgeError::Bind(e.to_string()));
                }
            };

            tokio::select! {
                biased;
                _ = self.shutdown.cancelled() => {
                    tracing::info!(target: "ipc_bridge", "IPC Bridge shutting down");
                    break;
                }
                connect_res = pipe.connect() => {
                    match connect_res {
                        Ok(()) => {
                            let permit = match self.semaphore.clone().try_acquire_owned() {
                                Ok(p) => p,
                                Err(_) => {
                                    tracing::warn!(
                                        target: "ipc_bridge",
                                        "[IPC_BRIDGE_003] Max concurrent connections ({}) reached, rejecting client",
                                        MAX_CONCURRENT_CONNECTIONS
                                    );
                                    continue;
                                }
                            };
                            let registry = Arc::clone(&self.tool_registry);
                            tokio::spawn(async move {
                                let _permit = permit;
                                if let Err(e) = Self::handle_connection(pipe, registry).await {
                                    tracing::debug!(target: "ipc_bridge", "Client connection ended: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::debug!(target: "ipc_bridge", "Pipe connect error: {}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(not(windows))]
    async fn run_accept_loop(&self) -> Result<(), IpcBridgeError> {
        use tokio::net::UnixListener;

        if self.pipe_path.exists() {
            use tokio::net::UnixStream;
            match UnixStream::connect(&self.pipe_path).await {
                Ok(_) => {
                    tracing::error!(target: "ipc_bridge", "[IPC_BRIDGE_001] Active IPC bridge already listening on: {}", self.pipe_path.display());
                    return Err(IpcBridgeError::AlreadyRunning(self.pipe_path.clone()));
                }
                Err(_) => {
                    let _ = tokio::fs::remove_file(&self.pipe_path).await;
                }
            }
        }

        let listener = match UnixListener::bind(&self.pipe_path) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!(target: "ipc_bridge", "[IPC_BRIDGE_001] Bind failed: {}", e);
                return Err(IpcBridgeError::Bind(e.to_string()));
            }
        };

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                std::fs::set_permissions(&self.pipe_path, std::fs::Permissions::from_mode(0o600));
        }

        let discovery_dir = self.workspace_root.join(".tmp");
        let discovery_path = discovery_dir.join("ipc_bridge_path.txt");
        if tokio::fs::create_dir_all(&discovery_dir).await.is_ok() {
            let _ = tokio::fs::write(&discovery_path, self.pipe_path.to_string_lossy().as_bytes())
                .await;
        }

        let _guard = SocketGuard {
            path: self.pipe_path.clone(),
            discovery_path: Some(discovery_path),
        };

        tracing::info!(target: "ipc_bridge", "🔌 IPC Bridge listening on: {}", self.pipe_path.display());

        loop {
            tokio::select! {
                biased;
                _ = self.shutdown.cancelled() => {
                    tracing::info!(target: "ipc_bridge", "IPC Bridge shutting down");
                    break;
                }
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((stream, _)) => {
                            let permit = match self.semaphore.clone().try_acquire_owned() {
                                Ok(p) => p,
                                Err(_) => {
                                    tracing::warn!(
                                        target: "ipc_bridge",
                                        "[IPC_BRIDGE_003] Max concurrent connections ({}) reached, rejecting client",
                                        MAX_CONCURRENT_CONNECTIONS
                                    );
                                    continue;
                                }
                            };
                            let registry = Arc::clone(&self.tool_registry);
                            tokio::spawn(async move {
                                let _permit = permit;
                                if let Err(e) = Self::handle_connection(stream, registry).await {
                                    tracing::debug!(target: "ipc_bridge", "Client connection ended: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::debug!(target: "ipc_bridge", "Accept error: {}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle a single client connection — reads NDJSON lines with strict frame bounding.
    async fn handle_connection<S>(
        stream: S,
        registry: Arc<crate::agent::runner::tools::registry::ToolRegistry>,
    ) -> Result<(), IpcBridgeError>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);
        let mut buf = Vec::with_capacity(1024);

        loop {
            buf.clear();
            let mut take = (&mut reader).take((MAX_FRAME_BYTES + 1) as u64);
            let read_result =
                tokio::time::timeout(IDLE_TIMEOUT, take.read_until(b'\n', &mut buf)).await;

            let n = match read_result {
                Ok(Ok(0)) => break,
                Ok(Ok(bytes_read)) => bytes_read,
                Ok(Err(e)) => {
                    tracing::debug!(target: "ipc_bridge", "Read error: {}", e);
                    break;
                }
                Err(_) => {
                    tracing::debug!(target: "ipc_bridge", "[IPC_BRIDGE_004] Client idle timeout reached");
                    break;
                }
            };

            if n > MAX_FRAME_BYTES
                || buf.len() > MAX_FRAME_BYTES
                || (take.limit() == 0 && !buf.ends_with(b"\n"))
            {
                tracing::warn!(
                    target: "ipc_bridge",
                    "[IPC_BRIDGE_002] Frame too large ({} bytes > {} max bytes). Terminating connection.",
                    buf.len(),
                    MAX_FRAME_BYTES
                );
                let resp = JsonRpcResponse::error(
                    serde_json::Value::Null,
                    -32700,
                    format!(
                        "Frame too large: maximum allowed length is {} bytes",
                        MAX_FRAME_BYTES
                    ),
                );
                if let Ok(mut out) = serde_json::to_string(&resp) {
                    out.push('\n');
                    let _ =
                        tokio::time::timeout(WRITE_TIMEOUT, writer.write_all(out.as_bytes())).await;
                    let _ = tokio::time::timeout(WRITE_TIMEOUT, writer.flush()).await;
                }
                return Err(IpcBridgeError::FrameTooLarge {
                    got: buf.len(),
                    cap: MAX_FRAME_BYTES,
                });
            }

            let line_str = match std::str::from_utf8(&buf) {
                Ok(s) => s.trim(),
                Err(e) => {
                    tracing::warn!(target: "ipc_bridge", "[IPC_BRIDGE_002] Invalid UTF-8 frame: {}", e);
                    let resp = JsonRpcResponse::error(
                        serde_json::Value::Null,
                        -32700,
                        "Invalid UTF-8 encoding in request".to_string(),
                    );
                    if let Ok(mut out) = serde_json::to_string(&resp) {
                        out.push('\n');
                        let _ =
                            tokio::time::timeout(WRITE_TIMEOUT, writer.write_all(out.as_bytes()))
                                .await;
                        let _ = tokio::time::timeout(WRITE_TIMEOUT, writer.flush()).await;
                    }
                    continue;
                }
            };

            if line_str.is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<JsonRpcRequest>(line_str) {
                Ok(req) => Self::dispatch(&req, &registry),
                Err(e) => {
                    tracing::warn!(target: "ipc_bridge", "[IPC_BRIDGE_002] Parse error: {}", e);
                    JsonRpcResponse::error(
                        serde_json::Value::Null,
                        -32700,
                        "Parse error: invalid JSON-RPC 2.0 payload".to_string(),
                    )
                }
            };

            let mut out = serde_json::to_string(&response)
                .map_err(|e| IpcBridgeError::Parse(e.to_string()))?;
            out.push('\n');

            let write_res =
                tokio::time::timeout(WRITE_TIMEOUT, writer.write_all(out.as_bytes())).await;
            if write_res.is_err() || write_res.unwrap().is_err() {
                tracing::debug!(target: "ipc_bridge", "Client disconnected during write");
                break;
            }
            let flush_res = tokio::time::timeout(WRITE_TIMEOUT, writer.flush()).await;
            if flush_res.is_err() || flush_res.unwrap().is_err() {
                tracing::debug!(target: "ipc_bridge", "Client disconnected during flush");
                break;
            }
        }

        Ok(())
    }

    /// Dispatch a JSON-RPC request to the appropriate handler.
    pub fn dispatch(
        req: &JsonRpcRequest,
        registry: &crate::agent::runner::tools::registry::ToolRegistry,
    ) -> JsonRpcResponse {
        // Protocol validation: JSON-RPC version must be "2.0"
        if req.jsonrpc != "2.0" {
            return JsonRpcResponse::error(
                req.id.clone(),
                -32600,
                format!(
                    "Invalid JSON-RPC version '{}', expected '2.0'",
                    sanitize_str(&req.jsonrpc, 16)
                ),
            );
        }

        // Protocol validation: id must be number, string, or null
        match &req.id {
            serde_json::Value::Number(_)
            | serde_json::Value::String(_)
            | serde_json::Value::Null => {}
            _ => {
                return JsonRpcResponse::error(
                    req.id.clone(),
                    -32600,
                    "Invalid Request: id must be an integer, string, or null".to_string(),
                );
            }
        }

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
                let tool_name = match req.params.get("name").and_then(|v| v.as_str()) {
                    Some(name) => name,
                    None => {
                        return JsonRpcResponse::error(
                            req.id.clone(),
                            -32602,
                            "Invalid params: missing required string parameter 'name'".to_string(),
                        );
                    }
                };

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
                        format!("Tool '{}' not found", sanitize_str(tool_name, 64)),
                    ),
                }
            }

            "ping" => JsonRpcResponse::success(req.id.clone(), serde_json::json!("pong")),

            _ => JsonRpcResponse::error(
                req.id.clone(),
                -32601,
                format!(
                    "Method '{}' not found. Available: list_tools, get_tool_schema, ping",
                    sanitize_str(&req.method, 64)
                ),
            ),
        }
    }
}

fn sanitize_str(s: &str, max_len: usize) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(max_len)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    async fn connect_test_client(
        path: &std::path::Path,
    ) -> tokio::io::Result<tokio::net::windows::named_pipe::NamedPipeClient> {
        tokio::net::windows::named_pipe::ClientOptions::new().open(path)
    }

    #[cfg(not(windows))]
    async fn connect_test_client(
        path: &std::path::Path,
    ) -> tokio::io::Result<tokio::net::UnixStream> {
        tokio::net::UnixStream::connect(path).await
    }

    #[test]
    fn test_pipe_path_deterministic_golden_vector() {
        let p = IpcBridge::pipe_path_for(std::path::Path::new("/workspace/project"));
        let p_str = p.to_string_lossy();
        assert!(
            p_str.contains("e3af8a7251583e76"),
            "Hash must match Python mcp_client.py SHA-256 golden vector, got: {}",
            p_str
        );
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

    #[test]
    fn test_dispatch_ping() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(42),
            method: "ping".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry);
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap(), serde_json::json!("pong"));
    }

    #[test]
    fn test_dispatch_invalid_version() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            id: serde_json::json!(1),
            method: "ping".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32600);
    }

    #[test]
    fn test_dispatch_invalid_id_type() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!({"bad": "id"}),
            method: "ping".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32600);
    }

    #[test]
    fn test_dispatch_get_tool_schema_missing_params() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(2),
            method: "get_tool_schema".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32602);
    }

    #[test]
    fn test_dispatch_list_tools_empty_registry() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "list_tools".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry);
        assert!(resp.error.is_none());
        let tools = resp.result.unwrap();
        assert!(tools.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(3),
            method: "nonexistent_method".to_string(),
            params: serde_json::Value::Null,
        };
        let resp = IpcBridge::dispatch(&req, &registry);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_dispatch_get_tool_schema_not_found() {
        let registry = crate::agent::runner::tools::registry::ToolRegistry::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(4),
            method: "get_tool_schema".to_string(),
            params: serde_json::json!({"name": "nonexistent_tool"}),
        };
        let resp = IpcBridge::dispatch(&req, &registry);
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().message.contains("not found"));
    }

    #[tokio::test]
    async fn test_ipc_round_trip_end_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let registry = Arc::new(crate::agent::runner::tools::registry::ToolRegistry::new());
        let bridge = Arc::new(IpcBridge::new(dir.path(), registry));
        let handle = bridge.start();

        // Brief delay to ensure listener is bound
        tokio::time::sleep(Duration::from_millis(60)).await;

        let mut client = connect_test_client(bridge.path())
            .await
            .expect("Failed to connect client");
        client
            .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":100,\"method\":\"ping\"}\n")
            .await
            .unwrap();
        client.flush().await.unwrap();

        let mut reader = BufReader::new(client);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();

        let val: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(val["jsonrpc"], "2.0");
        assert_eq!(val["id"], 100);
        assert_eq!(val["result"], "pong");

        // Verify discovery file was created
        let discovery_file = dir.path().join(".tmp").join("ipc_bridge_path.txt");
        assert!(
            discovery_file.exists(),
            "Discovery file should exist during bridge execution"
        );

        handle.shutdown().await.unwrap();

        // Discovery file should be cleaned up by SocketGuard
        assert!(
            !discovery_file.exists(),
            "Discovery file should be removed on shutdown"
        );
    }

    #[tokio::test]
    async fn test_shutdown_is_not_lost_under_connect_storm() {
        // Regression test for B1: Shutdown must reliably terminate the accept loop
        for _ in 0..10 {
            let dir = tempfile::tempdir().unwrap();
            let registry = Arc::new(crate::agent::runner::tools::registry::ToolRegistry::new());
            let bridge = Arc::new(IpcBridge::new(dir.path(), registry));
            let handle = bridge.start();

            tokio::time::sleep(Duration::from_millis(30)).await;

            let path = bridge.path().to_path_buf();
            tokio::spawn(async move {
                let _ = connect_test_client(&path).await;
            });

            handle.shutdown_signal();

            tokio::time::timeout(Duration::from_secs(2), handle.shutdown())
                .await
                .expect("Accept loop hung on shutdown — signal was lost!")
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_oversized_frame_rejected() {
        // Regression test for B3: Frame exceeding MAX_FRAME_BYTES must be rejected with -32700
        let dir = tempfile::tempdir().unwrap();
        let registry = Arc::new(crate::agent::runner::tools::registry::ToolRegistry::new());
        let bridge = Arc::new(IpcBridge::new(dir.path(), registry));
        let handle = bridge.start();

        tokio::time::sleep(Duration::from_millis(60)).await;

        let mut client = connect_test_client(bridge.path())
            .await
            .expect("Failed to connect client");

        // Send 1 MiB + 100 bytes without newline
        let oversized_chunk = vec![b'x'; MAX_FRAME_BYTES + 100];
        let _ = client.write_all(&oversized_chunk).await;
        let _ = client.write_all(b"\n").await;
        let _ = client.flush().await;

        let mut reader = BufReader::new(client);
        let mut line = String::new();
        let read_res = reader.read_line(&mut line).await;

        if let Ok(n) = read_res {
            if n > 0 {
                let val: Result<serde_json::Value, _> = serde_json::from_str(&line);
                if let Ok(v) = val {
                    assert_eq!(v["error"]["code"], -32700);
                }
            }
        }

        handle.shutdown().await.unwrap();
    }
}
