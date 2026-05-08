//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[data_weighting]` in tracing logs.

use std::collections::HashMap;

pub struct DataWeighting;

impl DataWeighting {
    /// Returns the default weights for various context components.
    /// Higher values mean higher importance (more likely to be preserved).
    pub fn default_weights() -> HashMap<String, f32> {
        let mut weights = HashMap::new();
        weights.insert("identity".to_string(), 2.0);
        weights.insert("mission_goal".to_string(), 2.0);
        weights.insert("directives".to_string(), 1.8);
        weights.insert("findings".to_string(), 1.5);
        weights.insert("history".to_string(), 1.2);
        weights.insert("repo_map".to_string(), 0.8);
        weights.insert("memory".to_string(), 0.7);
        weights.insert("swarm_context".to_string(), 1.0);
        weights
    }
}

// Metadata: [data_weighting]
