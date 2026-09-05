//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / conflict
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct ConflictManager {
    leases: DashMap<PathBuf, (String, Instant)>,
    ttl: Duration,
}

impl ConflictManager {
    pub fn new() -> Self {
        Self {
            leases: DashMap::new(),
            ttl: Duration::from_secs(30),
        }
    }

    pub fn acquire_lease(&self, path: PathBuf, agent_id: String) -> Result<(), AppError> {
        let now = Instant::now();

        self.leases
            .retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);

        if let Some(entry) = self.leases.get(&path) {
            let (holder, timestamp) = entry.value();
            if holder != &agent_id && now.duration_since(*timestamp) < self.ttl {
                return Err(AppError::Forbidden(format!(
                    "Path {:?} is currently locked by agent '{}' (lease expires in {:?}). Please retry with backoff.",
                    path,
                    holder,
                    self.ttl.checked_sub(now.duration_since(*timestamp)).unwrap_or(Duration::from_secs(0))
                )));
            }
        }

        self.leases.insert(path, (agent_id, now));
        Ok(())
    }

    pub fn release_lease(&self, path: &Path) {
        self.leases.remove(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_conflict_manager_lease() {
        let manager = ConflictManager::new();
        let path = PathBuf::from("workspace/file.txt");

        assert!(manager
            .acquire_lease(path.clone(), "agent-a".to_string())
            .is_ok());
        assert!(manager
            .acquire_lease(path.clone(), "agent-a".to_string())
            .is_ok());

        let err = manager.acquire_lease(path.clone(), "agent-b".to_string());
        assert!(err.is_err());

        manager.release_lease(&path);

        assert!(manager
            .acquire_lease(path.clone(), "agent-b".to_string())
            .is_ok());
    }

    #[test]
    fn test_conflict_manager_ttl_expiry() {
        let mut manager = ConflictManager::new();
        manager.ttl = Duration::from_millis(10);

        let path = PathBuf::from("workspace/file.txt");

        assert!(manager
            .acquire_lease(path.clone(), "agent-a".to_string())
            .is_ok());

        sleep(Duration::from_millis(15));

        assert!(manager
            .acquire_lease(path.clone(), "agent-b".to_string())
            .is_ok());
    }
}
