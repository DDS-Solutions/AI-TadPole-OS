//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / capabilities
//! - **Primary Entrypoints**: `ModelCapabilities`, `lacks_native_tool_bridge`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, specta::Type)]
pub struct ModelCapabilities {
    #[serde(default, alias = "supportsTools")]
    pub supports_tools: bool,
    #[serde(default, alias = "supportsVision")]
    pub supports_vision: bool,
    #[serde(default, alias = "supportsStructuredOutput")]
    pub supports_structured_output: bool,
    #[serde(default, alias = "supportsReasoning")]
    pub supports_reasoning: bool,
    #[serde(default, alias = "supportsHaltingTool")]
    pub supports_halting_tool: bool,
    #[serde(default, alias = "contextWindow")]
    pub context_window: u32,
    #[serde(default, alias = "maxOutputTokens")]
    pub max_output_tokens: u32,
}

/// Model families that cannot accept an OpenAI-style `tools` array at all.
/// Shared by the capability registry and `ModelConfig::supports_native_tools`
/// so the two can no longer drift apart.
pub(crate) const NATIVE_TOOL_INCOMPATIBLE_SLUGS: &[&str] = &["phi-3", "phi3", "stable-code"];

/// Returns true when the model must not be sent native tool definitions.
///
/// This is intentionally narrower than `ModelCapabilities::supports_tools`: reasoning
/// families (o1/o3, R1) are flagged as tool-less in the registry, but suppressing the
/// whole toolbelt for them at runtime would strip agents of every skill.
pub fn lacks_native_tool_bridge(model_id: &str) -> bool {
    let id = model_id.to_ascii_lowercase();
    NATIVE_TOOL_INCOMPATIBLE_SLUGS
        .iter()
        .any(|s| id.contains(s))
}

struct CapabilityPattern {
    slugs: &'static [&'static str],
    update: fn(&mut ModelCapabilities),
}

