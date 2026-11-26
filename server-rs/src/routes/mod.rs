//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / mod
//!
//! ### AI Assist Note
//! - Company operations register under the authenticated business API.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

// ─── Core Protocols & Protocol Upgrades ─────────────────────────────────────
pub mod pagination;
pub mod ws;

// ─── Sovereign Agent & Communication Gateways ───────────────────────────────
pub mod a2a;
pub mod agent;
pub mod audio;
pub mod business;
pub mod outward_routes;
pub mod remote;
pub mod skills;

// ─── Neural Infrastructure & Environment ────────────────────────────────────
pub mod backup;
pub mod deploy;
pub mod engine_control;
pub mod env_schema;
pub mod model_manager;
pub mod nodes;
pub mod system;

// ─── Knowledge, Memory & Intelligence ───────────────────────────────────────
pub mod cas;
pub mod docs;
pub mod intelligence;
#[cfg(feature = "vector-memory")]
pub mod knowledge;
#[cfg(not(feature = "vector-memory"))]
pub mod knowledge_fallback;
pub mod memory;
pub mod templates;

// ─── Governance, Continuous Execution & Telemetry ───────────────────────────
pub mod benchmarks;
pub mod continuity;
pub mod governance;
pub mod health;
pub mod metrics;
pub mod missions;
pub mod oversight;

// ─── Route-Level Integration Test Suites ────────────────────────────────────
#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod auth_tests;
#[cfg(test)]
mod business_tests;
#[cfg(test)]
mod integration_tests;
pub mod mcp;
#[cfg(test)]
mod mcp_test;
#[cfg(test)]
mod ws_tests;
