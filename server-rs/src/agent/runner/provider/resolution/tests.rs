//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / resolution / tests
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none

use super::*;
use crate::agent::model_routing::is_local_endpoint;
use crate::agent::types::{ModelConfig, ModelProvider};

#[test]
fn test_extract_model_param_size() {
    assert_eq!(extract_model_param_size("gemma4:12b"), Some(12.0));
    assert_eq!(extract_model_param_size("llama3:70b-instruct"), Some(70.0));
    assert_eq!(extract_model_param_size("qwen2.5:32b"), Some(32.0));
    assert_eq!(extract_model_param_size("phi3:3.8b"), Some(3.8));
    assert_eq!(extract_model_param_size("gemma4:e4b"), Some(4.0));
    assert_eq!(extract_model_param_size("mixtral:8x7b"), Some(56.0));
    assert_eq!(extract_model_param_size("mixtral:8x22b"), Some(176.0));
    assert_eq!(extract_model_param_size("llama3.1:405b"), Some(405.0));
    assert_eq!(extract_model_param_size("model:15.0b"), Some(15.0));
    assert_eq!(extract_model_param_size("model:15.1b"), Some(15.1));
    assert_eq!(extract_model_param_size("model:16b"), Some(16.0));
    assert_eq!(extract_model_param_size("phi3.5-safe:latest"), None);
}

#[test]
fn test_is_oversized_model() {
    assert!(is_oversized_model("llama3:70b"));
    assert!(is_oversized_model("qwen2.5:32b"));
    assert!(is_oversized_model("qwen2.5:72b"));
    assert!(is_oversized_model("llama3.1:405b"));
    assert!(is_oversized_model("nemotron-3.5-lightning:latest"));
    assert!(is_oversized_model("mixtral:8x7b"));
    assert!(is_oversized_model("mixtral:8x22b"));
    assert!(is_oversized_model("model:15.1b"));
    assert!(is_oversized_model("model:16b"));
    assert!(is_oversized_model("deepseek-r1:latest"));

    // Safe models (<= 15.0B)
    assert!(!is_oversized_model("model:15.0b"));
    assert!(!is_oversized_model("gemma4:12b"));
    assert!(!is_oversized_model("gemma4:e4b"));
    assert!(!is_oversized_model("phi3.5-safe:latest"));
    assert!(!is_oversized_model("llama3.2:3b"));
    assert!(!is_oversized_model("llama3:latest"));
    assert!(!is_oversized_model("qwen2.5:latest"));
}

#[test]
fn test_estimate_model_param_size_untagged_and_oversized() {
    // Known family defaults
    assert_eq!(estimate_model_param_size("llama3:latest"), Some(8.0));
    assert_eq!(estimate_model_param_size("llama3.2:latest"), Some(3.0));
    assert_eq!(estimate_model_param_size("mistral:latest"), Some(7.0));
    assert_eq!(estimate_model_param_size("qwen2.5:latest"), Some(7.0));
    assert_eq!(estimate_model_param_size("phi3.5-safe:latest"), Some(3.8));

    // Untagged unvetted model returns None and is treated as oversized/unsafe
    assert_eq!(
        estimate_model_param_size("unknown-custom-model:latest"),
        None
    );
    assert!(is_oversized_model("unknown-custom-model:latest"));

    // 15.0B boundary check
    assert!(!is_oversized_model("test-model:15b"));
    assert!(!is_oversized_model("test-model:15.0b"));
    assert!(is_oversized_model("test-model:15.1b"));
    assert!(is_oversized_model("test-model:16b"));
}

#[test]
fn test_select_best_fallback_model_prefers_phi35_safe() {
    let candidates = [
        "nemotron-3.5-lightning:latest",
        "phi3.5-safe:latest",
        "gemma4:12b",
    ];
    let chosen = select_best_fallback_model(candidates.iter().copied(), None);
    assert_eq!(chosen, Some("phi3.5-safe:latest".to_string()));
}

#[test]
fn test_select_best_fallback_model_case_insensitive() {
    let candidates = [
        "Nemotron-3.5-Lightning:latest",
        "Phi3.5-Safe:latest",
        "gemma4:12b",
    ];
    let chosen = select_best_fallback_model(candidates.iter().copied(), None);
    assert_eq!(chosen, Some("Phi3.5-Safe:latest".to_string()));
}