/// Slugs are matched against a normalized id (see [`normalize_model_id`]), which is
/// lowercased, has `/`, `:`, `_` and spaces folded into `-`, and is prefixed with `-`.
/// A slug starting with `-` therefore matches either a segment boundary or the start
/// of the id (`-o1` matches `o1-mini` and `openai/o1-mini`).
const CAPABILITY_PATTERNS: &[CapabilityPattern] = &[
    CapabilityPattern {
        slugs: NATIVE_TOOL_INCOMPATIBLE_SLUGS,
        update: |c| c.supports_tools = false,
    },
    CapabilityPattern {
        slugs: &[
            "vision",
            "-vl",
            "llava",
            "minicpm-v",
            "moondream",
            "gpt-4o",
            "gpt-5",
            "claude-3",
            "claude-4",
            "claude-5",
            "gemini-1.5",
            "gemini-2.0",
            "gemini-3",
            "pixtral",
        ],
        update: |c| c.supports_vision = true,
    },
    CapabilityPattern {
        slugs: &[
            "gpt-4o",
            "gpt-5",
            "gpt-3.5-turbo",
            "gemini",
            "-pro",
            "-flash",
        ],
        update: |c| c.supports_structured_output = true,
    },
    CapabilityPattern {
        slugs: &[
            "reasoning",
            "-o1",
            "-o3",
            "-o4",
            "deepseek-r1",
            "-r1",
            "mai-thinking",
        ],
        update: |c| {
            c.supports_reasoning = true;
            c.supports_tools = false;
        },
    },
    // Granular Family Overrides
    CapabilityPattern {
        // o3 / o4 reasoning models support function calling (unlike o1-mini/preview).
        slugs: &["-o3", "-o4"],
        update: |c| c.supports_tools = true,
    },
    CapabilityPattern {
        slugs: &["gpt-4o"],
        update: |c| {
            c.context_window = 128_000;
            c.max_output_tokens = 16_384;
            c.supports_tools = true;
            c.supports_vision = true;
            c.supports_structured_output = true;
        },
    },
    CapabilityPattern {
        slugs: &["gpt-5"],
        update: |c| {
            c.context_window = 256_000;
            c.max_output_tokens = 16_384;
            c.supports_tools = true;
            c.supports_vision = true;
            c.supports_structured_output = true;
        },
    },
    CapabilityPattern {
        slugs: &["gemini-1.5", "gemini-2.0"],
        update: |c| {
            c.context_window = 1_000_000;
            c.max_output_tokens = 8_192;
            c.supports_vision = true;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["gemini-3"],
        update: |c| {
            c.context_window = 2_000_000;
            c.max_output_tokens = 16_384;
            c.supports_vision = true;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["claude-3"],
        update: |c| {
            c.context_window = 200_000;
            c.max_output_tokens = 4_096;
            c.supports_vision = true;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["claude-4", "claude-5"],
        update: |c| {
            c.context_window = 500_000;
            c.max_output_tokens = 8_192;
            c.supports_vision = true;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["deepseek-r1"],
        update: |c| {
            c.context_window = 64_000;
            c.supports_reasoning = true;
            c.supports_vision = false;
        },
    },
    CapabilityPattern {
        slugs: &["llama-3", "llama3", "llama-4", "llama4", "mistral"],
        update: |c| {
            c.context_window = 128_000;
            c.supports_tools = true;
        },
    },
    CapabilityPattern {
        slugs: &["gemma-4", "gemma4"],
        update: |c| {
            c.supports_tools = true;
            c.context_window = 128_000;
        },
    },
];

/// Lowercases the id, folds common separators into `-`, and prefixes `-` so that
/// boundary-anchored slugs (`-o1`, `-vl`) also match at the start of the id.
fn normalize_model_id(model_id: &str) -> String {
    let folded: String = model_id
        .to_lowercase()
        .chars()
        .map(|c| match c {
            '/' | ':' | '_' | ' ' => '-',
            other => other,
        })
        .collect();
    format!("-{}", folded)
}

impl ModelCapabilities {
    /// ### IMR-01: Intelligent Inference
    /// Automatically infers capabilities based on the Model ID.
    pub fn infer_from_id(model_id: &str) -> Self {
        let id = normalize_model_id(model_id);
        let mut caps = Self {
            context_window: 32_768,
            max_output_tokens: 4_096,
            supports_tools: true,
            supports_halting_tool: true,
            ..Self::default()
        };

        for pattern in CAPABILITY_PATTERNS {
            if pattern.slugs.iter().any(|s| id.contains(s)) {
                (pattern.update)(&mut caps);
            }
        }

        // Final Edge Case logic not easily captured by slugs
        if id.contains("gemini-1.5-pro") {
            caps.context_window = 2_000_000;
        }
        if (id.contains("gemma-4") || id.contains("gemma4"))
            && (id.contains("12b")
                || id.contains("26b")
                || id.contains("moe")
                || id.contains("31b"))
        {
            caps.context_window = 256_000;
        }

        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_suffix_is_not_vision() {
        // `-v` used to match version suffixes like `-v0.1`.
        assert!(!ModelCapabilities::infer_from_id("mistral-7b-instruct-v0.3").supports_vision);
        assert!(!ModelCapabilities::infer_from_id("some-model-v2").supports_vision);
    }

    #[test]
    fn detects_vision_families() {
        for id in [
            "llava:13b",
            "bakllava",
            "qwen2.5-vl-7b",
            "llama-3.2-11b-vision",
            "minicpm-v:8b",
            "moondream",
            "pixtral-12b-2409",
        ] {
            assert!(ModelCapabilities::infer_from_id(id).supports_vision, "{id}");
        }
    }

    #[test]
    fn detects_openai_reasoning_prefixes() {
        let o1 = ModelCapabilities::infer_from_id("o1-mini");
        assert!(o1.supports_reasoning);
        assert!(!o1.supports_tools);

        let o1_routed = ModelCapabilities::infer_from_id("openai/o1-preview");
        assert!(o1_routed.supports_reasoning);

        let o3 = ModelCapabilities::infer_from_id("o3-mini");
        assert!(o3.supports_reasoning);
        assert!(o3.supports_tools);

        // `gpt-4o` must not be mistaken for an o-series reasoning model.
        assert!(!ModelCapabilities::infer_from_id("gpt-4o").supports_reasoning);
    }

    #[test]
    fn colon_tags_still_match_size_rules() {
        let caps = ModelCapabilities::infer_from_id("gemma4:31b");
        assert_eq!(caps.context_window, 256_000);
    }

    #[test]
    fn native_tool_bridge_exclusions() {
        assert!(lacks_native_tool_bridge("phi3-mini"));
        assert!(lacks_native_tool_bridge("Phi-3-medium"));
        assert!(lacks_native_tool_bridge("stable-code:3b"));
        // Reasoning models keep their toolbelt at runtime.
        assert!(!lacks_native_tool_bridge("deepseek-r1"));
        assert!(!lacks_native_tool_bridge("o1-mini"));
        assert!(!lacks_native_tool_bridge("phi-4"));
    }
}
