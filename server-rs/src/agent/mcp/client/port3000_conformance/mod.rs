//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Port 3000 Conformance
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Locally mocked HTTP and stdio conformance gate harness for AI-Tadpole-OS to GEV Port 3000 Client Contract.
//! - `[Structural]` Exhaustive evidence gate coverage for Task 6.2 unblocking.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: assertion failures identify the violated Port 3000 gate.
//! - **Telemetry Targets**: deterministic local mock HTTP and stdio transports only.
//! - **Witness Tests**: `port3000_conformance::{gates_config,gates_schema,gates_http,gates_adaptive}::test_gate_*`

pub(crate) mod harness;

#[cfg(test)]
mod tests;
