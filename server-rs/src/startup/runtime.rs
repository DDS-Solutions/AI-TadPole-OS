//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Startup / Tokio Runtime & Crash Reconciliation
//! - **Primary Entrypoints**: `build_custom_runtime`, `reconcile_crashes`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

/// Configures and builds a custom multi-threaded Tokio runtime.
pub fn build_custom_runtime() -> anyhow::Result<tokio::runtime::Runtime> {
    let default_stack = if cfg!(debug_assertions) {
        16 * 1024 * 1024 // 16 MB for debug builds (unoptimized async state machines on Windows)
    } else {
        4 * 1024 * 1024 // 4 MB for release builds
    };

    let stack_size = std::env::var("TOKIO_THREAD_STACK_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default_stack);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(
            std::thread::available_parallelism()
                .map(|n| n.get().max(4))
                .unwrap_or(4),
        )
        .max_blocking_threads(32)
        .thread_name("tadpole-worker")
        .thread_stack_size(stack_size)
        .enable_all()
        .build();

    match rt {
        Ok(r) => Ok(r),
        Err(e) => {
            let err_msg = format!("❌ FATAL: Failed to initialize Tokio runtime: {:?}", e);
            eprintln!("{}", err_msg);
            if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
                let _ = std::fs::write(
                    std::path::Path::new(&root).join("sidecar_boot_error.log"),
                    &err_msg,
                );
            }
            Err(anyhow::anyhow!(err_msg))
        }
    }
}

/// Scans for structured JSON crash logs in `.tmp/crashes/` and updates the database state accordingly.
pub async fn reconcile_crashes(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    // Sweep orphaned pending oversight_log entries from prior interrupted runtime
    let swept = sqlx::query(
        "UPDATE oversight_log SET status = 'rejected', decision = 'interrupted', decided_at = CURRENT_TIMESTAMP, decided_by = 'system_recovery' WHERE status = 'pending'"
    )
    .execute(pool)
    .await;
    if let Ok(res) = swept {
        if res.rows_affected() > 0 {
            tracing::warn!(
                "🔧 [Reconciler] Swept {} orphaned pending oversight entries to rejected/interrupted.",
                res.rows_affected()
            );
        }
    }

    let crashes_dir = if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
        std::path::PathBuf::from(root).join(".tmp").join("crashes")
    } else {
        std::path::PathBuf::from(".tmp").join("crashes")
    };

    if tokio::fs::metadata(&crashes_dir).await.is_ok() {
        if let Ok(mut entries) = tokio::fs::read_dir(&crashes_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    // Parse structured JSON
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            let msg = val
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("Unknown panic");
                            let loc = val
                                .get("location")
                                .and_then(|l| l.as_str())
                                .unwrap_or("unknown location");
                            let timestamp_str = val
                                .get("timestamp")
                                .map(|t| t.to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            tracing::warn!("🔧 [Reconciler] Found crash log from panic: {} at {}. Reconciling database...", msg, loc);

                            // Find all active/pending missions in the database that were interrupted by crash
                            let active_missions: Vec<(String, String)> = sqlx::query_as::<_, (String, String)>(
                                "SELECT id, title FROM mission_history WHERE status IN ('active', 'pending') ORDER BY created_at DESC"
                            )
                            .fetch_all(pool)
                            .await?;

                            for (mission_id, mission_title) in active_missions {
                                tracing::warn!(
                                    "🔧 [Reconciler] Reconciling interrupted mission {} ('{}') as crashed.",
                                    mission_id,
                                    mission_title
                                );

                                // Insert a fatal step log
                                let log_id = uuid::Uuid::new_v4().to_string();
                                let now = chrono::Utc::now();
                                let log_text = format!(
                                    "🚨 ENGINE CRASHED: {}\nLocation: {}\nTimestamp: {}",
                                    msg, loc, timestamp_str
                                );

                                let _ = sqlx::query(
                                    "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp, hash)
                                     VALUES (?1, ?2, 'system', 'system', ?3, 'fatal', ?4, '')"
                                )
                                .bind(&log_id)
                                .bind(&mission_id)
                                .bind(&log_text)
                                .bind(now)
                                .execute(pool)
                                .await;

                                // Update the mission status to failed with completion timestamp
                                let _ = sqlx::query("UPDATE mission_history SET status = 'failed', updated_at = ?1 WHERE id = ?2")
                                    .bind(now)
                                    .bind(&mission_id)
                                    .execute(pool)
                                    .await;
                            }
                        }
                    }
                    // Delete the crash file once processed
                    if let Err(e) = tokio::fs::remove_file(&path).await {
                        tracing::error!(
                            "🚨 [Reconciler] Failed to remove reconciled crash file {:?}: {:?}",
                            path,
                            e
                        );
                    }
                }
            }
        }
    }

    // Sweep any orphaned active/pending missions from prior run (e.g. process termination without crash dump)
    let orphaned_active: Vec<(String, String)> = sqlx::query_as::<_, (String, String)>(
        "SELECT id, title FROM mission_history WHERE status IN ('active', 'pending') ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (m_id, m_title) in orphaned_active {
        tracing::warn!(
            "🔧 [Reconciler] Reconciling orphaned mission {} ('{}') as failed across restart.",
            m_id,
            m_title
        );
        let log_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let log_text = "⚠️ [Recovery] Mission interrupted by ungraceful engine shutdown or process termination. Marked failed.";

        let _ = sqlx::query(
            "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp, hash)
             VALUES (?1, ?2, 'system', 'system', ?3, 'warning', ?4, '')"
        )
        .bind(&log_id)
        .bind(&m_id)
        .bind(log_text)
        .bind(now)
        .execute(pool)
        .await;

        let _ = sqlx::query(
            "UPDATE mission_history SET status = 'failed', updated_at = ?1 WHERE id = ?2 AND status IN ('active', 'pending')"
        )
        .bind(now)
        .bind(&m_id)
        .execute(pool)
        .await;
    }

    // Sweep any agents left as 'busy' so they boot in 'idle' state
    let _ = sqlx::query(
        "UPDATE agents SET status = 'idle', active_mission = NULL, current_task = NULL WHERE status = 'busy'"
    )
    .execute(pool)
    .await;

    Ok(())
}
