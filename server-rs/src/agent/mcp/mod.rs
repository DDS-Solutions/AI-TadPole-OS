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
pub mod native;
pub mod registry;
pub mod resolve;
pub mod telemetry;
#[cfg(test)]
pub mod test_support;
pub mod transport;
pub mod types;

// Re-export core items for backward compatibility across server-rs and macros
pub use authz::{decode_mcp_tool_name, is_mcp_tool_authorized};
#[cfg(test)]
pub use client::McpClient;
#[cfg(test)]
pub use config::McpServerConfig;
pub use config::{validate_mcp_server_config, McpConfig};
pub use context::AgentScope;
pub use host::McpHost;
pub use types::{McpResult, McpToolHub, McpToolStats};
