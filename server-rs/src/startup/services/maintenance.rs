//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / maintenance
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[SQLite]`
//! - **Witness Tests**: none declared

use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct SqliteMaintenanceService;

#[async_trait]
impl SystemService for SqliteMaintenanceService {
    fn name(&self) -> &'static str {
        "SqliteMaintenance"
    }

    fn registry_key(&self) -> &'static str {
        "SqliteMaintenance"
    }

    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let pool = app_state.resources.pool.clone();

        app_state
            .resources
            .set_subsystem_status("SqliteMaintenance", crate::types::SubsystemStatus::Ready);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match sqlx::query("PRAGMA wal_checkpoint(TRUNCATE);").execute(&pool).await {
                            Ok(_) => tracing::debug!("🧹 [SQLite] WAL Auto-Checkpoint (TRUNCATE) completed successfully"),
                            Err(e) => tracing::warn!("⚠️ [SQLite] WAL Checkpoint warning: {}", e),
                        }
                    }
                    res = shutdown_rx.changed() => {
                        match res {
                            Ok(_) => {
                                if *shutdown_rx.borrow() {
                                    tracing::info!("🛑 [SQLite] Executing final WAL checkpoint flush on shutdown...");
                                    if let Err(e) = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE);").execute(&pool).await {
                                        tracing::warn!("⚠️ [SQLite] Final WAL checkpoint on shutdown failed: {}", e);
                                    }
                                    break;
                                }
                            }
                            Err(_) => {
                                tracing::debug!("🔌 [SQLite] Shutdown channel closed.");
                                break;
                            }
                        }
                    }
                }
            }
        });

        tracing::info!(
            "🧹 [SQLite] WAL Auto-Checkpoint & Maintenance service launched (5m interval)."
        );
        Ok(())
    }
}

pub struct StateBackupDaemonService;

#[async_trait]
impl SystemService for StateBackupDaemonService {
    fn name(&self) -> &'static str {
        "StateBackupDaemon"
    }

    fn registry_key(&self) -> &'static str {
        "StateBackupDaemon"
    }

    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let pool = app_state.resources.pool.clone();
        let base_dir = app_state.base_dir.clone();

        app_state
            .resources
            .set_subsystem_status("StateBackupDaemon", crate::types::SubsystemStatus::Ready);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // 1 hour
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut config = crate::services::backup::load_backup_config(&base_dir);
                        if config.enabled && !config.destination_path.trim().is_empty() {
                            tracing::info!("⏰ [BackupDaemon] Executing scheduled hourly state snapshot...");
                            if let Err(e) = crate::services::backup::run_backup_cycle(&pool, &base_dir, &mut config).await {
                                tracing::error!("🚨 [BackupDaemon] Hourly backup failed: {:?}", e);
                            }
                        }
                    }
                    res = shutdown_rx.changed() => {
                        match res {
                            Ok(_) => {
                                if *shutdown_rx.borrow() {
                                    tracing::info!("🛑 [BackupDaemon] Graceful shutdown signal received.");
                                    break;
                                }
                            }
                            Err(_) => {
                                tracing::debug!("🔌 [BackupDaemon] Shutdown channel closed.");
                                break;
                            }
                        }
                    }
                }
            }
        });

        tracing::info!("🛡️ [BackupDaemon] Automated state backup daemon initialized (hourly cadence, 48h retention).");
        Ok(())
    }
}
