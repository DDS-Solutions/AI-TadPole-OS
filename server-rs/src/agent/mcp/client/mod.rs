//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Client Facade
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Unified client facade delegating seamlessly across stdio subprocesses and remote HTTP endpoints.
//! - `[Structural]` Spec-conformance for MCP 2026-07-28 and backward compatibility with 2024-11-05.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InfrastructureError`, `AppError::BadRequest`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `client::tests::*`

pub mod adaptive;
pub mod http;
pub mod jsonrpc;
#[cfg(test)]
pub mod port3000_conformance;
pub mod stdio;

pub use adaptive::{AdaptiveMcpClient, AdaptiveState};
pub use http::McpHttpClient;
pub use jsonrpc::{
    make_meta, make_tool_call_meta, JsonRpcError, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, JSONRPC_HEADER_MISMATCH, JSONRPC_INTERNAL_ERROR, JSONRPC_INVALID_PARAMS,
    JSONRPC_INVALID_REQUEST, JSONRPC_METHOD_NOT_FOUND, JSONRPC_UNSUPPORTED_PROTOCOL_VERSION,
};
pub use stdio::McpStdioClient;

use crate::error::AppError;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

pub const DEFAULT_MCP_CALL_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_MCP_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(15);
pub const MAX_CONSECUTIVE_SKIPPED_LINES: usize = 50;
pub const MAX_LINE_LENGTH_BYTES: usize = 4 * 1024 * 1024; // 4 MB safety bound

pub const MCP_PROTOCOL_2026_07_28: &str = "2026-07-28";
pub const MCP_PROTOCOL_2024_11_05: &str = "2024-11-05";

// ---------------------------------------------------------------------------
// Unified McpClient enum facade
// ---------------------------------------------------------------------------

pub enum McpClient {
    Stdio(McpStdioClient),
    Http(McpHttpClient),
    Adaptive(Box<AdaptiveMcpClient>),
}

impl std::fmt::Debug for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdio(s) => s.fmt(f),
            Self::Http(h) => h.fmt(f),
            Self::Adaptive(a) => a.fmt(f),
        }
    }
}

impl McpClient {
    pub async fn spawn(
        server_name: &str,
        program: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
    ) -> Result<Self, AppError> {
        let stdio = McpStdioClient::spawn(server_name, program, args, env).await?;
        Ok(Self::Stdio(stdio))
    }

    pub async fn spawn_stdio(
        server_name: &str,
        program: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
    ) -> Result<Self, AppError> {
        Self::spawn(server_name, program, args, env).await
    }

    pub async fn spawn_stdio_with_cwd(
        server_name: &str,
        program: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
        cwd: Option<&str>,
    ) -> Result<Self, AppError> {
        let stdio = McpStdioClient::spawn_with_cwd(server_name, program, args, env, cwd).await?;
        Ok(Self::Stdio(stdio))
    }

    pub fn connect_http(
        server_name: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        protocol_version: Option<&str>,
    ) -> Result<Self, AppError> {
        let http = McpHttpClient::new(server_name, url, headers, protocol_version)?;
        Ok(Self::Http(http))
    }

    pub fn connect_adaptive(
        server_name: &str,
        http_config: crate::agent::mcp::config::McpHttpConfig,
        stdio_fallback: Option<crate::agent::mcp::config::McpStdioConfig>,
    ) -> Result<Self, AppError> {
        let adaptive = AdaptiveMcpClient::new(server_name, http_config, stdio_fallback)?;
        Ok(Self::Adaptive(Box::new(adaptive)))
    }

    pub async fn initialize(&mut self) -> Result<(), AppError> {
        match self {
            Self::Stdio(client) => client.initialize().await,
            Self::Http(client) => client.initialize().await,
            Self::Adaptive(client) => client.initialize().await,
        }
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Value>, AppError> {
        match self {
            Self::Stdio(client) => client.list_tools().await,
            Self::Http(client) => client.list_tools().await,
            Self::Adaptive(client) => client.list_tools().await,
        }
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, AppError> {
        match self {
            Self::Stdio(client) => client.call_tool(name, arguments).await,
            Self::Http(client) => client.call_tool(name, arguments).await,
            Self::Adaptive(client) => client.call_tool(name, arguments).await,
        }
    }

    pub async fn call_tool_with_operation_id(
        &mut self,
        name: &str,
        arguments: Value,
        operation_id: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Stdio(client) => {
                client
                    .call_tool_with_operation_id(name, arguments, operation_id)
                    .await
            }
            Self::Http(client) => {
                client
                    .call_tool_with_operation_id(name, arguments, operation_id)
                    .await
            }
            Self::Adaptive(client) => {
                client
                    .call_tool_with_operation_id(name, arguments, operation_id)
                    .await
            }
        }
    }

    pub async fn shutdown(&mut self) -> Result<(), AppError> {
        match self {
            Self::Stdio(client) => client.shutdown().await,
            Self::Http(client) => client.shutdown().await,
            Self::Adaptive(client) => client.shutdown().await,
        }
    }

    pub fn server_name(&self) -> &str {
        match self {
            Self::Stdio(client) => &client.server_name,
            Self::Http(client) => &client.server_name,
            Self::Adaptive(client) => client.server_name(),
        }
    }

    pub fn protocol_version(&self) -> Option<&str> {
        match self {
            Self::Stdio(client) => client.protocol_version.as_deref(),
            Self::Http(client) => client.protocol_version.as_deref(),
            Self::Adaptive(client) => client.protocol_version(),
        }
    }

    pub fn capabilities(&self) -> Option<&Value> {
        match self {
            Self::Stdio(client) => client.capabilities.as_ref(),
            Self::Http(client) => client.capabilities.as_ref(),
            Self::Adaptive(client) => client.capabilities(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_enum_delegation() {
        let client = McpClient::connect_http(
            "cloud-agent",
            "https://agent.example.com/mcp",
            None,
            Some("2026-07-28"),
        )
        .unwrap();

        assert_eq!(client.server_name(), "cloud-agent");
        assert_eq!(client.protocol_version(), Some("2026-07-28"));
    }
}
