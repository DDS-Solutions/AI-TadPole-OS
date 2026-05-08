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
        ModelCapabilities::infer_from_id(model_id)
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

// Metadata: [capability_matrix]

// Metadata: [capability_matrix]
