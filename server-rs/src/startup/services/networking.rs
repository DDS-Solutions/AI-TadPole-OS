//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / networking
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Discovery]`
//! - **Witness Tests**: none declared

use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct SwarmDiscoveryService;

#[async_trait]
impl SystemService for SwarmDiscoveryService {
    fn name(&self) -> &'static str {
        "SwarmDiscovery"
    }
    fn is_critical(&self) -> bool {
        true
    }
    fn registry_key(&self) -> &'static str {
        "Network"
    }
    fn start_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(10)
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        app_state
            .resources
            .set_subsystem_status("Network", crate::types::SubsystemStatus::Warming(0.0));

        let manager =
            match crate::services::discovery::SwarmDiscoveryManager::new(app_state.clone()) {
                Ok(mgr) => mgr,
                Err(e) => {
                    tracing::error!("📡 [Discovery] Failed to initialize mDNS manager: {}", e);
                    app_state.resources.set_subsystem_status(
                        "Network",
                        crate::types::SubsystemStatus::Failed(e.to_string()),
                    );
                    return Err(anyhow::anyhow!(e));
                }
            };

        // Note: manager.start registers mDNS services and spawns a background peer browser loop
        // which retains a cloned Arc<ServiceDaemon>, ensuring discovery remains active after manager is dropped.
        let shutdown_rx = context.shutdown_rx;
        match manager.start(shutdown_rx) {
            Ok(()) => {
                app_state
                    .resources
                    .set_subsystem_status("Network", crate::types::SubsystemStatus::Ready);
                Ok(())
            }
            Err(e) => {
                tracing::error!("📡 [Discovery] Failed to start mDNS manager: {}", e);
                app_state.resources.set_subsystem_status(
                    "Network",
                    crate::types::SubsystemStatus::Failed(e.to_string()),
                );
                Err(anyhow::anyhow!(e))
            }
        }
    }
}

pub struct SwarmPulseService;

#[async_trait]
impl SystemService for SwarmPulseService {
    fn name(&self) -> &'static str {
        "SwarmPulse"
    }
    fn is_critical(&self) -> bool {
        true
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let shutdown_rx = context.shutdown_rx;
        app_state
            .resources
            .set_subsystem_status("SwarmPulse", crate::types::SubsystemStatus::Ready);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            crate::telemetry::pulse::spawn_pulse_loop(app_state_clone.clone(), shutdown_rx).await;
            tracing::warn!("⚠️ [SwarmPulse] Pulse telemetry loop terminated.");
        });
        Ok(())
    }
}
