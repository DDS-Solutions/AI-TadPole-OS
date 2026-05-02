//! @docs ARCHITECTURE:Infrastructure
//! 
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[profiler.rs]` in tracing logs.

use serde::{Deserialize, Serialize};
use parking_lot::Mutex;
use sysinfo::System;

/// A snapshot of the system's compute pipeline.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputeProfile {
    pub cpu_usage: f32, // percentage
    pub memory_used: u64, // bytes
    pub memory_total: u64, // bytes
    pub active_processes: usize,
    pub gpu_usage: Option<f32>, // sysinfo doesn't easily track cross-platform GPU natively yet
}

pub struct HardwareProfiler {
    sys: Mutex<System>,
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

    pub fn get_profile(&self) -> ComputeProfile {
        let mut sys = self.sys.lock();
        // To get accurate CPU usage, we just refresh CPU
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let cpus = sys.cpus();
        let cpu_usage = if !cpus.is_empty() {
            let total_usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum();
            total_usage / (cpus.len() as f32)
        } else {
            0.0
        };

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
        assert!(profile.memory_total > 0, "Memory total should be greater than 0");
        assert!(profile.active_processes > 0, "There should be at least some active processes");
        
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
        let sys = profiler.sys.lock();
        // Since we refresh cpu in new(), cpus() should not be empty
        assert!(!sys.cpus().is_empty());
    }
}

// Metadata: [profiler]

// Metadata: [profiler]
