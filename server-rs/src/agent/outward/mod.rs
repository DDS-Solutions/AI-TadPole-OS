//! @docs ARCHITECTURE:Agent
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

pub mod customer_catalog;
pub mod outward_gateway;

#[allow(unused_imports)]
pub use customer_catalog::{
    CatalogItem, CustomerCatalog, DEFAULT_MAX_CONTEXT_ITEMS, MAX_CATALOG_ITEMS,
};
#[allow(unused_imports)]
pub use outward_gateway::{
    A2aAgentCard, A2aSkill, BusinessProfile, OutwardGateway, DEFAULT_A2A_PROTOCOL_VERSION,
    DEFAULT_MODEL_PROFILE,
};
