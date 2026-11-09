//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / agent
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

pub mod authority;
pub mod entity;
pub mod wire;

pub use authority::*;
pub use entity::*;
pub use wire::*;
