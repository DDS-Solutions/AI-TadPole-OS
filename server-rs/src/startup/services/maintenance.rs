//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **SQLite WAL Auto-Checkpoint & Database Maintenance Background Service**
//! Periodically executes non-blocking `PRAGMA wal_checkpoint(TRUNCATE)` to reclaim disk space,
//! compact WAL logs, and execute a final flush during engine shutdown.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: SQLite lock timeout, pool disconnection, or checkpoint error.
//! - **Telemetry Link**: Search `[maintenance]` in tracing logs.

use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct SqliteMaintenanceService;

#[async_trait]
impl SystemService for SqliteMaintenanceService {
    fn name(&self) -> &'static str {
        "SqliteMaintenance"
    }

    fn registry_key(&self) -> &'static str {
        "sqlite_wal_maintenance"
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
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match sqlx::query("PRAGMA wal_checkpoint(TRUNCATE);").execute(&pool).await {
                            Ok(_) => tracing::debug!("🧹 [SQLite] WAL Auto-Checkpoint (TRUNCATE) completed successfully"),
                            Err(e) => tracing::warn!("⚠️ [SQLite] WAL Checkpoint warning: {}", e),
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [SQLite] Executing final WAL checkpoint flush on shutdown...");
                            let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE);").execute(&pool).await;
                            break;
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

// Metadata: [maintenance]
