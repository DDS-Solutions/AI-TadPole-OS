//! @docs ARCHITECTURE:Runner
//! 
//! ### AI Assist Note
//! **Provider Abstraction**: Decouples the engine from specific LLM vendors 
//! (Gemini, Groq, OpenAI). Uses a concrete `ProviderVariant` enum to avoid 
//! async trait object overhead. Implements **Privacy Guard** (SEC-04) to block 
//! external traffic when local-only mode is active.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing API keys, unknown provider protocol, or rate limit 
//!   (RPM/TPM) breach. Falling back to `NullProvider` ensures missions degrade 
//!   gracefully instead of crashing.
//! - **Trace Scope**: `server-rs::agent::runner::provider`

use super::{AgentRunner, RunContext};
use crate::agent::null_provider::{NullProvider, NullReason};
use crate::agent::types::TokenUsage;
use std::sync::Arc;

// ─────────────────────────────────────────────────────────
//  PROVIDER VARIANT ENUM
// ─────────────────────────────────────────────────────────

/// Concrete enum representing all supported LLM provider backends.
pub(crate) enum ProviderVariant {
    /// Google Gemini API (Native tool-calling).
    Gemini(crate::agent::gemini::GeminiProvider),
    /// Groq high-speed inference (Llama/Whisper).
    Groq(crate::agent::groq::GroqProvider),
    /// OpenAI and compatible proxies (Ollama, Inception).
    OpenAI(crate::agent::openai::OpenAIProvider),
    /// Fallback "Degraded" provider for missing keys or unknown protocols.
    Null(NullProvider),
}

