//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Provider Fallback**: Implements the model fallback priority lists and proactive failovers.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Failover loop cycles without finding a candidate or missing agent registry config.
//!

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::model_routing::{is_local_model_config};
use crate::agent::types::ModelConfig;

impl AgentRunner {
    pub(crate) fn collect_fallback_candidates(
        &self,
        ctx: &RunContext,
        seen: &std::collections::HashSet<(String, String)>,
    ) -> Vec<ModelConfig> {
        let Some(agent_entry) = self.state.registry.agents.get(&ctx.agent_id) else {
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

        let privacy_mode_active = self
            .state
            .governance
            .is_privacy_mode_enabled(Some(&ctx.mission_id));
        let primary_is_local = is_local_model_config(&ctx.model_config);

        candidate_slots
            .into_iter()
            .filter(|slot| {
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
        if health == ProviderStatus::Red {
            tracing::warn!(
                "🚨 [Provider] {} is in RED state. Attempting proactive failover...",
                ctx.provider_name
            );

            let fallbacks = self.collect_fallback_candidates(ctx, seen);
            if !fallbacks.is_empty() {
                let fallback = &fallbacks[0];
                tracing::warn!(
                    "🔄 [Provider Failover] Proactive switch to {} (model: {}) because primary is RED",
                    fallback.provider.to_string(),
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
        }
    }
}
