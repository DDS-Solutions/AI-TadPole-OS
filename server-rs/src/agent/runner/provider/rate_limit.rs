//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / rate_limit
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::TokenUsage;

impl AgentRunner {
    pub(crate) async fn acquire_rate_limit(
        &self,
        agent_id: &str,
        current_ctx: &RunContext,
        system_prompt: &str,
        user_message: &str,
    ) {
        let limiter_key = format!(
            "{}:{}:{}",
            agent_id, current_ctx.provider_name, current_ctx.model_config.model_id
        );
        let effective_rpm = current_ctx.model_config.rpm.or_else(|| {
            if current_ctx.model_config.provider == crate::agent::types::ModelProvider::Openrouter {
                Some(30) // Default 30 RPM for OpenRouter shared pools
            } else {
                None
            }
        });

        let limiter = self
            .state
            .resources
            .rate_limiters
            .entry(limiter_key)
            .or_insert_with(|| {
                std::sync::Arc::new(crate::agent::rate_limiter::RateLimiter::new(
                    effective_rpm,
                    current_ctx.model_config.tpm,
                ))
            })
            .value()
            .clone();

        if limiter.is_active() {
            let estimated_tokens = ((system_prompt.len() + user_message.len()) as f64 / 3.5) as u32;
            limiter.acquire(estimated_tokens).await;
        }

        // Inter-turn pacing delay for cloud providers to prevent burst rate limits
        if current_ctx.model_config.provider == crate::agent::types::ModelProvider::Openrouter {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    pub(crate) fn record_rate_limit_usage(
        &self,
        agent_id: &str,
        current_ctx: &RunContext,
        usage: Option<&TokenUsage>,
    ) {
        use std::sync::atomic::Ordering;
        if let Some(u) = usage {
            let limiter_key = format!(
                "{}:{}:{}",
                agent_id, current_ctx.provider_name, current_ctx.model_config.model_id
            );
            if let Some(limiter_ref) = self.state.resources.rate_limiters.get(&limiter_key) {
                let limiter = limiter_ref.value();
                if limiter.is_active() {
                    limiter.record_usage(u.total_tokens);
                    self.state
                        .governance
                        .tpm_accumulator
                        .fetch_add(u.total_tokens as usize, Ordering::Relaxed);
                }
            }
        }
    }

    pub(crate) fn update_provider_failure(
        &self,
        provider_id: &str,
        amber_threshold: u32,
        red_threshold: u32,
    ) -> (u32, crate::agent::types::ProviderStatus) {
        use crate::agent::types::ProviderStatus;
        use std::sync::atomic::Ordering;

        let failures = self
            .state
            .registry
            .provider_failures
            .entry(provider_id.to_string())
            .or_insert_with(|| std::sync::atomic::AtomicU32::new(0));
        let count = failures.fetch_add(1, Ordering::Relaxed) + 1;

        let new_status = if count >= red_threshold {
            ProviderStatus::Red
        } else if count >= amber_threshold {
            ProviderStatus::Amber
        } else {
            ProviderStatus::Green
        };

        self.state
            .registry
            .provider_health
            .insert(provider_id.to_string(), new_status);

        (count, new_status)
    }

    pub(crate) fn force_provider_red(&self, provider_id: &str, red_threshold: u32) {
        use crate::agent::types::ProviderStatus;
        use std::sync::atomic::Ordering;

        let failures = self
            .state
            .registry
            .provider_failures
            .entry(provider_id.to_string())
            .or_insert_with(|| std::sync::atomic::AtomicU32::new(0));
        failures.store(red_threshold, Ordering::Relaxed);

        self.state
            .registry
            .provider_health
            .insert(provider_id.to_string(), ProviderStatus::Red);
    }
}
