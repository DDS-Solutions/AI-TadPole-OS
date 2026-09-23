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
    let crashes_dir = if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
        std::path::PathBuf::from(root).join(".tmp").join("crashes")
    } else {
        std::path::PathBuf::from(".tmp").join("crashes")
    };

    if tokio::fs::metadata(&crashes_dir).await.is_err() {
        return Ok(());
    }

    let mut entries = tokio::fs::read_dir(&crashes_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
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

                    // Find the most recent active/pending mission in the database
                    let active_mission: Option<(String, String)> = sqlx::query_as::<_, (String, String)>(
                        "SELECT id, title FROM mission_history WHERE status IN ('active', 'pending') ORDER BY created_at DESC LIMIT 1"
                    )
                    .fetch_optional(pool)
                    .await?;

                    if let Some((mission_id, mission_title)) = active_mission {
                        tracing::warn!(
                            "🔧 [Reconciler] Reconciling active mission {} ('{}') as crashed.",
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

                        sqlx::query(
                            "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp, hash)
                             VALUES (?1, ?2, 'system', 'system', ?3, 'fatal', ?4, '')"
                        )
                        .bind(&log_id)
                        .bind(&mission_id)
                        .bind(&log_text)
                        .bind(now)
                        .execute(pool)
                        .await?;

                        // Update the mission status
                        sqlx::query("UPDATE mission_history SET status = 'failed' WHERE id = ?1")
                            .bind(&mission_id)
                            .execute(pool)
                            .await?;
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

    Ok(())
}
