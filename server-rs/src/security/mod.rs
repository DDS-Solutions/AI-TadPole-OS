//! Cryptographic Security & Audit Infrastructure - The Shield Layer
//!
//! Orchestrates the high-security protocols for secret management, 
//! permission gating, and continuous audit monitoring. This layer 
//! ensures that all mission data remains isolated and tamper-evident.
//!
//! @docs ARCHITECTURE:ShieldLayer
//! @docs OPERATIONS_MANUAL:SecurityAudit
//!
//! ### AI Assist Note
//! **The Shield Layer**: This module manages the `permissions` engine 
//! and the `audit` log streaming. All non-standard filesystem access 
//! MUST be routed through the `permissions` gate prior to execution. 
//! `metering` tracks token consumption across the entire swarm.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Permission denial (Unauthorized access), Audit logging failure (disk full), or metering drift.
//! - **Telemetry Link**: Search for `[Security]` or `[Audit]` in `tracing` logs.
//! - **Trace Scope**: `server-rs::security`
//!
pub mod audit;
pub mod metering;
pub mod monitoring;
pub mod permissions;
pub mod scanner;
