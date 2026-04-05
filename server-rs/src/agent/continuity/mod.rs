//! @docs ARCHITECTURE:Continuity
//! 
//! ### AI Assist Note
//! **Continuity Subsystem**: Orchestrates the background persistence 
//! and long-running mission scheduling for the Tadpole OS engine. 
//! Features **Scheduled Mission Execution**: manages the lifecycle of 
//! recurring tasks and long-lived agent workflows. Implements 
//! **Mission Recovery**: ensures that interrupted missions can be 
//! resumed or rolled back based on the `ContinuityState`. AI agents 
//! should utilize this module for tasks that exceed the scope of a 
//! single user-interactive session (CONT-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Job execution stalls due to worker pool 
//!   exhaustion, database deadlocks during state-persistence, or 
//!   cron-expression validation failures.
//! - **Trace Scope**: `server-rs::agent::continuity`

pub mod executor;
pub mod scheduler;
pub mod types;
pub mod workflow;
