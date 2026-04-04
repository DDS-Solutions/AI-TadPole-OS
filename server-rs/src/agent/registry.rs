//! Model Registry — Global provider and model configuration
//!
//! Provides the primary interface for infrastructure discovery. This module 
//! identifies the baseline configuration required to communicate with 
//! external LLM providers.
//!
//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Dynamic Configuration**: System providers and models are now 
//! externalized to `data/*.json` files. The Rust functions in this module 
//! are legacy fallbacks; agents should prioritize the persistence-based 
//! loader in most production scenarios.

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
