//! API Surface — REST & WebSocket Routing
//!
//! Orchestrates the entry points for all external engine interactions, 
//! providing versioned REST endpoints and real-time WebSocket bridges.
//!
//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Routing Hierarchy**: Most API endpoints are grouped logically by 
//! subsystem (e.g., `agent`, `memory`, `oversight`) and versioned under 
//! `/v1/`. Real-time streams are exclusively handled via the `ws` module. 
//! High-latency tasks like `deploy` or `benchmarks` use an 
//! async-request pattern with `task_id` feedback.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404 (Incorrect route prefix), 405 (Method mismatch), or WebSocket handshake failure.
//! - **Telemetry Link**: Search for `[Route]` or `[Handler]` in `tracing` logs.
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

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod auth_tests;
#[cfg(test)]
mod mcp_test;
#[cfg(test)]
mod ws_tests;
