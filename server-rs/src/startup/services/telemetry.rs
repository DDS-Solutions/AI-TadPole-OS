//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / telemetry
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Heartbeat]`, `[MetricAggregator]`, `[TelemetryLogSink]`, `[HealthMonitor]`
//! - **Witness Tests**: none declared

use crate::startup::{StateQuery, StateResponse, SystemContext, SystemService};
use async_trait::async_trait;

pub struct HeartbeatService;

#[async_trait]
impl SystemService for HeartbeatService {
    fn name(&self) -> &'static str {
        "Heartbeat"
    }
    fn is_critical(&self) -> bool {
        true
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let context_clone = context.clone();
        let mut shutdown_rx = context.shutdown_rx.clone();
        let heartbeat_secs = context.config.heartbeat_secs;
        context
            .app_state
            .resources
            .set_subsystem_status("Heartbeat", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let boot_instant = std::time::Instant::now();
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(heartbeat_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut last_tpm = 0;
            let mut last_recruits = 0;
            let mut last_registry_json = None;
            let mut last_snapshot_map: Option<
                std::collections::BTreeMap<String, crate::types::SubsystemStatus>,
            > = None;
            loop {
                tokio::select! {
                    res = shutdown_rx.changed() => {
                        match res {
                            Ok(_) => {
                                if *shutdown_rx.borrow() {
                                    tracing::info!("🛑 [Heartbeat] Heartbeat Loop shutting down gracefully.");
                                    break;
                                }
                            }
                            Err(_) => {
                                tracing::debug!("🔌 [Heartbeat] Shutdown channel closed.");
                                break;
                            }
                        }
                    }
                    _ = interval.tick() => {
                        // Recovery wrapper: execute metrics collection safely
                        if let Err(e) = gather_and_emit_metrics(
                            &context_clone,
                            boot_instant,
                            &mut last_tpm,
                            &mut last_recruits,
                            &mut last_registry_json,
                            &mut last_snapshot_map,
                        ).await {
                            tracing::warn!("⚠️  [Heartbeat] Metrics collection failed: {:?}", e);
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

pub(crate) async fn gather_and_emit_metrics(
    context: &SystemContext,
    boot_instant: std::time::Instant,
    last_tpm: &mut usize,
    last_recruits: &mut u32,
    last_registry_json: &mut Option<serde_json::Value>,
    last_snapshot_map: &mut Option<
        std::collections::BTreeMap<String, crate::types::SubsystemStatus>,
    >,
) -> Result<(), anyhow::Error> {
    let app_state = &context.app_state;

    let active_agents = match context.query_state(StateQuery::GetActiveAgents).await {
        StateResponse::ActiveAgents(val) => val,
        other => {
            tracing::debug!("Unexpected response for GetActiveAgents: {:?}", other);
            0
        }
    };
    let swarm_depth = match context.query_state(StateQuery::GetMaxSwarmDepth).await {
        StateResponse::MaxSwarmDepth(val) => val,
        other => {
            tracing::debug!("Unexpected response for GetMaxSwarmDepth: {:?}", other);
            0
        }
    };
    let tpm_current = match context.query_state(StateQuery::GetTpmAccumulator).await {
        StateResponse::TpmAccumulator(val) => val,
        other => {
            tracing::debug!("Unexpected response for GetTpmAccumulator: {:?}", other);
            0
        }
    };
    let recruits_current = match context.query_state(StateQuery::GetRecruitCount).await {
        StateResponse::RecruitCount(val) => val,
        other => {
            tracing::debug!("Unexpected response for GetRecruitCount: {:?}", other);
            0
        }
    };

    // Calculate rates (deltas) non-destructively (SEC-07)
    let tpm = tpm_current.saturating_sub(*last_tpm);
    let recruits = recruits_current.saturating_sub(*last_recruits);

    *last_tpm = tpm_current;
    *last_recruits = recruits_current;

    let profile = app_state.resources.hardware_profiler.get_profile();
    let cpu = profile.cpu_usage;
    let memory_gb = profile.memory_used as f32 / (1024.0 * 1024.0 * 1024.0);
    let total_gb = profile.memory_total as f32 / (1024.0 * 1024.0 * 1024.0);

    // Accurate change-detection snapshot without stale cache leaks (REL-02)
    let current_snapshot: std::collections::BTreeMap<String, crate::types::SubsystemStatus> =
        app_state
            .resources
            .initialization_registry
            .iter()
            .map(|kv| (kv.key().clone(), kv.value().clone()))
            .collect();

    let registry_snapshot = if let (Some(last_json), Some(last_map)) =
        (last_registry_json.as_ref(), last_snapshot_map.as_ref())
    {
        if *last_map == current_snapshot {
            last_json.clone()
        } else {
            let json_val =
                serde_json::to_value(&current_snapshot).unwrap_or(serde_json::Value::Null);
            *last_snapshot_map = Some(current_snapshot);
            *last_registry_json = Some(json_val.clone());
            json_val
        }
    } else {
        let json_val = serde_json::to_value(&current_snapshot).unwrap_or(serde_json::Value::Null);
        *last_snapshot_map = Some(current_snapshot);
        *last_registry_json = Some(json_val.clone());
        json_val
    };

    app_state.emit_event(serde_json::json!({
        "type": "engine:health",
        "uptime": boot_instant.elapsed().as_secs(),
        "agentCount": active_agents,
        "activeAgents": active_agents,
        "maxDepth": swarm_depth,
        "tpm": tpm,
        "recruitCount": recruits,
        "cpu": cpu,
        "memory": memory_gb,
        "maxMemory": total_gb,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "initialization": registry_snapshot
    }));

    Ok(())
}

pub struct MetricAggregatorService;

#[async_trait]
impl SystemService for MetricAggregatorService {
    fn name(&self) -> &'static str {
        "MetricAggregator"
    }
    fn is_critical(&self) -> bool {
        true
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state
            .resources
            .set_subsystem_status("MetricAggregator", crate::types::SubsystemStatus::Ready);
        let aggregator_rx = crate::telemetry::TELEMETRY_TX.subscribe();
        let aggregator = crate::telemetry::aggregator::MetricAggregator::new(1000);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            tokio::select! {
                res = shutdown_rx.changed() => {
                    match res {
                        Ok(_) => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [MetricAggregator] Metric Aggregator shutting down gracefully.");
                            }
                        }
                        Err(_) => {
                            tracing::debug!("🔌 [MetricAggregator] Shutdown channel closed.");
                        }
                    }
                }
                _ = aggregator.run(aggregator_rx) => {
                    tracing::error!("🚨 [MetricAggregator] Metric Aggregator exited unexpectedly.");
                    app_state_clone.resources.set_subsystem_status(
                        "MetricAggregator",
                        crate::types::SubsystemStatus::Failed("Aggregator exited unexpectedly".to_string()),
                    );
                }
            }
        });
        Ok(())
    }
}

/// Background service that persists all telemetry events to rolling JSONL files
/// under `data/logs/telemetry-YYYY-MM-DD.jsonl` for post-hoc RCA by AI agents.
pub struct TelemetryLogSinkService;

#[async_trait]
impl SystemService for TelemetryLogSinkService {
    fn name(&self) -> &'static str {
        "TelemetryLogSink"
    }
    fn is_critical(&self) -> bool {
        false
    }
    fn registry_key(&self) -> &'static str {
        "TelemetryLogSink"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state.clone();
        let mut shutdown_rx = context.shutdown_rx;
        let base_dir = app_state.base_dir.clone();
        app_state
            .resources
            .set_subsystem_status("TelemetryLogSink", crate::types::SubsystemStatus::Ready);
        let sink_rx = crate::telemetry::TELEMETRY_TX.subscribe();
        let sink = crate::telemetry::sink::TelemetryLogSink::new(base_dir);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            tokio::select! {
                res = shutdown_rx.changed() => {
                    match res {
                        Ok(_) => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [TelemetryLogSink] Log sink shutting down gracefully.");
                            }
                        }
                        Err(_) => {
                            tracing::debug!("🔌 [TelemetryLogSink] Shutdown channel closed.");
                        }
                    }
                }
                _ = sink.run(sink_rx) => {
                    tracing::error!("🚨 [TelemetryLogSink] Log sink exited unexpectedly.");
                    app_state_clone.resources.set_subsystem_status(
                        "TelemetryLogSink",
                        crate::types::SubsystemStatus::Failed("Log sink exited unexpectedly".to_string()),
                    );
                }
            }
        });
        Ok(())
    }
}

pub struct SystemHealthMonitorService;

#[async_trait]
impl SystemService for SystemHealthMonitorService {
    fn name(&self) -> &'static str {
        "SystemHealthMonitor"
    }
    fn is_critical(&self) -> bool {
        true
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state
            .resources
            .set_subsystem_status("SystemHealthMonitor", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let boot_instant = std::time::Instant::now();
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut was_demoted = false;
            loop {
                tokio::select! {
                    res = shutdown_rx.changed() => {
                        match res {
                            Ok(_) => {
                                if *shutdown_rx.borrow() {
                                    tracing::info!("🛑 [HealthMonitor] System Health Monitor shutting down gracefully.");
                                    break;
                                }
                            }
                            Err(_) => {
                                tracing::debug!("🔌 [HealthMonitor] Shutdown channel closed.");
                                break;
                            }
                        }
                    }
                    _ = interval.tick() => {
                        let elapsed_secs = boot_instant.elapsed().as_secs();
                        let is_timeout = elapsed_secs > 30;
                        let critical_subsystems = vec![
                            "Database", "Agents", "MCP", "Heartbeat",
                            "SecurityEviction", "PrivacyGuard", "BudgetFlush", "SqliteMaintenance"
                        ];

                        let mut failed_detected = false;
                        for sub in critical_subsystems {
                            let status = app_state.resources.initialization_registry.get(sub).map(|r| r.value().clone());
                            match status {
                                Some(crate::types::SubsystemStatus::Failed(e)) => {
                                    tracing::error!("🚨 [HealthMonitor] Critical subsystem '{}' is in failed state: {}", sub, e);
                                    failed_detected = true;
                                }
                                Some(crate::types::SubsystemStatus::Warming(_)) | Some(crate::types::SubsystemStatus::NotStarted) => {
                                    if is_timeout {
                                        let err_msg = "Startup timeout (exceeded 30s in warming state)".to_string();
                                        app_state.resources.set_subsystem_status(sub, crate::types::SubsystemStatus::Failed(err_msg));
                                        tracing::error!("🚨 [HealthMonitor] Critical subsystem '{}' timed out warming up.", sub);
                                        failed_detected = true;
                                    }
                                }
                                _ => {}
                            }
                        }

                        if failed_detected {
                            let current_max = app_state.governance.max_agents.load(std::sync::atomic::Ordering::Relaxed);
                            if current_max > 2 {
                                tracing::warn!("🚨 [HealthMonitor] Critical failure active. Scaling down max_agents from {} to 2.", current_max);
                                app_state.governance.max_agents.store(2, std::sync::atomic::Ordering::Relaxed);
                                was_demoted = true;
                            }
                        } else if was_demoted {
                            // Subsystems have recovered to healthy
                            tracing::info!("✅ [HealthMonitor] All critical subsystems healthy. Restoring max_agents capacity.");
                            app_state.governance.max_agents.store(8, std::sync::atomic::Ordering::Relaxed);
                            was_demoted = false;
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

pub struct RecoverActiveAgentsService;

#[async_trait]
impl SystemService for RecoverActiveAgentsService {
    fn name(&self) -> &'static str {
        "RecoverActiveAgents"
    }
    fn is_critical(&self) -> bool {
        true
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        app_state
            .resources
            .set_subsystem_status("RecoverActiveAgents", crate::types::SubsystemStatus::Ready);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            crate::routes::agent::recover_active_agents(app_state_clone).await;
        });
        Ok(())
    }
}
