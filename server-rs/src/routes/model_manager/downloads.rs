//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - Preserve route behavior and security checks when reorganizing model operations.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / model_manager downloads
//! - **Primary Entrypoints**: get_model_catalog, pull_model, ollama_proxy_pull
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::NotFound, AppError::InternalServerError
//! - **Telemetry Targets**: [ModelStore]
use super::{execute_with_timeout, get_ollama_host, validate_url_security};
use crate::{agent::types::Validatable, error::AppError, state::AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

/// Returns a curated catalog of recommended models for the local swarm.
#[tracing::instrument(skip(_state), name = "infra_models::catalog")]
pub async fn get_model_catalog(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let catalog = serde_json::json!([
        {
            "id": "llama3:8b",
            "name": "Llama 3 (8B)",
            "provider": "ollama",
            "description": "Meta's balanced 8B open-weight model for logic and creative tasks.",
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
    pub node_id: String,
    pub tag: String,
}

impl Validatable for PullModelPayload {
    fn validate(&self) -> Result<(), String> {
        let tag = self.tag.trim();
        if tag.is_empty() {
            return Err("Model tag cannot be empty".to_string());
        }
        if tag.len() > 128 {
            return Err("Model tag exceeds maximum length of 128 characters".to_string());
        }
        if tag.contains("..") || tag.starts_with('/') || tag.starts_with('.') {
            return Err(format!(
                "Model tag '{}' contains invalid path traversal sequences",
                tag
            ));
        }
        if !tag.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == ':' || c == '.' || c == '-' || c == '/' || c == '_'
        }) {
            return Err(format!("Model tag '{}' contains invalid characters", tag));
        }
        let node_id = self.node_id.trim();
        if node_id.is_empty() {
            return Err("Node ID cannot be empty".to_string());
        }
        if node_id.len() > 64 {
            return Err("Node ID exceeds maximum length of 64 characters".to_string());
        }
        if node_id.contains("..") || node_id.starts_with('/') || node_id.starts_with('.') {
            return Err(format!(
                "Node ID '{}' contains invalid path traversal sequences",
                node_id
            ));
        }
        Ok(())
    }
}

/// Proxies a model pull request to a specific swarm node.
#[tracing::instrument(skip(_admin, state, payload), fields(node_id = %payload.node_id, tag = %payload.tag), name = "infra_models::pull")]
pub async fn pull_model(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PullModelPayload>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::BadRequest(e));
    }

    tracing::info!(
        "📥 [ModelStore] Requesting pull of {} on node {}",
        payload.tag,
        payload.node_id
    );

    if payload.node_id == "local" || payload.node_id == "localhost" {
        let ollama_payload = serde_json::json!({
            "name": payload.tag,
            "stream": false
        });
        let _ = ollama_proxy_pull(_admin, State(Arc::clone(&state)), Json(ollama_payload)).await?;
        tracing::info!(
            "✅ [ModelStore] Successfully initiated pull on local Ollama engine for {}",
            payload.tag
        );
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Model {} is being pulled to local node", payload.tag)
            })),
        ));
    }

    let node = state.registry.nodes.get(&payload.node_id).ok_or_else(|| {
        AppError::NotFound(format!("Node '{}' not found in swarm", payload.node_id))
    })?;

    // Validate and parse node address using robust url::Url parser
    let raw_address = node.address.trim();
    if raw_address.is_empty() {
        return Err(AppError::BadRequest(
            "Node network address is empty".to_string(),
        ));
    }

    let candidate_url = if raw_address.starts_with("http://") || raw_address.starts_with("https://")
    {
        raw_address.to_string()
    } else {
        format!("http://{}", raw_address)
    };

    let parsed = url::Url::parse(&candidate_url)
        .map_err(|e| AppError::BadRequest(format!("Node network address is invalid: {}", e)))?;

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(AppError::BadRequest(
            "Node network address must not contain user credentials".to_string(),
        ));
    }

    let host = parsed.host_str().ok_or_else(|| {
        AppError::BadRequest("Node network address is missing a valid host".to_string())
    })?;
    let port = parsed.port().unwrap_or(11434);

    let ollama_url = format!("http://{}:{}/api/pull", host, port);
    validate_url_security(&ollama_url)?;

    let ollama_payload = serde_json::json!({
        "name": payload.tag,
        "stream": false
    });

    let request = state
        .resources
        .http_client
        .post(&ollama_url)
        .header(
            "Authorization",
            format!("Bearer {}", state.security.deploy_token),
        )
        .json(&ollama_payload);

    let resp = execute_with_timeout(request, &state, "Node pull request").await?;

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
        let safe_err = state.security.secret_redactor.redact(&err);
        tracing::error!(
            "❌ [ModelStore] Pull failed on {} ({}): {}",
            node.name,
            status,
            safe_err
        );
        Err(AppError::InternalServerError(format!(
            "Bunker node '{}' reported error ({}): {}",
            node.name, status, safe_err
        )))
    }
}

/// A lightweight proxy handler forwarding model pulls to local Ollama.
#[tracing::instrument(skip(_admin, state, payload), name = "infra_models::ollama_proxy_pull")]
pub async fn ollama_proxy_pull(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let ollama_host = get_ollama_host().await;
    let clean_host = ollama_host.trim_end_matches('/');
    let base_no_v1 = clean_host
        .strip_suffix("/v1")
        .unwrap_or(clean_host)
        .trim_end_matches('/');
    let ollama_url = format!("{}/api/pull", base_no_v1);

    validate_url_security(&ollama_url)?;

    tracing::info!(
        "🔄 [ModelStore] Proxying pull request to local Ollama: {}",
        ollama_url
    );

    let request = state.resources.http_client.post(&ollama_url).json(&payload);

    let resp = execute_with_timeout(request, &state, "Local Ollama pull").await?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_else(|_| "{}".to_string());

    if status.is_success() {
        Ok((status, body))
    } else {
        let safe_body = state.security.secret_redactor.redact(&body);
        tracing::error!(
            "❌ [ModelStore] Local Ollama pull failed ({}): {}",
            status,
            safe_body
        );
        Err(AppError::InternalServerError(format!(
            "Ollama reported error: {}",
            safe_body
        )))
    }
}
