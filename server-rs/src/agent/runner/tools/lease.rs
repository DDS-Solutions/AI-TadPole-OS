//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[lease]` in tracing logs.

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

// Metadata: [lease]
