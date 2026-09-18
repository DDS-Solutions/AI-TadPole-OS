//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / jobs
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Continuity]`, `[Reaper]`, `[SpanWatchdog]`, `[MemoryCleanup]`, `[IngestionWorker]`, `[RecipeIngestion]`, `[IksDecay]`, `[IKS]`, `[IksEviction]`
//! - **Witness Tests**: none declared

use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct ContinuitySchedulerService;

#[async_trait]
impl SystemService for ContinuitySchedulerService {
    fn name(&self) -> &'static str {
        "ContinuityScheduler"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state
            .resources
            .set_subsystem_status("ContinuityScheduler", crate::types::SubsystemStatus::Ready);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            tokio::select! {
                res = shutdown_rx.changed() => {
                    match res {
                        Ok(_) => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [Continuity] Scheduled job executor shutting down gracefully.");
                            }
                        }
                        Err(_) => {
                            tracing::debug!("🔌 [Continuity] Shutdown channel closed.");
                        }
                    }
                }
                _ = crate::agent::continuity::executor::start_scheduler(app_state_clone.clone()) => {
                    tracing::error!("🚨 [Continuity] Scheduled job executor exited unexpectedly.");
                    app_state_clone.resources.set_subsystem_status(
                        "ContinuityScheduler",
                        crate::types::SubsystemStatus::Failed("Scheduler worker exited unexpectedly".to_string()),
                    );
                }
            }
        });
        tracing::info!("🕐 [Continuity] Scheduled job executor launched.");
        Ok(())
    }
}

pub struct SwarmReaperService;

#[async_trait]
impl SystemService for SwarmReaperService {
    fn name(&self) -> &'static str {
        "SwarmReaper"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state
            .resources
            .set_subsystem_status("SwarmReaper", crate::types::SubsystemStatus::Ready);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            tokio::select! {
                res = shutdown_rx.changed() => {
                    match res {
                        Ok(_) => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [Reaper] Swarm Reaper shutting down gracefully.");
                            }
                        }
                        Err(_) => {
                            tracing::debug!("🔌 [Reaper] Shutdown channel closed.");
                        }
                    }
                }
                _ = crate::agent::reaper::SwarmReaper::start(app_state_clone.clone()) => {
                    tracing::error!("🚨 [Reaper] Swarm Reaper exited unexpectedly.");
                    app_state_clone.resources.set_subsystem_status(
                        "SwarmReaper",
                        crate::types::SubsystemStatus::Failed("Reaper worker exited unexpectedly".to_string()),
                    );
                }
            }
        });
        tracing::info!("♻️ [Reaper] Swarm Reaper launched (48h retention policy).");
        Ok(())
    }
}

pub struct SpanWatchdogService;

#[async_trait]
impl SystemService for SpanWatchdogService {
    fn name(&self) -> &'static str {
        "SpanWatchdog"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state
            .resources
            .set_subsystem_status("SpanWatchdog", crate::types::SubsystemStatus::Ready);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            tokio::select! {
                res = shutdown_rx.changed() => {
                    match res {
                        Ok(_) => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [SpanWatchdog] Span Watchdog shutting down gracefully.");
                            }
                        }
                        Err(_) => {
                            tracing::debug!("🔌 [SpanWatchdog] Shutdown channel closed.");
                        }
                    }
                }
                _ = crate::telemetry::span_watchdog::SpanWatchdog::start_background_loop() => {
                    tracing::error!("🚨 [SpanWatchdog] Span Watchdog loop exited unexpectedly.");
                    app_state_clone.resources.set_subsystem_status(
                        "SpanWatchdog",
                        crate::types::SubsystemStatus::Failed("SpanWatchdog exited unexpectedly".to_string()),
                    );
                }
            }
        });
        tracing::info!("⏱️ [SpanWatchdog] Adaptive Span Lifecycle Watchdog service launched.");
        Ok(())
    }
}

pub struct MemoryCleanupService;

#[async_trait]
impl SystemService for MemoryCleanupService {
    fn name(&self) -> &'static str {
        "MemoryCleanup"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        app_state
            .resources
            .set_subsystem_status("MemoryCleanup", crate::types::SubsystemStatus::Ready);

        #[cfg(feature = "vector-memory")]
        {
            let shutdown_rx = context.shutdown_rx;
            let interval_secs = context.config.memory_cleanup_interval_secs;
            let memory_cleanup_pool = app_state.resources.pool.clone();
            tokio::spawn(async move {
                let mut shutdown_rx = shutdown_rx;
                let mut interval =
                    tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    tokio::select! {
                        res = shutdown_rx.changed() => {
                            match res {
                                Ok(_) => {
                                    if *shutdown_rx.borrow() {
                                        tracing::info!("🛑 [MemoryCleanup] Memory Cleanup shutting down gracefully.");
                                        break;
                                    }
                                }
                                Err(_) => {
                                    tracing::debug!("🔌 [MemoryCleanup] Shutdown channel closed.");
                                    break;
                                }
                            }
                        }
                        _ = interval.tick() => {
                            crate::agent::memory::VectorMemory::cleanup_orphaned_scopes(&memory_cleanup_pool).await;
                        }
                    }
                }
            });
        }
        Ok(())
    }
}

pub struct IngestionWorkerService;

#[async_trait]
impl SystemService for IngestionWorkerService {
    fn name(&self) -> &'static str {
        "IngestionWorker"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state
            .resources
            .set_subsystem_status("IngestionWorker", crate::types::SubsystemStatus::Ready);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            tokio::select! {
                res = shutdown_rx.changed() => {
                    match res {
                        Ok(_) => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [IngestionWorker] Ingestion Worker shutting down gracefully.");
                            }
                        }
                        Err(_) => {
                            tracing::debug!("🔌 [IngestionWorker] Shutdown channel closed.");
                        }
                    }
                }
                _ = crate::agent::connectors::start_ingestion_worker(app_state_clone.clone()) => {
                    tracing::error!("🚨 [IngestionWorker] Ingestion Worker exited unexpectedly.");
                    app_state_clone.resources.set_subsystem_status(
                        "IngestionWorker",
                        crate::types::SubsystemStatus::Failed("Ingestion worker exited unexpectedly".to_string()),
                    );
                }
            }
        });
        Ok(())
    }
}

pub struct RecipeIngestionService;

#[async_trait]
impl SystemService for RecipeIngestionService {
    fn name(&self) -> &'static str {
        "RecipeIngestion"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state
            .resources
            .set_subsystem_status("RecipeIngestion", crate::types::SubsystemStatus::Ready);
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            tokio::select! {
                res = shutdown_rx.changed() => {
                    match res {
                        Ok(_) => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [RecipeIngestion] Recipe Ingestion shutting down gracefully.");
                            }
                        }
                        Err(_) => {
                            tracing::debug!("🔌 [RecipeIngestion] Shutdown channel closed.");
                        }
                    }
                }
                _ = crate::agent::recipes::auto_ingest_recipes(app_state_clone.clone()) => {
                    tracing::error!("🚨 [RecipeIngestion] Recipe Ingestion exited unexpectedly.");
                    app_state_clone.resources.set_subsystem_status(
                        "RecipeIngestion",
                        crate::types::SubsystemStatus::Failed("Recipe ingestion worker exited unexpectedly".to_string()),
                    );
                }
            }
        });
        Ok(())
    }
}

#[cfg(feature = "vector-memory")]
pub struct IksDecayService;

#[cfg(feature = "vector-memory")]
#[async_trait]
impl SystemService for IksDecayService {
    fn name(&self) -> &'static str {
        "IksDecay"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let decay_interval_secs = context.config.iks_decay_interval_secs;
        app_state
            .resources
            .set_subsystem_status("IksDecay", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(decay_interval_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    res = shutdown_rx.changed() => {
                        match res {
                            Ok(_) => {
                                if *shutdown_rx.borrow() {
                                    tracing::info!("🛑 [IksDecay] IKS Decay shutting down gracefully.");
                                    break;
                                }
                            }
                            Err(_) => {
                                tracing::debug!("🔌 [IksDecay] Shutdown channel closed.");
                                break;
                            }
                        }
                    }
                    _ = interval.tick() => {
                        match app_state.resources.get_knowledge_store().await {
                            Ok(ks) => {
                                if let Err(e) = ks.decay_confidence().await {
                                    tracing::warn!("[IKS] Confidence decay pass failed: {:?}", e);
                                } else {
                                    tracing::debug!("[IKS] Confidence decay pass complete.");
                                }
                            }
                            Err(e) => {
                                tracing::warn!("[IKS] Could not acquire store for decay: {:?}", e);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

#[cfg(feature = "vector-memory")]
pub struct IksEvictionService;

#[cfg(feature = "vector-memory")]
#[async_trait]
impl SystemService for IksEvictionService {
    fn name(&self) -> &'static str {
        "IksEviction"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let eviction_interval_secs = context.config.iks_eviction_interval_secs;
        app_state
            .resources
            .set_subsystem_status("IksEviction", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let initial_delay = tokio::time::sleep(std::time::Duration::from_secs(30 * 60));
            tokio::pin!(initial_delay);
            let mut initial_passed = false;
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(eviction_interval_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    res = shutdown_rx.changed() => {
                        match res {
                            Ok(_) => {
                                if *shutdown_rx.borrow() {
                                    tracing::info!("🛑 [IksEviction] IKS Eviction shutting down gracefully.");
                                    break;
                                }
                            }
                            Err(_) => {
                                tracing::debug!("🔌 [IksEviction] Shutdown channel closed.");
                                break;
                            }
                        }
                    }
                    _ = &mut initial_delay, if !initial_passed => {
                        initial_passed = true;
                        tracing::debug!("[IksEviction] Initial 30-minute warmup delay elapsed.");
                    }
                    _ = interval.tick(), if initial_passed => {
                        match app_state.resources.get_knowledge_store().await {
                            Ok(ks) => match ks.evict_expired().await {
                                Ok(n) => {
                                    tracing::info!("[IKS] Eviction pass removed {} entries.", n);
                                }
                                Err(e) => {
                                    tracing::warn!("[IKS] Eviction pass failed: {:?}", e);
                                }
                            },
                            Err(e) => {
                                tracing::warn!("[IKS] Could not acquire store for eviction: {:?}", e);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}
