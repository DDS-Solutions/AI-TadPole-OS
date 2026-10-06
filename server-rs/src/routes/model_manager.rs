//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / model_manager
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::NotFound`, `AppError::Unauthorized`, `AppError::InternalServerError`
//! - **Telemetry Targets**: `[infra-trace]`, `[IMR]`
//! - **Witness Tests**: `model_manager::tests::*`

use crate::{
    agent::{
        capability_matrix::CapabilityMatrix,
        types::{ModelEntry, ProviderConfig, Validatable},
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
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

pub const OUTBOUND_REQUEST_TIMEOUT_SECS: u64 = 30;

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

pub async fn get_ollama_host() -> String {
    let raw_host = std::env::var("OLLAMA_HOST").unwrap_or_default();
    let host = raw_host.trim();
    if host.is_empty() {
        crate::networking::resolver::AddressResolver::resolve_local_url(11434).await
    } else {
        let candidate = if !host.starts_with("http://") && !host.starts_with("https://") {
            format!("http://{}", host)
        } else {
            host.to_string()
        };
        let clean = candidate.trim_end_matches('/');
        let base_no_v1 = clean.strip_suffix("/v1").unwrap_or(clean);
        crate::networking::resolver::AddressResolver::resolve_url_if_local(base_no_v1).await
    }
}

pub use crate::security::ssrf_guard::{is_host_private_or_loopback, validate_url_security};

enum ProtocolRouter {
    Anthropic,
    Google,
    OpenAI,
    Ollama,
}

impl ProtocolRouter {
    fn from_protocol(protocol: &str) -> Self {
        match protocol {
            "anthropic" => Self::Anthropic,
            "google" | "gemini" => Self::Google,
            "ollama" => Self::Ollama,
            _ => Self::OpenAI,
        }
    }

    async fn build_models_url(&self, protocol: &str, base_url: &str, is_sync: bool) -> String {
        match self {
            Self::Anthropic => {
                let base = if base_url.is_empty() {
                    "https://api.anthropic.com".to_string()
                } else {
                    crate::networking::resolver::AddressResolver::resolve_url_if_local(base_url)
                        .await
                };
                format!("{}/v1/models", base.trim_end_matches('/'))
            }
            Self::Google => {
                let base = if base_url.is_empty() {
                    "https://generativelanguage.googleapis.com".to_string()
                } else {
                    crate::networking::resolver::AddressResolver::resolve_url_if_local(base_url)
                        .await
                };
                format!("{}/v1beta/models", base.trim_end_matches('/'))
            }
            Self::Ollama => {
                let base = if base_url.is_empty() {
                    get_ollama_host().await
                } else {
                    crate::networking::resolver::AddressResolver::resolve_url_if_local(base_url)
                        .await
                };
                let clean = base.trim_end_matches('/');
                let base_no_v1 = clean
                    .strip_suffix("/v1")
                    .unwrap_or(clean)
                    .trim_end_matches('/');
                if is_sync {
                    format!("{}/api/tags", base_no_v1)
                } else {
                    format!("{}/api/version", base_no_v1)
                }
            }
            Self::OpenAI => {
                let base = if base_url.is_empty() {
                    match protocol {
                        "groq" => "https://api.groq.com/openai/v1".to_string(),
                        "mistral" => "https://api.mistral.ai/v1".to_string(),
                        "perplexity" => "https://api.perplexity.ai".to_string(),
                        "fireworks" => "https://api.fireworks.ai/inference/v1".to_string(),
                        "together" => "https://api.together.xyz/v1".to_string(),
                        "deepseek" => "https://api.deepseek.com/v1".to_string(),
                        "xai" => "https://api.x.ai/v1".to_string(),
                        "openrouter" => "https://openrouter.ai/api/v1".to_string(),
                        "cerebras" => "https://api.cerebras.ai/v1".to_string(),
                        "sambanova" => "https://api.sambanova.ai/v1".to_string(),
                        _ => "https://api.openai.com/v1".to_string(),
                    }
                } else {
                    crate::networking::resolver::AddressResolver::resolve_url_if_local(base_url)
                        .await
                };
                let base = base.trim_end_matches('/');
                if base.ends_with("/models") {
                    base.to_string()
                } else if base.contains(":11434") && !base.ends_with("/v1") {
                    format!("{}/v1/models", base)
                } else {
                    format!("{}/models", base)
                }
            }
        }
    }

    fn authenticate_request(
        &self,
        builder: reqwest::RequestBuilder,
        api_key: &str,
    ) -> reqwest::RequestBuilder {
        match self {
            Self::Anthropic => builder
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01"),
            Self::Google => builder.header("x-goog-api-key", api_key),
            Self::Ollama => builder,
            Self::OpenAI => builder.bearer_auth(api_key),
        }
    }

    fn parse_discovery_response(&self, body: &serde_json::Value, url: &str) -> Vec<String> {
        let mut discovered_ids = Vec::new();
        match self {
            Self::Anthropic => {
                if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
                    for m in data {
                        if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
                            discovered_ids.push(id.to_string());
                        }
                    }
                }
            }
            Self::Ollama => {
                if url.contains("/tags") {
                    if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
                        for m in models {
                            if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                                discovered_ids.push(name.to_string());
                            }
                        }
                    }
                } else if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
                    for m in data {
                        if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
                            discovered_ids.push(id.to_string());
                        }
                    }
                } else if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
                    for m in models {
                        if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                            discovered_ids.push(name.to_string());
                        }
                    }
                }
            }
            Self::Google => {
                if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
                    for m in models {
                        if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                            discovered_ids.push(name.replace("models/", ""));
                        }
                    }
                }
            }
            _ => {
                if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
                    for m in data {
                        if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
                            discovered_ids.push(id.to_string());
                        }
                    }
                }
            }
        }
        discovered_ids
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
                entry.capabilities = CapabilityMatrix::infer_capabilities(&entry.id);
            }
            entry
        })
        .collect();
    Ok(Json(models))
}

/// Updates or creates a model entry.
#[tracing::instrument(skip(state, entry), fields(model_id = %id), name = "infra_models::update")]
pub async fn update_model(
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
    if entry.capabilities.context_window == 0 {
        entry.capabilities = CapabilityMatrix::infer_capabilities(&id);
    }

    state.registry.models.insert(id.clone(), entry);
    state.save_models().await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated", "id": id })),
    ))
}

/// Deletes a model from the registry.
#[tracing::instrument(skip(state), fields(model_id = %id), name = "infra_models::delete")]
pub async fn delete_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
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

/// Tests a provider configuration by performing a dummy model list or balance check.
#[tracing::instrument(skip(state, config), fields(provider_id = %id), name = "infra_providers::test")]
pub async fn test_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<ProviderConfig>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("🔍 [Test Trace] Initiating handshake for provider: {}", id);

    let api_key = config
        .api_key
        .filter(|k| !k.is_empty())
        .or_else(|| {
            state
                .registry
                .providers
                .get(&id)
                .and_then(|p| p.api_key.clone())
        })
        .unwrap_or_default();
    let base_url = config.base_url.as_deref().unwrap_or("");
    let protocol = config.protocol.to_string().to_lowercase();

    if api_key.is_empty() && protocol != "ollama" && protocol != "local" {
        return Err(AppError::BadRequest(
            "Missing Secure API Key. Handshakes require authentication.".to_string(),
        ));
    }

    let router = ProtocolRouter::from_protocol(&protocol);
    let url = router.build_models_url(&protocol, base_url, false).await;

    validate_url_security(&url)?;

    let request = state.resources.http_client.get(&url);
    let mut request = router.authenticate_request(request, &api_key);

    if let Some(ref headers) = config.custom_headers {
        for (k, v) in headers {
            if let (Ok(h_name), Ok(h_val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                reqwest::header::HeaderValue::from_str(v),
            ) {
                request = request.header(h_name, h_val);
            }
        }
    }

    let send_fut = request.send();
    let res = tokio::time::timeout(Duration::from_secs(OUTBOUND_REQUEST_TIMEOUT_SECS), send_fut)
        .await
        .map_err(|_| {
            AppError::InternalServerError(format!(
                "Handshake timed out after {} seconds",
                OUTBOUND_REQUEST_TIMEOUT_SECS
            ))
        })?
        .map_err(|e| {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            AppError::InternalServerError(format!("Network Error: {}", safe_err))
        })?;

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

#[derive(serde::Deserialize)]
pub struct SyncModelsPayload {
    pub api_key: Option<String>,
}

/// Dynamic synchronization of models from a provider's discovery endpoint.
#[tracing::instrument(skip(state, payload), fields(provider_id = %id), name = "infra_providers::sync")]
pub async fn sync_provider_models(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<SyncModelsPayload>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("🔄 [IMR] Synchronizing models for provider: {}", id);

    let provider = state
        .registry
        .providers
        .get(&id)
        .ok_or_else(|| AppError::NotFound(format!("Provider '{}' not found", id)))?;

    let api_key = payload
        .api_key
        .filter(|k| !k.is_empty())
        .or_else(|| provider.api_key.clone())
        .unwrap_or_default();
    let base_url = provider.base_url.as_deref().unwrap_or("");
    let protocol = provider.protocol.to_string().to_lowercase();

    let router = ProtocolRouter::from_protocol(&protocol);
    let url = router.build_models_url(&protocol, base_url, true).await;

    validate_url_security(&url)?;

    let request = state.resources.http_client.get(&url);
    let request = router.authenticate_request(request, &api_key);

    let send_fut = request.send();
    let resp = tokio::time::timeout(Duration::from_secs(OUTBOUND_REQUEST_TIMEOUT_SECS), send_fut)
        .await
        .map_err(|_| {
            AppError::InternalServerError(format!(
                "Discovery timed out after {} seconds",
                OUTBOUND_REQUEST_TIMEOUT_SECS
            ))
        })?
        .map_err(|e| {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            AppError::InternalServerError(format!("Discovery network error: {}", safe_err))
        })?;

    if !resp.status().is_success() {
        return Err(AppError::Unauthorized(format!(
            "Discovery failed ({}): Provider rejected credentials or endpoint unreachable.",
            resp.status()
        )));
    }

    let body: serde_json::Value = resp.json().await.map_err(|_| {
        AppError::InternalServerError("Failed to parse discovery response.".to_string())
    })?;

    let discovered_ids = router.parse_discovery_response(&body, &url);
    let count = discovered_ids.len();
    let mut added = 0;

    for model_id in discovered_ids {
        let entry_id = format!("{}:{}", id, model_id.replace("/", "-"));
        if !state.registry.models.contains_key(&entry_id) {
            let capabilities = CapabilityMatrix::infer_capabilities(&model_id);
            let entry = ModelEntry {
                id: model_id.clone(),
                name: model_id.clone(),
                provider_id: id.clone(),
                provider: Some(provider.protocol),
                modality: if capabilities.supports_vision {
                    crate::agent::types::Modality::Vision
                } else {
                    crate::agent::types::Modality::Llm
                },
                capabilities,
                ..ModelEntry::default()
            };
            state.registry.models.insert(entry_id, entry);
            added += 1;
        }
    }

    if added > 0 {
        state.save_models().await?;
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "message": format!("Synchronized {} models ({} total discovered).", added, count),
            "added": added,
            "discovered": count
        })),
    ))
}

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

/// Proxies a model pull request to a specific swarm node.
#[tracing::instrument(skip(state, payload), fields(node_id = %payload.node_id, tag = %payload.tag), name = "infra_models::pull")]
pub async fn pull_model(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PullModelPayload>,
) -> Result<impl IntoResponse, AppError> {
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
        let _ = ollama_proxy_pull(State(Arc::clone(&state)), Json(ollama_payload)).await?;
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

    // Validate and normalize node address format before connecting
    let raw_address = node.address.trim();
    if raw_address.is_empty() || raw_address.contains('@') {
        return Err(AppError::BadRequest(
            "Node network address is invalid or malformed".to_string(),
        ));
    }

    let stripped_address = raw_address
        .strip_prefix("http://")
        .or_else(|| raw_address.strip_prefix("https://"))
        .unwrap_or(raw_address);

    let (host_port, _subpath) = match stripped_address.split_once('/') {
        Some((hp, rest)) => (hp, rest),
        None => (stripped_address, ""),
    };

    let clean_host_port = host_port.trim();
    if clean_host_port.is_empty()
        || clean_host_port.contains('@')
        || clean_host_port.contains('/')
        || clean_host_port.contains('\\')
    {
        return Err(AppError::BadRequest(
            "Node network address is invalid or malformed".to_string(),
        ));
    }

    let ollama_url = format!("http://{}/api/pull", clean_host_port);
    validate_url_security(&ollama_url)?;

    let ollama_payload = serde_json::json!({
        "name": payload.tag,
        "stream": false
    });

    let send_fut = state
        .resources
        .http_client
        .post(&ollama_url)
        .header(
            "Authorization",
            format!("Bearer {}", state.security.deploy_token),
        )
        .json(&ollama_payload)
        .send();

    let resp = tokio::time::timeout(Duration::from_secs(OUTBOUND_REQUEST_TIMEOUT_SECS), send_fut)
        .await
        .map_err(|_| {
            AppError::InternalServerError(format!(
                "Node pull request timed out after {} seconds",
                OUTBOUND_REQUEST_TIMEOUT_SECS
            ))
        })?
        .map_err(|e| {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            AppError::InternalServerError(format!("Failed to reach Bunker node: {}", safe_err))
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
#[tracing::instrument(skip(state, payload), name = "infra_models::ollama_proxy_pull")]
pub async fn ollama_proxy_pull(
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

    tracing::info!(
        "🔄 [ModelStore] Proxying pull request to local Ollama: {}",
        ollama_url
    );

    let send_fut = state
        .resources
        .http_client
        .post(&ollama_url)
        .json(&payload)
        .send();

    let resp = tokio::time::timeout(Duration::from_secs(OUTBOUND_REQUEST_TIMEOUT_SECS), send_fut)
        .await
        .map_err(|_| {
            AppError::InternalServerError(format!(
                "Local Ollama pull timed out after {} seconds",
                OUTBOUND_REQUEST_TIMEOUT_SECS
            ))
        })?
        .map_err(|e| {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            AppError::InternalServerError(format!("Failed to reach local Ollama: {}", safe_err))
        })?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_security_exact_host_validation() {
        assert!(validate_url_security("http://localhost:11434").is_ok());
        assert!(validate_url_security("http://127.0.0.1:8080").is_ok());
        assert!(validate_url_security("http://10.0.0.1:8000").is_ok());
        assert!(validate_url_security("http://10.0.0.1").is_ok());
        assert!(validate_url_security("https://api.openai.com/v1").is_ok());

        // Bypasses blocked
        assert!(validate_url_security("http://localhost.attacker.com/v1").is_err());
        assert!(validate_url_security("http://127.0.0.1.evil.com/v1").is_err());
        assert!(validate_url_security("http://external-api.com/v1").is_err());
    }

    #[tokio::test]
    async fn test_update_provider_preserves_existing_api_key() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let id = "test-provider".to_string();

        let initial_config = ProviderConfig {
            id: id.clone(),
            name: "Initial Provider".to_string(),
            protocol: crate::agent::types::ModelProvider::Openai,
            api_key: Some("super-secret-key-12345".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            icon: None,
            external_id: None,
            custom_headers: None,
            default_config: None,
            supports_steering_vectors: false,
            audio_model: None,
        };
        state.registry.providers.insert(id.clone(), initial_config);

        // Update without sending api_key (api_key: None)
        let update_req = ProviderConfig {
            id: id.clone(),
            name: "Updated Name".to_string(),
            protocol: crate::agent::types::ModelProvider::Openai,
            api_key: None,
            base_url: Some("https://api.openai.com/v1".to_string()),
            icon: None,
            external_id: None,
            custom_headers: None,
            default_config: None,
            supports_steering_vectors: false,
            audio_model: None,
        };

        let response = update_provider(
            crate::middleware::auth::RequireAdmin,
            State(state.clone()),
            Path(id.clone()),
            Json(update_req),
        )
        .await
        .expect("update succeeds");
        let into_resp = response.into_response();
        assert_eq!(into_resp.status(), StatusCode::OK);

        let stored = state.registry.providers.get(&id).unwrap();
        assert_eq!(stored.name, "Updated Name");
        assert_eq!(stored.api_key, Some("super-secret-key-12345".to_string()));
    }

    #[tokio::test]
    async fn test_update_provider_blocks_external_plain_http() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let id = "evil-provider".to_string();

        let evil_req = ProviderConfig {
            id: id.clone(),
            name: "Evil Provider".to_string(),
            protocol: crate::agent::types::ModelProvider::Openai,
            api_key: Some("test-key".to_string()),
            base_url: Some("http://attacker.example.com/v1".to_string()),
            icon: None,
            external_id: None,
            custom_headers: None,
            default_config: None,
            supports_steering_vectors: false,
            audio_model: None,
        };

        let res = update_provider(
            crate::middleware::auth::RequireAdmin,
            State(state),
            Path(id),
            Json(evil_req),
        )
        .await;

        match res {
            Err(AppError::BadRequest(msg)) => {
                assert!(msg.contains("Insecure transmission blocked"));
            }
            Err(other) => panic!("Expected BadRequest, got {:?}", other),
            Ok(_) => panic!("Expected error for external plain HTTP, but got Ok"),
        }
    }

    #[tokio::test]
    async fn test_protocol_router_ollama_url_normalization() {
        let router = ProtocolRouter::from_protocol("ollama");

        // When base_url contains /v1 and is_sync is false (handshake probe)
        let probe_url = router
            .build_models_url("ollama", "http://127.0.0.1:11434/v1", false)
            .await;
        assert_eq!(probe_url, "http://127.0.0.1:11434/api/version");

        // When base_url contains /v1 and is_sync is true (model sync)
        let sync_url = router
            .build_models_url("ollama", "http://127.0.0.1:11434/v1", true)
            .await;
        assert_eq!(sync_url, "http://127.0.0.1:11434/api/tags");

        // When base_url has no /v1
        let probe_clean = router
            .build_models_url("ollama", "http://127.0.0.1:11434", false)
            .await;
        assert_eq!(probe_clean, "http://127.0.0.1:11434/api/version");

        let sync_clean = router
            .build_models_url("ollama", "http://127.0.0.1:11434", true)
            .await;
        assert_eq!(sync_clean, "http://127.0.0.1:11434/api/tags");
    }

    #[tokio::test]
    async fn test_protocol_router_openai_local_11434() {
        let router = ProtocolRouter::from_protocol("openai");

        // For local Ollama port 11434 using openai protocol without /v1
        let models_url = router
            .build_models_url("openai", "http://127.0.0.1:11434", true)
            .await;
        assert_eq!(models_url, "http://127.0.0.1:11434/v1/models");

        // With /v1 already present
        let models_url_v1 = router
            .build_models_url("openai", "http://127.0.0.1:11434/v1", true)
            .await;
        assert_eq!(models_url_v1, "http://127.0.0.1:11434/v1/models");
    }
}
