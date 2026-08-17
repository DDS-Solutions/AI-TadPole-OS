//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[mod]` in tracing logs.

pub mod graph;
pub mod graph_store;
pub mod markdown_graph;

pub use graph::EXCLUDED_DIRS;

// Metadata: [mod]
