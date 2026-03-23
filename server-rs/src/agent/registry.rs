//! Central registry for provider and model configuration logic.
//!
//! This module identifies the baseline infrastructure required for the engine to
//! communicate with external LLM providers. Definitions are now persisted in
//! JSON configuration files in the `data/` directory to allow for dynamic
//! updates without recompilation.

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


