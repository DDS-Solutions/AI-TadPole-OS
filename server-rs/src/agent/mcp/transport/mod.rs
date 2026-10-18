//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Transport Factory
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Shell command validation and environment sanitization prior to stdio process creation.
//! - `[Structural]` Deterministic mode dispatch across Auto, PreferHttp, PreferStdio, Http, and Stdio.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Forbidden`, `AppError::BadRequest`
//! - **Telemetry Targets**: none declared

pub mod factory;
pub use factory::spawn_mcp_client;
