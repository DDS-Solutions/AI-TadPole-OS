//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Provider Dispatch**: Handles the main routing, oversight failover loops, and LLM call executions.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Failover loop bounds exceeded or network transport timeouts.
//!

use super::ProviderVariant;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::{TokenUsage, ToolCall, ToolDefinition};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};

impl AgentRunner {
    pub(crate) async fn dispatch_to_provider(
        &self,
        ctx: &RunContext,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        use crate::agent::types::ProviderStatus;

        let provider_id = ctx.provider_name.clone();
        let health = self
            .state
            .registry
            .provider_health
            .get(&provider_id)
            .map(|h| *h.value())
            .unwrap_or(ProviderStatus::Green);

        // Load failover settings from governance with potential per-agent override
        let (amber_threshold, red_threshold, max_attempts) =
            self.load_failover_thresholds(&ctx.agent_id);

        let client = (*self.state.resources.http_client).clone();
        let mut current_ctx = ctx.clone();
        let mut current_provider_id = provider_id.clone();

        // Track tried configurations to avoid retrying any failed setup (lowercased provider name)
        let mut seen = std::collections::HashSet::new();
        seen.insert((
            ctx.model_config.provider.to_string().to_lowercase(),
            ctx.model_config.model_id.clone(),
        ));

        // Pre-emptive Proactive Failover for RED status
        self.attempt_proactive_failover(
            ctx,
            health,
            &mut current_ctx,
            &mut current_provider_id,
            &mut seen,
        );

        // Rate Limiter Management
        self.acquire_rate_limit(&ctx.agent_id, &current_ctx, system_prompt, user_message)
            .await;

        let mut attempt: u32 = 0;

        loop {
            let provider = self.resolve_provider(&current_ctx, client.clone()).await;

            // Pre-flight check for MissingApiKey to trigger failover proactively
            if let ProviderVariant::Null(ref null_prov) = provider {
                if matches!(
                    null_prov.reason,
                    crate::agent::null_provider::NullReason::MissingApiKey { .. }
                ) {
                    tracing::warn!(
                        "⚠️ [Provider] API Key missing for sub-agent '{}' (provider: {}). Triggering pre-flight failover...",
                        ctx.agent_id,
                        current_provider_id
                    );
                    let fallbacks = self.collect_fallback_candidates(&current_ctx, &seen);
                    if !fallbacks.is_empty() {
                        let fallback = &fallbacks[0];
                        tracing::warn!(
                            "🔄 [Provider Failover] Pre-flight switch to {} (model: {}) due to missing primary API key",
                            fallback.provider.to_string(),
                            fallback.model_id
                        );
                        current_ctx.model_config = fallback.clone();
                        current_ctx.provider_name = fallback.provider.to_string().to_lowercase();
                        current_provider_id = current_ctx.provider_name.clone();
                        seen.insert((
                            fallback.provider.to_string().to_lowercase(),
                            fallback.model_id.clone(),
                        ));
                        attempt += 1;
                        if attempt > max_attempts {
                            tracing::error!("🚨 [Provider Failover] Max failover attempts ({}) exceeded. Failing with Auth error.", max_attempts);
                            return Err(AppError::InfrastructureError {
                                provider_id: match current_provider_id.as_str() {
                                    "anthropic" => crate::error::ProviderId::Anthropic,
                                    "openai" => crate::error::ProviderId::OpenAi,
                                    "gemini" => crate::error::ProviderId::Gemini,
                                    "groq" => crate::error::ProviderId::Groq,
                                    _ => crate::error::ProviderId::System,
                                },
                                kind: crate::error::InfrastructureErrorKind::ApiError,
                                detail: format!(
                                    "API key missing for provider: {}",
                                    current_provider_id
                                ),
                                help_link: None,
                            });
                        }
                        continue;
                    }
                }
            }

            // Determine timeout duration:
            // - Local Ollama: 300s (or 480s for synthesis/debrief calls) to allow slow CPU offloads
            // - Synthesis/debrief calls (user_message is empty) for cloud: 120s
            // - Otherwise: read from governance provider_timeout_secs (default: 60s)
            use std::sync::atomic::Ordering;
            let timeout_secs = if current_ctx.model_config.provider
                == crate::agent::types::ModelProvider::Ollama
            {
                if user_message.is_empty() {
                    900 // Give local Ollama synthesis/debrief calls up to 15 minutes for slow CPU/GPU offloads
                } else {
                    720 // Give standard local Ollama calls 720s (12 minutes) to avoid timeout during CPU prefill
                }
            } else if user_message.is_empty() {
                120
            } else {
                self.state
                    .governance
                    .provider_timeout_secs
                    .load(Ordering::Relaxed) as u64
            };

            let generate_fut = provider.generate(system_prompt, user_message, tools.clone());
            let result = match tokio::time::timeout(
                std::time::Duration::from_secs(timeout_secs),
                generate_fut,
            )
            .await
            {
                Ok(res) => res,
                Err(_) => {
                    let typed_provider = match current_provider_id.as_str() {
                        "anthropic" => ProviderId::Anthropic,
                        "openai" => ProviderId::OpenAi,
                        "gemini" => ProviderId::Gemini,
                        "groq" => ProviderId::Groq,
                        "mcp" => ProviderId::Mcp,
                        "audio" => ProviderId::Audio,
                        "runner" => ProviderId::Runner,
                        _ => ProviderId::System,
                    };
                    Err(AppError::InfrastructureError {
                        provider_id: typed_provider,
                        kind: InfrastructureErrorKind::Timeout,
                        detail: format!(
                            "Request timeout: generate call exceeded {} seconds",
                            timeout_secs
                        ),
                        help_link: None,
                    })
                }
            };

            match result {
                Ok((text, tool_calls, usage)) => {
                    // Success: Reset failures and restore Green status
                    self.state
                        .registry
                        .provider_failures
                        .remove(&current_provider_id);
                    self.state
                        .registry
                        .provider_health
                        .insert(current_provider_id, ProviderStatus::Green);

                    self.record_rate_limit_usage(&ctx.agent_id, &current_ctx, usage.as_ref());
                    return Ok((text, tool_calls, usage));
                }
                Err(e) => {
                    // Failure: Increment count and update status using helper
                    let (count, new_status) = self.update_provider_failure(
                        &current_provider_id,
                        amber_threshold,
                        red_threshold,
                    );

                    tracing::warn!(
                        "⚠️ [Provider] {} failed (count: {}). New status: {:?}",
                        current_provider_id,
                        count,
                        new_status
                    );

                    // Check if the error triggers failover using typed helper methods (captures all 5xx server errors)
                    let is_trigger = e.is_rate_limit()
                        || e.is_network_timeout()
                        || match &e {
                            AppError::Reqwest(req_err) => {
                                if let Some(status) = req_err.status() {
                                    status.is_server_error()
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        };

                    if is_trigger && attempt < max_attempts {
                        // Mark current provider status as RED explicitly on trigger.
                        self.force_provider_red(&current_provider_id, red_threshold);

                        // Find the next fallback candidate using the deduplicated helper
                        let fallbacks = self.collect_fallback_candidates(ctx, &seen);

                        if !fallbacks.is_empty() {
                            // Pick the first valid fallback
                            let fallback_config = &fallbacks[0];

                            // Sanitize error info to prevent leaking PII or credentials to UI/channels
                            let error_summary = match &e {
                                AppError::RateLimit(msg) => format!("Rate limit: {}", msg),
                                AppError::Reqwest(err) => {
                                    if let Some(status) = err.status() {
                                        format!("HTTP {} error", status.as_u16())
                                    } else {
                                        "Network timeout/error".to_string()
                                    }
                                }
                                _ => e.type_slug().replace('-', " ").to_uppercase(),
                            };

                            let proposed_desc = format!(
                                "Failover from {} ({}) to {} ({}) due to error: {}",
                                current_provider_id,
                                current_ctx.model_config.model_id,
                                fallback_config.provider,
                                fallback_config.model_id,
                                error_summary
                            );

                            self.broadcast_agent(
                                ctx,
                                &format!(
                                    "⚠️ Oversight: Model failover proposed from '{}' ({}) to '{}' ({}). CRITICAL REVIEW REQUIRED.",
                                    current_provider_id,
                                    current_ctx.model_config.model_id,
                                    fallback_config.provider,
                                    fallback_config.model_id
                                ),
                                "error",
                            );

                            let params_val = serde_json::json!({
                                "failed_provider": current_provider_id,
                                "failed_model": current_ctx.model_config.model_id,
                                "proposed_fallback_provider": fallback_config.provider.to_string(),
                                "proposed_fallback_model": fallback_config.model_id.clone(),
                                "error": error_summary,
                            });

                            let res = self
                                .submit_oversight_resolution(
                                    crate::agent::types::ToolCallAudit {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        agent_id: ctx.agent_id.clone(),
                                        mission_id: Some(ctx.mission_id.clone()),
                                        skill: "model_failover".to_string(),
                                        params: params_val,
                                        department: ctx.department.clone(),
                                        description: proposed_desc,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    },
                                    Some(ctx.mission_id.clone()),
                                )
                                .await?;

                            if !res.approved {
                                self.broadcast_sys(
                                    "❌ Failover rejected by Oversight. Execution aborted.",
                                    "error",
                                    Some(ctx.mission_id.clone()),
                                );
                                return Err(e);
                            }

                            // Use the approved slot configuration if overridden, otherwise proposed config
                            let mut final_config = fallback_config.clone();
                            let mut slot_used = "proposed fallback".to_string();
                            if let Some(slot) = &res.override_slot {
                                if let Some(entry) = self.state.registry.agents.get(&ctx.agent_id) {
                                    let agent = entry.value();
                                    let config_opt = match slot.as_str() {
                                        "planning" => agent.models.planning_slot.as_ref(),
                                        "execution" => agent.models.execution_slot.as_ref(),
                                        _ => Some(&agent.models.model),
                                    };
                                    if let Some(config) = config_opt {
                                        final_config = config.clone();
                                        slot_used = slot.clone();
                                    } else {
                                        tracing::warn!("Override slot '{}' requested during failover, but config is missing. Using proposed fallback instead.", slot);
                                    }
                                }
                            }

                            current_ctx.model_config = final_config.clone();
                            current_ctx.provider_name =
                                final_config.provider.to_string().to_lowercase();
                            current_provider_id = current_ctx.provider_name.clone();
                            seen.insert((
                                final_config.provider.to_string().to_lowercase(),
                                final_config.model_id.clone(),
                            ));

                            self.acquire_rate_limit(
                                &ctx.agent_id,
                                &current_ctx,
                                system_prompt,
                                user_message,
                            )
                            .await;

                            tracing::warn!(
                                "🔄 [Provider Failover] Switched to {} (model: {}) due to error: {:?}",
                                current_ctx.provider_name,
                                current_ctx.model_config.model_id,
                                e
                            );

                            self.broadcast_sys(
                                &format!(
                                    "🔄 Provider Failover: Switched to {} (model: {}) [slot: {}] due to error: {}",
                                    current_ctx.provider_name,
                                    current_ctx.model_config.model_id,
                                    slot_used,
                                    error_summary
                                ),
                                "warning",
                                Some(ctx.mission_id.clone()),
                            );

                            attempt += 1;
                            continue;
                        }
                    }

                    return Err(e);
                }
            }
        }
    }
}
