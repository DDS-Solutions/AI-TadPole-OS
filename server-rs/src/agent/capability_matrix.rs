//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Assist Note
//! **Inference Engine**: Conservative pattern-matching logic to detect model 
//! capabilities (Vision, Tools, Reasoning) based on model ID slugs. 
//! Part of IMR-01 (Intelligent Model Registry).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Fallback to conservative defaults (No Vision/Tools) if 
//!   pattern matching fails.
//! - **Trace Scope**: `server-rs::agent::capability_matrix`
//!
use crate::agent::types::ModelCapabilities;

/// Baseline capability engine for the Intelligent Model Registry.
/// Provides a static "source of truth" for model capabilities based on well-known IDs.
/// Part of IMR-01.
pub struct CapabilityMatrix;

impl CapabilityMatrix {
    /// Infers capabilities for a model based on its ID slug.
    /// Handles both cloud (OpenAI, Gemini) and local (Ollama, vLLM) naming conventions.
    pub fn infer_capabilities(model_id: &str) -> ModelCapabilities {
        let id = model_id.to_lowercase();
        let mut caps = ModelCapabilities {
            context_window: 32_768,
            max_output_tokens: 4_096,
            supports_tools: true,
            ..ModelCapabilities::default()
        };

        // --- Tool Support (Native Function Calling) ---
        // Generally True for modern models EXCEPT specific small ones or old ones
        if id.contains("phi-3") || id.contains("phi3") || id.contains("stable-code") {
            caps.supports_tools = false;
        }

        // --- Vision Support ---
        if id.contains("vision") || id.contains("-v") || id.contains("lava") 
           || id.contains("gpt-4o") || id.contains("claude-3") || id.contains("gemini-1.5")
           || id.contains("phi-3.5-vision") || id.contains("pixtral") 
        {
            caps.supports_vision = true;
        }

        // --- Structured Output (JSON Mode) ---
        if id.contains("gpt-4o") || id.contains("gpt-3.5-turbo") || id.contains("gemini")
           || id.contains("-pro") || id.contains("-flash")
        {
            caps.supports_structured_output = true;
        }

        // --- Reasoning Models ---
        if id.contains("reasoning") || id.contains("-o1") || id.contains("-o3") 
           || id.contains("deepseek-r1") || id.contains("-r1")
        {
            caps.supports_reasoning = true;
            caps.supports_tools = false; // Many early reasoning models don't support tools yet
        }

        // --- Specific Model Family Matching ---
        
        // OpenAI GPT-4o
        if id.contains("gpt-4o") {
            caps.context_window = 128_000;
            caps.max_output_tokens = 16_384;
            caps.supports_tools = true;
            caps.supports_vision = true;
            caps.supports_structured_output = true;
        }
        
        // Gemini 1.5 Series
        if id.contains("gemini-1.5") {
            caps.context_window = 1_000_000;
            if id.contains("pro") {
                caps.context_window = 2_000_000;
            }
            caps.max_output_tokens = 8_192;
            caps.supports_vision = true;
            caps.supports_tools = true;
        }

        // Claude 3.x
        if id.contains("claude-3") {
            caps.context_window = 200_000;
            caps.max_output_tokens = 4_096;
            caps.supports_vision = true;
            caps.supports_tools = true;
        }

        // DeepSeek R1
        if id.contains("deepseek-r1") || id.eq("r1") {
            caps.context_window = 64_000;
            caps.supports_reasoning = true;
            caps.supports_vision = false;
        }

        // Mistral / Llama 3 (Typical defaults)
        if id.contains("llama-3") || id.contains("llama3") || id.contains("mistral") {
            caps.context_window = 128_000;
            caps.supports_tools = true;
        }

        // Gemma 4 Series (2026)
        if id.contains("gemma-4") || id.contains("gemma4") {
            caps.supports_tools = true;
            if id.contains("26b") || id.contains("moe") || id.contains("31b") {
                caps.context_window = 256_000;
            } else {
                caps.context_window = 128_000; // Edge models (E2B/E4B)
            }
        }

        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpt4o_inference() {
        let caps = CapabilityMatrix::infer_capabilities("gpt-4o-2024-05-13");
        assert!(caps.supports_vision);
        assert!(caps.supports_tools);
        assert_eq!(caps.context_window, 128_000);
    }

    #[test]
    fn test_gemini_pro_inference() {
        let caps = CapabilityMatrix::infer_capabilities("gemini-1.5-pro");
        assert!(caps.context_window >= 1_000_000);
    }

    #[test]
    fn test_deepseek_r1_inference() {
        let caps = CapabilityMatrix::infer_capabilities("deepseek-r1");
        assert!(caps.supports_reasoning);
        assert!(!caps.supports_tools); // R1 currently disables tools in native mode
    }

    #[test]
    fn test_gemma_4_moe_inference() {
        let caps = CapabilityMatrix::infer_capabilities("gemma-4-26b-moe");
        assert_eq!(caps.context_window, 256_000);
        assert!(caps.supports_tools);
    }

    #[test]
    fn test_gemma_4_edge_inference() {
        let caps = CapabilityMatrix::infer_capabilities("gemma-4-e4b");
        assert_eq!(caps.context_window, 128_000);
        assert!(caps.supports_tools);
    }

    #[test]
    fn test_phi_3_tool_exclusion() {
        let caps = CapabilityMatrix::infer_capabilities("phi-3-mini");
        assert!(!caps.supports_tools);
    }

    #[test]
    fn test_pixtral_vision_detection() {
        let caps = CapabilityMatrix::infer_capabilities("pixtral-12b-2409");
        assert!(caps.supports_vision);
    }

    #[test]
    fn test_claude_3_context_window() {
        let caps = CapabilityMatrix::infer_capabilities("claude-3-5-sonnet");
        assert_eq!(caps.context_window, 200_000);
        assert!(caps.supports_vision);
    }

    #[test]
    fn test_generic_fallback() {
        let caps = CapabilityMatrix::infer_capabilities("unknown-model-x");
        assert_eq!(caps.context_window, 32_768);
        assert!(!caps.supports_vision);
    }
}
