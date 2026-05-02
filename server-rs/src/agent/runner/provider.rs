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
use crate::agent::provider_trait::LlmProvider;
use crate::agent::types::{TokenUsage, ToolCall, ToolDefinition};
use crate::error::AppError;
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
    /// Anthropic Claude 3.5 Sonnet (Native Messages API).
    Anthropic(crate::agent::anthropic::AnthropicProvider),
    /// Fallback "Degraded" provider for missing keys or unknown protocols.
    Null(NullProvider),
}

impl ProviderVariant {
    pub(crate) async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        match self {
            ProviderVariant::Gemini(p) => {
                p.generate(system_prompt, user_message, tools).await
            }
            ProviderVariant::Groq(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::OpenAI(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::Anthropic(p) => p.generate(system_prompt, user_message, tools).await,
            ProviderVariant::Null(p) => {
                p.generate(system_prompt, user_message, tools).await
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        match self {
            ProviderVariant::Gemini(p) => p.embed(text).await,
            ProviderVariant::Groq(p) => p.embed(text).await,
            ProviderVariant::OpenAI(p) => p.embed(text).await,
            ProviderVariant::Anthropic(p) => p.embed(text).await,
            ProviderVariant::Null(p) => p.embed(text).await,
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
        if self
            .state
            .governance
            .privacy_mode
            .load(std::sync::atomic::Ordering::Relaxed)
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
                    None => ProviderVariant::Null(NullProvider::new(
                        &ctx.agent_id,
                        NullReason::MissingApiKey {
                            env_var: "GOOGLE_API_KEY",
                        },
                    )),
                }
            }
            ModelProvider::Groq => match resolve_api_key(&ctx.model_config, "GROQ_API_KEY") {
                Some(key) => ProviderVariant::Groq(crate::agent::groq::GroqProvider::new(
                    client,
                    key,
                    ctx.model_config.clone(),
                )),
                None => ProviderVariant::Null(NullProvider::new(
                    &ctx.agent_id,
                    NullReason::MissingApiKey {
                        env_var: "GROQ_API_KEY",
                    },
                )),
            },
            ModelProvider::Openai | ModelProvider::Xai | ModelProvider::Openrouter | 
            ModelProvider::Mistral | ModelProvider::Perplexity | ModelProvider::Fireworks | 
            ModelProvider::Together | ModelProvider::Cerebras | ModelProvider::Sambanova => {
                let (env_var, default_url) = match ctx.model_config.provider {
                    ModelProvider::Xai => ("XAI_API_KEY", "https://api.x.ai/v1"),
                    ModelProvider::Openrouter => ("OPENROUTER_API_KEY", "https://openrouter.ai/api/v1"),
                    ModelProvider::Mistral => ("MISTRAL_API_KEY", "https://api.mistral.ai/v1"),
                    ModelProvider::Perplexity => ("PERPLEXITY_API_KEY", "https://api.perplexity.ai"),
                    ModelProvider::Fireworks => ("FIREWORKS_API_KEY", "https://api.fireworks.ai/inference/v1"),
                    ModelProvider::Together => ("TOGETHER_API_KEY", "https://api.together.xyz/v1"),
                    ModelProvider::Cerebras => ("CEREBRAS_API_KEY", "https://api.cerebras.ai/v1"),
                    ModelProvider::Sambanova => ("SAMBANOVA_API_KEY", "https://api.sambanova.ai/v1"),
                    _ => ("OPENAI_API_KEY", "https://api.openai.com/v1"),
                };

                match resolve_api_key(&ctx.model_config, env_var) {
                    Some(key) => {
                        let mut config = ctx.model_config.clone();
                        if config.base_url.is_none() {
                            config.base_url = Some(default_url.to_string());
                        }
                        ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                            client,
                            key,
                            config,
                        ))
                    }
                    None => ProviderVariant::Null(NullProvider::new(
                        &ctx.agent_id,
                        NullReason::MissingApiKey { env_var },
                    )),
                }
            }
            ModelProvider::Inception => {
                match resolve_api_key(&ctx.model_config, "INCEPTION_API_KEY") {
                    Some(key) => {
                        ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                            client,
                            key,
                            ctx.model_config.clone(),
                        ))
                    }
                    None => ProviderVariant::Null(NullProvider::new(
                        &ctx.agent_id,
                        NullReason::MissingApiKey {
                            env_var: "INCEPTION_API_KEY",
                        },
                    )),
                }
            }
            ModelProvider::Deepseek => {
                let api_key = resolve_api_key(&ctx.model_config, "DEEPSEEK_API_KEY")
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok());
                match api_key {
                    Some(key) => {
                        ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                            client,
                            key,
                            ctx.model_config.clone(),
                        ))
                    }
                    None => ProviderVariant::Null(NullProvider::new(
                        &ctx.agent_id,
                        NullReason::MissingApiKey {
                            env_var: "DEEPSEEK_API_KEY",
                        },
                    )),
                }
            }
            ModelProvider::Ollama => {
                let api_key = ctx
                    .model_config
                    .api_key
                    .clone()
                    .unwrap_or_else(|| "ollama".to_string());
                ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                    client,
                    api_key,
                    ctx.model_config.clone(),
                ))
            }
            ModelProvider::Anthropic => {
                match resolve_api_key(&ctx.model_config, "ANTHROPIC_API_KEY") {
                    Some(key) => {
                        ProviderVariant::Anthropic(crate::agent::anthropic::AnthropicProvider::new(
                            client,
                            key,
                            ctx.model_config.clone(),
                        ))
                    }
                    None => ProviderVariant::Null(NullProvider::new(
                        &ctx.agent_id,
                        NullReason::MissingApiKey {
                            env_var: "ANTHROPIC_API_KEY",
                        },
                    )),
                }
            }
        }
    }

    async fn dispatch_to_provider(
        &self,
        ctx: &RunContext,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        use crate::agent::types::ProviderStatus;
        use std::sync::atomic::Ordering;

        let provider_id = ctx.provider_name.clone();
        let health = self.state.registry.provider_health.get(&provider_id)
            .map(|h| *h.value())
            .unwrap_or(ProviderStatus::Green);

        // Pre-emptive Failover for RED status
        if health == ProviderStatus::Red {
            tracing::error!("🚨 [Provider] {} is in RED state. Attempting failover...", provider_id);
        }

        let client = (*self.state.resources.http_client).clone();
        let limiter_key = format!("{}:{}", ctx.provider_name, ctx.model_config.model_id);
        let limiter = self
            .state
            .resources
            .rate_limiters
            .entry(limiter_key.clone())
            .or_insert_with(|| {
                Arc::new(crate::agent::rate_limiter::RateLimiter::new(
                    ctx.model_config.rpm,
                    ctx.model_config.tpm,
                ))
            })
            .value()
            .clone();

        if limiter.is_active() {
            let estimated_tokens = ((system_prompt.len() + user_message.len()) as f64 / 3.5) as u32;
            limiter.acquire(estimated_tokens).await;
        }

        let provider = self.resolve_provider(ctx, client);
        let result = provider.generate(system_prompt, user_message, tools).await;

        match result {
            Ok((text, tool_calls, usage)) => {
                // Success: Reset failures and restore Green status
                self.state.registry.provider_failures.remove(&provider_id);
                self.state.registry.provider_health.insert(provider_id, ProviderStatus::Green);

                if limiter.is_active() {
                    if let Some(ref u) = usage {
                        limiter.record_usage(u.total_tokens);
                        self.state.governance.tpm_accumulator.fetch_add(
                            u.total_tokens as usize,
                            Ordering::Relaxed,
                        );
                    }
                }
                Ok((text, tool_calls, usage))
            }
            Err(e) => {
                // Failure: Increment count and update status
                let failures = self.state.registry.provider_failures.entry(provider_id.clone())
                    .or_insert_with(|| std::sync::atomic::AtomicU32::new(0));
                let count = failures.fetch_add(1, Ordering::Relaxed) + 1;

                let new_status = if count >= 5 {
                    ProviderStatus::Red
                } else if count >= 3 {
                    ProviderStatus::Amber
                } else {
                    ProviderStatus::Green
                };
                
                self.state.registry.provider_health.insert(provider_id.clone(), new_status);

                tracing::warn!(
                    "⚠️ [Provider] {} failed (count: {}). New status: {:?}",
                    provider_id, count, new_status
                );

                Err(e)
            }
        }
    }

    pub(crate) async fn check_budget(
        &self,
        ctx: &RunContext,
        _step_cost: f64,
        output_text: &str,
    ) -> Result<Option<String>, AppError> {
        let budget = ctx.budget_usd;
        let current_cost = ctx.current_cost_usd + _step_cost;

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

        Ok(None)
    }
}

