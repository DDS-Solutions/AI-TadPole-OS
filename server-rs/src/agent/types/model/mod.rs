//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / model
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

pub mod capabilities;
pub mod config;
pub mod provider;

pub use capabilities::*;
pub use config::*;
pub use provider::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_provider_from_str() {
        assert_eq!(
            ModelProvider::from_str("openai"),
            Some(ModelProvider::Openai)
        );
        assert_eq!(
            ModelProvider::from_str("Open-AI"),
            Some(ModelProvider::Openai)
        );
        assert_eq!(
            ModelProvider::from_str("ANTHROPIC"),
            Some(ModelProvider::Anthropic)
        );
        assert_eq!(
            ModelProvider::from_str("claude-3"),
            Some(ModelProvider::Anthropic)
        );
        assert_eq!(
            ModelProvider::from_str("google-ai-studio"),
            Some(ModelProvider::Google)
        );
        assert_eq!(
            ModelProvider::from_str("ollama"),
            Some(ModelProvider::Ollama)
        );
        assert_eq!(ModelProvider::from_str("Groq"), Some(ModelProvider::Groq));
        assert_eq!(ModelProvider::from_str("unknown-provider"), None);
    }

    #[test]
    fn test_model_config_supports_native_tools() {
        let mut cfg = ModelConfig::default();
        cfg.model_id = "gpt-4o".to_string();
        assert!(cfg.supports_native_tools());

        cfg.model_id = "phi3-mini".to_string();
        assert!(!cfg.supports_native_tools());

        cfg.model_id = "Phi-3-medium".to_string();
        assert!(!cfg.supports_native_tools());

        cfg.model_id = "llama-3.1-70b".to_string();
        assert!(cfg.supports_native_tools());
    }

    #[test]
    fn test_from_model_id_inference() {
        assert_eq!(
            ModelProvider::from_model_id("gpt-4o"),
            Some(ModelProvider::Openai)
        );
        assert_eq!(
            ModelProvider::from_model_id("chatgpt-4o-latest"),
            Some(ModelProvider::Openai)
        );
        assert_eq!(
            ModelProvider::from_model_id("o1-mini"),
            Some(ModelProvider::Openai)
        );
        assert_eq!(
            ModelProvider::from_model_id("o3-mini"),
            Some(ModelProvider::Openai)
        );
        assert_eq!(
            ModelProvider::from_model_id("openai/custom-model"),
            Some(ModelProvider::Openai)
        );
        assert_eq!(
            ModelProvider::from_model_id("claude-3-5-sonnet-20241022"),
            Some(ModelProvider::Anthropic)
        );
        assert_eq!(
            ModelProvider::from_model_id("gemini-2.0-flash"),
            Some(ModelProvider::Gemini)
        );
        assert_eq!(
            ModelProvider::from_model_id("deepseek-chat"),
            Some(ModelProvider::Deepseek)
        );
        assert_eq!(
            ModelProvider::from_model_id("gemma-4:31b"),
            Some(ModelProvider::OllamaCloud)
        );
        assert_eq!(
            ModelProvider::from_model_id("mistral:7b-cloud"),
            Some(ModelProvider::OllamaCloud)
        );
        // Non-ollama cloud service names must not falsely trigger OllamaCloud
        assert_ne!(
            ModelProvider::from_model_id("google-cloud-vertex/model"),
            Some(ModelProvider::OllamaCloud)
        );
    }
}
