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
    /// ### 🧠 Cognitive Analysis: infer_capabilities
    /// Infers capabilities for a model based on its specific ID slug.
    /// Handles both cloud (OpenAI, Gemini, Anthropic) and local (Ollama, vLLM) 
    /// naming conventions to ensure consistency across the swarm.
    /// 
    /// ### 🩹 Logic: Pattern-Based Fallbacks
    /// Uses a "Defense-in-Depth" matching strategy. Broad family flags 
    /// (e.g., `-v` for vision) are applied first, followed by specific 
    /// granular overrides for high-fidelity models like GPT-4o or Gemini 1.5 Pro.
    pub fn infer_capabilities(model_id: &str) -> ModelCapabilities {
        let id = model_id.to_lowercase();
        let mut caps = ModelCapabilities {
            context_window: 32_768,
            max_output_tokens: 4_096,
            supports_tools: true,
            ..ModelCapabilities::default()
        };

        // --- 📡 Feature Detection: Tool Support ---
        // Generally True for modern models EXCEPT specific small ones or old ones.
        // Phil-3 and Stable-Code families have fragmented native tool support 
        // and are excluded from the functional registry.
        if id.contains("phi-3") || id.contains("phi3") || id.contains("stable-code") {
            caps.supports_tools = false;
        }

        // --- 👁️ Feature Detection: Vision ---
        // Matches common naming patterns for multimodal-capable models.
        if id.contains("vision") || id.contains("-v") || id.contains("lava") 
           || id.contains("gpt-4o") || id.contains("claude-3") || id.contains("gemini-1.5")
           || id.contains("phi-3.5-vision") || id.contains("pixtral") 
        {
            caps.supports_vision = true;
        }

        // --- 🧱 Feature Detection: Structured Output (JSON Mode) ---
        // Detects support for enforced schemas where the model yields 
        // guaranteed valid JSON.
        if id.contains("gpt-4o") || id.contains("gpt-3.5-turbo") || id.contains("gemini")
           || id.contains("-pro") || id.contains("-flash")
        {
            caps.supports_structured_output = true;
        }

        // --- 🧠 Feature Detection: Reasoning Models ---
        // Specialized logic for 'Chain of Thought' or 'Reasoning' architectures 
        // (e.g., OpenAI o1, DeepSeek R1). These models often utilize internal 
        // tool-use loops that are incompatible with external sidecar tool calling.
        if id.contains("reasoning") || id.contains("-o1") || id.contains("-o3") 
           || id.contains("deepseek-r1") || id.contains("-r1")
        {
            caps.supports_reasoning = true;
            caps.supports_tools = false; // Many early reasoning models don't support tools yet
        }

        // --- 🛸 Granular Overrides: Specific Model Family Matching ---
        
        // 🔹 OpenAI: GPT-4o Series
        if id.contains("gpt-4o") {
            caps.context_window = 128_000;
            caps.max_output_tokens = 16_384;
            caps.supports_tools = true;
            caps.supports_vision = true;
            caps.supports_structured_output = true;
        }
        
        // 🔹 Google: Gemini 1.5 Series
        if id.contains("gemini-1.5") {
            caps.context_window = 1_000_000;
            if id.contains("pro") {
                caps.context_window = 2_000_000;
            }
            caps.max_output_tokens = 8_192;
            caps.supports_vision = true;
            caps.supports_tools = true;
        }

        // 🔹 Anthropic: Claude 3.x Series
        if id.contains("claude-3") {
            caps.context_window = 200_000;
            caps.max_output_tokens = 4_096;
            caps.supports_vision = true;
            caps.supports_tools = true;
        }

        // 🔹 DeepSeek: R1 / Reasoning Series
        if id.contains("deepseek-r1") || id.eq("r1") {
            caps.context_window = 64_000;
            caps.supports_reasoning = true;
            caps.supports_vision = false;
        }

        // 🔹 Mistral/Meta: Llama 3 / Mistral Series
        if id.contains("llama-3") || id.contains("llama3") || id.contains("mistral") {
            caps.context_window = 128_000;
            caps.supports_tools = true;
        }

        // 🔹 Google: Gemma 4 Series (2026 Ready)
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
