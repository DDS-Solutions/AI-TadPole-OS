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
    // 1. Initialize Tracing & Load Env
    startup::init_tracing();
    startup::load_environment();

    // 2. Initialize App State
    let app_state = Arc::new(AppState::new().await);

    // 3. Launch Background Tasks
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

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    tracing::info!("🛑 Tadpole OS Engine shutting down gracefully.");
    // Persist all agent state before exit
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
