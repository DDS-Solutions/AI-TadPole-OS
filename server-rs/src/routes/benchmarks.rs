//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **Performance Benchmarks (Metrics API)**: Orchestrates the 
//! stress testing and efficiency measurement for the Tadpole OS 
//! agent swarm. Features **Latency & Throughput Tracking**: provides 
//! endpoints for triggering and retrieving results from automated 
//! benchmark suites (e.g., `run_benchmark_suite`). Implements **Trend 
//! Analysis**: stores and compares historical performance data to 
//! identify regressions in model response times or token 
//! efficiency. AI agents should use these endpoints to validate 
//! system stability after significant architectural or provider-layer 
//! modifications (PERF-02).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 500 Internal Server Error due to database 
//!   connectivity issues during result persistence, or benchmark 
//!   timeouts during high-concurrency stress tests.
//! - **Telemetry Link**: Search for `[Benchmark]` or `[Performance]` 
//!   in `tracing` logs for suite execution milestones.
//! - **Trace Scope**: `server-rs::routes::benchmarks`

use crate::agent::benchmarks::{self, BenchmarkResult};

use crate::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_benchmarks(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match benchmarks::list_benchmarks(&state.resources.pool).await {
        Ok(results) => (StatusCode::OK, Json(results)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_benchmark_history(
    State(state): State<Arc<AppState>>,
    Path(test_id): Path<String>,
) -> impl IntoResponse {
    match benchmarks::get_benchmark_comparison(&state.resources.pool, &test_id).await {
        Ok(results) => (StatusCode::OK, Json(results)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_benchmark(
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<BenchmarkResult>,
) -> impl IntoResponse {
    if payload.id.is_empty() {
        payload.id = Uuid::new_v4().to_string();
    }

    match benchmarks::save_benchmark(&state.resources.pool, payload).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn trigger_benchmark(
    State(state): State<Arc<AppState>>,
    Path(test_id): Path<String>,
) -> impl IntoResponse {
    match benchmarks::run_benchmark_suite(&state.resources.pool, &test_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
