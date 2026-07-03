//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Neural Billing**: Manages the centralized registry of AI model pricing.
//! Provides the **Financial Ground Truth** for USD cost calculation based
//! on token usage metrics. Enforces **SEC-02 (Synchronous Ledgers)** by
//! applying deterministic fallback rates for unrecognized models.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Model ID mismatch causing incorrect billing, or
//!   stale static rates for rapidly evolving provider pricing.
//! - **Trace Scope**: `server-rs::agent::rates`

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Represents the financial cost of a specific AI model.
/// Rates are defined as USD per 1,000 tokens for calculation granularity.
pub struct ModelRate {
    /// Cost in USD for 1,000 input (prompt) tokens.
    pub input_cost_per_1k: f64,
    /// Cost in USD for 1,000 output (completion) tokens.
    pub output_cost_per_1k: f64,
}

/// Static registry of model rates (Cost per 1,000 tokens)
pub static MODEL_RATES: Lazy<HashMap<&'static str, ModelRate>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // OpenAI Models
    m.insert(
        "gpt-5.5-pro",
        ModelRate {
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.045,
        },
    );
    m.insert(
        "gpt-5.5",
        ModelRate {
            input_cost_per_1k: 0.005,
            output_cost_per_1k: 0.015,
        },
    );
    m.insert(
        "gpt-5.3-codex",
        ModelRate {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.009,
        },
    );
    m.insert(
        "gpt-5.2-preview",
        ModelRate {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.009,
        },
    );
    m.insert(
        "o4-mini-2026-02",
        ModelRate {
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0006,
        },
    );
    m.insert(
        "gpt-4o",
        ModelRate {
            input_cost_per_1k: 0.005,
            output_cost_per_1k: 0.015,
        },
    );
    m.insert(
        "gpt-4o-mini",
        ModelRate {
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0006,
        },
    );

    // Anthropic Models
    m.insert(
        "claude-5-fable",
        ModelRate {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
    );
    m.insert(
        "claude-5-mythos",
        ModelRate {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
    );
    m.insert(
        "claude-4.8-opus",
        ModelRate {
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.075,
        },
    );
    m.insert(
        "claude-4.5-sonnet",
        ModelRate {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
    );
    m.insert(
        "claude-4-sonnet",
        ModelRate {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
    );

    // Google Models
    m.insert(
        "gemini-3.5-pro",
        ModelRate {
            input_cost_per_1k: 0.00125,
            output_cost_per_1k: 0.00375,
        },
    );
    m.insert(
        "gemini-3.5-flash",
        ModelRate {
            input_cost_per_1k: 0.000075,
            output_cost_per_1k: 0.0003,
        },
    );
    m.insert(
        "gemini-3.1-pro-preview",
        ModelRate {
            input_cost_per_1k: 0.00125,
            output_cost_per_1k: 0.00375,
        },
    );
    m.insert(
        "gemini-3.1-flash-preview",
        ModelRate {
            input_cost_per_1k: 0.000075,
            output_cost_per_1k: 0.0003,
        },
    );
    m.insert(
        "gemini-3-pro-preview",
        ModelRate {
            input_cost_per_1k: 0.00125,
            output_cost_per_1k: 0.00375,
        },
    );
    m.insert(
        "gemini-3-flash-preview",
        ModelRate {
            input_cost_per_1k: 0.000075,
            output_cost_per_1k: 0.0003,
        },
    );
    m.insert(
        "gemini-1.5-pro",
        ModelRate {
            input_cost_per_1k: 0.00125,
            output_cost_per_1k: 0.00375,
        },
    );
    m.insert(
        "gemini-1.5-flash",
        ModelRate {
            input_cost_per_1k: 0.000075,
            output_cost_per_1k: 0.0003,
        },
    );

    // Meta Models
    m.insert(
        "llama-3.3-70b-instruct",
        ModelRate {
            input_cost_per_1k: 0.00035,
            output_cost_per_1k: 0.0004,
        },
    );

    // Groq Models
    m.insert(
        "llama-3.3-70b-versatile",
        ModelRate {
            input_cost_per_1k: 0.00059,
            output_cost_per_1k: 0.00079,
        },
    );
    m.insert(
        "mixtral-8x7b-32768",
        ModelRate {
            input_cost_per_1k: 0.00027,
            output_cost_per_1k: 0.00027,
        },
    );

    // DeepSeek Models
    m.insert(
        "deepseek-v3.2",
        ModelRate {
            input_cost_per_1k: 0.00014,
            output_cost_per_1k: 0.00028,
        },
    );
    m.insert(
        "deepseek-r1",
        ModelRate {
            input_cost_per_1k: 0.00055,
            output_cost_per_1k: 0.00219,
        },
    );

    m
});

/// Calculates the cost in USD for a given token usage and model.
///
/// # Parameters
/// - `model_id`: The ID of the model used (e.g., "gpt-4o").
/// - `input_tokens`: The number of tokens sent in the request.
/// - `output_tokens`: The number of tokens received in the response.
///
/// # Returns
/// The calculated USD cost as an `f64`. If the model is not in the registry,
/// a standard fallback rate is applied.
pub fn calculate_cost(model_id: &str, input_tokens: u32, output_tokens: u32) -> f64 {
    let rate = MODEL_RATES.get(model_id).unwrap_or(&ModelRate {
        input_cost_per_1k: 0.002, // Default fallback
        output_cost_per_1k: 0.006,
    });

    let input_cost = (input_tokens as f64 / 1000.0) * rate.input_cost_per_1k;
    let output_cost = (output_tokens as f64 / 1000.0) * rate.output_cost_per_1k;

    input_cost + output_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cost_gpt4o() {
        let cost = calculate_cost("gpt-4o", 1000, 1000);
        assert_eq!(cost, 0.005 + 0.015);
    }

    #[test]
    fn test_calculate_cost_unknown() {
        let cost = calculate_cost("unknown-model", 1000, 1000);
        // Default fallback: 0.002 + 0.006 = 0.008
        assert_eq!(cost, 0.008);
    }

    #[test]
    fn test_calculate_cost_gemini() {
        let cost = calculate_cost("gemini-1.5-flash", 10000, 10000);
        // input: 10 * 0.000075 = 0.00075
        // output: 10 * 0.0003 = 0.003
        assert!((cost - 0.00375).abs() < 1e-10);
    }
}

// Metadata: [rates]
