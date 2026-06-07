//! Shared model routing helpers for local-first and Privacy Shield decisions.

use crate::agent::types::{ModelConfig, ModelProvider};

pub const PRIVACY_FALLBACK_MODEL: &str = "gemma4:e4b";

/// Checks whether a provider/url pair targets a local endpoint.
pub(crate) fn is_local_endpoint(provider: &ModelProvider, url_opt: Option<&str>) -> bool {
    if matches!(provider, ModelProvider::Ollama) {
        return true;
    }

    let Some(url) = url_opt else {
        return false;
    };
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };

    let host_lower = host.to_lowercase();
    if host_lower == "localhost" || host_lower == "host.docker.internal" {
        return true;
    }

    let host_clean = host_lower
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(&host_lower);

    if let Ok(ip) = host_clean.parse::<std::net::IpAddr>() {
        return match ip {
            std::net::IpAddr::V4(ipv4) => {
                ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local()
            }
            std::net::IpAddr::V6(ipv6) => ipv6.is_loopback(),
        };
    }

    false
}

/// Checks whether a full model config targets a local endpoint.
pub(crate) fn is_local_model_config(config: &ModelConfig) -> bool {
    is_local_endpoint(&config.provider, config.base_url.as_deref())
}

/// Deterministic local fallback used when Privacy Shield is active and no
/// configured local slot is available.
pub(crate) fn privacy_fallback_config() -> ModelConfig {
    ModelConfig {
        provider: ModelProvider::Ollama,
        model_id: PRIVACY_FALLBACK_MODEL.to_string(),
        api_key: None,
        base_url: None,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_is_local_with_or_without_base_url() {
        assert!(is_local_endpoint(&ModelProvider::Ollama, None));
        assert!(is_local_endpoint(
            &ModelProvider::Ollama,
            Some("https://example.com")
        ));
    }

    #[test]
    fn test_openai_compatible_local_urls_are_local() {
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://localhost:11434")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://host.docker.internal:11434")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://127.0.0.1:1234/v1")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://10.0.0.1:11434")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://10.0.0.1:11434")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://10.0.0.1:11434")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://[::1]:11434")
        ));
    }

    #[test]
    fn test_cloud_urls_and_missing_base_url_are_not_local() {
        assert!(!is_local_endpoint(
            &ModelProvider::Openai,
            Some("https://api.openai.com/v1")
        ));
        assert!(!is_local_endpoint(
            &ModelProvider::Gemini,
            Some("https://generativelanguage.googleapis.com")
        ));
        assert!(!is_local_endpoint(&ModelProvider::Openai, None));
    }

    #[test]
    fn test_privacy_fallback_config_is_ollama_gemma4() {
        let config = privacy_fallback_config();
        assert_eq!(config.provider, ModelProvider::Ollama);
        assert_eq!(config.model_id, PRIVACY_FALLBACK_MODEL);
        assert!(config.api_key.is_none());
        assert!(config.base_url.is_none());
    }
}
