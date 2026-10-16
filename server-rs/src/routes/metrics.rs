//! @docs ARCHITECTURE:TelemetryEngine
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / metrics
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Metrics]`
//! - **Witness Tests**: `metrics::tests::test_metrics_exposition`

use crate::state::AppState;
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use prometheus::{Encoder, TextEncoder};
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Prometheus metrics scraping endpoint.
#[tracing::instrument(skip(state), name = "system::metrics")]
pub async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // 1. Dynamically update current application states in Gauges before scrape
    crate::telemetry::TADPOLE_ACTIVE_AGENTS.set(state.registry.agents.len() as f64);

    crate::telemetry::TADPOLE_HEALTH_STATE.set(match state.health_state() {
        crate::types::SystemHealthState::Degraded => 0.0,
        crate::types::SystemHealthState::Warming => 1.0,
        crate::types::SystemHealthState::Ready => 2.0,
    });

    crate::telemetry::TADPOLE_MAX_SWARM_DEPTH
        .set(state.governance.max_swarm_depth.load(Ordering::Relaxed) as f64);

    crate::telemetry::TADPOLE_TPM_ACCUMULATOR
        .set(state.governance.tpm_accumulator.load(Ordering::Relaxed) as f64);

    crate::telemetry::TADPOLE_RECRUIT_COUNT
        .set(state.governance.recruit_count.load(Ordering::Relaxed) as f64);

    // 2. Gather and encode metrics
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("❌ [Metrics] Failed to encode prometheus metrics: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {}", e),
        )
            .into_response();
    }

    // 3. Return as standard raw text response with anti-caching headers
    let res = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
        )
        .header(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        )
        .body(axum::body::Body::from(buffer));

    match res {
        Ok(r) => r.into_response(),
        Err(e) => {
            tracing::error!("❌ [Metrics] Failed to build response: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build metrics response: {}", e),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_exposition() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let res = metrics_handler(State(state)).await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let cache_header = res.headers().get(header::CACHE_CONTROL);
        assert!(cache_header.is_some());
        assert_eq!(
            cache_header.unwrap().to_str().unwrap(),
            "no-store, no-cache, must-revalidate"
        );
    }
}
