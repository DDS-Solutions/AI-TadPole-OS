//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP HTTP Limits
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Bounded resource limits matching MCP 2026-07-28 specification.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::mcp::client::http::tests::*`

pub const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024; // 1 MiB spec limit
pub const MAX_RESPONSE_BODY_BYTES: usize = 4 * 1024 * 1024; // 4 MiB
pub const MAX_SSE_LINE_BYTES: usize = 64 * 1024; // 64 KiB
pub const MAX_SSE_EVENTS: usize = 1024;
pub const MAX_SSE_NOTIFICATIONS: usize = 256;
pub const MAX_OPERATION_BINDINGS: usize = 1024;
pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
