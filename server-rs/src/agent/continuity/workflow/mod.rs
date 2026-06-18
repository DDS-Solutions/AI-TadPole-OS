//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **Continuity Workflow Engine**: Orchestrates the deterministic
//! sequence of agent tasks and long-running state machine
//! resumption. Features **Multi-Step Orchestration**: enables the
//! piping of results between agents using template placeholders
//! (`{{context_keys}}`). Implements **Durable Run Persistence**:
//! every workflow step is committed to the `workflow_runs` and
//! `workflow_step_runs` tables, ensuring that the engine can
//! reconstruct the execution path after a system restart. AI agents
//! should utilize the `context` object to maintain state across
//! asynchronous task boundaries (CONT-03).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Placeholder injection failures due to missing
//!   context keys, step-order conflicts in the database, or agent
//!   timeouts during high-concurrency workflow bursts.
//! - **Trace Scope**: `server-rs::agent::continuity::workflow`

pub mod types;
pub mod helpers;
pub mod executor;
pub mod engine;

#[cfg(test)]
mod tests;

pub use types::*;
pub use engine::WorkflowEngine;

// Metadata: [workflow]

// Metadata: [mod]

// Metadata: [mod]
