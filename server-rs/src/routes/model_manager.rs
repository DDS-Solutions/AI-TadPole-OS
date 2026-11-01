//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - Keep route exports stable while model operations are separated by responsibility.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / model_manager
//! - **Primary Entrypoints**: Provider, model, discovery, and local download handlers
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError mappings are defined in server-rs/src/error/metadata.rs.
//! - **Telemetry Targets**: none declared
use crate::{error::AppError, state::AppState};
use std::time::Duration;

pub const OUTBOUND_REQUEST_TIMEOUT_SECS: u64 = 30;

async fn execute_with_timeout(
    request: reqwest::RequestBuilder,
    state: &AppState,
    operation_name: &str,
) -> Result<reqwest::Response, AppError> {
    let send_fut = request.send();
    tokio::time::timeout(Duration::from_secs(OUTBOUND_REQUEST_TIMEOUT_SECS), send_fut)
        .await
        .map_err(|_| {
            AppError::InternalServerError(format!(
                "{} timed out after {} seconds",
                operation_name, OUTBOUND_REQUEST_TIMEOUT_SECS
            ))
        })?
        .map_err(|e| {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            AppError::InternalServerError(format!("{} network error: {}", operation_name, safe_err))
        })
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

mod discovery;
mod downloads;
mod models;
mod providers;

pub use discovery::{sync_provider_models, test_provider, SyncModelsPayload};
pub use downloads::{get_model_catalog, ollama_proxy_pull, pull_model, PullModelPayload};
pub use models::{delete_model, get_models, update_model};
pub use providers::{delete_provider, get_providers, update_provider, ProviderResponse};

#[cfg(test)]
use crate::agent::types::{ProviderConfig, Validatable};
#[cfg(test)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
#[cfg(test)]
use discovery::ProtocolRouter;
#[cfg(test)]
use std::sync::Arc;

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

    #[test]
    fn test_pull_model_tag_validation() {
        // Valid tags
        let valid_payload = PullModelPayload {
            node_id: "local".to_string(),
            tag: "llama3:latest".to_string(),
        };
        assert!(valid_payload.validate().is_ok());

        let valid_nested = PullModelPayload {
            node_id: "node-1".to_string(),
            tag: "library/deepseek-r1:8b_v1.0".to_string(),
        };
        assert!(valid_nested.validate().is_ok());

        // Invalid: empty tag
        let empty_tag = PullModelPayload {
            node_id: "local".to_string(),
            tag: "   ".to_string(),
        };
        assert!(empty_tag.validate().is_err());

        // Invalid: path traversal
        let traversal_tag = PullModelPayload {
            node_id: "local".to_string(),
            tag: "../../etc/shadow".to_string(),
        };
        assert!(traversal_tag.validate().is_err());

        // Invalid: special shell characters
        let injection_tag = PullModelPayload {
            node_id: "local".to_string(),
            tag: "llama3;rm -rf /".to_string(),
        };
        assert!(injection_tag.validate().is_err());

        // Invalid: empty node_id
        let empty_node = PullModelPayload {
            node_id: "".to_string(),
            tag: "llama3".to_string(),
        };
        assert!(empty_node.validate().is_err());
    }

    #[tokio::test]
    async fn test_update_provider_synchronizes_mismatched_id() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let path_id = "canonical-provider-id".to_string();

        let req_with_mismatched_id = ProviderConfig {
            id: "attacker-chosen-mismatched-id".to_string(),
            name: "Mismatched Provider".to_string(),
            protocol: crate::agent::types::ModelProvider::Openai,
            api_key: Some("secret-key".to_string()),
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
            Path(path_id.clone()),
            Json(req_with_mismatched_id),
        )
        .await
        .expect("update succeeds");

        let into_resp = response.into_response();
        assert_eq!(into_resp.status(), StatusCode::OK);

        // Verify stored provider's internal ID matches the registry path ID
        let stored = state.registry.providers.get(&path_id).unwrap();
        assert_eq!(stored.id, path_id);
    }
}
