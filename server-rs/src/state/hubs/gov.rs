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
    /// Number of agents currently executing tasks.
    pub active_agents: AtomicU32,
    /// Total number of recruitment operations performed.
    pub recruit_count: AtomicU32,
    /// Global TPM accumulator for telemetry.
    pub tpm_accumulator: AtomicUsize,
    /// Privacy Shield: When true, all external cloud provider traffic is blocked.
    pub privacy_mode: AtomicBool,
}

// Metadata: [gov]

// Metadata: [gov]
