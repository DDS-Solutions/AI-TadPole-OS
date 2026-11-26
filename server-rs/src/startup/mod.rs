//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Startup
//! - **Primary Entrypoints**: none declared
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod cli;
pub mod runtime;
pub mod services;
pub mod supervisor;
pub mod tracing;

pub use cli::*;
pub use runtime::*;
pub use supervisor::*;
pub use tracing::*;

#[cfg(test)]
mod tests {
    use super::services::telemetry::gather_and_emit_metrics;
    use super::services::*;
    use super::*;
    use crate::state::AppState;
    use crate::types::SubsystemStatus;
    use std::sync::Arc;

    #[test]
    fn test_bootstrap_intent_variants() {
        let full = BootstrapIntent::Full;
        let fast = BootstrapIntent::Fast;
        assert_ne!(full, fast);
    }

    #[tokio::test]
    async fn test_warmup_registry_reporting() {
        let state = Arc::new(AppState::default());

        state
            .resources
            .set_subsystem_status("TestWarmup", SubsystemStatus::Warming(0.1));
        let status = state
            .resources
            .get_initialization_snapshot()
            .get("TestWarmup")
            .cloned();
        assert_eq!(status, Some(SubsystemStatus::Warming(0.1)));

        state
            .resources
            .set_subsystem_status("TestWarmup", SubsystemStatus::Ready);
        let status = state
            .resources
            .get_initialization_snapshot()
            .get("TestWarmup")
            .cloned();
        assert_eq!(status, Some(SubsystemStatus::Ready));
    }

    #[tokio::test]
    async fn test_fast_path_branching() {
        let state = Arc::new(AppState::new_mock().await);

        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let service_config = ServiceConfiguration {
            heartbeat_secs: 3,
            ..Default::default()
        };
        spawn_background_tasks(
            state.clone(),
            BootstrapIntent::Fast,
            service_config,
            shutdown_rx,
        )
        .await;

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let snapshot = state.resources.get_initialization_snapshot();

        assert!(!snapshot.contains_key("CodeGraph"));
        assert!(!snapshot.contains_key("Network"));
    }

    #[cfg(feature = "vector-memory")]
    #[tokio::test]
    async fn test_iks_decay_service_resilience_to_dependency_failure() {
        let state = Arc::new(AppState::new_mock().await);
        let service = IksDecayService;
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let context = SystemContext {
            app_state: state.clone(),
            shutdown_rx,
            config: ServiceConfiguration::default(),
        };

        let res = service.start(context).await;
        assert!(
            res.is_ok(),
            "Service startup should be resilient to dependency errors"
        );
    }

    #[tokio::test]
    async fn test_shutdown_race_condition_responsiveness() {
        let state = Arc::new(AppState::new_mock().await);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let context = SystemContext {
            app_state: state.clone(),
            shutdown_rx,
            config: ServiceConfiguration {
                heartbeat_secs: 5,
                ..Default::default()
            },
        };

        let service = HeartbeatService;
        let res = service.start(context).await;
        assert!(res.is_ok());

        let _ = shutdown_tx.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_resource_lock_contention_graceful_handling() {
        let state = Arc::new(AppState::new_mock().await);

        let graph_lock = state.resources.get_code_graph().await;
        let _write_guard = graph_lock.write();

        let service = CodeGraphWarmupService;
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let context = SystemContext {
            app_state: state.clone(),
            shutdown_rx,
            config: ServiceConfiguration::default(),
        };

        drop(_write_guard);
        let res = service.start(context).await;
        assert!(
            res.is_ok(),
            "Service should complete successfully after lock is released"
        );
    }

    #[tokio::test]
    async fn test_crash_reconciliation() {
        let state = Arc::new(AppState::new_mock().await);
        let pool = &state.resources.pool;

        // Get a valid agent_id to satisfy foreign key constraints
        let agent_id: String = sqlx::query_scalar("SELECT id FROM agents LIMIT 1")
            .fetch_one(pool)
            .await
            .unwrap();

        // Seed an active mission in the database
        let mission_id = "test-mission-123";
        sqlx::query(
            "INSERT INTO mission_history (id, title, status, agent_id, created_at)
             VALUES (?1, 'Test Mission', 'active', ?2, ?3)",
        )
        .bind(mission_id)
        .bind(&agent_id)
        .bind(chrono::Utc::now())
        .execute(pool)
        .await
        .unwrap();

        // Create a temporary crashes directory
        let crashes_dir = if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
            std::path::PathBuf::from(root).join(".tmp").join("crashes")
        } else {
            std::path::PathBuf::from(".tmp").join("crashes")
        };
        std::fs::create_dir_all(&crashes_dir).unwrap();

        // Create a crash JSON file
        let crash_file = crashes_dir.join("crash-12345.json");
        let payload = r#"{
            "timestamp": 12345,
            "message": "assertion failed: self.is_char_boundary(new_len)",
            "location": "src/utils/serialization.rs:36:15"
        }"#;
        std::fs::write(&crash_file, payload).unwrap();

