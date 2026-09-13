//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / lease
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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
