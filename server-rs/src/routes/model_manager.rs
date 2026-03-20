use crate::{
    agent::types::{ModelEntry, ProviderConfig},
    state::AppState,
    routes::error::AppError,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Returns all configured AI providers.
/// SEC-02: Strips `api_key` from the response so secrets are never sent to the frontend.
pub async fn get_providers(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let sanitized: Vec<serde_json::Value> = state.registry.providers
        .iter()
        .map(|kv| {
            let mut val = serde_json::to_value(kv.value()).unwrap_or_default();
            if let Some(obj) = val.as_object_mut() {
                // Replace actual key with a boolean flag so dashboard knows if it's configured
                let has_key = obj
                    .get("apiKey")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.trim().is_empty());
                obj.insert("apiKey".to_string(), serde_json::Value::Null);
                obj.insert("hasApiKey".to_string(), serde_json::Value::Bool(has_key));
            }
            val
        })
        .collect();
    Ok(Json(serde_json::json!(sanitized)))
}

/// Updates or creates a provider configuration.
pub async fn update_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<ProviderConfig>,
) -> Result<impl IntoResponse, AppError> {
    state.registry.providers.insert(id.clone(), config);
    state.save_providers().await;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated", "id": id })),
    ))
}

/// Deletes a provider configuration.
pub async fn delete_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("🗑️ [infra-trace] Deleting provider: {}", id);
    let removed = state.registry.providers.remove(&id);
    if removed.is_some() {
        tracing::info!("✅ [infra-trace] Successfully removed provider: {}", id);

        // Also remove all models associated with this provider
        let mut models_to_remove = Vec::new();
        for kv in state.registry.models.iter() {
            if kv.value().provider_id == id {
                models_to_remove.push(kv.key().clone());
            }
        }

        for model_id in models_to_remove {
            tracing::info!("🗑️ [infra-trace] Decommissioning associated model: {}", model_id);
            state.registry.models.remove(&model_id);
        }

        // Save both registries
        state.save_providers().await;
        state.save_models().await;

        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "status": "deleted", "id": id })),
        ))
    } else {
        tracing::warn!("⚠️ [infra-trace] Provider not found for deletion: {}", id);
        Err(AppError::NotFound(format!("Provider '{}' not found", id)))
    }
}

/// Returns all available models in the registry.
pub async fn get_models(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let models: Vec<ModelEntry> = state.registry.models.iter().map(|kv| kv.value().clone()).collect();
    Ok(Json(models))
}

/// Updates or creates a model entry.
pub async fn update_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(entry): Json<ModelEntry>,
) -> Result<impl IntoResponse, AppError> {
    state.registry.models.insert(id.clone(), entry);
    state.save_models().await;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated", "id": id })),
    ))
}

/// Deletes a model entry.
pub async fn delete_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("🗑️ [infra-trace] Deleting model: {}", id);
    let removed = state.registry.models.remove(&id);
    if removed.is_some() {
        tracing::info!("✅ [infra-trace] Successfully removed model: {}", id);
        state.save_models().await;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "status": "deleted", "id": id })),
        ))
    } else {
        tracing::warn!("⚠️ [infra-trace] Model not found for deletion: {}", id);
        Err(AppError::NotFound(format!("Model '{}' not found", id)))
    }
}

/// Performs a connectivity test for a provider.
pub async fn test_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<ProviderConfig>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("🔍 [Test Trace] Initiating handshake for provider: {}", id);

    let mut api_key = config.api_key.as_deref().unwrap_or("").to_string();
    let base_url = config.base_url.as_deref().unwrap_or("");
    let protocol = config.protocol.to_lowercase();

    // Fallback to saved API key if not provided in the request
    if api_key.is_empty() {
        if let Some(saved) = state.registry.providers.get(&id) {
            if let Some(saved_key) = &saved.api_key {
                api_key = saved_key.clone();
            }
        }
    }

    if api_key.is_empty() {
        return Err(AppError::BadRequest("Missing Secure API Key. handshakes require authentication.".to_string()));
    }

    let url = match protocol.as_str() {
        "anthropic" => format!(
            "{}/v1/models",
            if base_url.is_empty() {
                "https://api.anthropic.com"
            } else {
                base_url.trim_end_matches('/')
            }
        ),
        "google" => format!(
            "{}/models?key={}",
            if base_url.is_empty() {
                "https://generativelanguage.googleapis.com/v1"
            } else {
                base_url.trim_end_matches('/')
            },
            api_key
        ),
        "openai" | "inception" | "deepseek" | "ollama" | "groq" => {
            format!(
                "{}/models",
                if base_url.is_empty() {
                    if protocol == "groq" {
                        "https://api.groq.com/openai/v1"
                    } else {
                        "https://api.openai.com/v1"
                    }
                } else {
                    base_url.trim_end_matches('/')
                }
            )
        }
        _ => format!(
            "{}/models",
            if base_url.is_empty() {
                "https://api.openai.com/v1"
            } else {
                base_url.trim_end_matches('/')
            }
        ),
    };

    let mut request = state.resources.http_client.get(&url);

    if protocol != "google" {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    if protocol == "anthropic" {
        request = request.header("x-api-key", api_key);
        request = request.header("anthropic-version", "2023-06-01");
    }

    match request.send().await {
        Ok(res) => {
            if res.status().is_success() {
                tracing::info!("✅ [Test Trace] Handshake successful for {}", id);
                Ok((StatusCode::OK, Json(serde_json::json!({
                    "status": "success",
                    "message": format!("Handshake successful: {} endpoint is reactive.", config.name)
                }))))
            } else {
                let err_text = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                let safe_err = state.security.secret_redactor.redact(&err_text);
                tracing::warn!("❌ [Test Trace] Handshake failed for {}: {}", id, safe_err);
                Err(AppError::Unauthorized(format!("Handshake failed: {}", safe_err)))
            }
        }
        Err(e) => {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            tracing::error!("🚨 [Test Trace] Network error for {}: {}", id, safe_err);
            Err(AppError::InternalServerError(format!("Network Error: {}", safe_err)))
        }
    }
}
