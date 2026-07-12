//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[networking]` in tracing logs.

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
        match crate::services::discovery::SwarmDiscoveryManager::new(app_state.clone()) {
            Ok(manager) => {
                let shutdown_rx = context.shutdown_rx.clone();
                let start_fut = async move { manager.start(shutdown_rx) };
                match tokio::time::timeout(std::time::Duration::from_secs(10), start_fut).await {
                    Ok(Ok(_)) => {
                        app_state
                            .resources
                            .set_subsystem_status("Network", crate::types::SubsystemStatus::Ready);
                        Ok(())
                    }
                    Ok(Err(e)) => {
                        tracing::error!("📡 [Discovery] Failed to start mDNS manager: {}", e);
                        app_state.resources.set_subsystem_status(
                            "Network",
                            crate::types::SubsystemStatus::Failed(e.to_string()),
                        );
                        Err(anyhow::anyhow!(e))
                    }
                    Err(_) => {
                        let err_msg = "SwarmDiscoveryService timed out after 10s".to_string();
                        tracing::error!("📡 [Discovery] {}", err_msg);
                        app_state.resources.set_subsystem_status(
                            "Network",
                            crate::types::SubsystemStatus::Failed(err_msg.clone()),
                        );
                        Err(anyhow::anyhow!(err_msg))
                    }
                }
            }
            Err(e) => {
                tracing::error!("📡 [Discovery] Failed to initialize mDNS manager: {}", e);
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
        tokio::spawn(async move {
            crate::telemetry::pulse::spawn_pulse_loop(app_state, shutdown_rx).await;
        });
        Ok(())
    }
}

// Metadata: [networking]
