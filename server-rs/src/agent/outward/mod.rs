//! @docs ARCHITECTURE:Agent
//! @docs OPERATIONS_MANUAL:OutwardGateway
//!
//! ### AI Assist Note
//! Outward A2A Agent Gateway & Customer Knowledge Catalog for SMBs.
//! Silos customer-facing A2A interactions from internal codebase graphs.
//! Uses gemma4:e4b local model profile for execution.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Outward A2A schema validation errors, rate limits.
//! - **Trace Scope**: `server-rs::agent::outward` (Search for `[OutwardGateway]` in logs)

#[allow(dead_code)]
pub mod customer_catalog;
#[allow(dead_code)]
pub mod outward_gateway;

#[allow(unused_imports)]
pub use customer_catalog::*;
#[allow(unused_imports)]
pub use outward_gateway::*;
