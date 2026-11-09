//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / dispatch
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

use super::ProviderVariant;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::{ModelConfig, TokenUsage, ToolCall, ToolDefinition};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use std::sync::atomic::Ordering;

pub const OLLAMA_SYNTH_TIMEOUT_SECS: u64 = 900; // 15 minutes for local CPU/GPU debrief/synthesis
pub const OLLAMA_DEFAULT_TIMEOUT_SECS: u64 = 720; // 12 minutes for standard local inference
pub const CLOUD_SYNTH_TIMEOUT_SECS: u64 = 120; // 2 minutes for cloud debrief/synthesis
pub const OVERSIGHT_RESOLUTION_TIMEOUT_SECS: u64 = 30; // 30s timeout on human oversight resolution

fn determine_dispatch_timeout(
    model_provider: crate::agent::types::ModelProvider,
    user_message: &str,
    governance_timeout: u64,
) -> u64 {
    if model_provider == crate::agent::types::ModelProvider::Ollama {
        if user_message.is_empty() {
            OLLAMA_SYNTH_TIMEOUT_SECS
        } else {
            OLLAMA_DEFAULT_TIMEOUT_SECS
        }
    } else if user_message.is_empty() {
        CLOUD_SYNTH_TIMEOUT_SECS
    } else {
        governance_timeout
    }
}

