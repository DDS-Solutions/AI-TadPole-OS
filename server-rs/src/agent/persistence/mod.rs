//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Storage Synchronizer**: Bridges in-memory registries with persistent
//! **SQLite** and **JSON** storage. Orchestrates **Agent Reaping**
//! (`reap_stale_agents`) to ensure the swarm recovers from zombie/crashed
//! runs. Enforces **Credential Polarization** (SEC-02) by prioritizing
//! environment variables over disk-based JSON configs. Features
//! **Incremental Sync Manifests** to track external data ingestion
//! state across engine restarts.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: SQLite statement-cache staling across migrations
//!   (causing row decoding panics), JSON parsing errors in
//!   `infra_providers.json`, or heartbeat timeout misconfiguration
//!   leading to premature reaping.
//! - **Trace Scope**: `server-rs::agent::persistence`

pub mod agent_db;
pub mod blueprints;
pub mod infra_config;
pub mod sync_manifests;

pub use agent_db::*;
pub use blueprints::*;
pub use infra_config::*;
pub use sync_manifests::*;
