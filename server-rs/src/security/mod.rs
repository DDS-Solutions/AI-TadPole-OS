//! @docs ARCHITECTURE:ShieldLayer
//! @docs OPERATIONS_MANUAL:SecurityAudit
//!
//! ### AI Assist Note
//! **The Shield Layer**: Orchestrates the cryptographic protocols and
//! continuous audit monitoring for the Tadpole OS engine. Features
//! **Permission Gating** for all non-standard and high-risk operations
//! (Filesystem, Network), **Token Metering** across the entire swarm,
//! and **Tamper-Evident Audit Logging**. Coordinates with the `audit`
//! module to ensure that every high-level mission intent is recorded
//! for forensic reconstruction.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Permission denial (gating failure), audit logging
//!   latency under high-throughput mission execution, or metering drift
//!   due to provider-side token mismatches.
//! - **Telemetry Link**: Search for `[Security]` or `[Audit]` in `tracing`
//!   logs for policy enforcement milestones.
//! - **Trace Scope**: `server-rs::security`
//!
pub mod audit;
pub mod metering;
pub mod monitoring;
pub mod permissions;
pub mod scanner;

#[cfg(test)]
mod permission_tests;
