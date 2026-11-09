//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Port 3000 Conformance - Test Aggregator
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Exhaustive witness gate aggregator for Port 3000 Client Contract.
//!
//! ### 🔍 Debugging & Observability
//! - **Witness Tests**: `port3000_conformance::tests::{gates_config,gates_schema,gates_http,gates_adaptive}::test_gate_*`

use super::harness;

#[path = "gates_adaptive.rs"]
mod gates_adaptive;
#[path = "gates_config.rs"]
mod gates_config;
#[path = "gates_http.rs"]
mod gates_http;
#[path = "gates_schema.rs"]
mod gates_schema;
