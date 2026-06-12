use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct PrivacyGuardService;

#[async_trait]
impl SystemService for PrivacyGuardService {
    fn name(&self) -> &'static str { "PrivacyGuard" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("PrivacyGuard", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [PrivacyGuard] Privacy Guard shutting down gracefully.");
                    }
                }
                _ = crate::services::privacy::start_privacy_guard(app_state) => {}
            }
        });
        Ok(())
    }
}

pub struct SecurityEvictionService;

#[async_trait]
impl SystemService for SecurityEvictionService {
    fn name(&self) -> &'static str { "SecurityEviction" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let eviction_interval_secs = context.config.rate_limit_eviction_interval_secs;
        let max_bucket_age_secs = context.config.max_bucket_age_secs;
        let max_auth_age_secs = context.config.max_auth_age_secs;
        app_state.resources.set_subsystem_status("SecurityEviction", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            use std::time::Duration;
            let eviction_interval = Duration::from_secs(eviction_interval_secs);
            let max_bucket_age = Duration::from_secs(max_bucket_age_secs);
            let max_auth_age = Duration::from_secs(max_auth_age_secs);
            let mut interval = tokio::time::interval(eviction_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [SecurityEviction] Security Eviction shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        crate::middleware::rate_limit::evict_stale_buckets(max_bucket_age);
                        crate::middleware::auth_rate_limit::evict_expired_blocks(max_auth_age);
                        tracing::debug!("🧹 [Security] Rate limit bucket eviction completed");
                    }
                }
            }
        });
        let _ = app_state;
        Ok(())
    }
}

pub struct BudgetFlushService;

#[async_trait]
impl SystemService for BudgetFlushService {
    fn name(&self) -> &'static str { "BudgetFlush" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let interval_secs = context.config.budget_flush_interval_secs;
        app_state.resources.set_subsystem_status("BudgetFlush", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [BudgetFlush] Budget Flush shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        if let Err(e) = app_state.security.budget_guard.flush_to_db().await {
                            tracing::error!("🚨 [BudgetGuard] Failed to flush usage to DB: {}", e);
                        }
                    }
                }
            }
        });
        Ok(())
    }
}
