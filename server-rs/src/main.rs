//! Tadpole OS Engine - Core Entry Point
//!
//! This module initializes the asynchronous runtime, loads governance configuration,
//! and orchestrates the lifecycle of the swarm engine.
//!
//! @docs ARCHITECTURE:BootSequence

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
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 0. Setup Emergency Panic Hook (Catch silent crashes)
    std::panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        let location = panic_info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        
        let log_msg = format!("\n--- PANIC DETECTED ---\nMessage: {}\nLocation: {}\n----------------------\n", message, location);
        
        // Direct filesystem write (bypass tracing/logging stack)
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("D:\\TadpoleOS-Dev\\sidecar_panic.log")
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

    // 2. Initialize Tracing & Load Env
    startup::init_tracing();
    startup::load_environment();

    // 2. Initialize App State
    let app_state = Arc::new(AppState::new().await);

    // 3. Launch Background Tasks: Telemetry, budget tracking, and swarm health checks.
    startup::spawn_background_tasks(app_state.clone()).await;

    // 4. Build Router
    let app = router::create_router(app_state.clone());

    // 5. Start the Server
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

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

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

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
