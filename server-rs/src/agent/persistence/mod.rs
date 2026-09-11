//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod agent_db;
pub mod blueprints;
pub mod infra_config;
pub mod sync_manifests;

// Explicit Re-exports
#[allow(unused_imports)]
pub use agent_db::{
    claim_agent, execute_save_agent, load_agent_by_id, load_agent_by_id_db, load_agents_db,
    reap_stale_agents, release_agent, save_agent_db, save_agent_db_in_tx, save_agents_json,
    update_agent_heartbeat, FlatAgentRow,
};
#[allow(unused_imports)]
pub use blueprints::{delete_blueprint, execute_save_blueprint, load_blueprints, save_blueprint};
#[allow(unused_imports)]
pub use infra_config::{
    is_placeholder, load_models, load_providers, provider_env_var, save_models, save_providers,
};
#[allow(unused_imports)]
pub use sync_manifests::{
    complete_sync, get_all_sync_manifests, load_sync_manifests, sync_manifests_for_agent,
    update_sync_status,
};
