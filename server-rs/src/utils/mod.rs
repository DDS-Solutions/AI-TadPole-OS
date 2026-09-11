//! @docs ARCHITECTURE:UtilityFoundation
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Utilities / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod deduplicator;
pub mod fs_transaction;
pub mod graph;
pub mod parser;
pub mod security;
pub mod serialization;

#[cfg(feature = "vector-memory")]
pub mod data_weighting;

/// Returns true if the process is running inside a Docker/containerized environment.
pub fn is_docker() -> bool {
    if std::path::Path::new("/.dockerenv").exists() {
        return true;
    }
    if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
        if cgroup.contains("docker")
            || cgroup.contains("containerd")
            || cgroup.contains("kubepods")
            || cgroup.contains("lxc")
        {
            return true;
        }
    }
    false
}
