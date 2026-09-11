//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / infra_config
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::{ModelEntry, ProviderConfig};
use crate::error::AppError;
use tracing::{debug, error, info, warn};

const PROVIDERS_FILE: &str = "data/infra_providers.json";
const MODELS_FILE: &str = "data/infra_models.json";

pub(crate) async fn read_json_file<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> Option<T> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&content).ok()
}

/// ### 🔒 SEC-02: Credential Polarization Helper
/// Maps a protocol-specific provider ID to its canonical environment variable.
///
/// Returns `None` for providers that do not require secret keys or utilize
/// alternative authentication mechanisms (e.g., local Ollama instance).
pub fn provider_env_var(provider_id: &str) -> Option<&'static str> {
    match provider_id {
        "google" | "gemini" => Some("GOOGLE_API_KEY"),
        "groq" => Some("GROQ_API_KEY"),
        "openai" => Some("OPENAI_API_KEY"),
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "inception" => Some("INCEPTION_API_KEY"),
        "deepseek" => Some("DEEPSEEK_API_KEY"),
        "ollama-cloud" => Some("OLLAMA_CLOUD_API_KEY"),
        _ => None,
    }
}

pub fn is_placeholder(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    key_lower.starts_with("your-")
        || key_lower.contains("api-key-here")
        || key_lower.contains("placeholder")
        || key_lower == "sk-..."
}

/// Loads provider configurations from disk and overlays security context.
///
/// ### 🔒 SEC-02: Credential Polarization
/// API keys are NEVER loaded from the raw JSON file if a corresponding environment
/// variable (e.g., `GOOGLE_API_KEY`) is present. Environment variables are treated
/// as the Sovereign Root of Truth for credentials.
pub async fn load_providers(base_dir: &std::path::Path) -> Vec<ProviderConfig> {
    let providers_file = match crate::utils::security::validate_path(base_dir, PROVIDERS_FILE) {
        Ok(p) => p,
        Err(e) => {
            warn!(
                "⚠️ [Persistence] Refusing to load providers from untrusted path {:?}: {}",
                base_dir.join(PROVIDERS_FILE),
                e
            );
            return crate::agent::registry::get_default_providers();
        }
    };

    let mut providers = if providers_file.exists() {
        match read_json_file::<Vec<ProviderConfig>>(&providers_file).await {
            Some(mut list) => {
                // Filter placeholder keys from disk
                for p in &mut list {
                    if let Some(ref k) = p.api_key {
                        if is_placeholder(k) {
                            debug!("Ignored placeholder disk key for provider '{}'", p.id);
                            p.api_key = None;
                        }
                    }
                }
                list
            }
            None => {
                error!(
                    file = ?providers_file,
                    "❌ [Persistence] Provider JSON parse failure — falling back to defaults"
                );
                crate::agent::registry::get_default_providers()
            }
        }
    } else {
        // Fallback: Check RESOURCE_ROOT (e.g. bundled data)
        let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
        let bundled_file = std::path::Path::new(&resource_root).join(PROVIDERS_FILE);
        read_json_file::<Vec<ProviderConfig>>(&bundled_file)
            .await
            .unwrap_or_else(crate::agent::registry::get_default_providers)
    };

    // SEC-02: Override api_key from environment variables.
    for provider in &mut providers {
        if let Some(env_var) = provider_env_var(&provider.id) {
            if let Ok(key) = std::env::var(env_var) {
                let trimmed = key.trim();
                if !trimmed.is_empty() && !is_placeholder(trimmed) {
                    info!(
                        "[Auth] Environment variable '{}' successfully provided active credentials for provider '{}'",
                        env_var, provider.id
                    );
                    provider.api_key = Some(trimmed.to_string());
                } else if is_placeholder(trimmed) {
                    warn!(
                        "⚠️ [Auth] Ignored placeholder API key for provider '{}' (from env '{}')",
                        provider.id, env_var
                    );
                }
            }
        }
    }

    providers
}

