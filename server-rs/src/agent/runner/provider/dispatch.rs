//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / dispatch
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::ProviderVariant;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::{TokenUsage, ToolCall, ToolDefinition};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};

pub const OLLAMA_SYNTH_TIMEOUT_SECS: u64 = 900; // 15 minutes for local CPU/GPU debrief/synthesis
pub const OLLAMA_DEFAULT_TIMEOUT_SECS: u64 = 720; // 12 minutes for standard local inference
pub const CLOUD_SYNTH_TIMEOUT_SECS: u64 = 120; // 2 minutes for cloud debrief/synthesis
pub const OVERSIGHT_RESOLUTION_TIMEOUT_SECS: u64 = 30; // 30s timeout on human oversight resolution

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
                        | crate::agent::null_provider::NullReason::MissingBaseUrl { .. }
                ) {
                    tracing::warn!(
                        "⚠️ [Provider] Credentials/Base URL missing for agent '{}' (provider: {}). Triggering pre-flight failover...",
                        ctx.agent_id,
                        current_provider_id
                    );
                    let fallbacks = self.collect_fallback_candidates(&current_ctx, &seen);
                    if !fallbacks.is_empty() {
                        if attempt >= max_attempts {
                            tracing::error!(
                                "🚨 [Provider Failover] Max failover attempts ({}) exceeded. Failing with Auth error.",
                                max_attempts
                            );
                            return Err(AppError::InfrastructureError {
                                provider_id: ProviderId::from_name(&current_provider_id),
                                kind: InfrastructureErrorKind::ApiError,
                                detail: format!(
                                    "Max failover attempts ({}) exceeded. API key/URL missing for provider: {}",
                                    max_attempts,
                                    current_provider_id
                                ),
                                help_link: Some("https://docs.tadpoleos.dev/configuration/models".to_string()),
                            });
                        }

                        let fallback = &fallbacks[0];
                        tracing::warn!(
                            "🔄 [Provider Failover] Pre-flight switch to {} (model: {}) due to missing primary credentials",
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

                        let backoff_ms =
                            std::cmp::min(100 * (1 << attempt), 1000) + rand::random::<u64>() % 200;
                        tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                        continue;
                    } else if crate::utils::security::is_production_env() {
                        tracing::error!(
                            "🚨 [Provider] No fallback candidates available and credentials missing for '{}'. Aborting in production.",
                            current_provider_id
                        );
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::from_name(&current_provider_id),
                            kind: InfrastructureErrorKind::ApiError,
                            detail: format!(
                                "API key or base URL missing for provider: {} (no valid fallback candidates configured)",
                                current_provider_id
                            ),
                            help_link: Some("https://docs.tadpoleos.dev/configuration/models".to_string()),
                        });
                    } else {
                        tracing::warn!(
                            "⚠️ [Provider] No fallback candidates available for '{}'. Permitting graceful degradation (NullProvider) in dev/test environment.",
                            current_provider_id
                        );
                    }
                }
            }

            // Determine timeout duration:
            // - Local Ollama debrief/synthesis (user_message empty): OLLAMA_SYNTH_TIMEOUT_SECS (900s)
            // - Local Ollama standard execution: OLLAMA_DEFAULT_TIMEOUT_SECS (720s)
            // - Cloud debrief/synthesis (user_message empty): CLOUD_SYNTH_TIMEOUT_SECS (120s)
            // - Otherwise: read from governance provider_timeout_secs (default: 60s)
            use std::sync::atomic::Ordering;
            let timeout_secs = if current_ctx.model_config.provider
                == crate::agent::types::ModelProvider::Ollama
            {
                if user_message.is_empty() {
                    OLLAMA_SYNTH_TIMEOUT_SECS
                } else {
                    OLLAMA_DEFAULT_TIMEOUT_SECS
                }
            } else if user_message.is_empty() {
                CLOUD_SYNTH_TIMEOUT_SECS
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
                    let typed_provider = ProviderId::from_name(&current_provider_id);
                    Err(AppError::InfrastructureError {
                        provider_id: typed_provider,
                        kind: InfrastructureErrorKind::Timeout,
                        detail: format!(
                            "Request timeout: generate call exceeded {} seconds",
                            timeout_secs
                        ),
                        help_link: Some(
                            "https://docs.tadpoleos.dev/troubleshooting/timeouts".to_string(),
                        ),
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

                        // Find the next fallback candidate using current context
                        let fallbacks = self.collect_fallback_candidates(&current_ctx, &seen);

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

                            let audit_entry = crate::agent::types::ToolCallAudit {
                                id: uuid::Uuid::new_v4().to_string(),
                                agent_id: ctx.agent_id.clone(),
                                mission_id: Some(ctx.mission_id.clone()),
                                skill: "model_failover".to_string(),
                                params: params_val,
                                department: ctx.department.clone(),
                                description: proposed_desc,
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };

                            let oversight_fut = self.submit_oversight_resolution(
                                audit_entry,
                                Some(ctx.mission_id.clone()),
                            );

                            let res = match tokio::time::timeout(
                                std::time::Duration::from_secs(OVERSIGHT_RESOLUTION_TIMEOUT_SECS),
                                oversight_fut,
                            )
                            .await
                            {
                                Ok(Ok(resolution)) => resolution,
                                Ok(Err(submit_err)) => {
                                    tracing::error!(
                                        "🚫 [Oversight] Failed to submit oversight resolution: {:?}",
                                        submit_err
                                    );
                                    return Err(e);
                                }
                                Err(_) => {
                                    tracing::warn!(
                                        "⏱️ [Oversight] Resolution timed out after {}s; aborting failover",
                                        OVERSIGHT_RESOLUTION_TIMEOUT_SECS
                                    );
                                    return Err(e);
                                }
                            };

                            if !res.approved {
                                self.broadcast_sys(
                                    "❌ Failover rejected by Oversight. Execution aborted.",
                                    "error",
                                    Some(ctx.mission_id.clone()),
                                );
                                return Err(AppError::Forbidden(format!(
                                    "Failover from {} ({}) to {} ({}) rejected by Oversight",
                                    current_provider_id,
                                    current_ctx.model_config.model_id,
                                    fallback_config.provider,
                                    fallback_config.model_id
                                )));
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
                                        _ => {
                                            tracing::warn!("Override slot '{}' unknown during failover. Using default primary model.", slot);
                                            Some(&agent.models.model)
                                        }
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
                            let backoff_ms = std::cmp::min(100 * (1 << attempt), 1000)
                                + rand::random::<u64>() % 200;
                            tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                            continue;
                        }
                    }

                    return Err(e);
                }
            }
        }
    }
}
