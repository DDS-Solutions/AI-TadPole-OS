//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / health
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::ServiceUnavailable`
//! - **Telemetry Targets**: none declared
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

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
    /// Emergency stop remains latched until the protected engine resume action.
    pub dispatch_paused: bool,
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

    // Merkle audit trail integrity verification (last 10 records)
    let audit_ok = match state.security.audit_trail.verify_last_n(10, None).await {
        Ok((verified, total)) => total == 0 || verified == total,
        Err(_) => false,
    };

    let mut health_state = state.health_state();
    if (!db_ok || !audit_ok) && health_state != crate::types::SystemHealthState::Degraded {
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

    let http_status = match health_state {
        crate::types::SystemHealthState::Ready => StatusCode::OK,
        crate::types::SystemHealthState::Warming | crate::types::SystemHealthState::Degraded => {
            StatusCode::SERVICE_UNAVAILABLE
        }
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
            dispatch_paused: state
                .comms
                .engine_stopped
                .load(std::sync::atomic::Ordering::Acquire),
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

    #[tokio::test]
    async fn health_reports_dispatch_latch() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .comms
            .engine_stopped
            .store(true, std::sync::atomic::Ordering::Release);
        let response = health_check(State(state)).await.unwrap().into_response();
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["dispatch_paused"], true);
    }
}