// Metadata: [provider]

// Metadata: [provider]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    #[tokio::test]
    async fn test_accumulate_usage() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        
        let mut total = Some(TokenUsage { input_tokens: 10, output_tokens: 5, total_tokens: 15 });
        let local = Some(TokenUsage { input_tokens: 20, output_tokens: 10, total_tokens: 30 });
        
        runner.accumulate_usage(&mut total, local);
        
        let tot = total.unwrap();
        assert_eq!(tot.input_tokens, 30);
        assert_eq!(tot.output_tokens, 15);
        assert_eq!(tot.total_tokens, 45);
    }

    #[tokio::test]
    async fn test_check_budget_exceeded() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            budget_usd: 1.0,
            current_cost_usd: 1.06,
            ..RunContext::default()
        };

        let res = runner.check_budget(&ctx, 0.0, "Result text").await.unwrap();
        assert!(res.is_some());
        assert!(res.unwrap().contains("Budget Exceeded"));
    }

    #[tokio::test]
    async fn test_check_budget_safe() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            budget_usd: 1.0,
            current_cost_usd: 0.5,
            ..RunContext::default()
        };

        let res = runner.check_budget(&ctx, 0.1, "Result text").await.unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_resolve_api_key() {
        let mut config = crate::agent::types::ModelConfig {
            api_key: Some("config-key".to_string()),
            ..crate::agent::types::ModelConfig::default()
        };
        
        // Priority should be config
        let key = resolve_api_key(&config, "UNUSED_ENV_VAR");
        assert_eq!(key, Some("config-key".to_string()));

        // Fallback to env
        config.api_key = None;
        std::env::set_var("TEST_PROVIDER_KEY", "env-key");
        let key = resolve_api_key(&config, "TEST_PROVIDER_KEY");
        assert_eq!(key, Some("env-key".to_string()));
    }
}