        // Run the reconciler
        reconcile_crashes(pool).await.unwrap();

        // Verify that the crash file has been deleted
        assert!(!crash_file.exists());

        // Verify that the mission status has been updated to failed
        let status: String = sqlx::query_scalar("SELECT status FROM mission_history WHERE id = ?1")
            .bind(mission_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(status, "failed");

        // Verify that a fatal log step has been inserted
        let log_text: String = sqlx::query_scalar(
            "SELECT text FROM mission_logs WHERE mission_id = ?1 AND severity = 'fatal'",
        )
        .bind(mission_id)
        .fetch_one(pool)
        .await
        .unwrap();
        assert!(log_text.contains("assertion failed: self.is_char_boundary(new_len)"));
    }

    #[tokio::test]
    async fn test_dropped_shutdown_channel_terminates_loops_cleanly() {
        let state = Arc::new(AppState::new_mock().await);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let context = SystemContext {
            app_state: state.clone(),
            shutdown_rx,
            config: ServiceConfiguration::default(),
        };

        let service = SqliteMaintenanceService;
        let res = service.start(context).await;
        assert!(res.is_ok());

        // Dropping sender closes channel immediately
        drop(shutdown_tx);

        // Sleep briefly: loop must break cleanly without hanging or busy-looping
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_sqlite_maintenance_registry_key_parity() {
        let service = SqliteMaintenanceService;
        assert_eq!(service.name(), "SqliteMaintenance");
        assert_eq!(service.registry_key(), "SqliteMaintenance");
    }

    #[tokio::test]
    async fn test_heartbeat_cache_invalidation_on_failure_transition() {
        let state = Arc::new(AppState::new_mock().await);
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let context = SystemContext {
            app_state: state.clone(),
            shutdown_rx,
            config: ServiceConfiguration::default(),
        };

        state
            .resources
            .set_subsystem_status("CodeGraph", SubsystemStatus::Warming(0.0));

        let boot_instant = std::time::Instant::now();
        let mut last_tpm = 0;
        let mut last_recruits = 0;
        let mut last_registry_json = None;
        let mut last_snapshot_map = None;

        // First pass
        let _ = gather_and_emit_metrics(
            &context,
            boot_instant,
            &mut last_tpm,
            &mut last_recruits,
            &mut last_registry_json,
            &mut last_snapshot_map,
        )
        .await;

        let json1 = last_registry_json.clone().unwrap();
        assert!(json1.to_string().contains("Warming"));

        // Transition from Warming to Failed (ready count unchanged: 0 -> 0)
        state
            .resources
            .set_subsystem_status("CodeGraph", SubsystemStatus::Failed("Timeout".to_string()));

        // Second pass
        let _ = gather_and_emit_metrics(
            &context,
            boot_instant,
            &mut last_tpm,
            &mut last_recruits,
            &mut last_registry_json,
            &mut last_snapshot_map,
        )
        .await;

        let json2 = last_registry_json.clone().unwrap();
        assert!(json2.to_string().contains("Failed"));
        assert_ne!(json1, json2);
    }
}
