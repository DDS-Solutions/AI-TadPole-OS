//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP HTTP Transport Submodule
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` MCP 2026-07-28 Stateless Streamable HTTP specification conformance.
//! - `[Structural]` Request-scoped SSE isolation: each SSE connection maps 1:1 with an HTTP POST.
//! - `[Structural]` Omission of Origin, Mcp-Session-Id, and Last-Event-ID per Port 3000 IPC contract.
//! - `[Structural]` Memory-bounded operation ID binding cache (MAX_OPERATION_BINDINGS = 1,024).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InfrastructureError`, `AppError::Conflict`, `AppError::Forbidden`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::mcp::client::http::tests::*`

pub mod body;
pub mod classify;
pub mod client;
pub mod headers;
pub mod limits;
pub mod sse;

pub use classify::{HttpDiscoveryFailureKind, HttpTransportFailureKind};
pub use client::McpHttpClient;
pub use headers::{
    encode_header_value_if_needed, extract_and_validate_tool_headers, hash_operation_binding,
    is_valid_header_token, primitive_header_value,
};
pub use limits::*;

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    pub use super::classify::tests::*;
    #[allow(unused_imports)]
    pub use super::client::tests::*;
    #[allow(unused_imports)]
    pub use super::headers::tests::*;
    #[allow(unused_imports)]
    pub use super::sse::tests::*;
}
