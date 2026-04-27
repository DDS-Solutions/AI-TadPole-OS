//! @docs ARCHITECTURE:Governance
//! 
//! ### AI Assist Note
//! **Governance Logic**: API handlers for managing Role Blueprints. 
//! Orchestrates the CRUD operations for standardized agent templates, ensuring 
//! that recruitment blueprints are synchronized with the primary database.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Database connection timeouts during blueprint retrieval or unique constraint violations on save.
//! - **Telemetry Link**: Search `[Governance]` in server traces.
//!

use crate::agent::types::RoleBlueprint;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// ### ⚖️ Governance: Blueprint Discovery
/// Returns a list of all registered Role Blueprints.
pub async fn list_blueprints(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match crate::agent::persistence::load_blueprints(&state.resources.pool).await {
        Ok(blueprints) => (StatusCode::OK, Json(blueprints)).into_response(),
        Err(e) => {
            tracing::error!("❌ [Governance] Failed to load blueprints: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

/// ### ⚖️ Governance: Promote to Role
/// Registers or updates a Role Blueprint in the persistence layer.
pub async fn save_blueprint(
    State(state): State<Arc<AppState>>,
    Json(blueprint): Json<RoleBlueprint>,
) -> impl IntoResponse {
    // Basic validation
    if blueprint.id.is_empty() || blueprint.name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Blueprint ID and Name are required").into_response();
    }

    match crate::agent::persistence::save_blueprint(&state.resources.pool, &blueprint).await {
        Ok(_) => {
            tracing::info!("✅ [Governance] Role Blueprint '{}' saved successfully", blueprint.id);
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!("❌ [Governance] Failed to save blueprint: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

/// ### ⚖️ Governance: Role Retirement
/// Deletes a Role Blueprint from the system.
pub async fn delete_blueprint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match crate::agent::persistence::delete_blueprint(&state.resources.pool, &id).await {
        Ok(_) => {
            tracing::warn!("🗑️ [Governance] Role Blueprint '{}' retired.", id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!("❌ [Governance] Failed to delete blueprint: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

// Metadata: [governance]

// Metadata: [governance]