fn format_failover_error_summary(e: &AppError) -> String {
    match e {
        AppError::RateLimit(msg) => format!("Rate limit: {}", msg),
        AppError::Reqwest(err) => {
            if let Some(status) = err.status() {
                format!("HTTP {} error", status.as_u16())
            } else {
                "Network timeout/error".to_string()
            }
        }
        _ => e.type_slug().replace('-', " ").to_uppercase(),
    }
}

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

        let (amber_threshold, red_threshold, max_attempts) =
            self.load_failover_thresholds(&ctx.identity.agent_id);

        let client = (*self.state.resources.http_client).clone();
        let mut current_ctx = ctx.clone();
        let mut current_provider_id = provider_id.clone();

        let mut seen = std::collections::HashSet::new();
        seen.insert((
            ctx.model_config.provider.to_string().to_lowercase(),
            ctx.model_config.model_id.clone(),
        ));

        self.attempt_proactive_failover(
            ctx,
            health,
            &mut current_ctx,
            &mut current_provider_id,
            &mut seen,
        );

        self.acquire_rate_limit(
            &ctx.identity.agent_id,
            &current_ctx,
            system_prompt,
            user_message,
        )
        .await;

        let mut attempt: u32 = 0;

        loop {
            let provider = self.resolve_provider(&current_ctx, client.clone()).await;

            if let ProviderVariant::Null(ref null_prov) = provider {
                let is_missing_creds = matches!(
                    null_prov.reason,
                    crate::agent::null_provider::NullReason::MissingApiKey { .. }
                        | crate::agent::null_provider::NullReason::MissingBaseUrl { .. }
                );
                if is_missing_creds {
                    let maybe_fallback = self
                        .handle_preflight_null_failover(
                            &current_ctx,
                            &current_provider_id,
                            &seen,
                            attempt,
                            max_attempts,
                        )
                        .await?;
                    if let Some(fallback) = maybe_fallback {
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
                    }
                }
            }

            let gov_timeout = self
                .state
                .governance
                .provider_timeout_secs
                .load(Ordering::Relaxed) as u64;
            let timeout_secs = determine_dispatch_timeout(
                current_ctx.model_config.provider,
                user_message,
                gov_timeout,
            );

            let priced_config = match &provider {
                ProviderVariant::OpenAI(p) => p.config.clone(),
                _ => current_ctx.model_config.clone(),
            };
            let free = matches!(&provider, ProviderVariant::Null(_));
            let tool_bytes = tools
                .as_ref()
                .and_then(|t| serde_json::to_vec(t).ok())
                .map_or(0, |bytes| bytes.len());
            let input_estimate = ((system_prompt.len() + user_message.len() + tool_bytes) as f64
                / 3.5)
                .ceil() as u64;
            let price = |input, output| {
                if free {
                    0.0
                } else {
                    crate::agent::rates::calculate_cost_with_custom_rates(
                        priced_config.input_cost_per_m,
                        priced_config.output_cost_per_m,
                        Some(&priced_config.provider.to_string()),
                        &priced_config.model_id,
                        input,
                        output,
                    )
                }
            };
            let estimate = price(
                input_estimate,
                priced_config.max_tokens.unwrap_or(4096) as u64,
            );
            if self.check_budget(ctx, estimate, "").await?.is_some() {
                return Err(AppError::Forbidden(
                    "Mission budget exhausted before provider dispatch".to_string(),
                ));
            }
            let mut reservation = self
                .state
                .security
                .budget_guard
                .reserve_call(
                    &ctx.identity.agent_id,
                    &ctx.mission_state.mission_id,
                    estimate,
                    ctx.mission_state.budget_usd,
                )
                .await?
                .ok_or_else(|| {
                    AppError::Forbidden("Persistent agent or mission quota exhausted".to_string())
                })?;

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
                    let cost = usage
                        .as_ref()
                        .map_or(0.0, |u| price(u.input_tokens, u.output_tokens));
                    reservation.settle(cost)?;
                    let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
                        "type": "agent:cost_estimate",
                        "agent_id": ctx.identity.agent_id,
                        "mission_id": ctx.mission_state.mission_id,
                        "model": priced_config.model_id,
                        "pricing_source": if free { "degraded_provider" } else {
                            crate::agent::rates::pricing_source(priced_config.input_cost_per_m, priced_config.output_cost_per_m,
                                Some(&priced_config.provider.to_string()), &priced_config.model_id)
                        },
                        "cost_usd": cost,
                        "cost_is_estimate": true,
                        "input_tokens_estimated": input_estimate,
                        "usage": usage,
                    }));
                    ctx.execution_metrics.record_cost(cost);
                    ctx.dispatch_accounted.store(true, Ordering::Release);
                    if let Some(mut agent) =
                        self.state.registry.agents.get_mut(&ctx.identity.agent_id)
                    {
                        agent.economics.cost_usd += cost;
                        if let Some(u) = &usage {
                            agent.economics.tokens_used += u.total_tokens;
                            agent.economics.token_usage = u.clone();
                        }
                    }
                    self.state
                        .registry
                        .provider_failures
                        .remove(&current_provider_id);
                    self.state
                        .registry
                        .provider_health
                        .insert(current_provider_id, ProviderStatus::Green);
                    self.record_rate_limit_usage(
                        &ctx.identity.agent_id,
                        &current_ctx,
                        usage.as_ref(),
                    );
                    return Ok((text, tool_calls, usage));
                }
                Err(e) => {
                    drop(reservation);
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

                    let is_trigger = e.is_rate_limit()
                        || e.is_network_timeout()
                        || match &e {
                            AppError::Reqwest(req_err) => {
                                req_err.status().is_some_and(|s| s.is_server_error())
                            }
                            _ => false,
                        };

                    if is_trigger && attempt < max_attempts {
                        self.force_provider_red(&current_provider_id, red_threshold);
                        let fallbacks = self.collect_fallback_candidates(&current_ctx, &seen);
                        if !fallbacks.is_empty() {
                            let fallback_config = &fallbacks[0];
                            let error_summary = format_failover_error_summary(&e);
                            let final_config = self
                                .evaluate_failover_oversight(
                                    &current_ctx,
                                    &current_provider_id,
                                    fallback_config,
                                    &error_summary,
                                )
                                .await
                                .map_err(|_| e)?;

                            current_ctx.model_config = final_config.clone();
                            current_ctx.provider_name =
                                final_config.provider.to_string().to_lowercase();
                            current_provider_id = current_ctx.provider_name.clone();
                            seen.insert((
                                final_config.provider.to_string().to_lowercase(),
                                final_config.model_id.clone(),
                            ));

                            self.acquire_rate_limit(
                                &ctx.identity.agent_id,
                                &current_ctx,
                                system_prompt,
                                user_message,
                            )
                            .await;
                            tracing::warn!(
                                "🔄 [Provider Failover] Switched to {} (model: {})",
                                current_ctx.provider_name,
                                current_ctx.model_config.model_id
                            );
                            self.broadcast_sys(
                                &format!("🔄 Provider Failover: Switched to {} (model: {}) due to error: {}", current_ctx.provider_name, current_ctx.model_config.model_id, error_summary),
                                "warning",
                                Some(ctx.mission_state.mission_id.clone()),
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

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::agent::types::{EngineAgent, ModelConfig, ModelProvider};
    use crate::state::AppState;
    use axum::{routing::post, Json, Router};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[tokio::test]
    async fn planning_and_synthesis_costs_block_next_call_and_finalize_once() {
        let count = Arc::new(AtomicUsize::new(0));
        let calls = count.clone();
        let app = Router::new().route("/v1/chat/completions", post(move || {
            let calls = calls.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Json(serde_json::json!({
                    "choices": [{"message": {"role": "assistant", "content": "done"}, "finish_reason": "stop"}],
                    "usage": {"prompt_tokens": 1, "completion_tokens": 100, "total_tokens": 101}
                }))
            }
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let state = Arc::new(AppState::new_minimal_mock().await);
        let mut agent = EngineAgent::default();
        agent.identity.id = "accounted-agent".to_string();
        agent.economics.budget_usd = 1.0;
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);
        crate::agent::mission::create_mission_with_id(
            &state.resources.pool,
            "accounted-mission",
            "accounted-agent",
            "accounting",
            0.25,
        )
        .await
        .unwrap();
        let runner = AgentRunner::new(state.clone());
        let ctx = RunContext {
            identity: crate::agent::runner::AgentIdentity {
                agent_id: "accounted-agent".to_string(),
                ..Default::default()
            },
            mission_state: crate::agent::runner::MissionState {
                mission_id: "accounted-mission".to_string(),
                budget_usd: 0.25,
                ..Default::default()
            },
            provider_name: "openai".to_string(),
            model_config: ModelConfig {
                provider: ModelProvider::Openai,
                model_id: "gpt-4o-mini".to_string(),
                api_key: Some("local-test-only".to_string()),
                base_url: Some(format!("http://{address}/v1")),
                max_tokens: Some(100),
                input_cost_per_m: Some(0.0),
                output_cost_per_m: Some(1000.0),
                ..Default::default()
            },
            ..Default::default()
        };
        let (_, _, first_usage) = runner
            .call_provider(&ctx, "planning", "safe task", None)
            .await
            .unwrap();
        let (_, _, second_usage) = runner
            .call_provider_for_synthesis(&ctx, "synthesis", None)
            .await
            .unwrap();
        assert!(matches!(
            runner
                .call_provider(&ctx, "next turn", "safe task", None)
                .await,
            Err(AppError::Forbidden(_))
        ));
        assert_eq!(count.load(Ordering::SeqCst), 2);
        let mut combined = first_usage;
        runner.accumulate_usage(&mut combined, second_usage);
        runner.finalize_run(&ctx, "done", &combined).await.unwrap();
        let cost: f64 = sqlx::query_scalar(
            "SELECT cost_usd FROM mission_history WHERE id = 'accounted-mission'",
        )
        .fetch_one(&state.resources.pool)
        .await
        .unwrap();
        assert!((cost - 0.2).abs() < 1e-9);
        for query in [
            "SELECT used_usd FROM agent_quotas WHERE entity_id = 'accounted-agent'",
            "SELECT used_usd FROM mission_quotas WHERE cluster_id = 'accounted-mission'",
        ] {
            let used_micros: i64 = sqlx::query_scalar(query)
                .fetch_one(&state.resources.pool)
                .await
                .unwrap();
            assert_eq!(used_micros, 200_000);
        }
        assert!(
            (state
                .registry
                .agents
                .get("accounted-agent")
                .unwrap()
                .economics
                .cost_usd
                - 0.2)
                .abs()
                < 1e-9
        );
        assert_eq!(
            state
                .registry
                .agents
                .get("accounted-agent")
                .unwrap()
                .economics
                .tokens_used,
            202
        );
        server.abort();
        let _ = server.await;
    }
}
