//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod environment;
pub mod okf_gate;
pub mod profiler;
