//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Persistence
//! - **Primary Entrypoints**: `AppState::save_agents`, `AppState::flush_all`, `AppState::save_governance_settings`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[State]`, `[System]`, `[Governance]`
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

use super::AppState;
use crate::error::AppError;

impl AppState {
    /// Persists all current agent states to the database in a single transaction.
    /// Batched to avoid N individual round-trips (was the #1 shutdown bottleneck).
    ///
    /// Snapshots in-memory agent states first to drop DashMap shard locks immediately,
    /// preventing blocking and thread stalls during asynchronous database I/O.
    pub async fn save_agents(&self) -> Result<(), AppError> {
        let _write = self.comms.agent_write_gate.lock().await;
        let agents_to_save: Vec<crate::agent::types::EngineAgent> = self
            .registry
            .agents
            .iter()
            .map(|kv| kv.value().clone())
            .collect();

        // Batch all saves into a single transaction (1 fsync vs N fsyncs)
        let mut tx = self.resources.pool.begin().await?;
        let mut pending_versions: Vec<(String, u32)> = Vec::new();
        for mut agent in agents_to_save {
            match crate::agent::persistence::save_agent_db_in_tx(&mut tx, &mut agent).await {
                Ok(next_ver) => {
                    pending_versions.push((agent.identity.id.clone(), next_ver));
                }
                Err(err) => {
                    tracing::error!(
                        agent_id = %agent.identity.id,
                        error = %err,
                        "❌ [State] Failed to persist agent during batched save_agents"
                    );
                    let _ = tx.rollback().await;
                    return Err(err);
                }
            }
        }

        tx.commit().await?;

        for (agent_id, next_ver) in pending_versions {
            if let Some(mut agent_entry) = self.registry.agents.get_mut(&agent_id) {
                agent_entry.version = agent_entry.version.max(next_ver);
            }
        }

        Ok(())
    }

    /// Persists all provider configurations to disk.
    pub async fn save_providers(&self) -> Result<(), AppError> {
        let providers_vec: Vec<crate::agent::types::ProviderConfig> = self
            .registry
            .providers
            .iter()
            .map(|kv| kv.value().clone())
            .collect();
        crate::agent::persistence::save_providers(&self.base_dir, providers_vec).await
    }

    /// Persists all model metadata to disk.
    pub async fn save_models(&self) -> Result<(), AppError> {
        let models_vec: Vec<crate::agent::types::ModelEntry> = self
            .registry
            .models
            .iter()
            .map(|kv| kv.value().clone())
            .collect();
        crate::agent::persistence::save_models(&self.base_dir, models_vec).await
    }

    /// Persists governance settings to data/governance_settings.json atomically.
    pub fn save_governance_settings(&self) {
        if let Err(error) = self.try_save_governance_settings() {
            tracing::error!("[Governance] Failed to persist settings: {}", error);
        }
    }

    pub fn try_save_governance_settings(&self) -> Result<(), AppError> {
        let _write = self.comms.settings_write_gate.lock();
        let settings_path = self.base_dir.join("data").join("governance_settings.json");
        let payload = crate::routes::oversight::OversightSettingsPayload {
            dispatch_paused: Some(
                self.comms
                    .engine_stopped
                    .load(std::sync::atomic::Ordering::Acquire),
            ),
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
            std::fs::create_dir_all(parent)
                .map_err(|error| AppError::InternalServerError(error.to_string()))?;
        }

        let json_str = serde_json::to_string_pretty(&payload)
            .map_err(|error| AppError::InternalServerError(error.to_string()))?;
        {
            let tmp_path = settings_path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
            let write_result = std::fs::write(&tmp_path, json_str.as_bytes())
                .and_then(|_| std::fs::rename(&tmp_path, &settings_path));

            match write_result {
                Ok(_) => {
                    tracing::info!(
                        "🛡️ [Governance] Settings persisted successfully to {}",
                        settings_path.display()
                    );
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&tmp_path);
                    tracing::error!(
                        "🚨 [Governance] Failed to persist settings to {}: {:?}",
                        settings_path.display(),
                        e
                    );
                    return Err(AppError::InternalServerError(e.to_string()));
                }
            }
        }
        Ok(())
    }

    /// Flushes all volatile buffers to persistent storage.
    ///
    /// ### 💾 Persistence Guarantee
    /// Aggregates agent registry states, model updates, and budget meter logs
    /// into a batched transaction. This is the primary safety valve for
    /// graceful engine shutdowns.
    pub async fn flush_all(&self) -> Result<(), AppError> {
        tracing::info!(
            "💾 [System] Flushing all volatile buffers and registries to persistence..."
        );

        // 1. Persist Registries
        self.save_agents().await?;
        if let Err(e) = self.save_providers().await {
            tracing::error!(
                "❌ [State] Failed to persist providers during save_all: {:?}",
                e
            );
        }
        if let Err(e) = self.save_models().await {
            tracing::error!(
                "❌ [State] Failed to persist models during save_all: {:?}",
                e
            );
        }

        // 2. Flush Telemetry & Metering
        if let Err(e) = self.security.budget_guard.flush_to_db().await {
            tracing::error!("🚨 [System] Failed to flush budget data: {}", e);
        }

        // 3. Passive WAL checkpoint on shutdown to fold transaction journals
        if let Err(e) = crate::db::checkpoint_wal(&self.resources.pool).await {
            tracing::warn!(
                "⚠️ [Database] Passive WAL checkpoint during flush_all encountered warning: {}",
                e
            );
        }

        Ok(())
    }
}
