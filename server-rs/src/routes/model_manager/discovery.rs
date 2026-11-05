//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - Preserve route behavior and security checks when reorganizing model operations.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / model_manager provider discovery
//! - **Primary Entrypoints**: ProtocolRouter, test_provider, sync_provider_models
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::Unauthorized, AppError::InternalServerError
//! - **Telemetry Targets**: [IMR]
use super::{execute_with_timeout, get_ollama_host, validate_url_security};
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
use std::sync::Arc;

const DEFAULT_PROVIDER_URLS: &[(&str, &str)] = &[
    ("groq", "https://api.groq.com/openai/v1"),
    ("mistral", "https://api.mistral.ai/v1"),
    ("perplexity", "https://api.perplexity.ai"),
    ("fireworks", "https://api.fireworks.ai/inference/v1"),
    ("together", "https://api.together.xyz/v1"),
    ("deepseek", "https://api.deepseek.com/v1"),
    ("xai", "https://api.x.ai/v1"),
    ("openrouter", "https://openrouter.ai/api/v1"),
    ("cerebras", "https://api.cerebras.ai/v1"),
    ("sambanova", "https://api.sambanova.ai/v1"),
];

fn default_openai_compatible_url(protocol: &str) -> String {
    for &(proto, url) in DEFAULT_PROVIDER_URLS {
        if protocol.eq_ignore_ascii_case(proto) {
            return url.to_string();
        }
    }
    "https://api.openai.com/v1".to_string()
}

pub(super) enum ProtocolRouter {
    Anthropic,
    Google,
    OpenAI,
    Ollama,
}

impl ProtocolRouter {
    pub(super) fn from_protocol(protocol: &str) -> Self {
        match protocol {
            "anthropic" => Self::Anthropic,
            "google" | "gemini" => Self::Google,
            "ollama" => Self::Ollama,
            _ => Self::OpenAI,
        }
    }

    pub(super) async fn build_models_url(
        &self,
        protocol: &str,
        base_url: &str,
        is_sync: bool,
    ) -> String {
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
                    default_openai_compatible_url(protocol)
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

    fn parse_discovery_response(
        &self,
        body: &serde_json::Value,
        is_tags_endpoint: bool,
    ) -> Vec<String> {
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
                if is_tags_endpoint {
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

/// Tests a provider configuration by performing a dummy model list or balance check.
#[tracing::instrument(skip(_admin, state, config), fields(provider_id = %id), name = "infra_providers::test")]
pub async fn test_provider(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<ProviderConfig>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(e) = config.validate() {
        return Err(AppError::BadRequest(e));
    }

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

    let res = execute_with_timeout(request, &state, "Handshake").await?;

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
#[tracing::instrument(skip(_admin, state, payload), fields(provider_id = %id), name = "infra_providers::sync")]
pub async fn sync_provider_models(
    _admin: crate::middleware::auth::RequireAdmin,
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

    let resp = execute_with_timeout(request, &state, "Discovery").await?;

    if !resp.status().is_success() {
        return Err(AppError::Unauthorized(format!(
            "Discovery failed ({}): Provider rejected credentials or endpoint unreachable.",
            resp.status()
        )));
    }

    let body: serde_json::Value = resp.json().await.map_err(|_| {
        AppError::InternalServerError("Failed to parse discovery response.".to_string())
    })?;

    let discovered_ids = router.parse_discovery_response(&body, true);
    let count = discovered_ids.len();
    let mut added = 0;

    for model_id in discovered_ids {
        let entry_id = format!("{}:{}", id, model_id.replace("/", "-"));
        if !state.registry.models.contains_key(&entry_id) {
            let capabilities = CapabilityMatrix::infer_capabilities(&model_id);
            let entry = ModelEntry {
                id: entry_id.clone(),
                provider_model_id: Some(model_id.clone()),
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