#[test]
fn test_select_best_fallback_model_preserves_requested_safe_model() {
    let candidates = ["phi3.5-safe:latest", "gemma4:12b"];
    // Requested model is gemma4:12b-cloud; matches local gemma4:12b
    let chosen = select_best_fallback_model(candidates.iter().copied(), Some("gemma4:12b-cloud"));
    assert_eq!(chosen, Some("gemma4:12b".to_string()));

    // Slug matching: openai/gpt-4o matches local gpt-4o
    let candidates_slug = ["gpt-4o", "phi3.5-safe:latest"];
    let chosen_slug =
        select_best_fallback_model(candidates_slug.iter().copied(), Some("openai/gpt-4o"));
    assert_eq!(chosen_slug, Some("gpt-4o".to_string()));
}

#[test]
fn test_select_best_fallback_model_rejects_all_oversized() {
    let candidates = [
        "llama3:70b",
        "qwen2.5:32b",
        "llama3.1:405b",
        "nemotron-3.5-lightning:latest",
        "unknown-untagged-model:latest",
    ];
    let chosen = select_best_fallback_model(candidates.iter().copied(), None);
    assert_eq!(chosen, None);
}

#[test]
fn test_select_best_fallback_model_ignores_embedding_models_and_handles_bert_boundary() {
    let candidates = [
        "nomic-embed-text:latest",
        "bge-large:latest",
        "distilbert-base:latest",
        "robert-chat:latest",
        "eclipse:latest",
    ];
    let chosen = select_best_fallback_model(candidates.iter().copied(), None);
    // robert-chat and eclipse are recognized chat models; eclipse should not collide with clip
    assert!(
        chosen == Some("robert-chat:latest".to_string())
            || chosen == Some("eclipse:latest".to_string())
    );
}

#[test]
fn test_alignment_preserves_local_and_custom_gateway() {
    // 1. Local endpoint with Claude model ID must NOT mutate provider
    let local_config = ModelConfig {
        provider: ModelProvider::Openai,
        model_id: "claude-3-5-sonnet".to_string(),
        base_url: Some("http://127.0.0.1:8000/v1".to_string()),
        ..Default::default()
    };
    assert!(is_local_endpoint(
        &local_config.provider,
        local_config.base_url.as_deref()
    ));
    assert!(!is_official_openai_endpoint(
        local_config.base_url.as_deref()
    ));

    // 2. Custom gateway endpoint must NOT mutate provider
    let gateway_config = ModelConfig {
        provider: ModelProvider::Openai,
        model_id: "claude-3-5-sonnet".to_string(),
        base_url: Some("https://llm-gateway.corp.net/v1".to_string()),
        ..Default::default()
    };
    assert!(!is_official_openai_endpoint(
        gateway_config.base_url.as_deref()
    ));

    // 3. Official cloud OpenAI endpoint with Claude model ID DOES qualify for auto-alignment
    let cloud_config = ModelConfig {
        provider: ModelProvider::Openai,
        model_id: "claude-3-5-sonnet".to_string(),
        base_url: Some("https://api.openai.com/v1".to_string()),
        ..Default::default()
    };
    assert!(is_official_openai_endpoint(
        cloud_config.base_url.as_deref()
    ));
}

#[test]
fn test_construct_probe_urls_preserves_subpaths() {
    // Standard /v1 URL
    let (models, tags) = construct_probe_urls("http://127.0.0.1:11434/v1");
    assert_eq!(models, "http://127.0.0.1:11434/v1/models");
    assert_eq!(tags, "http://127.0.0.1:11434/api/tags");

    // Base URL without /v1
    let (models, tags) = construct_probe_urls("http://127.0.0.1:11434");
    assert_eq!(models, "http://127.0.0.1:11434/v1/models");
    assert_eq!(tags, "http://127.0.0.1:11434/api/tags");

    // Subpath proxy with /v1
    let (models, tags) = construct_probe_urls("http://gateway.local:8080/ollama/v1");
    assert_eq!(models, "http://gateway.local:8080/ollama/v1/models");
    assert_eq!(tags, "http://gateway.local:8080/ollama/api/tags");

    // Subpath proxy without /v1
    let (models, tags) = construct_probe_urls("http://gateway.local:8080/ollama");
    assert_eq!(models, "http://gateway.local:8080/ollama/v1/models");
    assert_eq!(tags, "http://gateway.local:8080/ollama/api/tags");
}

#[test]
fn test_select_best_fallback_model_none_when_empty() {
    let candidates: [&str; 0] = [];
    let chosen = select_best_fallback_model(candidates.iter().copied(), None);
    assert_eq!(chosen, None);
}
