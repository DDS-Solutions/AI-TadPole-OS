//! @docs ARCHITECTURE:TelemetryEngine
//!
//! ### AI Assist Note
//! **Prometheus Metrics (Exporter Route)**: Orchestrates the public-facing
//! Prometheus /metrics scraping endpoint. Automatically fetches and binds
//! dynamic application state properties (active agents, health status,
//! token accumulations, and recursion depths) to Gauges prior to serialization.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Prometheus encoding failures (will log and return 500),
//!   or state lock contentions if reading values.
//! - **Telemetry Link**: Search for `/metrics` requests in routing logs.
//! - **Trace Scope**: `server-rs::routes::metrics`

use crate::state::AppState;
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// Prometheus metrics scraping endpoint.
#[tracing::instrument(skip(state), name = "system::metrics")]
pub async fn metrics_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 1. Dynamically update current application states in Gauges before scrape
    crate::telemetry::TADPOLE_ACTIVE_AGENTS.set(state.registry.agents.len() as f64);
    
    crate::telemetry::TADPOLE_HEALTH_STATE.set(match state.health_state() {
        crate::types::SystemHealthState::Degraded => 0.0,
        crate::types::SystemHealthState::Warming => 1.0,
        crate::types::SystemHealthState::Ready => 2.0,
    });

    crate::telemetry::TADPOLE_MAX_SWARM_DEPTH.set(
        state.governance.max_swarm_depth.load(Ordering::Relaxed) as f64,
    );
    
    crate::telemetry::TADPOLE_TPM_ACCUMULATOR.set(
        state.governance.tpm_accumulator.load(Ordering::Relaxed) as f64,
    );
    
    crate::telemetry::TADPOLE_RECRUIT_COUNT.set(
        state.governance.recruit_count.load(Ordering::Relaxed) as f64,
    );

    // 2. Gather and encode metrics
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("❌ [Metrics] Failed to encode prometheus metrics: {:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 3. Return as standard raw text response
    let res = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
        )
        .body(axum::body::Body::from(buffer));
        
    match res {
        Ok(r) => r.into_response(),
        Err(e) => {
            tracing::error!("❌ [Metrics] Failed to build response: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// Metadata: [metrics]
