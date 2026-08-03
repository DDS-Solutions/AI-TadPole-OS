//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **SEC-02 Infrastructure Config Persistence**: Handles loading and saving of `infra_providers.json` and `infra_models.json`. Enforces Credential Polarization by prioritizing environment variables over disk-based JSON secrets.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Failed JSON serialization or env var override parsing mismatch.
//! - **Telemetry Link**: Search for `[InfraConfig]` in server logs.

use crate::agent::types::{ModelEntry, ProviderConfig};
use crate::error::AppError;

const PROVIDERS_FILE: &str = "data/infra_providers.json";
const MODELS_FILE: &str = "data/infra_models.json";

pub(crate) async fn read_json_file<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Option<T> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&content).ok()
}

/// ### 🔒 SEC-02: Credential Polarization Helper
/// Maps a protocol-specific provider ID to its canonical environment variable.
///
/// Returns `None` for providers that do not require secret keys or utilize
/// alternative authentication mechanisms (e.g., local Ollama instance).
pub(crate) fn provider_env_var(provider_id: &str) -> Option<&'static str> {
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

pub(crate) fn is_placeholder(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    key_lower.starts_with("your-")
        || key_lower.contains("api-key-here")
        || key_lower.contains("placeholder")
}

/// Loads provider configurations from disk and overlays security context.
///
/// ### 🔒 SEC-02: Credential Polarization
/// API keys are NEVER loaded from the raw JSON file if a corresponding environment
/// variable (e.g., `GOOGLE_API_KEY`) is present. Environment variables are treated
/// as the Sovereign Root of Truth for credentials.
pub async fn load_providers(base_dir: &std::path::Path) -> Vec<ProviderConfig> {
    let providers_file = crate::utils::security::validate_path(base_dir, PROVIDERS_FILE)
        .unwrap_or_else(|_| {
            crate::utils::security::SafePath::from_trusted(base_dir.join(PROVIDERS_FILE))
        });
    let mut providers = if providers_file.exists() {
        read_json_file::<Vec<ProviderConfig>>(&providers_file)
            .await
            .unwrap_or_else(|| {
                tracing::error!(
                    file = ?providers_file,
                    "❌ [Persistence] Provider JSON parse failure — falling back to defaults"
                );
                crate::agent::registry::get_default_providers()
            })
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
                    provider.api_key = Some(trimmed.to_string());
                } else if is_placeholder(trimmed) {
                    tracing::warn!(
                        "⚠️ [Auth] Ignored placeholder API key for provider '{}' (from env '{}')",
                        provider.id,
                        env_var
                    );
                }
            }
        }
    }

    providers
}

/// ### 🔒 SEC-02: Credential Redaction Pass
/// Persists provider configurations to disk after sanitizing sensitive tokens.
///
/// ### 🛰️ Security Note: Identity Leakage Prevention
/// API keys are stripped and replaced with `serde_json::Value::Null` before disk write.
/// This ensures that repo-wide exports or cloud backups do not accidentally contain
/// the `NEURAL_TOKEN` or cloud provider secrets.
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
    tokio::fs::write(providers_file, content)
        .await
        .map_err(AppError::Io)?;
    Ok(())
}

/// Loads the model registry from disk.
pub async fn load_models(base_dir: &std::path::Path) -> Vec<ModelEntry> {
    let models_file =
        crate::utils::security::validate_path(base_dir, MODELS_FILE).unwrap_or_else(|_| {
            crate::utils::security::SafePath::from_trusted(base_dir.join(MODELS_FILE))
        });
    if models_file.exists() {
        if let Some(models) = read_json_file::<Vec<ModelEntry>>(&models_file).await {
            return models;
        }
        tracing::error!(file = ?models_file, "❌ [Persistence] Model JSON parse failure — falling back to defaults");
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

/// Persists all model entries to disk.
pub async fn save_models(
    base_dir: &std::path::Path,
    models: Vec<ModelEntry>,
) -> Result<(), AppError> {
    let models_file = crate::utils::security::validate_path(base_dir, MODELS_FILE)?;
    let content = serde_json::to_string_pretty(&models)?;
    tokio::fs::write(models_file, content)
        .await
        .map_err(AppError::Io)?;
    Ok(())
}
