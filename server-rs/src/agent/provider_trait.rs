//! Multi-LLM Provider Interface - Provider Abstraction Layer
//!
//! Defines the unified trait for all model providers. Decouples the 
//! core reasoning engine from specific API implementations (Google, OpenAI, etc).
//!
//! @docs ARCHITECTURE:LlmProviderInterface
//!
//! ### AI Assist Note
//! **Unified Trait Design**: `Send + Sync` is mandatory for cross-thread 
//! execution in `tokio::spawn` contexts. `LlmProvider` abstracts both 
//! generation (`generate`) and vectorization (`embed`).

use crate::agent::gemini::GeminiTool;
use crate::agent::types::{GeminiFunctionCall, TokenUsage};
use async_trait::async_trait;

/// Unified interface for all LLM providers.
///
/// Implement this trait on any provider struct to make it usable by the runner.
/// The runner calls `resolve_provider()` in `runner/provider.rs` which returns a
/// `Box<dyn LlmProvider>`, decoupling the dispatch logic from individual provider details.
///
/// The `Send + Sync` bounds are required because `dispatch_to_provider` is an async fn
/// that holds `Box<dyn LlmProvider>` across `.await` points inside `tokio::spawn` contexts.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generates a response from the LLM.
    ///
    /// # Returns
    /// `(text_response, function_calls, optional_token_usage)`
    async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<GeminiTool>>,
    ) -> anyhow::Result<(String, Vec<GeminiFunctionCall>, Option<TokenUsage>)>;

    /// Generates vector embeddings for a given text.
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
}
