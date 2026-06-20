//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[mod]` in tracing logs.

pub mod code_graph;
pub mod networking;
pub mod telemetry;
pub mod security;
pub mod jobs;

pub use code_graph::{CodeGraphDbRefreshService, CodeGraphWarmupService};
pub use networking::{SwarmDiscoveryService, SwarmPulseService};
pub use telemetry::{HeartbeatService, MetricAggregatorService, SystemHealthMonitorService, RecoverActiveAgentsService};
pub use security::{BudgetFlushService, PrivacyGuardService, SecurityEvictionService};
pub use jobs::{ContinuitySchedulerService, IngestionWorkerService, MemoryCleanupService, RecipeIngestionService, SwarmReaperService};

#[cfg(feature = "vector-memory")]
pub use jobs::{IksDecayService, IksEvictionService};

// Metadata: [mod]
