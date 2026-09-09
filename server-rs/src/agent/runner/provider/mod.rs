//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod dispatch;
pub mod fallback;
pub mod rate_limit;
pub mod resolution;

#[cfg(test)]
pub mod mock_provider_tests;
#[cfg(test)]
pub mod privacy_cluster_tests;
#[cfg(test)]
pub mod tests;

use crate::agent::null_provider::NullProvider;
use crate::agent::provider_trait::LlmProvider;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::{TokenUsage, ToolCall, ToolDefinition};
use crate::error::AppError;

/// Concrete enum representing all supported LLM provider backends.
pub(crate) enum ProviderVariant {
    /// Google Gemini API (Native tool-calling).
    Gemini(crate::agent::gemini::GeminiProvider),
    /// Groq high-speed inference (Llama/Whisper).
    Groq(crate::agent::groq::GroqProvider),
    /// OpenAI and compatible proxies (Ollama, Inception).
    OpenAI(crate::agent::openai::OpenAIProvider),
    /// Anthropic Claude 3.5 Sonnet (Native Messages API).
    Anthropic(crate::agent::anthropic::AnthropicProvider),
    /// Fallback "Degraded" provider for missing keys, privacy enforcement, or test scenarios.
    Null(NullProvider),
}

impl ProviderVariant {
    pub(crate) async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        let result = match self {
            ProviderVariant::Gemini(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::Groq(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::OpenAI(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::Anthropic(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::Null(p) => p.generate(system_prompt, user_message, tools).await,
        };
        result.map_err(|e| self.normalize_error(e))
    }

    #[allow(dead_code)]
    pub(crate) async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let result = match self {
            ProviderVariant::Gemini(p) => p.embed(text).await,
            ProviderVariant::Groq(p) => p.embed(text).await,
            ProviderVariant::OpenAI(p) => p.embed(text).await,
            ProviderVariant::Anthropic(p) => p.embed(text).await,
            ProviderVariant::Null(p) => p.embed(text).await,
        };
        result.map_err(|e| self.normalize_error(e))
    }

    fn normalize_error(&self, err: AppError) -> AppError {
        match err {
            AppError::InfrastructureError { .. }
            | AppError::RateLimit(_)
            | AppError::QuantizationFallback { .. } => err,
            _ => {
                let provider_id = match self {
                    ProviderVariant::Gemini(_) => crate::error::ProviderId::Gemini,
                    ProviderVariant::Groq(_) => crate::error::ProviderId::Groq,
                    ProviderVariant::OpenAI(_) => crate::error::ProviderId::OpenAi,
                    ProviderVariant::Anthropic(_) => crate::error::ProviderId::Anthropic,
                    ProviderVariant::Null(_) => crate::error::ProviderId::Runner,
                };
                crate::error::AppError::InfrastructureError {
                    provider_id,
                    kind: crate::error::InfrastructureErrorKind::ApiError,
                    detail: err.to_string(),
                    help_link: None,
                }
            }
        }
    }
}

impl AgentRunner {
    /// Accumulates token usage from a tool call into the mission total.
    pub(crate) fn accumulate_usage(
        &self,
        total: &mut Option<TokenUsage>,
        local: Option<TokenUsage>,
    ) {
        if let Some(loc) = local {
            if let Some(tot) = total {
                tot.input_tokens += loc.input_tokens;
                tot.output_tokens += loc.output_tokens;
                tot.total_tokens += loc.total_tokens;
            } else {
                *total = Some(loc);
            }
        }
    }

    /// Routes the generation request to the correct LLM provider.
    pub(crate) async fn call_provider(
        &self,
        ctx: &RunContext,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        self.dispatch_to_provider(ctx, system_prompt, user_message, tools)
            .await
    }

    /// Calls the provider for a synthesis/follow-up step. Supporting tools here
    /// allows specialists to 'Self-Heal' from sub-agent failures.
    pub(crate) async fn call_provider_for_synthesis(
        &self,
        ctx: &RunContext,
        prompt: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        let mut warning_instruction = "CRITICAL INSTRUCTION: You MUST provide a deterministic resolution. If the sub-agent result is unsatisfactory, use your tools to find an alternative or call 'complete_mission' with the findings so far.".to_string();

        if let Some(agent_entry) = self.state.registry.agents.get(&ctx.agent_id) {
            if let Some(custom_prompt) = agent_entry
                .value()
                .metadata
                .get("synthesis_prompt_override")
                .and_then(|v| v.as_str())
            {
                warning_instruction = custom_prompt.to_string();
            }
        }

        let synthesis_prompt = format!("{}\n\n{}", prompt, warning_instruction);
        self.dispatch_to_provider(ctx, &synthesis_prompt, "", tools)
            .await
    }

    pub(crate) async fn check_budget(
        &self,
        ctx: &RunContext,
        step_cost: f64,
        output_text: &str,
    ) -> Result<Option<String>, AppError> {
        let budget = ctx.budget_usd;
        let current_cost = ctx.current_cost_usd + step_cost;

        if budget > 0.0 && current_cost >= (budget * 1.05) {
            tracing::warn!(
                "⚠️ [Governance] Budget exceeded for mission {}: ${:.4} / ${:.4}",
                ctx.mission_id,
                current_cost,
                budget
            );
            return Ok(Some(format!(
                "(PAUSED: Budget Exceeded ${:.4}/${:.4}) {}",
                current_cost, budget, output_text
            )));
        }

        if let Some(sub_budget) = ctx.sub_budget_usd {
            let current_db_cost = match crate::agent::mission::get_mission_by_id(
                &self.state.resources.pool,
                &ctx.mission_id,
            )
            .await
            {
                Ok(Some(m)) => m.cost_usd,
                _ => ctx.current_cost_usd,
            };
            let accumulated_by_sub_agent =
                ((current_db_cost + step_cost) - ctx.current_cost_usd).max(0.0);

            if sub_budget > 0.0 && accumulated_by_sub_agent >= (sub_budget * 1.05) {
                tracing::warn!(
                    "⚠️ [Governance] Sub-budget exceeded for agent {} in mission {}: ${:.4} / ${:.4}",
                    ctx.agent_id,
                    ctx.mission_id,
                    accumulated_by_sub_agent,
                    sub_budget
                );
                return Ok(Some(format!(
                    "(PAUSED: Sub-budget Exceeded ${:.4}/${:.4}) {}",
                    accumulated_by_sub_agent, sub_budget, output_text
                )));
            }
        }

        Ok(None)
    }
}
