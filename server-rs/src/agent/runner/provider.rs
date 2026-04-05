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
//
//  Using a concrete enum instead of Box<dyn LlmProvider> avoids
//  the 'future not Send' issue that arises when holding a trait object
//  across an `.await` point inside a tokio::spawn context.
//
//  All variants are Send + Sync because the underlying structs are.
// ─────────────────────────────────────────────────────────

/// Concrete enum representing all supported LLM provider backends.
///
/// Using a box-less enum avoids the 'Send' issues endemic to trait objects
/// while maintaining high-performance dispatch across async boundaries.
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
/// back to the named environment variable. Returns `None` only when neither
/// source provides a non-empty value.
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

    /// Routes the generation request to the correct LLM provider using the shared HTTP client.
    /// Enforces RPM/TPM rate limits when configured on the model.
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

    /// Calls the provider for a synthesis/follow-up step (no tool definitions).
    pub(crate) async fn call_provider_for_synthesis(
        &self,
        ctx: &RunContext,
        prompt: &str,
    ) -> anyhow::Result<(
        String,
        Vec<crate::agent::types::GeminiFunctionCall>,
        Option<crate::agent::types::TokenUsage>,
    )> {
        let synthesis_prompt = format!("{}\n\nCRITICAL INSTRUCTION: You MUST provide a clear, textual, conversational response to this synthesis request. Do NOT output a blank response.", prompt);
        self.dispatch_to_provider(ctx, &synthesis_prompt, "", None)
            .await
    }

    /// Resolves the correct `ProviderVariant` for the given context.
    ///
    /// **Never returns a hard error for a missing API key or unknown provider.**
    /// Instead returns `ProviderVariant::Null` so the mission completes as "degraded"
    /// rather than "failed", and the dashboard can surface a clear warning.
    ///
    /// Set `TADPOLE_NULL_PROVIDERS=true` to force all providers to null (CI/Test mode).
    pub(crate) fn resolve_provider(
        &self,
        ctx: &RunContext,
        client: reqwest::Client,
    ) -> ProviderVariant {
        tracing::info!(
            "🔍 [Provider] Resolving provider '{}' for agent '{}'",
            ctx.provider_name,
            ctx.agent_id
        );
        // CI / test override: force every provider to null
        if std::env::var("TADPOLE_NULL_PROVIDERS").as_deref() == Ok("true") {
            tracing::info!(
                "🧪 [Provider] TADPOLE_NULL_PROVIDERS=true — using NullProvider for agent '{}'",
                ctx.agent_id
            );
            return ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::TestMode));
        }

        // 🛡️ Privacy Guard: Enforce local-only traffic if Privacy Mode is active
        if self
            .state
            .governance
            .privacy_mode
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let p_lower = ctx.provider_name.to_lowercase();
            // Allow Ollama, local models, or anything explicitly tagged as 'local'
            if !p_lower.contains("ollama") && !p_lower.contains("local") {
                tracing::warn!(
                    "🛡️ [PrivacyShield] Blocked external provider '{}' for agent '{}' due to active Privacy Mode.",
                    ctx.provider_name,
                    ctx.agent_id
                );
                return ProviderVariant::Null(NullProvider::new(
                    &ctx.agent_id,
                    NullReason::PrivacyModeEnforced,
                ));
            }
        }

        let p_name = ctx.provider_name.to_lowercase();
        match p_name.as_str() {
            "google" | "gemini" | "google-ai-studio" => {
                tracing::info!(
                    "📡 [Runner] Calling Gemini API for agent {}...",
                    ctx.agent_id
                );
                match resolve_api_key(&ctx.model_config, "GOOGLE_API_KEY") {
                    Some(key) => {
                        ProviderVariant::Gemini(crate::agent::gemini::GeminiProvider::new(
                            client,
                            key,
                            ctx.model_config.clone(),
                        ))
                    }
                    None => {
                        tracing::warn!("⚠️  [Provider] No GOOGLE_API_KEY — activating NullProvider for agent '{}'", ctx.agent_id);
                        ProviderVariant::Null(NullProvider::new(
                            &ctx.agent_id,
                            NullReason::MissingApiKey {
                                env_var: "GOOGLE_API_KEY",
                            },
                        ))
                    }
                }
            }
            "groq" => {
                tracing::info!("📡 [Runner] Calling Groq API for agent {}...", ctx.agent_id);
                match resolve_api_key(&ctx.model_config, "GROQ_API_KEY") {
                    Some(key) => ProviderVariant::Groq(crate::agent::groq::GroqProvider::new(
                        client,
                        key,
                        ctx.model_config.clone(),
                    )),
                    None => {
                        tracing::warn!("⚠️  [Provider] No GROQ_API_KEY — activating NullProvider for agent '{}'", ctx.agent_id);
                        ProviderVariant::Null(NullProvider::new(
                            &ctx.agent_id,
                            NullReason::MissingApiKey {
                                env_var: "GROQ_API_KEY",
                            },
                        ))
                    }
                }
            }
            "openai" | "alibaba" | "mistral" | "meta" | "xai" => {
                tracing::info!(
                    "📡 [Runner] Calling OpenAI-compatible API (Alias: {}) for agent {}...",
                    ctx.provider_name,
                    ctx.agent_id
                );

                let key = match p_name.as_str() {
                    "alibaba" => resolve_api_key(&ctx.model_config, "ALIBABA_API_KEY")
                        .or_else(|| resolve_api_key(&ctx.model_config, "DASHSCOPE_API_KEY")),
                    "mistral" => resolve_api_key(&ctx.model_config, "MISTRAL_API_KEY"),
                    "xai" => resolve_api_key(&ctx.model_config, "XAI_API_KEY"),
                    _ => None,
                }
                .or_else(|| resolve_api_key(&ctx.model_config, "OPENAI_API_KEY"));

                match key {
                    Some(key) => {
                        ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                            client,
                            key,
                            ctx.model_config.clone(),
                        ))
                    }
                    None => {
                        let env_var = match p_name.as_str() {
                            "alibaba" => "ALIBABA_API_KEY",
                            "mistral" => "MISTRAL_API_KEY",
                            "xai" => "XAI_API_KEY",
                            _ => "OPENAI_API_KEY",
                        };
                        tracing::warn!(
                            "⚠️  [Provider] No {} — activating NullProvider for agent '{}'",
                            env_var,
                            ctx.agent_id
                        );
                        ProviderVariant::Null(NullProvider::new(
                            &ctx.agent_id,
                            NullReason::MissingApiKey { env_var },
                        ))
                    }
                }
            }
            "inception" => {
                tracing::info!(
                    "📡 [Runner] Calling Inception API for agent {}...",
                    ctx.agent_id
                );
                match resolve_api_key(&ctx.model_config, "INCEPTION_API_KEY") {
                    Some(key) => {
                        ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                            client,
                            key,
                            ctx.model_config.clone(),
                        ))
                    }
                    None => {
                        tracing::warn!("⚠️  [Provider] No INCEPTION_API_KEY — activating NullProvider for agent '{}'", ctx.agent_id);
                        ProviderVariant::Null(NullProvider::new(
                            &ctx.agent_id,
                            NullReason::MissingApiKey {
                                env_var: "INCEPTION_API_KEY",
                            },
                        ))
                    }
                }
            }
            "deepseek" => {
                tracing::info!(
                    "📡 [Runner] Calling DeepSeek API for agent {}...",
                    ctx.agent_id
                );
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
                    None => {
                        tracing::warn!("⚠️  [Provider] No DEEPSEEK_API_KEY — activating NullProvider for agent '{}'", ctx.agent_id);
                        ProviderVariant::Null(NullProvider::new(
                            &ctx.agent_id,
                            NullReason::MissingApiKey {
                                env_var: "DEEPSEEK_API_KEY",
                            },
                        ))
                    }
                }
            }
            "ollama" => {
                tracing::info!(
                    "📡 [Runner] Calling Ollama API for agent {}...",
                    ctx.agent_id
                );
                // Ollama does not require an API key — use a placeholder
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
            // SEC-02: Fuzzy resolution for descriptive Provider IDs (e.g. "ollama-local")
            unknown if unknown.contains("ollama") => {
                tracing::info!(
                    "📡 [Runner] Fuzzy-matched Ollama protocol for provider '{}'",
                    unknown
                );
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
            unknown if unknown.contains("openai") || unknown.contains("groq") => {
                tracing::info!(
                    "📡 [Runner] Fuzzy-matched OpenAI protocol for provider '{}'",
                    unknown
                );
                match resolve_api_key(&ctx.model_config, "OPENAI_API_KEY") {
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
                            env_var: "OPENAI_API_KEY",
                        },
                    )),
                }
            }
            unknown if unknown.contains("google") || unknown.contains("gemini") => {
                tracing::info!(
                    "📡 [Runner] Fuzzy-matched Google protocol for provider '{}'",
                    unknown
                );
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
            unknown => {
                tracing::warn!(
                    "⚠️  [Provider] Unknown provider '{}' — activating NullProvider for agent '{}'",
                    unknown,
                    ctx.agent_id
                );
                ProviderVariant::Null(NullProvider::new(
                    &ctx.agent_id,
                    NullReason::UnknownProvider {
                        name: unknown.to_string(),
                    },
                ))
            }
        }
    }

    /// Shared provider dispatch loop.
    ///
    /// Handles RPM/TPM rate limiting, provider resolution, and usage tracking
    /// across all model protocols. All generation requests aggregate here.
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

        // Shared Rate Limiter Key based on model configuration
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

        // Resolve provider via factory — never hard-errors on missing keys
        let provider = self.resolve_provider(ctx, client);

        let result = provider.generate(system_prompt, user_message, tools).await;

        // Record actual token usage against the limiter window
        if limiter.is_active() {
            if let Ok((_, _, Some(ref usage))) = &result {
                limiter.record_usage(usage.total_tokens);
                self.state.governance.tpm_accumulator.fetch_add(
                    usage.total_tokens as usize,
                    std::sync::atomic::Ordering::Relaxed,
                );
            }
        }

        result
    }

    // ─────────────────────────────────────────────────────────
    //  ERROR HANDLING
    // ─────────────────────────────────────────────────────────

    // Handles provider-level errors: resets agent state, fails the mission, logs.
    // Applies secret redaction to prevent API keys from leaking into UI.
    // ─────────────────────────────────────────────────────────
    //  BUDGET ENFORCEMENT
    // ─────────────────────────────────────────────────────────

    /// Checks if the mission has exceeded its budget. Returns early-exit message if breached.
    pub(crate) async fn check_budget(
        &self,
        ctx: &RunContext,
        _step_cost: f64,
        output_text: &str,
    ) -> anyhow::Result<Option<String>> {
        if let Some(mission) =
            crate::agent::mission::get_mission_by_id(&self.state.resources.pool, &ctx.mission_id)
                .await?
        {
            if mission.cost_usd >= mission.budget_usd {
                tracing::warn!("⚠️ [Protocol] Budget limit reached for Mission {}. Automatic shutdown initiated.", ctx.mission_id);

                self.broadcast_sys(&format!("⚠️ PROTOCOL ALERT: Mission {} exceeded budget (${:.4}). Swarm auto-paused.", mission.title, mission.budget_usd), "warning", Some(ctx.mission_id.clone()));

                crate::agent::mission::update_mission(
                    &self.state.resources.pool,
                    &ctx.mission_id,
                    crate::agent::types::MissionStatus::Paused,
                    0.0,
                )
                .await?;
                crate::agent::mission::log_step(
                    &self.state.resources.pool,
                    &ctx.mission_id,
                    &ctx.agent_id,
                    "Finance Analyst",
                    &format!("Emergency Pause: Neural cost (${:.4}) has exceeded allocated budget (${:.4}).", mission.cost_usd, mission.budget_usd),
                    "warning",
                    None
                ).await?;

                self.broadcast_agent_status(&ctx.agent_id, &ctx.mission_id, "idle");
                return Ok(Some(format!("(PAUSED: Budget Exceeded) {}", output_text)));
            }
        }
        Ok(None)
    }
}
