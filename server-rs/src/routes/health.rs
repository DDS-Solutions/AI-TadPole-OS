//! Engine Health & Heartbeat — API
//!
//! Provides a lightweight endpoint for verifying the operational status of the 
//! Rust engine and retrieving basic system-level telemetry.
//!
//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Static Heartbeat**: This endpoint returns an immediate JSON response 
//! (`tadpole_online_rust`). It does not perform deep subsystem diagnostic 
//! scans. For detailed agent health, use `/v1/oversight/security/health`.

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
