//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits
//! - **Architecture**: `@docs ARCHITECTURE:Registry`
//!
//! ### Module Layout
//! | Sub-module        | Responsibility                                   | Change Cadence |
//! |-------------------|--------------------------------------------------|----------------|
//! | `ports`           | Trait definitions + DTOs (public API contract)   | Slowest        |
//! | `observation`     | Token defense + injection sanitization (pure fns)| On prompt work |
//! | `routing`         | Model slot selection + prompt delegation         | On routing work|
//! | `identity`        | Hardcoded orchestrator trust boundary predicate  | Rarest         |
//! | `mission_state`   | Status/spec adapters + Sentinel Gate policy      | On state work  |
//! | `transaction`     | ACID state transaction + rollback RAII guard     | On tx work     |
//! | `orchestrator`    | Parallel tool execution + doom-loop detection    | On exec work   |
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` All public symbols are re-exported here — external callers use `service_traits::*`
//!   unchanged. Zero modifications required in any caller.
//!
//! ### 🔍 Debugging & Observability
//! - **Witness Tests**: All 15 colocated tests across sub-modules (run via
//!   `cargo test agent::runner::service_traits`)

pub mod identity;
pub mod mission_state;
pub mod observation;
pub mod orchestrator;
pub mod ports;
pub mod routing;
pub mod transaction;

// ---------------------------------------------------------------------------
// Zero-diff re-exports — 100% backward compatibility for all external callers
// ---------------------------------------------------------------------------

// Ports / DTOs
pub use ports::{
    AclServiceTrait, AgentMissionState, MissionStateManager, ModelRouter, PromptRendererTrait,
    PromptService, SlotKind, SlotSelection, ToolExecutor, ToolOrchestrationResult,
    ToolOrchestrator, WorkflowCoordinator,
};

// Observation / Token Defense
pub use observation::{
    classify_failure, format_fenced_observation, offload_large_tool_response,
    sanitize_observation_content, truncate_observation, DEFAULT_INDIVIDUAL_TOOL_TOKEN_THRESHOLD,
    DEFAULT_PREVIEW_CHARS, DEFAULT_TOTAL_TOOL_TOKEN_THRESHOLD, FAILURE_MESSAGE_TRUNCATION_LENGTH,
    MAX_TOOL_OUTPUT_CHARS,
};

// Routing
pub use routing::{DefaultModelRouter, DefaultPromptService};

// Identity
pub use identity::IdentityService;

// Mission State
pub use mission_state::{requires_verification, DefaultMissionStateManager};

// Transactions
pub use transaction::{StateTransaction, UndoOp};

// Orchestrator
pub use orchestrator::DefaultToolOrchestrator;
