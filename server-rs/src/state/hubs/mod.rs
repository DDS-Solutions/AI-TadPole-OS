//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **State Hubs (Neural Architecture)**: Orchestrates the
//! categorization and initialization of sub-states for the Tadpole OS
//! engine. Features **Logical Separation of Concerns**: organizes
//! communication (`comm`), governance (`gov`), registry (`reg`),
//! security (`sec`), and hardware resources (`res`) into discrete
//! modules. Implements **Unified State Aggregation**: provides a
//! composite structure (`AppState`) that integrates these hubs
//! into a single thread-safe context for routing and agent
//! execution. AI agents should use this module as a map to
//! navigate common system services (e.g., event broadcasting,
//! secret redacting, or pool access) (STA-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Circular dependencies between physical hubs due
//!   to incorrect re-exports, initialization order failures leading
//!   to `None` references in `AppState`, or high-frequency lock
//!   contention in the `reg` hub.
//! - **Trace Scope**: `server-rs::state::hubs`

pub mod comm;
pub mod gov;
pub mod reg;
pub mod res;
pub mod sec;

// Metadata: [mod]

// Metadata: [mod]
