//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / lease
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Behavioral]` Tool lease guards ensure exclusive file access and release locks on drop (enforced_by: `test_tool_lease_guard_lifecycle`).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_tool_lease_guard_lifecycle`

use crate::agent::runner::tools::error::ToolExecutionError;
use std::sync::Arc;

pub struct ToolLeaseGuard {
    conflict_manager: Arc<crate::security::conflict::ConflictManager>,
    path: std::path::PathBuf,
    active: bool,
}

impl ToolLeaseGuard {
    pub fn acquire(
        conflict_manager: Arc<crate::security::conflict::ConflictManager>,
        path: std::path::PathBuf,
        agent_id: String,
    ) -> Result<Self, ToolExecutionError> {
        conflict_manager
            .acquire_lease(path.clone(), agent_id)
            .map_err(ToolExecutionError::AppError)?;
        Ok(Self {
            conflict_manager,
            path,
            active: true,
        })
    }

    pub fn release(&mut self) {
        if self.active {
            self.conflict_manager.release_lease(&self.path);
            self.active = false;
        }
    }
}

impl Drop for ToolLeaseGuard {
    fn drop(&mut self) {
        if self.active {
            self.conflict_manager.release_lease(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_tool_lease_guard_lifecycle() {
        let cm = Arc::new(crate::security::conflict::ConflictManager::new());
        let path = PathBuf::from("workspace/test.txt");

        {
            let guard = ToolLeaseGuard::acquire(cm.clone(), path.clone(), "agent_1".to_string())
                .expect("Acquire should succeed");
            assert!(cm
                .acquire_lease(path.clone(), "agent_2".to_string())
                .is_err());
            drop(guard);
        }

        assert!(cm
            .acquire_lease(path.clone(), "agent_2".to_string())
            .is_ok());
    }
}
