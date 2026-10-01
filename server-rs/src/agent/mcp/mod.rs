//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Module Root
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Thin module root re-exporting public symbols preserving full backward compatibility.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `mcp::*`

pub mod authz;
pub mod client;
pub mod client_pool;
pub mod config;
pub mod config_store;
pub mod context;
pub mod exec;
pub mod gate;
pub mod host;
pub mod ipc_bridge;
pub mod native;
pub mod registry;
pub mod resolve;
pub mod telemetry;
#[cfg(test)]
pub mod test_support;
pub mod transport;
pub mod types;

// Re-export core items for backward compatibility across server-rs and macros
pub use authz::{
    decode_mcp_tool_name, encode_mcp_tool_name, is_mcp_server_authorized, is_mcp_tool_authorized,
};
pub use client::McpClient;
pub use client_pool::{ClientHandle, ClientPool};
pub use config::{
    is_valid_environment_name, resolve_mcp_environment, resolve_mcp_headers,
    validate_mcp_server_config, McpConfig, McpServerConfig,
};
pub use config_store::McpConfigStore;
pub use context::{AgentScope, ExecutionContext};
pub use exec::{
    execute_legacy_skill, DEFAULT_SKILL_TIMEOUT, MAX_SKILL_STDERR_BYTES, MAX_SKILL_STDOUT_BYTES,
};
pub use gate::{sanitize_reflected, PermissionGate};
pub use host::{McpHost, DEFAULT_EXTERNAL_TOOL_TIMEOUT, DEFAULT_MCP_DISCOVERY_TIMEOUT};
pub use ipc_bridge::IpcBridge;
pub use native::{
    call_mcp_tool, get_mcp_tool_info, get_symbol_body, list_file_symbols, list_mcp_tools,
    recruit_specialist, run_integrity_check, InspectEngineHealthHandler,
    DEFAULT_INTEGRITY_CHECK_TIMEOUT,
};
pub use registry::{McpRegistry, ToolHandler};
pub use resolve::{resolve_tool, ResolvedTool};
pub use telemetry::McpTelemetry;
pub use transport::spawn_mcp_client;
pub use types::{McpResult, McpToolHub, McpToolStats};
