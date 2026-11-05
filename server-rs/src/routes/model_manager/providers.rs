//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - Preserve route behavior and security checks when reorganizing model operations.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / model_manager providers
//! - **Primary Entrypoints**: get_providers, update_provider, delete_provider
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::NotFound
//! - **Telemetry Targets**: [infra-trace]
use super::validate_url_security;
use crate::{
    agent::types::{ProviderConfig, Validatable},
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

#[derive(serde::Serialize)]
pub struct ProviderResponse {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub base_url: Option<String>,
    pub protocol: crate::agent::types::ModelProvider,
    pub external_id: Option<String>,
    pub supports_steering_vectors: bool,
    pub audio_model: Option<String>,
    pub has_api_key: bool,
}

impl From<&ProviderConfig> for ProviderResponse {
    fn from(config: &ProviderConfig) -> Self {
        let has_key = config
            .api_key
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty());
        Self {
            id: config.id.clone(),
            name: config.name.clone(),
            icon: config.icon.clone(),
            base_url: config.base_url.clone(),
            protocol: config.protocol,
            external_id: config.external_id.clone(),
            supports_steering_vectors: config.supports_steering_vectors,
            audio_model: config.audio_model.clone(),
            has_api_key: has_key,
        }
    }
}

/// Returns all configured AI providers.
/// SEC-02: Strips `api_key` from the response so secrets are never sent to the frontend.
#[tracing::instrument(skip(state), name = "infra_providers::list")]
pub async fn get_providers(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let sanitized: Vec<ProviderResponse> = state
        .registry
        .providers
        .iter()
        .map(|kv| ProviderResponse::from(kv.value()))
        .collect();
    Ok(Json(sanitized))
}

/// Updates or creates a provider configuration with key-preserving merge semantics.
#[tracing::instrument(skip(_admin, state, config), fields(provider_id = %id), name = "infra_providers::update")]
pub async fn update_provider(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<ProviderConfig>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(e) = config.validate() {
        return Err(AppError::BadRequest(e));
    }

    if let Some(ref base_url) = config.base_url {
        if !base_url.trim().is_empty() {
            validate_url_security(base_url.trim())?;
        }
    }

    let mut merged_config = config;
    merged_config.id = id.clone();

    // SEC-02 Safe Merge: If incoming api_key is None, preserve the existing stored key
    if merged_config.api_key.is_none() {
        if let Some(existing) = state.registry.providers.get(&id) {
            merged_config.api_key = existing.api_key.clone();
        }
    }

    state.registry.providers.insert(id.clone(), merged_config);
    state.save_providers().await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated", "id": id })),
    ))
}

/// Deletes a provider configuration and cleans up associated models.
#[tracing::instrument(skip(_admin, state), fields(provider_id = %id), name = "infra_providers::delete")]
pub async fn delete_provider(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("🗑️ [infra-trace] Deleting provider: {}", id);

    // Guard: Check if any registered agents are actively configured with models from this provider
    let in_use_by: Vec<String> = state
        .registry
        .agents
        .iter()
        .filter(|kv| {
            let a = kv.value();
            let matches_provider = |cfg: &crate::agent::types::ModelConfig| {
                state
                    .registry
                    .models
                    .get(&cfg.model_id)
                    .map_or(false, |m| m.provider_id == id)
                    || cfg.model_id.starts_with(&format!("{}:", id))
            };
            matches_provider(&a.models.model)
                || a.models
                    .planning_slot
                    .as_ref()
                    .is_some_and(matches_provider)
                || a.models
                    .execution_slot
                    .as_ref()
                    .is_some_and(matches_provider)
        })
        .map(|kv| kv.key().clone())
        .collect();

    if !in_use_by.is_empty() {
        tracing::warn!(
            "⚠️ [infra-trace] Provider '{}' is currently referenced by agents: {:?}",
            id,
            in_use_by
        );
    }

    let removed = state.registry.providers.remove(&id);
    if removed.is_some() {
        tracing::info!("✅ [infra-trace] Successfully removed provider: {}", id);

        // Also remove all models associated with this provider
        let initial_count = state.registry.models.len();
        state
            .registry
            .models
            .retain(|_, model| model.provider_id != id);
        let removed_count = initial_count - state.registry.models.len();
        if removed_count > 0 {
            tracing::info!(
                "🗑️ [infra-trace] Decommissioned {} associated models for provider: {}",
                removed_count,
                id
            );
        }

        // Save both registries
        state.save_providers().await?;
        state.save_models().await?;

        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "status": "deleted", "id": id })),
        ))
    } else {
        tracing::warn!("⚠️ [infra-trace] Provider not found for deletion: {}", id);
        Err(AppError::NotFound(format!("Provider '{}' not found", id)))
    }
}
