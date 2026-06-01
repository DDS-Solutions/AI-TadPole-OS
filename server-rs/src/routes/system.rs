//! @docs ARCHITECTURE:Gateways
//!
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[system.rs]` in tracing logs.

use crate::error::AppError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

/// Exposes the hardware profile of the Tadpole OS engine for sovereign compute telemetry.
#[tracing::instrument(skip(state), name = "system::compute_profile")]
pub async fn get_compute_profile(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let profile = state.resources.hardware_profiler.get_profile();
    Ok((StatusCode::OK, Json(profile)))
}

/// Retrieves the synchronization status and metrics for all workspaces/connectors.
#[tracing::instrument(skip(state), name = "system::workspaces_status")]
pub async fn get_workspaces_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let manifests = crate::agent::persistence::get_all_sync_manifests(&state.resources.pool).await?;
    Ok((StatusCode::OK, Json(manifests)))
}

/// Retrieves the list of files in the workspace (relative paths).
#[tracing::instrument(skip(state), name = "system::workspace_files")]
pub async fn get_workspace_files(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let base_dir = &state.resources.base_dir;
    let base_dir_clone = base_dir.clone();
    
    let files = tokio::task::spawn_blocking(move || {
        let mut list = Vec::new();
        for entry in walkdir::WalkDir::new(&base_dir_clone)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != "target"
                    && name != "node_modules"
                    && name != ".git"
                    && name != "dist"
                    && name != ".tmp"
                    && name != ".gemini"
            })
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                if let Ok(rel_path) = entry.path().strip_prefix(&base_dir_clone) {
                    let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
                    list.push(rel_path_str);
                }
            }
            if list.len() >= 5000 {
                break;
            }
        }
        list
    })
    .await
    .map_err(|e| AppError::InternalServerError(format!("Workspace files listing panicked: {}", e)))?;

    Ok((StatusCode::OK, Json(files)))
}

// Metadata: [system]

// Metadata: [system]
