//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Execution Engine
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Child processes must be sandboxed, bounded, and killed on drop.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared

pub mod skill;
pub use skill::{
    execute_legacy_skill, DEFAULT_SKILL_TIMEOUT, MAX_SKILL_STDERR_BYTES, MAX_SKILL_STDOUT_BYTES,
};
