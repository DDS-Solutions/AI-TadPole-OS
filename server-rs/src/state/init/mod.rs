//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init
//! - **Primary Entrypoints**: none declared
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod channels;
pub mod databases;
pub mod security;
pub mod services;

pub use channels::*;
pub use databases::*;
pub use security::*;
pub use services::*;
