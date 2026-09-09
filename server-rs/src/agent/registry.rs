//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / registry
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::{ModelEntry, ProviderConfig};

/// Returns the exhaustive list of supported LLM providers.
///
/// # Note
/// This is now a legacy fallback. System providers should be loaded from
/// `data/infra_providers.json` via the persistence module.
pub fn get_default_providers() -> Vec<ProviderConfig> {
    // Returning empty as configurations are now externalized.
    Vec::new()
}

/// Returns the exhaustive list of supported LLM models.
///
/// # Note
/// This is now a legacy fallback. System models should be loaded from
/// `data/infra_models.json` via the persistence module.
pub fn get_default_models() -> Vec<ModelEntry> {
    // Returning empty as configurations are now externalized.
    Vec::new()
}
