//! @docs ARCHITECTURE:IKS
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod search;
pub mod store;
pub mod types;

#[cfg(test)]
pub mod tests;

#[allow(unused_imports)]
pub use store::KnowledgeStore;
#[allow(unused_imports)]
pub use types::{
    AddKnowledgeRequest, KnowledgeEntry, KnowledgeSearchRequest, SecurityTier, DEFAULT_TTL_DAYS,
};