/// ### 🔒 SEC-02: Credential Redaction Pass & Atomic Persistence
/// Persists provider configurations to disk after sanitizing sensitive tokens.
///
/// ### 🛰️ Security Note: Identity Leakage Prevention
/// API keys are stripped and replaced with `serde_json::Value::Null` before disk write.
/// Writes atomically via temp file rename.
pub async fn save_providers(
    base_dir: &std::path::Path,
    providers: Vec<ProviderConfig>,
) -> Result<(), AppError> {
    let providers_file = crate::utils::security::validate_path(base_dir, PROVIDERS_FILE)?;
    let sanitized: Vec<serde_json::Value> = providers
        .iter()
        .map(|p| {
            let mut val = serde_json::to_value(p).unwrap_or_default();
            if let Some(obj) = val.as_object_mut() {
                obj.insert("api_key".to_string(), serde_json::Value::Null);
                obj.remove("apiKey");
            }
            val
        })
        .collect();

    let content = serde_json::to_string_pretty(&sanitized)?;
    let tmp_file = providers_file.with_extension("tmp");
    tokio::fs::write(&tmp_file, content)
        .await
        .map_err(AppError::Io)?;
    tokio::fs::rename(&tmp_file, providers_file)
        .await
        .map_err(AppError::Io)?;

    Ok(())
}

/// Loads the model registry from disk.
pub async fn load_models(base_dir: &std::path::Path) -> Vec<ModelEntry> {
    let models_file = match crate::utils::security::validate_path(base_dir, MODELS_FILE) {
        Ok(p) => p,
        Err(e) => {
            warn!(
                "⚠️ [Persistence] Refusing to load models from untrusted path {:?}: {}",
                base_dir.join(MODELS_FILE),
                e
            );
            return crate::agent::registry::get_default_models();
        }
    };

    if models_file.exists() {
        if let Some(models) = read_json_file::<Vec<ModelEntry>>(&models_file).await {
            return models;
        }
        error!(file = ?models_file, "❌ [Persistence] Model JSON parse failure — falling back to defaults");
    } else {
        // Fallback: Check RESOURCE_ROOT
        let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
        let bundled_file = std::path::Path::new(&resource_root).join(MODELS_FILE);
        if let Some(models) = read_json_file::<Vec<ModelEntry>>(&bundled_file).await {
            return models;
        }
    }
    crate::agent::registry::get_default_models()
}

/// Persists the model registry to disk atomically.
pub async fn save_models(
    base_dir: &std::path::Path,
    models: Vec<ModelEntry>,
) -> Result<(), AppError> {
    let models_file = crate::utils::security::validate_path(base_dir, MODELS_FILE)?;
    let content = serde_json::to_string_pretty(&models)?;
    let tmp_file = models_file.with_extension("tmp");
    tokio::fs::write(&tmp_file, content)
        .await
        .map_err(AppError::Io)?;
    tokio::fs::rename(&tmp_file, models_file)
        .await
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_credential_redaction_and_atomic_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path();
        std::fs::create_dir_all(base_path.join("data")).unwrap();

        let providers = vec![ProviderConfig {
            id: "google".to_string(),
            name: "Google AI".to_string(),
            icon: None,
            protocol: crate::agent::types::ModelProvider::Google,
            base_url: None,
            api_key: Some("secret-key-12345".to_string()),
            external_id: None,
            custom_headers: None,
            default_config: None,
            supports_steering_vectors: false,
            audio_model: None,
        }];

        save_providers(base_path, providers).await.unwrap();

        // Read saved file directly and verify api_key was stripped to null
        let saved_json: Vec<serde_json::Value> =
            read_json_file(&base_path.join("data/infra_providers.json"))
                .await
                .unwrap();
        assert_eq!(saved_json.len(), 1);
        assert!(saved_json[0].get("api_key").unwrap().is_null());
    }

    #[test]
    fn test_is_placeholder() {
        assert!(is_placeholder("your-api-key-here"));
        assert!(is_placeholder("YOUR-GOOGLE-KEY"));
        assert!(is_placeholder("placeholder_key"));
        assert!(is_placeholder("sk-..."));
        assert!(!is_placeholder("AIzaSyD-actual-api-key-12345"));
    }
}
