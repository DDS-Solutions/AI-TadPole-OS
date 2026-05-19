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

// Metadata: [system]

// Metadata: [system]
