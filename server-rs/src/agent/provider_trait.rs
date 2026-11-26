//! @docs ARCHITECTURE:LlmProviderInterface
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / provider_trait
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::{TokenUsage, ToolCall, ToolDefinition};
use crate::error::AppError;
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
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError>;

    /// Generates vector embeddings for a given text.
    #[allow(dead_code)]
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError>;
}
