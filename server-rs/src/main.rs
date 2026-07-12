#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::enum_variant_names,
    clippy::collapsible_match,
    clippy::unnecessary_map_or,
    clippy::derivable_impls,
    clippy::redundant_closure
)]
//! @docs ARCHITECTURE:Networking
//! @docs OPERATIONS_MANUAL:Lifecycle
//!
//! ### AI Assist Note
//! **Engine Entry Point**: Orchestrates the high-speed lifecycle of the
//! Tadpole OS swarm engine. Manages **Stage 0 Workspace Detection**,
//! where the environment variable `WORKSPACE_ROOT` is used to recalibrate
//! the current working directory for system-wide portability. Handles
//! **Graceful Shutdown Orchestration**, ensuring that all systemic
//! registries (Agents, Providers, Models) are flushed and persisted to
//! SQLite/JSON before process termination.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Port in use (8000), `.env` validation failure
//!   (panics during boot), or LanceDB file lock contention preventing
//!   AppState initialization.
//! - **Telemetry Link**: Search for `[Main]` or `[Sidecar]` in `tracing`
//!   logs for boot sequence milestones.
//! - **Trace Scope**: `server-rs::main`

use crate::state::AppState;
use std::{net::SocketAddr, sync::Arc};

mod adapter;
mod agent;
mod bridge;
mod config;
mod db;
#[cfg(test)]
mod db_tests;
mod env_schema;
pub mod error;
mod intelligence;
mod middleware;
mod networking;
mod router;
mod routes;
mod secret_redactor;
mod security;
mod services;
mod startup;
mod state;
mod system;
mod telemetry;
mod types;
mod utils;

fn main() -> anyhow::Result<()> {
    // 1. Load configuration and validate environment variables early
    let config = crate::config::Config::load()?;

    // 2. Set current working directory to WORKSPACE_ROOT if set and valid
    if let Some(ref canonical_path) = config.workspace_root {
        if let Err(e) = std::env::set_current_dir(canonical_path) {
            eprintln!("⚠️ [Sidecar] Failed to change directory to canonicalized WORKSPACE_ROOT ({:?}): {:?}", canonical_path, e);
        } else {
            println!(
                "🏠 [Sidecar] Workspace Root Set and Canonicalized: {:?}",
                canonical_path
            );
        }
    }

    // ### 🛠️ Resiliency: Emergency Panic Hook
    // Captures accidental runtime panics (e.g., index-out-of-bounds or failed
    // unwrap) and writes a high-fidelity diagnostic log to the workspace root.
    // This bypasses the normal `tracing` facade to ensure the failure context
    // is persisted even if the telemetry stack is what triggered the crash.
    std::panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let raw_log_msg = format!(
            "\n--- PANIC DETECTED ---\nMessage: {}\nLocation: {}\n----------------------\n",
            message, location
        );

        // SEC-04: Clean panic messages of secrets before writing to disk
        let redactor = crate::secret_redactor::SecretRedactor::from_env();
        let log_msg = redactor.scrub(&raw_log_msg);

        // Try to find a writable path for the log
        let log_path = if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
            std::path::PathBuf::from(root).join("sidecar_panic.log")
        } else {
            std::path::PathBuf::from("sidecar_panic.log")
        };

        // Direct filesystem write (bypass tracing/logging stack) with restrictive permissions on Unix
        let mut options = std::fs::OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        if let Err(e) = options.open(&log_path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{}", log_msg)
        }) {
            eprintln!("CRITICAL: Failed to write emergency panic log: {}", e);
        }

        eprintln!("{}", log_msg);
    }));

    println!("🚀 [Sidecar] Initializing Tokio Runtime...");

    // ### 🧵 Resource Calibration: Custom Tokio Runtime
    let rt = startup::build_custom_runtime()?;

    rt.block_on(async_main(config))
}

async fn async_main(config: crate::config::Config) -> anyhow::Result<()> {
    // --- [STAGE: INTENT DETECTION] ---
    // Detect flags that don't require the full engine (Code Graph, mDNS, etc.)
    // Optimized for sub-100ms response for administrative queries.
    let args: Vec<String> = std::env::args().collect();

    // Hyper-Fast Path: Handle version/help before ANY initialization.
    if let Some(()) = startup::handle_admin_cli(&args)? {
        return Ok(());
    }

    // 2. Initialize Tracing & Load Env
    startup::init_tracing(config.disable_telemetry);
    startup::load_environment();

    let intent = startup::detect_bootstrap_intent(&args);

    if intent == startup::BootstrapIntent::Fast {
        tracing::debug!("🏃 [Main] Entering Fast-Path (Intent: {:?})", intent);
    }

    // 2. Initialize App State
    let app_state: Arc<AppState> = match AppState::new().await {
        Ok(state) => Arc::new(state),
        Err(e) => {
            tracing::error!("🚨 [Main] FATAL: Failed to initialize AppState: {:?}", e);
            eprintln!("🚨 [Main] FATAL: Failed to initialize AppState: {:?}", e);
            return Err(anyhow::anyhow!(e));
        }
    };

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let service_config = startup::ServiceConfiguration {
        heartbeat_secs: config.heartbeat_interval_secs,
        ..Default::default()
    };

    // 3. Launch Background Tasks: Telemetry, budget tracking, and swarm health checks.
    startup::spawn_background_tasks(app_state.clone(), intent, service_config, shutdown_rx).await;

    // 4. Build Router
    let app = router::create_router(app_state.clone());

    // 5. Start the Server
    tracing::info!(
        "🚀 Tadpole OS Engine v{} listening on {}",
        env!("CARGO_PKG_VERSION"),
        config.socket_addr
    );

    // ### 📡 Networking: Endpoint Initialization (TCP Bind)
    // Dispatches the engine to the specified loopback port.
    // Includes specific fault-handling for port-binding failures (e.g.,
    // zombie sidecar instances using Port 8000).
    let listener = match tokio::net::TcpListener::bind(config.socket_addr).await {
        Ok(l) => l,
        Err(e) => {
            let msg = if e.kind() == std::io::ErrorKind::AddrInUse {
                format!("❌ FATAL ERROR: Port {} is already in use (os error 10048). Please ensure no other instances of 'server-rs' are running.", config.port)
            } else {
                format!(
                    "❌ FATAL ERROR: Failed to bind to {}: {:?}",
                    config.socket_addr, e
                )
            };
            tracing::error!("{}", msg);
            eprintln!("{}", msg);
            return Err(anyhow::anyhow!(msg));
        }
    };

    // --- [STAGE: RUN] ---
    // Start the Axum server and listen for incoming connections.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    // Signal all background tasks to shut down gracefully
    let _ = shutdown_tx.send(true);

    // --- [STAGE: SHUTDOWN] ---

    tracing::info!("🛑 Tadpole OS Engine shutting down gracefully.");
    // 6. Persistence: Save all systemic registries and flush buffers before exiting.
    // This ensures that metering costs, agent status, and infrastructure configs are fully persisted.
    app_state.flush_all().await;
    app_state.save_agents().await;
    app_state.save_providers().await;
    app_state.save_models().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("🚨 [Main] Failed to install Ctrl+C handler: {:?}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!("🚨 [Main] Failed to install SIGTERM handler: {:?}", e);
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("🛑 Shutdown signal received, draining connections...");
}

// Metadata: [main]
