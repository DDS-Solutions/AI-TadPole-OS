//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **Engine Health (Static Heartbeat)**: Orchestrates the lightweight 
//! verification of the Tadpole OS engine's operational status. 
//! Features **Real-Time Telemetry Summary**: returns the current 
//! engine version, server timestamp, and active agent count. 
//! Implements **Low-Overhead Monitoring**: designed for 
//! high-frequency polling by load balancers and dashboard health 
//! indicators without impacting systemic performance. AI agents 
//! should distinguish between this static heartbeat and deep 
//! diagnostic scans provided by the `/v1/oversight/security/health` 
//! endpoint (SYS-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 503 Service Unavailable if the Axum router 
//!   is saturated, incorrect agent counts due to registry sync delays, 
//!   or timestamp drift across distributed bunkers.
//! - **Telemetry Link**: Search for `tadpole_online_rust` in health 
//!   dashboard logs for uptime verification.
//! - **Trace Scope**: `server-rs::routes::health`

use crate::state::AppState;
use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

/// Heartbeat status response containing system telemetry.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Operational status string.
    pub status: String,
    /// Current engine version from Cargo.toml.
    pub version: String,
    /// ISO 8601 server timestamp.
    pub heartbeat: String,
    /// Count of currently registered agent nodes.
    pub active_agents: usize,
}

/// A simple heartbeat endpoint that mirrors the old `router.get("/health")` in Express.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "tadpole_online_rust".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        heartbeat: chrono::Utc::now().to_rfc3339(),
        active_agents: state.registry.agents.len(),
    })
}
