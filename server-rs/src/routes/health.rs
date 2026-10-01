//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / health
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::ServiceUnavailable`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `health::tests::test_health_check_operational`

use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

/// Heartbeat status response containing system telemetry and feature flags.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Operational status string.
    pub status: String,
    /// Overall health state of the engine.
    pub health_state: crate::types::SystemHealthState,
    /// Current engine version from Cargo.toml.
    pub version: String,
    /// ISO 8601 server timestamp.
    pub heartbeat: String,
    /// Total registered agent nodes in memory.
    pub registered_agents: usize,
    /// Count of currently non-idle agent nodes.
    pub active_agents: usize,
    /// List of enabled compile-time features (e.g., "neural-audio", "vector-memory").
    pub features: Vec<String>,
}

/// A simple heartbeat endpoint that mirrors the old `router.get("/health")` in Express.
#[tracing::instrument(skip(state), name = "system::health")]
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut features = Vec::new();
    if cfg!(feature = "neural-audio") {
        features.push("neural-audio".to_string());
    }
    if cfg!(feature = "vector-memory") {
        features.push("vector-memory".to_string());
    }

    // Quick database ping
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.resources.pool)
        .await
        .is_ok();

    let mut health_state = state.health_state();
    if !db_ok && health_state != crate::types::SystemHealthState::Degraded {
        health_state = crate::types::SystemHealthState::Degraded;
    }

    let status = match health_state {
        crate::types::SystemHealthState::Ready => "tadpole_online_rust".to_string(),
        crate::types::SystemHealthState::Warming => "tadpole_warming_rust".to_string(),
        crate::types::SystemHealthState::Degraded => "tadpole_degraded_rust".to_string(),
    };

    let registered_agents = state.registry.agents.len();
    let active_agents = state
        .registry
        .agents
        .iter()
        .filter(|a| a.health.status != "idle")
        .count();

    let http_status = if health_state == crate::types::SystemHealthState::Degraded {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    Ok((
        http_status,
        Json(HealthResponse {
            status,
            health_state,
            version: env!("CARGO_PKG_VERSION").to_string(),
            heartbeat: chrono::Utc::now().to_rfc3339(),
            registered_agents,
            active_agents,
            features,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_probe_status() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let res = health_check(State(state)).await;
        assert!(res.is_ok());
    }
}
