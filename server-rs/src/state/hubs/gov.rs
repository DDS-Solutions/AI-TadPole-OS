//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **Policy Controller**: Centralizes atomic constraints for swarm depth,
//! agent counts, and global resource quotas. Features a **Privacy Shield**
//! that, when toggled, blocks all external cloud provider traffic. Enforces
//! **Governance Hierarchy** by propagating budget and recursion limits
//! across the entire agentic lifecycle.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Atomic counter overflow in `tpm_accumulator`,
//!   deadlocks in `default_budget_usd` (parking_lot::RwLock), or
//!   unsynchronized privacy mode states across distributed workers.
//! - **Trace Scope**: `server-rs::state::hubs::gov`

use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};

/// Hub for system limits and automated policy enforcement.
pub struct GovernanceHub {
    /// Global setting: whether to auto-approve low-risk skills.
    pub auto_approve_safe_skills: AtomicBool,
    /// Maximum allowed agents in the swarm.
    pub max_agents: AtomicU32,
    /// Maximum allowed clusters.
    pub max_clusters: AtomicU32,
    /// Maximum depth for agent recursion/spawning.
    pub max_swarm_depth: AtomicU32,
    /// Maximum token length for a single task.
    pub max_task_length: AtomicUsize,
    /// Default budget allocated to new agents (in USD).
    pub default_budget_usd: RwLock<f64>,
    /// Default AI intelligence model used when none is explicitly configured.
    pub default_model: RwLock<String>,
    /// Default model provider used when none is explicitly configured.
    pub default_provider: RwLock<String>,
    /// Number of agents currently executing tasks.
    pub active_agents: AtomicU32,
    /// Total number of recruitment operations performed.
    pub recruit_count: AtomicU32,
    /// Global TPM accumulator for telemetry.
    pub tpm_accumulator: AtomicUsize,
    /// Privacy Shield: When true, all external cloud provider traffic is blocked.
    pub privacy_mode: AtomicBool,
    /// Failover Amber threshold (failures before status becomes Amber)
    pub failover_amber_threshold: AtomicU32,
    /// Failover Red threshold (failures before status becomes Red)
    pub failover_red_threshold: AtomicU32,
    /// Failover Max attempts (max retries to alternate models)
    pub failover_max_attempts: AtomicU32,
    /// Default timeout for LLM provider generation calls (in seconds)
    pub provider_timeout_secs: AtomicU32,
    /// Test mode flag to route all LLMs through NullProvider (TADPOLE_NULL_PROVIDERS=true)
    pub null_providers_test_mode: AtomicBool,
    /// Deprecated endpoints mapped to (Sunset Date, Alternate Link)
    pub deprecated_routes: RwLock<std::collections::HashMap<String, (String, String)>>,
    /// Per-cluster privacy mode override map (cluster_id -> privacy_mode boolean)
    pub cluster_privacy_policies: dashmap::DashMap<String, bool>,
}

impl GovernanceHub {
    /// Evaluates effective privacy mode for a given optional cluster_id.
    /// Returns true if global privacy_mode is active OR if the specified cluster_id is air-gapped.
    pub fn is_privacy_mode_enabled(&self, cluster_id: Option<&str>) -> bool {
        if self.privacy_mode.load(std::sync::atomic::Ordering::Relaxed) {
            return true;
        }
        if let Some(cid) = cluster_id {
            if let Some(entry) = self.cluster_privacy_policies.get(cid) {
                return *entry.value();
            }
        }
        false
    }
}

// Metadata: [gov]
