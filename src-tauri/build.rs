//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: System Core / build
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

fn main() {
    tauri_build::build()
}
