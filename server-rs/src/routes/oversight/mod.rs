//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Governance & Oversight Gateway**: Orchestrates the REST surface
//! for **Human-in-the-Loop** verification and global engine
//! constraints. Features **Oversight Decision Flows**: manages the
//! unblocking and termination of autonomous agent tasks based on
//! human approval. Implements **Merkle Hash-Chain Recording**: every
//! oversight decision is committed to a tamper-evident audit ledger
//! with digital signature verification. AI agents should monitor the
//! `oversight_queue` and provide clear technical context for all
//! pending decisions to minimize human review friction (GOV-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404 on resolution due to resolver timeout,
//!   decisions stalling in the queue due to missing human input, or
//!   Merkle integrity failures on tampered log entries.
//! - **Telemetry Link**: Search for `⚖️ [Oversight]` in `tracing` logs
//!   for decision lifecycle events.
//! - **Trace Scope**: `server-rs::routes::oversight`

mod ledger;
mod quotas;
mod security;

// Re-export all public handler functions and types so `routes::oversight::*` paths
// continue to work without modification in router.rs, test_oversight.rs, and state/mod.rs.
pub use ledger::{
    decide_oversight, get_ledger, get_pending, update_settings, OversightSettingsPayload,
};
#[allow(unused_imports)] // Used by agent::test_oversight (test-only)
pub(crate) use ledger::{resolve_oversight_decision, verify_oversight_signature};

pub use quotas::{
    get_mission_quotas, get_security_quotas, update_agent_quota, update_mission_quota,
};
pub use security::{
    get_agent_health, get_audit_trail, get_integrity_status, get_policies, get_security_snapshot,
    update_policy,
};

pub(crate) const OVS_SIG_WINDOW_MS: i64 = 300_000;
pub(crate) const DB_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub(crate) fn is_production_env() -> bool {
    std::env::var("TADPOLE_ENV")
        .or_else(|_| std::env::var("ENV"))
        .map(|v| v.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
}

// Metadata: [oversight]
