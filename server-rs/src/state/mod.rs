//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / mod
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[Engine]`, `[Oversight]`

use dashmap::DashMap;
use std::sync::Arc;

pub mod hubs;
pub mod init;
#[cfg(test)]
pub mod mock;
pub mod persistence;

#[cfg(test)]
pub use init::create_payment_router;

use hubs::comm::CommunicationHub;
use hubs::gov::GovernanceHub;
use hubs::reg::RegistryHub;
use hubs::res::ResourceHub;
use hubs::sec::SecurityHub;

use crate::error::AppError;

/// The global application state shared across all routes via Axum State.
/// Decomposed into logical hubs for modularity.
pub struct AppState {
    /// Manages real-time communication channels (logs, events, telemetry, audio).
    pub comms: Arc<CommunicationHub>,
    /// Manages operational limits and policy settings.
    pub governance: Arc<GovernanceHub>,
    /// Manages entities like agents, providers, models, and skills.
    pub registry: Arc<RegistryHub>,
    /// Manages security features like auditing, budget enforcement, and scanning.
    pub security: Arc<SecurityHub>,
    /// Manages shared system resources (DB pool, HTTP client, file contexts).
    pub resources: Arc<ResourceHub>,
    /// Whether mirror mode is active (read from environment/config).
    pub mirror_mode: bool,
    /// Active drift alerts observed in mirror mode.
    pub drift_alerts: Arc<DashMap<String, serde_json::Value>>,
    /// Global workspace root directory for data persistence.
    pub base_dir: std::path::PathBuf,
    /// Unique session ID generated at boot, used to correlate engine:boot / engine:shutdown events.
    pub session_id: String,
}

impl AppState {
    /// ### 🏁 Boot Sequence: Engine Initialization (new)
    /// Orchestrates secret verification, SQLite hydration, hub assembly, and passive hot-reload.
    pub async fn new() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();
        tracing::info!("🏁 [Engine] Starting AppState initialization...");

        let base_dir = init::resolve_workspace_root();
        let channels = init::init_channels();
        let security_tokens = init::load_security_tokens()?;
        let pool = init::init_database_pool(&base_dir).await?;
        let http_client = init::init_http_client()?;
        let audio_cache = init::init_audio_cache(&base_dir).await;

        let secret_redactor = Arc::new(crate::secret_redactor::SecretRedactor::from_env());
        let permission_policy = Arc::new(crate::security::permissions::PermissionPolicy::new(
            pool.clone(),
        ));

        let max_concurrent_runners = init::parse_max_concurrent_runners();
        let comms = init::build_comms_hub(&channels, max_concurrent_runners);
        let governance = init::build_governance_hub(max_concurrent_runners);
        init::hydrate_governance_settings(&base_dir, &comms, &governance);

        let registry = init::build_registry_hub(
            &base_dir,
            &pool,
            &comms,
            &channels.event_tx,
            permission_policy.clone(),
        )
        .await?;
        init::validate_tool_configurations(&registry)?;

        let security =
            init::build_security_hub(&pool, secret_redactor, permission_policy, security_tokens)?;
        init::verify_boot_audit_ledger(security.audit_trail.clone());

        let resources = init::build_resource_hub(&base_dir, &pool, http_client, audio_cache);
        init::set_initial_subsystem_statuses(&resources);
        init::spawn_skill_hot_reloader(registry.clone(), base_dir.clone());

        let mirror_mode = std::env::var("MIRROR_MODE")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Self {
            comms,
            governance,
            registry,
            security,
            resources,
            mirror_mode,
            drift_alerts: Arc::new(DashMap::new()),
            base_dir,
            session_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// ### 📡 Observability: System Broadcast (broadcast_sys)
    /// Publishes a redacted high-priority system log entry to connected telemetry consumers.
    pub fn broadcast_sys(&self, text: &str, severity: &str, mission_id: Option<String>) {
        let safe_text = self.security.secret_redactor.redact(text);
        let entry = crate::types::LogEntry::new("System", &safe_text, severity, mission_id);
        let _ = self.comms.tx.send(entry);
    }

    /// Helper to broadcast an agent-sourced log with identity metadata.
    pub fn broadcast_agent(
        &self,
        text: &str,
        severity: &str,
        mission_id: Option<String>,
        agent_id: &str,
        agent_name: &str,
    ) {
        let safe_text = self.security.secret_redactor.redact(text);
        let mut entry = crate::types::LogEntry::new("Agent", &safe_text, severity, mission_id);
        entry.agent_id = Some(agent_id.to_string());
        entry.agent_name = Some(agent_name.to_string());
        let _ = self.comms.tx.send(entry);
    }

    /// Helper to broadcast an arbitrary Engine event.
    pub fn emit_event(&self, event: serde_json::Value) {
        let mut full_event = event;
        let seq = self
            .comms
            .event_sequence
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if let Some(obj) = full_event.as_object_mut() {
            obj.insert("_seq".to_string(), serde_json::json!(seq));
        }

        let _ = self.comms.event_tx.send(full_event);
    }

    /// Returns the overall health state of the Swarm Engine based on critical subsystems.
    pub fn health_state(&self) -> crate::types::SystemHealthState {
        let mut critical_subsystems = vec![
            "Database",
            "Agents",
            "MCP",
            "Heartbeat",
            "SecurityEviction",
            "PrivacyGuard",
            "BudgetFlush",
        ];

        if self
            .resources
            .initialization_registry
            .contains_key("Network")
        {
            critical_subsystems.push("Network");
        }
        if self
            .resources
            .initialization_registry
            .contains_key("CodeGraph")
        {
            critical_subsystems.push("CodeGraph");
        }

        let mut warming = false;
        for sub in critical_subsystems {
            let Some(status) = self.resources.initialization_registry.get(sub) else {
                warming = true;
                continue;
            };
            use crate::types::{SubsystemStatus, SystemHealthState};
            match status.value() {
                SubsystemStatus::Failed(_) => return SystemHealthState::Degraded,
                SubsystemStatus::Warming(_) | SubsystemStatus::NotStarted => warming = true,
                SubsystemStatus::Ready => {}
            }
        }

        if warming {
            crate::types::SystemHealthState::Warming
        } else {
            crate::types::SystemHealthState::Ready
        }
    }

    /// ### ⏳ Governance: Oversight Synchronization (yield_phase_transition)
    /// Yields mission execution back to the Tokio scheduler and emits a phase transition event.
    pub async fn yield_phase_transition(&self, agent_id: &str, phase: &str) {
        tracing::debug!(
            "⏳ [Oversight] Agent {} yielding at boundary: {}",
            agent_id,
            phase
        );

        self.emit_event(serde_json::json!({
            "type": "agent:phase_transition",
            "agent_id": agent_id,
            "phase": phase
        }));

        tokio::task::yield_now().await;
    }
}
