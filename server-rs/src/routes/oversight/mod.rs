//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`, `AppError::Sqlx`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: declared in submodules

mod ledger;
mod quotas;
mod security;

// Re-export all public handler functions and types so `routes::oversight::*` paths
// continue to work without modification in router.rs, test_oversight.rs, and state/mod.rs.
pub use ledger::{
    decide_oversight, get_ledger, get_pending, get_settings, update_settings,
    OversightSettingsPayload,
};
#[allow(unused_imports)]
pub(crate) use ledger::{resolve_oversight_decision, verify_oversight_signature};

pub use quotas::{
    build_quota_summary, get_mission_quotas, get_security_quotas, update_agent_quota,
    update_mission_quota,
};
pub use security::{
    get_agent_health, get_audit_trail, get_integrity_status, get_policies, get_security_snapshot,
    update_policy,
};

pub(crate) const OVS_SIG_WINDOW_MS: i64 = 300_000;
pub(crate) const DB_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub(crate) use crate::utils::security::is_production_env;
