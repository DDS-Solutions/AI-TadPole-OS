//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod code_graph;
pub mod jobs;
pub mod maintenance;
pub mod networking;
pub mod security;
pub mod telemetry;

pub use code_graph::{CodeGraphDbRefreshService, CodeGraphWarmupService};
pub use jobs::{
    ContinuitySchedulerService, IngestionWorkerService, MemoryCleanupService,
    RecipeIngestionService, SpanWatchdogService, SwarmReaperService,
};
pub use maintenance::SqliteMaintenanceService;
pub use networking::{SwarmDiscoveryService, SwarmPulseService};
pub use security::{BudgetFlushService, PrivacyGuardService, SecurityEvictionService};
pub use telemetry::{
    HeartbeatService, MetricAggregatorService, RecoverActiveAgentsService,
    SystemHealthMonitorService, TelemetryLogSinkService,
};

#[cfg(feature = "vector-memory")]
pub use jobs::{IksDecayService, IksEvictionService};
