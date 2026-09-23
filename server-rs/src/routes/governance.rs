//! @docs ARCHITECTURE:Governance
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / governance
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::NotFound`, `AppError::Sqlx`, `AppError::InternalServerError`
//! - **Telemetry Targets**: `[Governance]`
//! - **Witness Tests**: `governance::tests::test_blueprint_crud`

use crate::agent::types::RoleBlueprint;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

/// ### ⚖️ Governance: Blueprint Discovery
/// Returns a list of all registered Role Blueprints.
#[tracing::instrument(skip(state), name = "governance::list_blueprints")]
pub async fn list_blueprints(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let blueprints = crate::agent::persistence::load_blueprints(&state.resources.pool).await?;
    Ok((StatusCode::OK, Json(blueprints)))
}

/// ### ⚖️ Governance: Promote to Role
/// Registers or updates a Role Blueprint in the persistence layer.
#[tracing::instrument(skip(state, blueprint), name = "governance::save_blueprint")]
pub async fn save_blueprint(
    State(state): State<Arc<AppState>>,
    Json(blueprint): Json<RoleBlueprint>,
) -> Result<impl IntoResponse, AppError> {
    let id = blueprint.id.trim();
    let name = blueprint.name.trim();

    if id.is_empty()
        || id.len() > 64
        || !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::BadRequest(
            "Blueprint ID must be 1-64 characters and contain only alphanumeric, dash, or underscore".into(),
        ));
    }

    if name.is_empty() || name.len() > 128 {
        return Err(AppError::BadRequest(
            "Blueprint Name must be 1-128 characters".into(),
        ));
    }

    crate::agent::persistence::save_blueprint(&state.resources.pool, &blueprint).await?;

    tracing::info!(
        "✅ [Governance] Role Blueprint '{}' saved successfully",
        blueprint.id
    );

    state.emit_event(json!({
        "type": "governance:blueprint_saved",
        "id": blueprint.id,
        "name": blueprint.name,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    Ok(StatusCode::OK)
}

/// ### ⚖️ Governance: Role Retirement
/// Deletes a Role Blueprint from the system.
#[tracing::instrument(skip(state), name = "governance::delete_blueprint")]
pub async fn delete_blueprint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let trimmed_id = id.trim();
    if trimmed_id.is_empty() {
        return Err(AppError::BadRequest("Blueprint ID cannot be empty".into()));
    }

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM role_blueprints WHERE id = ?1)")
            .bind(trimmed_id)
            .fetch_one(&state.resources.pool)
            .await
            .map_err(AppError::Sqlx)?;

    if !exists {
        return Err(AppError::NotFound(format!(
            "Role Blueprint '{}' not found",
            trimmed_id
        )));
    }

    crate::agent::persistence::delete_blueprint(&state.resources.pool, trimmed_id).await?;

    tracing::warn!("🗑️ [Governance] Role Blueprint '{}' retired.", trimmed_id);

    state.emit_event(json!({
        "type": "governance:blueprint_deleted",
        "id": trimmed_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blueprint_crud() {
        let state = Arc::new(AppState::new_minimal_mock().await);

        // Save invalid blueprint
        let invalid_bp = RoleBlueprint {
            id: "bad id with spaces!".into(),
            name: "Invalid BP".into(),
            department: "eng".into(),
            description: "".into(),
            skills: "[]".into(),
            workflows: "[]".into(),
            mcp_tools: "[]".into(),
            requires_oversight: false,
            model_id: None,
            created_at: Some(chrono::Utc::now()),
        };
        let save_res = save_blueprint(State(state.clone()), Json(invalid_bp)).await;
        assert!(save_res.is_err());

        // Delete nonexistent
        let del_res = delete_blueprint(State(state.clone()), Path("nonexistent-bp".into())).await;
        assert!(del_res.is_err());
    }
}
