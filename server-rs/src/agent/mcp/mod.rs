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
pub mod config;
pub mod host;
pub mod native;
pub mod registry;
pub mod types;

// Re-export core items for backward compatibility across server-rs and macros
pub use authz::{
    decode_mcp_tool_name, encode_mcp_tool_name, is_mcp_server_authorized, is_mcp_tool_authorized,
};
pub use client::McpClient;
pub use config::{
    is_valid_environment_name, resolve_mcp_environment, resolve_mcp_headers,
    validate_mcp_server_config, McpConfig, McpServerConfig,
};
pub use host::{
    McpHost, DEFAULT_EXTERNAL_TOOL_TIMEOUT, DEFAULT_MCP_DISCOVERY_TIMEOUT, DEFAULT_SKILL_TIMEOUT,
};
pub use native::{
    get_symbol_body, list_file_symbols, recruit_specialist, run_integrity_check,
    InspectEngineHealthHandler, DEFAULT_INTEGRITY_CHECK_TIMEOUT,
};
pub use registry::{McpRegistry, ToolHandler};
pub use types::{McpResult, McpToolHub, McpToolStats};
