//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - Grouped execution context owns identity, mission state and environment; retain scope across callers.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / fallback
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::ProviderVariant;
use crate::agent::model_routing::is_local_model_config;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::ModelConfig;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};

pub const OVERSIGHT_RESOLUTION_TIMEOUT_SECS: u64 = 30;

pub(super) fn format_failover_error_summary(e: &AppError) -> String {
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

pub(super) fn is_failover_trigger(e: &AppError) -> bool {
    e.is_rate_limit()
        || e.is_network_timeout()
        || match e {
            AppError::Reqwest(req_err) => req_err.status().is_some_and(|s| s.is_server_error()),
            _ => false,
        }
}

fn build_failover_audit_entry(
    ctx: &RunContext,
    current_provider_id: &str,
    fallback_config: &ModelConfig,
    error_summary: &str,
    description: String,
) -> crate::agent::types::ToolCallAudit {
    crate::agent::types::ToolCallAudit {
        id: uuid::Uuid::new_v4().to_string(),
        agent_id: ctx.identity.agent_id.clone(),
        mission_id: Some(ctx.mission_state.mission_id.clone()),
        skill: "model_failover".to_string(),
        params: serde_json::json!({
            "failed_provider": current_provider_id,
            "failed_model": ctx.model_config.model_id,
            "proposed_fallback_provider": fallback_config.provider.to_string(),
            "proposed_fallback_model": fallback_config.model_id.clone(),
            "error": error_summary,
        }),
        department: ctx.identity.department.clone(),
        description,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

fn is_candidate_eligible(
    slot: &ModelConfig,
    seen: &std::collections::HashSet<(String, String)>,
    privacy_mode_active: bool,
    primary_is_local: bool,
    allow_cloud: bool,
) -> bool {
    let key = (
        slot.provider.to_string().to_lowercase(),
        slot.model_id.clone(),
    );
    if seen.contains(&key) {
        return false;
    }
    let is_local = is_local_model_config(slot);
    if privacy_mode_active && !is_local {
        return false;
    }
    if primary_is_local && !is_local && !allow_cloud {
        return false;
    }
    true
}

impl AgentRunner {
    pub(crate) fn collect_fallback_candidates(
        &self,
        ctx: &RunContext,
        seen: &std::collections::HashSet<(String, String)>,
    ) -> Vec<ModelConfig> {
        let Some(agent_entry) = self.state.registry.agents.get(&ctx.identity.agent_id) else {
            return Vec::new();
        };
        let a = agent_entry.value();

        let allow_cloud = a
            .metadata
            .get("allow_cloud_failover")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut candidate_slots = Vec::new();
        let active_slot = a.models.active_model_slot.as_deref().unwrap_or("default");

        // Priority order cascade based on the active slot
        let slots = match active_slot {
            "planning" => vec![&a.models.execution_slot, &a.models.planning_slot],
            "execution" => vec![&a.models.planning_slot, &a.models.execution_slot],
            _ => vec![&a.models.planning_slot, &a.models.execution_slot],
        };

        for cfg in slots.into_iter().flatten() {
            candidate_slots.push(cfg.clone());
        }
        // Base model config is always the ultimate default fallback
        candidate_slots.push(a.models.model.clone());

        let privacy_mode_active = self.is_privacy_active(ctx);
        let primary_is_local = is_local_model_config(&ctx.model_config);

        candidate_slots
            .into_iter()
            .filter(|s| {
                is_candidate_eligible(s, seen, privacy_mode_active, primary_is_local, allow_cloud)
            })
            .collect()
    }

    pub(crate) fn load_failover_thresholds(&self, agent_id: &str) -> (u32, u32, u32) {
        use std::sync::atomic::Ordering;
        let mut amber_threshold = self
            .state
            .governance
            .failover_amber_threshold
            .load(Ordering::Relaxed);
        let mut red_threshold = self
            .state
            .governance
            .failover_red_threshold
            .load(Ordering::Relaxed);
        let mut max_attempts = self
            .state
            .governance
            .failover_max_attempts
            .load(Ordering::Relaxed);

        if let Some(agent_entry) = self.state.registry.agents.get(agent_id) {
            let a = agent_entry.value();
            if let Some(val) = a
                .metadata
                .get("failover_amber_threshold")
                .and_then(|v| v.as_u64())
            {
                amber_threshold = val as u32;
            }
            if let Some(val) = a
                .metadata
                .get("failover_red_threshold")
                .and_then(|v| v.as_u64())
            {
                red_threshold = val as u32;
            }
            if let Some(val) = a
                .metadata
                .get("failover_max_attempts")
                .and_then(|v| v.as_u64())
            {
                max_attempts = val as u32;
            }
        }
        (amber_threshold, red_threshold, max_attempts)
    }

    pub(crate) fn attempt_proactive_failover(
        &self,
        ctx: &RunContext,
        health: crate::agent::types::ProviderStatus,
        current_ctx: &mut RunContext,
        current_provider_id: &mut String,
        seen: &mut std::collections::HashSet<(String, String)>,
    ) {
        use crate::agent::types::ProviderStatus;
        if health != ProviderStatus::Red {
            return;
        }
        tracing::warn!(
            "🚨 [Provider] {} is in RED state. Attempting proactive failover...",
            ctx.provider_name
        );
        let fallbacks = self.collect_fallback_candidates(ctx, seen);
        let Some(fallback) = fallbacks.first() else {
            return;
        };
        tracing::warn!(
            "🔄 [Provider Failover] Proactive switch to {} ({}) because primary is RED",
            fallback.provider,
            fallback.model_id
        );
        current_ctx.model_config = fallback.clone();
        current_ctx.provider_name = fallback.provider.to_string().to_lowercase();
        *current_provider_id = current_ctx.provider_name.clone();
        seen.insert((
            fallback.provider.to_string().to_lowercase(),
            fallback.model_id.clone(),
        ));
    }

    pub(super) async fn handle_preflight_null_failover(
        &self,
        ctx: &RunContext,
        current_provider_id: &str,
        seen: &std::collections::HashSet<(String, String)>,
        attempt: u32,
        max_attempts: u32,
    ) -> Result<Option<ModelConfig>, AppError> {
        tracing::warn!(
            "⚠️ [Provider] Credentials/Base URL missing for agent '{}' (provider: {}). Triggering pre-flight failover...",
            ctx.identity.agent_id,
            current_provider_id
        );
        let fallbacks = self.collect_fallback_candidates(ctx, seen);
        if fallbacks.is_empty() && !crate::utils::security::is_production_env() {
            tracing::warn!(
                "⚠️ [Provider] Permitting NullProvider in dev/test for '{}'.",
                current_provider_id
            );
            return Ok(None);
        }
        if fallbacks.is_empty() {
            let detail = format!("API key or base URL missing for provider: {} (no valid fallback candidates configured)", current_provider_id);
            tracing::error!("🚨 [Provider] No fallback candidates: {}", detail);
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::from_name(current_provider_id),
                kind: InfrastructureErrorKind::ApiError,
                detail,
                help_link: Some("https://docs.tadpoleos.dev/configuration/models".to_string()),
            });
        }

        if attempt >= max_attempts {
            tracing::error!("🚨 [Provider Failover] Max failover attempts ({}) exceeded. Failing with Auth error.", max_attempts);
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::from_name(current_provider_id),
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "Max failover attempts ({}) exceeded. API key/URL missing for provider: {}",
                    max_attempts, current_provider_id
                ),
                help_link: Some("https://docs.tadpoleos.dev/configuration/models".to_string()),
            });
        }

        let fallback = fallbacks[0].clone();
        tracing::warn!(
            "🔄 [Provider Failover] Pre-flight switch to {} (model: {})",
            fallback.provider.to_string(),
            fallback.model_id
        );
        Ok(Some(fallback))
    }

    pub(super) async fn try_null_preflight_failover(
        &self,
        provider: &ProviderVariant,
        ctx: &RunContext,
        provider_id: &str,
        seen: &std::collections::HashSet<(String, String)>,
        attempt: u32,
        max_attempts: u32,
    ) -> Result<Option<ModelConfig>, AppError> {
        let ProviderVariant::Null(ref null_prov) = provider else {
            return Ok(None);
        };
        let is_missing_creds = matches!(
            null_prov.reason,
            crate::agent::null_provider::NullReason::MissingApiKey { .. }
                | crate::agent::null_provider::NullReason::MissingBaseUrl { .. }
        );
        if !is_missing_creds {
            return Ok(None);
        }
        self.handle_preflight_null_failover(ctx, provider_id, seen, attempt, max_attempts)
            .await
    }

    fn resolve_override_slot_config(
        &self,
        agent_id: &str,
        slot: &str,
        fallback_config: &ModelConfig,
    ) -> ModelConfig {
        let Some(entry) = self.state.registry.agents.get(agent_id) else {
            return fallback_config.clone();
        };
        let agent = entry.value();
        let config_opt = match slot {
            "planning" => agent.models.planning_slot.as_ref(),
            "execution" => agent.models.execution_slot.as_ref(),
            _ => Some(&agent.models.model),
        };
        config_opt
            .cloned()
            .unwrap_or_else(|| fallback_config.clone())
    }

    pub(super) async fn evaluate_failover_oversight(
        &self,
        ctx: &RunContext,
        current_provider_id: &str,
        fallback_config: &ModelConfig,
        error_summary: &str,
    ) -> Result<ModelConfig, AppError> {
        let proposed_desc = format!(
            "Failover from {} ({}) to {} ({}) due to error: {}",
            current_provider_id,
            ctx.model_config.model_id,
            fallback_config.provider,
            fallback_config.model_id,
            error_summary
        );

        self.broadcast_agent(
            ctx,
            &format!(
                "⚠️ Oversight: Model failover proposed from '{}' ({}) to '{}' ({}). CRITICAL REVIEW REQUIRED.",
                current_provider_id, ctx.model_config.model_id, fallback_config.provider, fallback_config.model_id
            ),
            "error",
        );

        let audit_entry = build_failover_audit_entry(
            ctx,
            current_provider_id,
            fallback_config,
            error_summary,
            proposed_desc,
        );

        let oversight_fut = self
            .submit_oversight_resolution(audit_entry, Some(ctx.mission_state.mission_id.clone()));

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
                return Err(AppError::Forbidden(
                    "Oversight submission failed".to_string(),
                ));
            }
            Err(_) => {
                tracing::warn!(
                    "⏱️ [Oversight] Resolution timed out after {}s; aborting failover",
                    OVERSIGHT_RESOLUTION_TIMEOUT_SECS
                );
                return Err(AppError::Forbidden(
                    "Oversight resolution timed out".to_string(),
                ));
            }
        };

        if !res.approved {
            self.broadcast_sys(
                "❌ Failover rejected by Oversight. Execution aborted.",
                "error",
                Some(ctx.mission_state.mission_id.clone()),
            );
            return Err(AppError::Forbidden(format!(
                "Failover from {} ({}) to {} ({}) rejected by Oversight",
                current_provider_id,
                ctx.model_config.model_id,
                fallback_config.provider,
                fallback_config.model_id
            )));
        }

        let slot = res.override_slot.as_deref().unwrap_or("");
        Ok(self.resolve_override_slot_config(&ctx.identity.agent_id, slot, fallback_config))
    }
}
