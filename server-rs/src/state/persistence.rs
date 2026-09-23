//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Persistence
//! - **Primary Entrypoints**: `AppState::save_agents`, `AppState::flush_all`, `AppState::save_governance_settings`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::AppState;

impl AppState {
    /// Persists all current agent states to the database in a single transaction.
    /// Batched to avoid N individual round-trips (was the #1 shutdown bottleneck).
    pub async fn save_agents(&self) {
        // Batch all saves into a single transaction (1 fsync vs N fsyncs)
        match self.resources.pool.begin().await {
            Ok(mut tx) => {
                for mut entry in self.registry.agents.iter_mut() {
                    let agent = entry.value_mut();
                    if let Err(err) =
                        crate::agent::persistence::save_agent_db_in_tx(&mut tx, agent).await
                    {
                        tracing::error!(
                            agent_id = %agent.identity.id,
                            error = %err,
                            "❌ [State] Failed to persist agent during batched save_agents"
                        );
                    }
                }
                if let Err(err) = tx.commit().await {
                    tracing::error!(
                        error = %err,
                        "❌ [State] Failed to commit agent batch transaction"
                    );
                }
            }
            Err(err) => {
                tracing::error!(
                    error = %err,
                    "❌ [State] Failed to begin agent batch transaction — falling back to individual saves"
                );
                // Fallback: individual saves (degraded but functional)
                for mut entry in self.registry.agents.iter_mut() {
                    let agent = entry.value_mut();
                    if let Err(err) =
                        crate::agent::persistence::save_agent_db(&self.resources.pool, agent).await
                    {
                        tracing::error!(
                            agent_id = %agent.identity.id,
                            error = %err,
                            "❌ [State] Failed to persist agent during fallback save_agents"
                        );
                    }
                }
            }
        }
    }

    /// Persists all provider configurations to disk.
    pub async fn save_providers(&self) {
        let providers_vec: Vec<crate::agent::types::ProviderConfig> = self
            .registry
            .providers
            .iter()
            .map(|kv| kv.value().clone())
            .collect();
        if let Err(e) =
            crate::agent::persistence::save_providers(&self.base_dir, providers_vec).await
        {
            tracing::error!("❌ [State] Failed to persist providers to disk: {:?}", e);
        }
    }

    /// Persists all model metadata to disk.
    pub async fn save_models(&self) {
        let models_vec: Vec<crate::agent::types::ModelEntry> = self
            .registry
            .models
            .iter()
            .map(|kv| kv.value().clone())
            .collect();
        if let Err(e) = crate::agent::persistence::save_models(&self.base_dir, models_vec).await {
            tracing::error!("❌ [State] Failed to persist models to disk: {:?}", e);
        }
    }

    /// Persists governance settings to data/governance_settings.json.
    pub fn save_governance_settings(&self) {
        let settings_path = self.base_dir.join("data").join("governance_settings.json");
        let payload = crate::routes::oversight::OversightSettingsPayload {
            auto_approve_safe_skills: Some(
                self.governance
                    .auto_approve_safe_skills
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            privacy_mode: Some(
                self.governance
                    .privacy_mode
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            cluster_privacy_policies: Some(
                self.governance
                    .cluster_privacy_policies
                    .iter()
                    .map(|kv| (kv.key().clone(), *kv.value()))
                    .collect(),
            ),
            max_agents: Some(
                self.governance
                    .max_agents
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            max_clusters: Some(
                self.governance
                    .max_clusters
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            max_swarm_depth: Some(
                self.governance
                    .max_swarm_depth
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            max_task_length: Some(
                self.governance
                    .max_task_length
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            default_budget_usd: Some(*self.governance.default_budget_usd.read()),
            default_model: Some(self.governance.default_model.read().clone()),
            default_provider: Some(self.governance.default_provider.read().clone()),
        };

        if let Some(parent) = settings_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            if let Err(e) = std::fs::write(&settings_path, json_str) {
                tracing::error!(
                    "🚨 [Governance] Failed to persist settings to {}: {:?}",
                    settings_path.display(),
                    e
                );
            } else {
                tracing::info!(
                    "🛡️ [Governance] Settings persisted successfully to {}",
                    settings_path.display()
                );
            }
        }
    }

    /// Flushes all volatile buffers to persistent storage.
    ///
    /// ### 💾 Persistence Guarantee
    /// Aggregates agent registry states, model updates, and budget meter logs
    /// into a batched transaction. This is the primary safety valve for
    /// graceful engine shutdowns.
    pub async fn flush_all(&self) {
        tracing::info!(
            "💾 [System] Flushing all volatile buffers and registries to persistence..."
        );

        // 1. Persist Registries
        self.save_agents().await;
        self.save_providers().await;
        self.save_models().await;

        // 2. Flush Telemetry & Metering
        if let Err(e) = self.security.budget_guard.flush_to_db().await {
            tracing::error!("🚨 [System] Failed to flush budget data: {}", e);
        }
    }
}
