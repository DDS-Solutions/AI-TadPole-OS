//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / missions
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - [Structural] Deterministic internal state integrity and strict interface contract compliance.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::NotFound, AppError::Sqlx, AppError::InternalServerError
//! - **Telemetry Targets**: none declared

pub mod catalog;
pub mod contracts;
pub mod dossier;
pub mod pins;

#[cfg(test)]
mod tests;

pub use catalog::get_missions_catalog;
pub use contracts::*;
pub use dossier::get_mission_dossier;
pub use pins::toggle_mission_pin;
