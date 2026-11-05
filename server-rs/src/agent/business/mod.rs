//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Isolated business workspaces use a common enquiry process and explicit connector contracts.
//! ### 🔍 Debugging & Observability
//! [Business] traces identify company, workspace and immutable run context.

mod connector;
mod service;
mod sops;
pub mod types;

pub use service::BusinessService;

#[cfg(test)]
mod tests;
