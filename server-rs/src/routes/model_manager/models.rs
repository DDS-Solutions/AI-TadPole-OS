//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - Preserve route behavior and security checks when reorganizing model operations.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / model_manager models
//! - **Primary Entrypoints**: get_models, update_model, delete_model
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::NotFound
//! - **Telemetry Targets**: [infra-trace]
use crate::{
    agent::{
        capability_matrix::CapabilityMatrix,
        types::{ModelEntry, Validatable},
    },
    error::AppError,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Returns all available agent models.
#[tracing::instrument(skip(state), name = "infra_models::list")]
pub async fn get_models(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let models: Vec<ModelEntry> = state
        .registry
        .models
        .iter()
        .map(|kv| {
            let mut entry = kv.value().clone();
            if entry.capabilities.context_window == 0 {
                if let Some(cw) = entry.context_window {
                    entry.capabilities.context_window = cw;
                } else {
                    entry.capabilities = CapabilityMatrix::infer_capabilities(&entry.id);
                }
            }
            entry
        })
        .collect();
    Ok(Json(models))
}

/// Updates or creates a model entry.
#[tracing::instrument(skip(_admin, state, entry), fields(model_id = %id), name = "infra_models::update")]
pub async fn update_model(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(entry): Json<ModelEntry>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(e) = entry.validate() {
        return Err(AppError::BadRequest(e));
    }

    if !state.registry.providers.contains_key(&entry.provider_id) {
        return Err(AppError::BadRequest(format!(
            "Provider '{}' is not registered in the neural infrastructure.",
            entry.provider_id
        )));
    }

    let mut entry = entry;
    entry.id = id.clone();
    if let Some(cw) = entry.context_window {
        entry.capabilities.context_window = cw;
    } else if entry.capabilities.context_window == 0 {
        entry.capabilities = CapabilityMatrix::infer_capabilities(&id);
    }

    tracing::info!(
        "🔄 [infra-trace] Model updated: id={}, context_window={:?}, input_cost_per_m={:?}, output_cost_per_m={:?}",
        id, entry.context_window, entry.input_cost_per_m, entry.output_cost_per_m
    );

    state.registry.models.insert(id.clone(), entry);
    state.save_models().await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated", "id": id })),
    ))
}

/// Deletes a model from the registry.
#[tracing::instrument(skip(_admin, state), fields(model_id = %id), name = "infra_models::delete")]
pub async fn delete_model(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let in_use_by: Vec<String> = state
        .registry
        .agents
        .iter()
        .filter(|kv| {
            let a = kv.value();
            a.models.model.model_id == id
                || a.models
                    .planning_slot
                    .as_ref()
                    .is_some_and(|m| m.model_id == id)
                || a.models
                    .execution_slot
                    .as_ref()
                    .is_some_and(|m| m.model_id == id)
        })
        .map(|kv| kv.key().clone())
        .collect();

    if !in_use_by.is_empty() {
        tracing::warn!(
            "⚠️ [infra-trace] Deleting model '{}' currently referenced by agents: {:?}",
            id,
            in_use_by
        );
    }

    tracing::info!("🗑️ [infra-trace] Deleting model: {}", id);
    let removed = state.registry.models.remove(&id);
    if removed.is_some() {
        tracing::info!("✅ [infra-trace] Successfully removed model: {}", id);
        state.save_models().await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "status": "deleted", "id": id })),
        ))
    } else {
        tracing::warn!("⚠️ [infra-trace] Model not found for deletion: {}", id);
        Err(AppError::NotFound(format!("Model '{}' not found", id)))
    }
}
