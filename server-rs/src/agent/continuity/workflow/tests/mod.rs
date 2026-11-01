//! @docs ARCHITECTURE:Continuity:Workflow
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / workflow_tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - [Structural] Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

mod cascades;
mod context;
mod crud;
mod execution;
