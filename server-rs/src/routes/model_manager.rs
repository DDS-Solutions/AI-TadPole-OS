//! Model Manager — LLM orchestration and lifecycle control
//!
//! @docs ARCHITECTURE:Agent

use crate::{
    agent::types::{ModelEntry, ProviderConfig},
    routes::error::AppError,
    state::AppState,
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
    let sanitized: Vec<serde_json::Value> = state
        .registry
        .providers
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
    if let Err(e) = config.validate() {
        return Err(AppError::BadRequest(e));
    }
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
            tracing::info!(
                "🗑️ [infra-trace] Decommissioning associated model: {}",
                model_id
            );
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
pub async fn get_models(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let models: Vec<ModelEntry> = state
        .registry
        .models
        .iter()
        .map(|kv| kv.value().clone())
        .collect();
    Ok(Json(models))
}

/// Updates or creates a model entry.
pub async fn update_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(entry): Json<ModelEntry>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(e) = entry.validate() {
        return Err(AppError::BadRequest(e));
    }

    // Verify provider existence for swarm consistency
    if !state.registry.providers.contains_key(&entry.provider_id) {
        return Err(AppError::BadRequest(format!(
            "Provider '{}' is not registered in the neural infrastructure.",
            entry.provider_id
        )));
    }

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
        return Err(AppError::BadRequest(
            "Missing Secure API Key. handshakes require authentication.".to_string(),
        ));
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
        "google" => {
            if base_url.is_empty() {
                "https://generativelanguage.googleapis.com/v1/models".to_string()
            } else {
                let mut b = base_url.trim_end_matches('/').to_string();
                if !b.contains("/v1") && !b.contains("/beta") {
                    b.push_str("/v1");
                }
                format!("{}/models", b)
            }
        }
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

    // SEC-01: Prevent transmission of API keys over unencrypted HTTP for external endpoints.
    if url.starts_with("http://") && !url.contains("localhost") && !url.contains("127.0.0.1") {
        return Err(AppError::BadRequest(
            "Insecure transmission blocked: API keys cannot be sent over HTTP to external providers. Use HTTPS."
                .to_string(),
        ));
    }

    let mut request = state.resources.http_client.get(&url);

    if protocol == "google" {
        // SEC-03: Transitioned from query param to header to prevent exposure in logs/history.
        request = request.header("x-goog-api-key", &api_key);
    } else {
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
                Ok((
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "status": "success",
                        "message": format!("Handshake successful: {} endpoint is reactive.", config.name)
                    })),
                ))
            } else {
                let err_text = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                let safe_err = state.security.secret_redactor.redact(&err_text);
                tracing::warn!("❌ [Test Trace] Handshake failed for {}: {}", id, safe_err);
                Err(AppError::Unauthorized(format!(
                    "Handshake failed: {}",
                    safe_err
                )))
            }
        }
        Err(e) => {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            tracing::error!("🚨 [Test Trace] Network error for {}: {}", id, safe_err);
            Err(AppError::InternalServerError(format!(
                "Network Error: {}",
                safe_err
            )))
        }
    }
}

/// Returns a curated catalog of recommended models for the local swarm.
pub async fn get_model_catalog(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let catalog = serde_json::json!([
        {
            "id": "llama3:8b",
            "name": "Llama 3 (8B)",
            "provider": "ollama",
            "description": "Meta's most capable 8B model. Balanced for logic and creative tasks.",
            "size": "4.7GB",
            "vram": "8GB",
            "tags": ["General", "Logic"]
        },
        {
            "id": "phi3:latest",
            "name": "Phi-3 Mini",
            "provider": "ollama",
            "description": "Microsoft's high-performance 3.8B small language model.",
            "size": "2.3GB",
            "vram": "4GB",
            "tags": ["Fast", "Efficiency"]
        },
        {
            "id": "mistral:latest",
            "name": "Mistral (7B)",
            "provider": "ollama",
            "description": "The original sovereign open-weight champion.",
            "size": "4.1GB",
            "vram": "6GB",
            "tags": ["Original", "Balanced"]
        },
        {
            "id": "nomic-embed-text:latest",
            "name": "Nomic Embed",
            "provider": "ollama",
            "description": "High-fidelity text embeddings for RAG and Vector Memory.",
            "size": "274MB",
            "vram": "1GB",
            "tags": ["RAG", "Embeddings"]
        }
    ]);

    Ok(Json(catalog))
}

#[derive(serde::Deserialize)]
pub struct PullModelPayload {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub tag: String,
}

/// Proxies a model pull request to a specific swarm node.
pub async fn pull_model(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PullModelPayload>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!(
        "📥 [ModelStore] Requesting pull of {} on node {}",
        payload.tag,
        payload.node_id
    );

    // 1. Locate the node
    let node = state.registry.nodes.get(&payload.node_id).ok_or_else(|| {
        AppError::NotFound(format!("Node '{}' not found in swarm", payload.node_id))
    })?;

    // 2. Prepare the Ollama pull request
    // We hit the Tadpole Engine's unified proxy on the target node.
    let ollama_url = format!("http://{}/v1/api/pull", node.address);
    let ollama_payload = serde_json::json!({
        "name": payload.tag,
        "stream": false
    });

    // 3. Dispatch the request with Swarm Authentication (NEURAL_TOKEN)
    let resp = state
        .resources
        .http_client
        .post(&ollama_url)
        .header(
            "Authorization",
            format!("Bearer {}", state.security.deploy_token),
        )
        .json(&ollama_payload)
        .send()
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to reach Bunker node: {}", e))
        })?;

    if resp.status().is_success() {
        tracing::info!(
            "✅ [ModelStore] Successfully initiated pull on {}",
            node.name
        );
        Ok((
            StatusCode::OK,
            Json(
                serde_json::json!({ "status": "success", "message": format!("Model {} is being pulled to {}", payload.tag, node.name) }),
            ),
        ))
    } else {
        let status = resp.status();
        let err = resp
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        tracing::error!(
            "❌ [ModelStore] Pull failed on {} ({}): {}",
            node.name,
            status,
            err
        );
        Err(AppError::InternalServerError(format!(
            "Bunker node '{}' reported error ({}): {}",
            node.name, status, err
        )))
    }
}

/// A lightweight proxy handler that forwards local model pull requests
/// from the Engine port (8000) to the local Ollama instance (11434).
/// This ensures a unified "Swarm Gateway" for all node operations.
pub async fn ollama_proxy_pull(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let ollama_host =
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
    let ollama_url = format!("{}/api/pull", ollama_host);

    tracing::info!(
        "🔄 [ModelStore] Proxying pull request to local Ollama: {}",
        ollama_url
    );

    let resp = state
        .resources
        .http_client
        .post(&ollama_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to reach local Ollama: {}", e))
        })?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_else(|_| "{}".to_string());

    if status.is_success() {
        Ok((status, body))
    } else {
        tracing::error!(
            "❌ [ModelStore] Local Ollama pull failed ({}): {}",
            status,
            body
        );
        Err(AppError::InternalServerError(format!(
            "Ollama reported error: {}",
            body
        )))
    }
}
