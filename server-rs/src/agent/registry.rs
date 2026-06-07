//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Discovery Hub**: Provides the primary interface for infrastructure
//! discovery and baseline configuration. While production models are
//! externalized to `data/*.json`, this module remains the logic-anchor
//! for **Infrastructure Fallbacks**.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing or empty default provider lists, causing
//!   agent initialization to stall.
//! - **Trace Scope**: `server-rs::agent::registry`

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

// Metadata: [registry]

// Metadata: [registry]
