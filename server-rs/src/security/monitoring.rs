use parking_lot::RwLock;
use serde::Serialize;
use std::sync::Arc;
use sysinfo::{MemoryRefreshKind, System};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SandboxType {
    None,
    Docker,
    Kubernetes,
    Podman,
    GenericContainer,
}

/// Snapshot of system health and defense posture.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemDefenseStats {
    /// Ratio of used RAM to total RAM (0.0 to 1.0).
    pub memory_pressure: f64,
    /// Global CPU load across all cores (0.0 to 1.0).
    pub cpu_load: f64,
    /// Human-readable status of the sandbox (e.g., "ACTIVE").
    pub sandbox_status: String,
    /// Detected sandbox environment (e.g., Docker, Kubernetes).
    pub sandbox_type: SandboxType,
    /// Whether the engine is confirmed to be running in an isolated environment.
    pub is_isolated: bool,
}

/// Real-time monitor for system resources and environment isolation.
pub struct SecurityMonitor {
    /// Shared access to the sysinfo state.
    sys: Arc<RwLock<System>>,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys: Arc::new(RwLock::new(sys)),
        }
    }

    pub fn get_system_defense_stats(&self) -> SystemDefenseStats {
        let mut sys = self.sys.write();
        // Refresh only what we need for efficiency
        sys.refresh_memory_specifics(MemoryRefreshKind::new().with_ram());
        sys.refresh_cpu_usage();

        let total_mem = sys.total_memory() as f64;
        let used_mem = sys.used_memory() as f64;
        let memory_pressure = if total_mem > 0.0 {
            used_mem / total_mem
        } else {
            0.0
        };

        let cpu_load = sys.global_cpu_info().cpu_usage() as f64 / 100.0;

        let (sandbox_type, is_isolated) = self.detect_sandbox_internal();
        let sandbox_status = if is_isolated { "ACTIVE" } else { "INACTIVE" }.to_string();

        SystemDefenseStats {
            memory_pressure,
            cpu_load,
            sandbox_status,
            sandbox_type,
            is_isolated,
        }
    }

    fn detect_sandbox_internal(&self) -> (SandboxType, bool) {
        // 1. Check for Docker
        if std::path::Path::new("/.dockerenv").exists() {
            return (SandboxType::Docker, true);
        }

        // 2. Check for Kubernetes
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            return (SandboxType::Kubernetes, true);
        }

        // 3. Check for Podman (often has certain files/env vars)
        if std::env::var("container")
            .map(|v| v == "podman")
            .unwrap_or(false)
        {
            return (SandboxType::Podman, true);
        }

        // 4. Heuristic: Check for common container mount points
        if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
            if cgroup.contains("docker") || cgroup.contains("kubepods") {
                return (SandboxType::GenericContainer, true);
            }
        }

        (SandboxType::None, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_monitor_instantiation() {
        let monitor = SecurityMonitor::new();
        let stats = monitor.get_system_defense_stats();

        // Verify metrics are within plausible ranges
        assert!(stats.memory_pressure >= 0.0 && stats.memory_pressure <= 1.0);
        assert!(stats.cpu_load >= 0.0 && stats.cpu_load <= 1.0);
        assert!(stats.sandbox_status == "ACTIVE" || stats.sandbox_status == "INACTIVE");
    }

    #[test]
    fn test_sandbox_type_consistency() {
        let monitor = SecurityMonitor::new();
        let stats = monitor.get_system_defense_stats();

        match stats.sandbox_type {
            SandboxType::None => assert!(!stats.is_isolated),
            _ => assert!(stats.is_isolated),
        }
    }
}
