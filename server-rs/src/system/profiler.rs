//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / profiler
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sysinfo::System;

/// A snapshot of the system's compute pipeline.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputeProfile {
    pub cpu_usage: f32,    // percentage
    pub memory_used: u64,  // bytes
    pub memory_total: u64, // bytes
    pub active_processes: usize,
    pub gpu_usage: Option<f32>, // sysinfo doesn't easily track cross-platform GPU natively yet
}

pub struct HardwareProfiler {
    sys: Mutex<System>,
}

impl Default for HardwareProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareProfiler {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        // pre-warm
        sys.refresh_all();
        Self {
            sys: Mutex::new(sys),
        }
    }

    /// Returns the number of logical CPU cores detected on the host.
    #[allow(dead_code)]
    pub fn cpu_count(&self) -> usize {
        self.sys.lock().cpus().len()
    }

    /// Retrieves the current compute profile snapshot.
    ///
    /// # Note on Sampling Resolution
    /// `sysinfo` computes CPU utilization based on delta ticks between consecutive
    /// refreshes. Callers should poll at intervals of ≥200ms for representative metrics.
    pub fn get_profile(&self) -> ComputeProfile {
        let mut sys = self.sys.lock();
        // To get accurate CPU usage, we refresh CPU
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::nothing(),
        );

        let cpus = sys.cpus();
        let raw_cpu_usage = if !cpus.is_empty() {
            let total_usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum();
            total_usage / (cpus.len() as f32)
        } else {
            0.0
        };
        let cpu_usage = raw_cpu_usage.clamp(0.0, 100.0);

        let memory_used = sys.used_memory();
        let memory_total = sys.total_memory();
        let active_processes = sys.processes().len();

        ComputeProfile {
            cpu_usage,
            memory_used,
            memory_total,
            active_processes,
            gpu_usage: None,
        }
    }
}

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_profile_gathering() {
        let profiler = HardwareProfiler::new();
        let profile = profiler.get_profile();

        // 1. Verify metrics are non-zero (assuming the test machine is running)
        assert!(
            profile.memory_total > 0,
            "Memory total should be greater than 0"
        );
        assert!(
            profile.active_processes > 0,
            "There should be at least some active processes"
        );

        // 2. CPU usage might be 0 on a cold start or idle machine, but should be a valid f32
        assert!(profile.cpu_usage >= 0.0 && profile.cpu_usage <= 100.0);

        // 3. Serialization check
        let json = serde_json::to_string(&profile).expect("Failed to serialize profile");
        assert!(json.contains("cpu_usage"));
        assert!(json.contains("memory_used"));
    }

    #[test]
    fn test_hardware_profiler_prewarm() {
        let profiler = HardwareProfiler::new();
        // Since we refresh cpu in new(), cpu_count() should be greater than 0
        assert!(profiler.cpu_count() > 0);
    }
}
