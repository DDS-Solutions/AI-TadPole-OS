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
    DEFAULT_PROVIDER_URLS
        .iter()
        .find(|&&(proto, _)| protocol.eq_ignore_ascii_case(proto))
        .map(|&(_, url)| url.to_string())
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
}

async fn resolve_base_or_default(base_url: &str, default_url: &str) -> String {
    if base_url.is_empty() {
        default_url.to_string()
    } else {
        crate::networking::resolver::AddressResolver::resolve_url_if_local(base_url).await
    }
}

async fn build_ollama_models_url(base_url: &str, is_sync: bool) -> String {
    let base = if base_url.is_empty() {
        get_ollama_host().await
    } else {
        crate::networking::resolver::AddressResolver::resolve_url_if_local(base_url).await
    };
    let clean = base.trim_end_matches('/');
    let base_no_v1 = clean
        .strip_suffix("/v1")
        .unwrap_or(clean)
        .trim_end_matches('/');
    let suffix = if is_sync { "api/tags" } else { "api/version" };
    format!("{}/{}", base_no_v1, suffix)
}

async fn build_openai_models_url(protocol: &str, base_url: &str) -> String {
    let raw = if base_url.is_empty() {
        default_openai_compatible_url(protocol)
    } else {
        crate::networking::resolver::AddressResolver::resolve_url_if_local(base_url).await
    };
    let base = raw.trim_end_matches('/');
    if base.ends_with("/models") {
        base.to_string()
    } else if base.contains(":11434") && !base.ends_with("/v1") {
        format!("{}/v1/models", base)
    } else {
        format!("{}/models", base)
    }
}

fn extract_string_field_list(
    body: &serde_json::Value,
    array_key: &str,
    item_key: &str,
) -> Vec<String> {
    let Some(items) = body.get(array_key).and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|m| m.get(item_key).and_then(|v| v.as_str()).map(str::to_string))
        .collect()
}

fn parse_ollama_discovery(body: &serde_json::Value, is_tags_endpoint: bool) -> Vec<String> {
    if is_tags_endpoint {
        return extract_string_field_list(body, "models", "name");
    }
    let data_ids = extract_string_field_list(body, "data", "id");
    if !data_ids.is_empty() {
        data_ids
    } else {
        extract_string_field_list(body, "models", "name")
    }
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
                let base = resolve_base_or_default(base_url, "https://api.anthropic.com").await;
                format!("{}/v1/models", base.trim_end_matches('/'))
            }
            Self::Google => {
                let default_url = "https://generativelanguage.googleapis.com";
                let base = resolve_base_or_default(base_url, default_url).await;
                format!("{}/v1beta/models", base.trim_end_matches('/'))
            }
            Self::Ollama => build_ollama_models_url(base_url, is_sync).await,
            Self::OpenAI => build_openai_models_url(protocol, base_url).await,
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
        match self {
            Self::Anthropic | Self::OpenAI => extract_string_field_list(body, "data", "id"),
            Self::Google => extract_string_field_list(body, "models", "name")
                .into_iter()
                .map(|n| n.replace("models/", ""))
                .collect(),
            Self::Ollama => parse_ollama_discovery(body, is_tags_endpoint),
        }
    }
}

fn apply_custom_headers(
    mut request: reqwest::RequestBuilder,
    custom_headers: &Option<std::collections::HashMap<String, String>>,
) -> reqwest::RequestBuilder {
    let Some(headers) = custom_headers else {
        return request;
    };
    for (k, v) in headers {
        let Ok(h_name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) else {
            continue;
        };
        let Ok(h_val) = reqwest::header::HeaderValue::from_str(v) else {
            continue;
        };
        request = request.header(h_name, h_val);
    }
    request
}

fn resolve_provider_api_key(provided: Option<&str>, state: &AppState, provider_id: &str) -> String {
    provided
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .or_else(|| {
            state
                .registry
                .providers
                .get(provider_id)
                .and_then(|p| p.api_key.clone())
        })
        .unwrap_or_default()
}

/// Tests a provider configuration by performing a dummy model list or balance check.
#[tracing::instrument(skip(_admin, state, config), fields(provider_id = %id), name = "infra_providers::test")]
pub async fn test_provider(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<ProviderConfig>,
) -> Result<impl IntoResponse, AppError> {
    config.validate().map_err(AppError::BadRequest)?;
    tracing::info!("🔍 [Test Trace] Initiating handshake for provider: {}", id);

    let api_key = resolve_provider_api_key(config.api_key.as_deref(), &state, &id);
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
    let request = router.authenticate_request(request, &api_key);
    let request = apply_custom_headers(request, &config.custom_headers);

    let res = execute_with_timeout(request, &state, "Handshake").await?;
    if !res.status().is_success() {
        let err_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        let safe_err = state.security.secret_redactor.redact(&err_text);
        tracing::warn!("❌ [Test Trace] Handshake failed for {}: {}", id, safe_err);
        return Err(AppError::Unauthorized(format!(
            "Handshake failed: {}",
            safe_err
        )));
    }

    tracing::info!("✅ [Test Trace] Handshake successful for {}", id);
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "message": format!("Handshake successful: {} endpoint is reactive.", config.name)
        })),
    ))
}

#[derive(serde::Deserialize)]
pub struct SyncModelsPayload {
    pub api_key: Option<String>,
}

fn upsert_discovered_models(
    state: &AppState,
    provider_id: &str,
    protocol: crate::agent::types::ModelProvider,
    discovered_ids: Vec<String>,
) -> usize {
    let mut added = 0;
    for model_id in discovered_ids {
        let entry_id = format!("{}:{}", provider_id, model_id.replace('/', "-"));
        if state.registry.models.contains_key(&entry_id) {
            continue;
        }
        let capabilities = CapabilityMatrix::infer_capabilities(&model_id);
        let modality = if capabilities.supports_vision {
            crate::agent::types::Modality::Vision
        } else {
            crate::agent::types::Modality::Llm
        };
        let entry = ModelEntry {
            id: entry_id.clone(),
            provider_model_id: Some(model_id.clone()),
            name: model_id,
            provider_id: provider_id.to_string(),
            provider: Some(protocol),
            modality,
            capabilities,
            ..ModelEntry::default()
        };
        state.registry.models.insert(entry_id, entry);
        added += 1;
    }
    added
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
    let base_url = provider.base_url.clone().unwrap_or_default();
    let protocol_enum = provider.protocol;
    let protocol = protocol_enum.to_string().to_lowercase();
    drop(provider);

    let router = ProtocolRouter::from_protocol(&protocol);
    let url = router.build_models_url(&protocol, &base_url, true).await;
    validate_url_security(&url)?;

    let request = router.authenticate_request(state.resources.http_client.get(&url), &api_key);
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
    let added = upsert_discovered_models(&state, &id, protocol_enum, discovered_ids);

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
