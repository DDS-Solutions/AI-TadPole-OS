//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / main
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic internal state integrity.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Sidecar]`, `[Main]`
//! - **Witness Tests**: none declared

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

        // Also write a structured crash JSON to .tmp/crashes/
        let crashes_dir = if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
            std::path::PathBuf::from(root).join(".tmp").join("crashes")
        } else {
            std::path::PathBuf::from(".tmp").join("crashes")
        };

        if std::fs::create_dir_all(&crashes_dir).is_ok() {
            let timestamp_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let crash_file = crashes_dir.join(format!("crash-{}.json", timestamp_ms));

            let clean_message = redactor.scrub(&message);
            let clean_location = redactor.scrub(&location);

            let json_payload = format!(
                "{{\n  \"timestamp\": {},\n  \"message\": {:?},\n  \"location\": {:?}\n}}",
                timestamp_ms, clean_message, clean_location
            );

            if let Err(e) = std::fs::write(&crash_file, json_payload) {
                eprintln!(
                    "CRITICAL: Failed to write emergency structured crash log: {}",
                    e
                );
            }
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

    // 2b. Reconcile any previous runtime crashes before starting background tasks
    if let Err(e) = startup::reconcile_crashes(&app_state.resources.pool).await {
        tracing::error!("🚨 [Main] Failed to reconcile previous crashes: {:?}", e);
    }

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
    // ### 📡 Networking: Endpoint Initialization (TCP Bind)
    // Dispatches the engine to the specified loopback port.
    // Includes dynamic port fallback if the default port is already in use.
    let mut bound_listener = None;
    let mut active_port = config.port;
    let mut active_socket_addr = config.socket_addr;

    for p in 0..10 {
        let current_port = config.port + p;
        let addr_str = format!("{}:{}", config.bind_address, current_port);
        let addr: SocketAddr = match addr_str.parse() {
            Ok(a) => a,
            Err(e) => {
                let msg = format!(
                    "❌ FATAL ERROR: Failed to parse address {}: {:?}",
                    addr_str, e
                );
                tracing::error!("{}", msg);
                eprintln!("{}", msg);
                return Err(anyhow::anyhow!(msg));
            }
        };

        match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => {
                bound_listener = Some(l);
                active_port = current_port;
                active_socket_addr = addr;
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    tracing::warn!("⚠️ Port {} is in use, trying next port...", current_port);
                } else {
                    let msg = format!("❌ FATAL ERROR: Failed to bind to {}: {:?}", addr, e);
                    tracing::error!("{}", msg);
                    eprintln!("{}", msg);
                    return Err(anyhow::anyhow!(msg));
                }
            }
        }
    }

    let listener = match bound_listener {
        Some(l) => l,
        None => {
            let msg = format!(
                "❌ FATAL ERROR: Ports {} through {} are all in use. Please ensure no other instances of 'server-rs' are running.",
                config.port,
                config.port + 9
            );
            tracing::error!("{}", msg);
            eprintln!("{}", msg);
            return Err(anyhow::anyhow!(msg));
        }
    };

    tracing::info!(
        "🚀 Tadpole OS Engine v{} listening on {}",
        env!("CARGO_PKG_VERSION"),
        active_socket_addr
    );
    if active_port != config.port {
        tracing::warn!("🔄 Bound to dynamic fallback port: {}", active_port);
    }

    // Write active port to .tmp/active_port.json for auto-discovery
    let tmp_dir = std::path::Path::new(".tmp");
    if tokio::fs::create_dir_all(tmp_dir).await.is_ok() {
        let port_file = tmp_dir.join("active_port.json");
        let payload = format!(
            "{{\n  \"port\": {},\n  \"url\": \"http://{}:{}\"\n}}",
            active_port, config.bind_address, active_port
        );
        if let Err(e) = tokio::fs::write(&port_file, payload).await {
            tracing::error!("🚨 Failed to write active_port.json: {:?}", e);
        }
    }

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
