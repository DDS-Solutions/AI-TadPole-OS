//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Startup / Supervisor & Lifecycle Orchestration
//! - **Primary Entrypoints**: `ServiceConfiguration`, `SystemContext`, `SystemService`, `spawn_background_tasks`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::cli::BootstrapIntent;
use super::services;
use crate::state::AppState;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServiceConfiguration {
    pub heartbeat_secs: u64,
    pub rate_limit_eviction_interval_secs: u64,
    pub max_bucket_age_secs: u64,
    pub max_auth_age_secs: u64,
    pub memory_cleanup_interval_secs: u64,
    pub iks_decay_interval_secs: u64,
    pub iks_eviction_interval_secs: u64,
    pub budget_flush_interval_secs: u64,
}

impl Default for ServiceConfiguration {
    fn default() -> Self {
        Self {
            heartbeat_secs: 3,
            rate_limit_eviction_interval_secs: 300,
            max_bucket_age_secs: 120,
            max_auth_age_secs: 600,
            memory_cleanup_interval_secs: 6 * 3600,
            iks_decay_interval_secs: 6 * 3600,
            iks_eviction_interval_secs: 24 * 3600,
            budget_flush_interval_secs: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateQuery {
    GetMaxSwarmDepth,
    GetActiveAgents,
    GetTpmAccumulator,
    GetRecruitCount,
}

#[derive(Debug, Clone)]
pub enum StateResponse {
    MaxSwarmDepth(u32),
    ActiveAgents(u32),
    TpmAccumulator(usize),
    RecruitCount(u32),
}

#[derive(Clone)]
pub struct SystemContext {
    pub app_state: Arc<AppState>,
    pub shutdown_rx: tokio::sync::watch::Receiver<bool>,
    pub config: ServiceConfiguration,
}

impl SystemContext {
    pub async fn query_state(&self, query: StateQuery) -> StateResponse {
        match query {
            StateQuery::GetMaxSwarmDepth => {
                let val = self
                    .app_state
                    .governance
                    .max_swarm_depth
                    .load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::MaxSwarmDepth(val)
            }
            StateQuery::GetActiveAgents => {
                let val = self
                    .app_state
                    .governance
                    .active_agents
                    .load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::ActiveAgents(val)
            }
            StateQuery::GetTpmAccumulator => {
                let val = self
                    .app_state
                    .governance
                    .tpm_accumulator
                    .load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::TpmAccumulator(val)
            }
            StateQuery::GetRecruitCount => {
                let val = self
                    .app_state
                    .governance
                    .recruit_count
                    .load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::RecruitCount(val)
            }
        }
    }
}

#[async_trait]
pub trait SystemService: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_critical(&self) -> bool {
        false
    }
    fn registry_key(&self) -> &'static str {
        self.name()
    }
    fn start_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error>;
}

pub async fn spawn_background_tasks(
    app_state: Arc<AppState>,
    intent: BootstrapIntent,
    service_config: ServiceConfiguration,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let context = SystemContext {
        app_state,
        shutdown_rx: shutdown_rx.clone(),
        config: service_config,
    };

    // 1. Phased Boot Sequence: Run warmup tasks sequentially (ARCH-02)
    let mut warmup_tasks: Vec<Box<dyn SystemService>> = Vec::new();
    if intent == BootstrapIntent::Full {
        warmup_tasks.push(Box::new(services::CodeGraphWarmupService));
        warmup_tasks.push(Box::new(services::CodeGraphDbRefreshService));
        warmup_tasks.push(Box::new(services::RecoverActiveAgentsService));
    }

    for service in warmup_tasks {
        let name = service.name();
        let reg_key = service.registry_key();
        let is_crit = service.is_critical();
        let timeout_duration = service.start_timeout();

        if is_crit {
            tracing::debug!("Running warmup service: {}", name);
        }
        let start_fut = service.start(context.clone());
        match tokio::time::timeout(timeout_duration, start_fut).await {
            Ok(Ok(())) => {
                if is_crit {
                    tracing::debug!("Warmup service '{}' completed successfully", name);
                }
            }
            Ok(Err(e)) => {
                tracing::error!("🚨 [Service] Warmup service '{}' failed: {:?}", name, e);
                context.app_state.resources.set_subsystem_status(
                    reg_key,
                    crate::types::SubsystemStatus::Failed(e.to_string()),
                );
            }
            Err(_) => {
                let err_msg = format!("Warmup timeout (exceeded {}s)", timeout_duration.as_secs());
                tracing::error!(
                    "🚨 [Service] Warmup service '{}' timed out: {}",
                    name,
                    err_msg
                );
                context
                    .app_state
                    .resources
                    .set_subsystem_status(reg_key, crate::types::SubsystemStatus::Failed(err_msg));
            }
        }
    }

    // ── Gap 7: Emit engine:boot event after warmup phase completes ─────────────
    {
        use crate::telemetry::TELEMETRY_TX;
        use std::sync::atomic::Ordering;
        let boot_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let _ = TELEMETRY_TX.send(::serde_json::json!({
            "type": "engine:boot",
            "session_id": context.app_state.session_id.clone(),
            "timestamp": boot_ts,
            "version": env!("CARGO_PKG_VERSION"),
            "config": {
                "budget_flush_interval_secs": context.config.budget_flush_interval_secs,
                "null_providers_test_mode": context.app_state.governance.null_providers_test_mode.load(Ordering::Relaxed),
                "privacy_mode": context.app_state.governance.privacy_mode.load(Ordering::Relaxed),
                "max_agents": context.app_state.governance.max_agents.load(Ordering::Relaxed),
            }
        }));
    }

    // Create phased shutdown channels (ARCH-03)
    let (shutdown_tx_p1, shutdown_rx_p1) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p2, shutdown_rx_p2) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p3, shutdown_rx_p3) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p4, shutdown_rx_p4) = tokio::sync::watch::channel(false);

    // Coordinate phased shutdown sequence in response to global shutdown signal
    let mut global_shutdown_rx = shutdown_rx.clone();
    let shutdown_session_id = context.app_state.session_id.clone();
    tokio::spawn(async move {
        loop {
            if global_shutdown_rx.changed().await.is_err() {
                break;
            }
            if *global_shutdown_rx.borrow() {
                break;
            }
        }

        tracing::info!("🔔 [ShutdownOrchestrator] Initiating Phased Graceful Shutdown...");

        // ── Gap 7: Emit engine:shutdown before tearing down services ────────────
        {
            use crate::telemetry::TELEMETRY_TX;
            let shutdown_ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let _ = TELEMETRY_TX.send(::serde_json::json!({
                "type": "engine:shutdown",
                "session_id": shutdown_session_id,
                "timestamp": shutdown_ts,
                "reason": "graceful"
            }));
        }

        // Phase 1: Ingestion & Networking
        tracing::info!(
            "🛑 [ShutdownOrchestrator] Phase 1: Shutting down Ingestion & Networking..."
        );
        let _ = shutdown_tx_p1.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Phase 2: Telemetry & Monitoring
        tracing::info!(
            "🛑 [ShutdownOrchestrator] Phase 2: Shutting down Telemetry & Monitoring..."
        );
        let _ = shutdown_tx_p2.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Phase 3: Security & Cleanup
        tracing::info!("🛑 [ShutdownOrchestrator] Phase 3: Shutting down Security & Cleanup...");
        let _ = shutdown_tx_p3.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Phase 4: Persistence
        tracing::info!("🛑 [ShutdownOrchestrator] Phase 4: Shutting down Persistence...");
        let _ = shutdown_tx_p4.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        tracing::info!("✨ [ShutdownOrchestrator] Phased Graceful Shutdown complete.");
    });

    // 2. Spawn loop and network tasks concurrently
    let mut loop_services: Vec<Box<dyn SystemService>> = vec![
        Box::new(services::HeartbeatService),
        Box::new(services::ContinuitySchedulerService),
        Box::new(services::SwarmReaperService),
        Box::new(services::SpanWatchdogService),
        Box::new(services::MemoryCleanupService),
        Box::new(services::IngestionWorkerService),
        Box::new(services::PrivacyGuardService),
        Box::new(services::SecurityEvictionService),
        Box::new(services::MetricAggregatorService),
        Box::new(services::TelemetryLogSinkService),
        Box::new(services::BudgetFlushService),
        Box::new(services::SqliteMaintenanceService),
        Box::new(services::RecipeIngestionService),
        Box::new(services::SystemHealthMonitorService),
        Box::new(crate::services::cognitive_memory::CognitiveMemoryPipelineService),
    ];

    #[cfg(feature = "vector-memory")]
    {
        loop_services.push(Box::new(services::IksDecayService));
        loop_services.push(Box::new(services::IksEvictionService));
    }

    if intent == BootstrapIntent::Full {
        loop_services.push(Box::new(services::SwarmDiscoveryService));
        loop_services.push(Box::new(services::SwarmPulseService));
    }

    for service in loop_services {
        let name = service.name();
        let reg_key = service.registry_key();
        let is_crit = service.is_critical();
        let timeout_duration = service.start_timeout();

        // Select shutdown receiver based on phase category
        let phase_rx = match name {
            "SwarmDiscovery"
            | "SwarmPulse"
            | "RecipeIngestion"
            | "IngestionWorker"
            | "ContinuityScheduler" => shutdown_rx_p1.clone(),
            "Heartbeat" | "MetricAggregator" | "TelemetryLogSink" | "SystemHealthMonitor" => {
                shutdown_rx_p2.clone()
            }
            "PrivacyGuard" | "SecurityEviction" | "MemoryCleanup" | "IksDecay" | "IksEviction"
            | "SwarmReaper" => shutdown_rx_p3.clone(),
            "BudgetFlush" | "SqliteMaintenance" => shutdown_rx_p4.clone(),
            _ => shutdown_rx.clone(),
        };

        let mut context_clone = context.clone();
        context_clone.shutdown_rx = phase_rx;

        tokio::spawn(async move {
            if is_crit {
                tracing::debug!("Starting background service: {}", name);
            }
            let start_fut = service.start(context_clone.clone());
            match tokio::time::timeout(timeout_duration, start_fut).await {
                Ok(Ok(())) => {
                    if is_crit {
                        tracing::debug!("Background service '{}' started successfully", name);
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!(
                        "🚨 [Service] Background service '{}' failed to start: {:?}",
                        name,
                        e
                    );
                    context_clone.app_state.resources.set_subsystem_status(
                        reg_key,
                        crate::types::SubsystemStatus::Failed(e.to_string()),
                    );
                }
                Err(_) => {
                    let err_msg =
                        format!("Startup timeout (exceeded {}s)", timeout_duration.as_secs());
                    tracing::error!(
                        "🚨 [Service] Background service '{}' timed out starting: {}",
                        name,
                        err_msg
                    );
                    context_clone.app_state.resources.set_subsystem_status(
                        reg_key,
                        crate::types::SubsystemStatus::Failed(err_msg),
                    );
                }
            }
        });
    }
}
