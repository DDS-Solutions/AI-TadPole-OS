//! Engine Entry Point — System initialization and server start
//!
//! Orchestrates the high-speed lifecycle of the swarm engine, managing
//! runtime initialization, telemetry, and graceful shutdown.
//!
//! @docs ARCHITECTURE:Networking
//! @docs OPERATIONS_MANUAL:Lifecycle
//!
//! ### AI Assist Note
//! **Workspace Context**: This engine attempts to detect and switch to `WORKSPACE_ROOT` 
//! from the environment during early boot (Stage 0). All subsequent file operations 
//! are relative to this root for portability.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Port in use (addr:8000), `.env` validation failure (panic), or LanceDB lock.
//! - **Telemetry Link**: Search for `[Main]` or `[Sidecar]` in `tracing` logs.
//! - **Trace Scope**: `server-rs::main`

use crate::state::AppState;
use std::{net::SocketAddr, sync::Arc};

mod adapter;
mod agent;
mod db;
#[cfg(test)]
mod db_tests;
mod env_schema;
mod middleware;
mod router;
mod routes;
mod secret_redactor;
mod security;
mod services;
mod startup;
mod state;
mod telemetry;
mod types;
mod utils;

fn main() -> anyhow::Result<()> {
    // 0. Setup Emergency Panic Hook (Catch silent crashes)
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

        let log_msg = format!(
            "\n--- PANIC DETECTED ---\nMessage: {}\nLocation: {}\n----------------------\n",
            message, location
        );

        // Try to find a writable path for the log
        let log_path = if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
            std::path::PathBuf::from(root).join("sidecar_panic.log")
        } else {
            std::path::PathBuf::from("sidecar_panic.log")
        };

        // Direct filesystem write (bypass tracing/logging stack)
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "{}", log_msg)
            });

        eprintln!("{}", log_msg);
    }));

    // 1. Capture Workspace Context (Critical for sidecar portability)
    if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
        let root_path = std::path::Path::new(&root);
        if root_path.exists() {
            let _ = std::env::set_current_dir(root_path);
            // Manual println since tracing isn't up yet
            println!("🏠 [Sidecar] Workspace Root Set: {:?}", root_path);
        }
    }

    println!("🚀 [Sidecar] Initializing Tokio Runtime...");

    // 2. Manual Runtime Setup (To catch tokio::rt::init panics)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build();

    let rt = match rt {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("❌ FATAL: Failed to initialize Tokio runtime: {:?}", e);
            eprintln!("{}", err_msg);
            // Try to log it
            if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
                let _ = std::fs::write(std::path::Path::new(&root).join("sidecar_boot_error.log"), &err_msg);
            }
            return Err(anyhow::anyhow!(err_msg));
        }
    };

    rt.block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    // --- [STAGE: INTENT DETECTION] ---
    // Detect flags that don't require the full engine (Code Graph, mDNS, etc.)
    // Optimized for sub-100ms response for administrative queries.
    let args: Vec<String> = std::env::args().collect();
    
    // Hyper-Fast Path: Handle version/help before ANY initialization.
    // This bypasses Tokio runtime setup, environment validation, and resource allocation.
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        println!("Tadpole OS Engine v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Tadpole OS - Sovereign AI Swarm Engine\n");
        println!("Usage: server-rs [OPTIONS]\n");
        println!("Options:");
        println!("  -v, --version    Show version and exit");
        println!("  -h, --help       Show this help and exit");
        println!("  --status         Show engine status and exit (Fast Path)");
        println!("  --port <PORT>    Set the port to listen on (Default: 8000)");
        return Ok(());
    }

    // 2. Initialize Tracing & Load Env
    startup::init_tracing();
    startup::load_environment();

    let is_fast_path = args.iter().any(|arg| arg == "--status");

    let intent = if is_fast_path {
        startup::BootstrapIntent::Fast
    } else {
        startup::BootstrapIntent::Full
    };

    if is_fast_path {
        tracing::debug!("🏃 [Main] Entering Fast-Path (Intent: {:?})", intent);
    }

    // 2. Initialize App State
    let app_state = match AppState::new().await {
        Ok(state) => Arc::new(state),
        Err(e) => {
            tracing::error!("🚨 FATAL: Failed to initialize AppState: {:?}", e);
            eprintln!("🚨 FATAL: Failed to initialize AppState: {:?}", e);
            return Err(e);
        }
    };

    // 3. Launch Background Tasks: Telemetry, budget tracking, and swarm health checks.
    startup::spawn_background_tasks(app_state.clone(), intent).await;

    // 4. Build Router
    let app = router::create_router(app_state.clone());

    // 5. Start the Server
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let bind_addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr: SocketAddr = format!("{}:{}", bind_addr, port).parse()?;

    tracing::info!(
        "🚀 Tadpole OS Engine v{} listening on {}",
        env!("CARGO_PKG_VERSION"),
        addr
    );

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            let msg = if e.kind() == std::io::ErrorKind::AddrInUse {
                format!("❌ FATAL ERROR: Port {} is already in use (os error 10048). Please ensure no other instances of 'server-rs' are running.", port)
            } else {
                format!("❌ FATAL ERROR: Failed to bind to {}: {:?}", addr, e)
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
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("🛑 Shutdown signal received, draining connections...");
}
