//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Services / Backup
//! - **Primary Entrypoints**: `crypto`, `snapshot`, `daemon`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

pub mod crypto;
pub mod daemon;
pub mod snapshot;

pub use daemon::*;