impl ProviderVariant {
    pub(crate) async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<crate::agent::gemini::GeminiTool>>,
    ) -> anyhow::Result<(
        String,
        Vec<crate::agent::types::GeminiFunctionCall>,
        Option<TokenUsage>,
    )> {
        match self {
            ProviderVariant::Gemini(p) => {
                let combined = format!("{}\n\nUSER MESSAGE:\n{}", system_prompt, user_message);
                p.generate(&combined, tools).await
            }
            ProviderVariant::Groq(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::OpenAI(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::Null(p) => {
                use crate::agent::provider_trait::LlmProvider;
                p.generate(system_prompt, user_message, tools).await
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        match self {
            ProviderVariant::Gemini(p) => {
                use crate::agent::provider_trait::LlmProvider;
                p.embed(text).await
            }
            ProviderVariant::Groq(p) => {
                use crate::agent::provider_trait::LlmProvider;
                p.embed(text).await
            }
            ProviderVariant::OpenAI(p) => {
                use crate::agent::provider_trait::LlmProvider;
                p.embed(text).await
            }
            ProviderVariant::Null(p) => {
                use crate::agent::provider_trait::LlmProvider;
                p.embed(text).await
            }
        }
    }
}

/// Resolves an API key: prefers the per-agent config override, then falls
/// back to the named environment variable.
fn resolve_api_key(config: &crate::agent::types::ModelConfig, env_var: &str) -> Option<String> {
    config
        .api_key
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .or_else(|| std::env::var(env_var).ok())
}

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  PROVIDER DISPATCH
    // ─────────────────────────────────────────────────────────

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
        tools: Option<Vec<crate::agent::gemini::GeminiTool>>,
    ) -> anyhow::Result<(
        String,
        Vec<crate::agent::types::GeminiFunctionCall>,
        Option<crate::agent::types::TokenUsage>,
    )> {
        self.dispatch_to_provider(ctx, system_prompt, user_message, tools)
            .await
    }

    /// Calls the provider for a synthesis/follow-up step. Supporting tools here
    /// allows specialists to 'Self-Heal' from sub-agent failures.
    pub(crate) async fn call_provider_for_synthesis(
        &self,
        ctx: &RunContext,
        prompt: &str,
        tools: Option<Vec<crate::agent::gemini::GeminiTool>>,
    ) -> anyhow::Result<(
        String,
        Vec<crate::agent::types::GeminiFunctionCall>,
        Option<crate::agent::types::TokenUsage>,
    )> {
        let synthesis_prompt = format!("{}\n\nCRITICAL INSTRUCTION: You MUST provide a deterministic resolution. If the sub-agent result is unsatisfactory, use your tools to find an alternative or call 'complete_mission' with the findings so far.", prompt);
        self.dispatch_to_provider(ctx, &synthesis_prompt, "", tools)
            .await
    }

    /// Resolves the correct `ProviderVariant` for the given context.
    pub(crate) fn resolve_provider(
        &self,
        ctx: &RunContext,
        client: reqwest::Client,
    ) -> ProviderVariant {
        use crate::agent::types::ModelProvider;

        tracing::info!(
            "🔍 [Provider] Resolving provider '{}' for agent '{}'",
            ctx.provider_name,
            ctx.agent_id
        );

        if std::env::var("TADPOLE_NULL_PROVIDERS").as_deref() == Ok("true") {
            return ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::TestMode));
        }

        // SEC-04: Privacy Mode Enforcement
        if self.state.governance.privacy_mode.load(std::sync::atomic::Ordering::Relaxed)
            && ctx.model_config.provider != ModelProvider::Ollama
        {
            return ProviderVariant::Null(NullProvider::new(
                &ctx.agent_id,
                NullReason::PrivacyModeEnforced,
            ));
        }

        match ctx.model_config.provider {
            ModelProvider::Google | ModelProvider::Gemini => {
                match resolve_api_key(&ctx.model_config, "GOOGLE_API_KEY") {
                    Some(key) => {
                        ProviderVariant::Gemini(crate::agent::gemini::GeminiProvider::new(
                            client,
                            key,
                            ctx.model_config.clone(),
                        ))
                    }
                    None => ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::MissingApiKey { env_var: "GOOGLE_API_KEY" })),
                }
            }
            ModelProvider::Groq => {
                match resolve_api_key(&ctx.model_config, "GROQ_API_KEY") {
                    Some(key) => ProviderVariant::Groq(crate::agent::groq::GroqProvider::new(client, key, ctx.model_config.clone())),
                    None => ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::MissingApiKey { env_var: "GROQ_API_KEY" })),
                }
            }
            ModelProvider::Openai | ModelProvider::Xai | ModelProvider::Openrouter => {
                let env_var = match ctx.model_config.provider {
                    ModelProvider::Xai => "XAI_API_KEY",
                    ModelProvider::Openrouter => "OPENROUTER_API_KEY",
                    _ => "OPENAI_API_KEY",
                };

                match resolve_api_key(&ctx.model_config, env_var) {
                    Some(key) => ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(client, key, ctx.model_config.clone())),
                    None => ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::MissingApiKey { env_var })),
                }
            }
            ModelProvider::Inception => {
                match resolve_api_key(&ctx.model_config, "INCEPTION_API_KEY") {
                    Some(key) => ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(client, key, ctx.model_config.clone())),
                    None => ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::MissingApiKey { env_var: "INCEPTION_API_KEY" })),
                }
            }
            ModelProvider::Deepseek => {
                let api_key = resolve_api_key(&ctx.model_config, "DEEPSEEK_API_KEY").or_else(|| std::env::var("OPENAI_API_KEY").ok());
                match api_key {
                    Some(key) => ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(client, key, ctx.model_config.clone())),
                    None => ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::MissingApiKey { env_var: "DEEPSEEK_API_KEY" })),
                }
            }
            ModelProvider::Ollama => {
                let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| "ollama".to_string());
                ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(client, api_key, ctx.model_config.clone()))
            }
            ModelProvider::Anthropic => {
                 match resolve_api_key(&ctx.model_config, "ANTHROPIC_API_KEY") {
                    Some(key) => {
                         ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(client, key, ctx.model_config.clone()))
                    }
                    None => ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::MissingApiKey { env_var: "ANTHROPIC_API_KEY" })),
                }
            }
        }
    }



    async fn dispatch_to_provider(
        &self,
        ctx: &RunContext,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<crate::agent::gemini::GeminiTool>>,
    ) -> anyhow::Result<(
        String,
        Vec<crate::agent::types::GeminiFunctionCall>,
        Option<crate::agent::types::TokenUsage>,
    )> {
        let client = (*self.state.resources.http_client).clone();
        let limiter_key = format!("{}:{}", ctx.provider_name, ctx.model_config.model_id);
        let limiter = self.state.resources.rate_limiters.entry(limiter_key.clone()).or_insert_with(|| {
            Arc::new(crate::agent::rate_limiter::RateLimiter::new(ctx.model_config.rpm, ctx.model_config.tpm))
        }).value().clone();

        if limiter.is_active() {
            let estimated_tokens = ((system_prompt.len() + user_message.len()) as f64 / 3.5) as u32;
            limiter.acquire(estimated_tokens).await;
        }

        let provider = self.resolve_provider(ctx, client);
        let result = provider.generate(system_prompt, user_message, tools).await;

        if limiter.is_active() {
            if let Ok((_, _, Some(ref usage))) = &result {
                limiter.record_usage(usage.total_tokens);
                self.state.governance.tpm_accumulator.fetch_add(usage.total_tokens as usize, std::sync::atomic::Ordering::Relaxed);
            }
        }
        result
    }

    pub(crate) async fn check_budget(
        &self,
        ctx: &RunContext,
        _step_cost: f64,
        output_text: &str,
    ) -> anyhow::Result<Option<String>> {
        // PERF: Use cached values from RunContext to avoid DB roundtrips on every turn.
        // We implement a 5% "Completion Buffer" to allow final synthesis reports 
        // to finalize even if the precise limit is reached mid-turn.
        let budget = ctx.budget_usd;
        let current_cost = ctx.current_cost_usd + _step_cost;

        if budget > 0.0 && current_cost >= (budget * 1.05) {
             tracing::warn!("⚠️ [Governance] Budget exceeded for mission {}: ${:.4} / ${:.4}", ctx.mission_id, current_cost, budget);
             return Ok(Some(format!("(PAUSED: Budget Exceeded ${:.4}/${:.4}) {}", current_cost, budget, output_text)));
        }

        Ok(None)
    }
}
