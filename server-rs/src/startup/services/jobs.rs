use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct ContinuitySchedulerService;

#[async_trait]
impl SystemService for ContinuitySchedulerService {
    fn name(&self) -> &'static str { "ContinuityScheduler" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("ContinuityScheduler", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [Continuity] Scheduled job executor shutting down gracefully.");
                    }
                }
                _ = crate::agent::continuity::executor::start_scheduler(app_state) => {}
            }
        });
        tracing::info!("🕐 [Continuity] Scheduled job executor launched.");
        Ok(())
    }
}

pub struct SwarmReaperService;

#[async_trait]
impl SystemService for SwarmReaperService {
    fn name(&self) -> &'static str { "SwarmReaper" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("SwarmReaper", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [Reaper] Swarm Reaper shutting down gracefully.");
                    }
                }
                _ = crate::agent::reaper::SwarmReaper::start(app_state) => {}
            }
        });
        tracing::info!("♻️ [Reaper] Swarm Reaper launched (48h retention policy).");
        Ok(())
    }
}

pub struct MemoryCleanupService;

#[async_trait]
impl SystemService for MemoryCleanupService {
    fn name(&self) -> &'static str { "MemoryCleanup" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let shutdown_rx = context.shutdown_rx;
        let interval_secs = context.config.memory_cleanup_interval_secs;
        app_state.resources.set_subsystem_status("MemoryCleanup", crate::types::SubsystemStatus::Ready);
        #[cfg(not(feature = "vector-memory"))]
        let _ = (app_state, shutdown_rx, interval_secs);

        #[cfg(feature = "vector-memory")]
        {
            let memory_cleanup_pool = app_state.resources.pool.clone();
            tokio::spawn(async move {
                let mut shutdown_rx = shutdown_rx;
                crate::agent::memory::VectorMemory::cleanup_orphaned_scopes(&memory_cleanup_pool).await;
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    tokio::select! {
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [MemoryCleanup] Memory Cleanup shutting down gracefully.");
                                break;
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
    fn name(&self) -> &'static str { "IngestionWorker" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("IngestionWorker", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [IngestionWorker] Ingestion Worker shutting down gracefully.");
                    }
                }
                _ = crate::agent::connectors::start_ingestion_worker(app_state) => {}
            }
        });
        Ok(())
    }
}

pub struct RecipeIngestionService;

#[async_trait]
impl SystemService for RecipeIngestionService {
    fn name(&self) -> &'static str { "RecipeIngestion" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("RecipeIngestion", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [RecipeIngestion] Recipe Ingestion shutting down gracefully.");
                    }
                }
                _ = crate::agent::recipes::auto_ingest_recipes(app_state) => {}
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
    fn name(&self) -> &'static str { "IksDecay" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let decay_interval_secs = context.config.iks_decay_interval_secs;
        app_state.resources.set_subsystem_status("IksDecay", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(decay_interval_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [IksDecay] IKS Decay shutting down gracefully.");
                            break;
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
    fn name(&self) -> &'static str { "IksEviction" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let eviction_interval_secs = context.config.iks_eviction_interval_secs;
        app_state.resources.set_subsystem_status("IksEviction", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(30 * 60)).await;
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(eviction_interval_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [IksEviction] IKS Eviction shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
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
