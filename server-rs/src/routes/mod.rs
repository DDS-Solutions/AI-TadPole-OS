//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **API Surface (REST & WebSocket)**: Orchestrates the entry points
//! for the Tadpole OS engine. Features a **Sovereign Routing
//! Hierarchy**: endpoints are grouped by subsystem (e.g., `agent`,
//! `memory`, `oversight`) and versioned under `/v1/`. Real-time mission
//! telemetry is exclusively handled via the `ws` (WebSocket) bridge.
//! High-latency operations (e.g., `deploy`, `benchmarks`) utilize an
//! **Asynchronous Response Pattern**, providing a `task_id` for
//! poll-based or socket-based status feedback (ROUTE-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404 (Incorrect subsystem route prefix), 405
//!   (Method mismatch), or WebSocket handshake failure due to
//!   missing subprotocols.
//! - **Telemetry Link**: Search for `[Route]` or `[Handler]` in
//!   `tracing` logs for request lifecycle benchmarks.
//! - **Trace Scope**: `server-rs::routes`

pub mod agent;
pub mod audio;
pub mod benchmarks;
pub mod continuity;
pub mod deploy;
pub mod docs;
pub mod engine_control;
pub mod env_schema;
pub mod error;
pub mod health;
pub mod mcp;
#[cfg(feature = "vector-memory")]
pub mod memory;
pub mod model_manager;
pub mod nodes;
pub mod oversight;
pub mod pagination;
pub mod skills;
pub mod templates;
pub mod ws;
pub mod system;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod auth_tests;
#[cfg(test)]
mod mcp_test;
#[cfg(test)]
mod ws_tests;
