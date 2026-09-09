//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / benchmarks
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Sqlx`
//! - **Telemetry Targets**: `[Benchmarks]`
//! - **Witness Tests**: `benchmarks::tests::test_create_benchmark_validation`, `benchmarks::tests::test_trigger_benchmark_accepted`

use crate::agent::benchmarks::{self, BenchmarkResult};
use crate::error::AppError;
use crate::routes::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBenchmarkRequest {
    pub name: String,
    pub category: String,
    pub test_id: String,
    pub mean_ms: f64,
    pub p95_ms: Option<f64>,
    pub p99_ms: Option<f64>,
    pub target_value: Option<String>,
    pub status: String,
    pub metadata: Option<String>,
}

impl CreateBenchmarkRequest {
    pub fn validate_and_into(self) -> Result<BenchmarkResult, AppError> {
        let name = self.name.trim().to_string();
        if name.is_empty() || name.len() > 128 {
            return Err(AppError::BadRequest(
                "Benchmark name must be non-empty and <= 128 characters".into(),
            ));
        }

        let test_id = self.test_id.trim().to_string();
        if test_id.is_empty()
            || test_id.len() > 64
            || !test_id
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AppError::BadRequest(
                "test_id must be non-empty, <= 64 chars, and alphanumeric/-/_".into(),
            ));
        }

        if !self.mean_ms.is_finite() || self.mean_ms < 0.0 {
            return Err(AppError::BadRequest(
                "mean_ms must be a non-negative finite number".into(),
            ));
        }

        Ok(BenchmarkResult {
            id: Uuid::new_v4().to_string(),
            name,
            category: self.category.trim().to_string(),
            test_id,
            mean_ms: self.mean_ms,
            p95_ms: self.p95_ms.filter(|v| v.is_finite() && *v >= 0.0),
            p99_ms: self.p99_ms.filter(|v| v.is_finite() && *v >= 0.0),
            target_value: self.target_value,
            status: self.status,
            metadata: self.metadata,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[tracing::instrument(skip(state), name = "metrics::list_benchmarks")]
pub async fn get_benchmarks(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (_page, per_page) = pagination.sanitize();
    let offset = pagination.offset() as u32;

    let (items, total_count) =
        benchmarks::list_benchmarks_paginated(&state.resources.pool, per_page, offset).await?;

    let response = PaginatedResponse::from_pre_sliced(
        items,
        total_count as u32,
        &pagination,
        "/v1/benchmarks",
    );
    Ok(Json(response))
}

#[tracing::instrument(skip(state), name = "metrics::get_benchmark_history")]
pub async fn get_benchmark_history(
    State(state): State<Arc<AppState>>,
    Path(test_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let trimmed_test_id = test_id.trim();
    if trimmed_test_id.is_empty() || trimmed_test_id.len() > 64 {
        return Err(AppError::BadRequest("Invalid test_id parameter".into()));
    }

    let results =
        benchmarks::get_benchmark_comparison(&state.resources.pool, trimmed_test_id).await?;
    Ok(Json(results))
}

#[tracing::instrument(skip(state, payload), name = "metrics::save_benchmark")]
pub async fn create_benchmark(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateBenchmarkRequest>,
) -> Result<impl IntoResponse, AppError> {
    let benchmark_result = payload.validate_and_into()?;

    benchmarks::save_benchmark(&state.resources.pool, benchmark_result.clone()).await?;

    tracing::info!(
        "📊 [Benchmarks] Saved benchmark result '{}' for test '{}'",
        benchmark_result.id,
        benchmark_result.test_id
    );

    Ok((StatusCode::CREATED, Json(benchmark_result)))
}

#[tracing::instrument(skip(state), name = "metrics::trigger_suite")]
pub async fn trigger_benchmark(
    State(state): State<Arc<AppState>>,
    Path(test_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let trimmed_test_id = test_id.trim().to_string();
    if trimmed_test_id.is_empty()
        || trimmed_test_id.len() > 64
        || !trimmed_test_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::BadRequest("Invalid test_id format".into()));
    }

    let state_clone = Arc::clone(&state);
    let tid_clone = trimmed_test_id.clone();
    tokio::spawn(async move {
        if let Err(e) = benchmarks::run_benchmark_suite(state_clone, &tid_clone).await {
            tracing::error!("❌ [Benchmarks] Background benchmark suite failed: {}", e);
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "accepted",
            "test_id": trimmed_test_id,
            "message": "Benchmark suite execution triggered in background"
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_benchmark_validation() {
        let req = CreateBenchmarkRequest {
            name: "Test Run".into(),
            category: "Runner".into(),
            test_id: "BM-RUN-01".into(),
            mean_ms: 12.5,
            p95_ms: Some(15.0),
            p99_ms: Some(20.0),
            target_value: Some("< 100ms".into()),
            status: "PASS".into(),
            metadata: None,
        };

        let res = req.validate_and_into();
        assert!(res.is_ok());
        let item = res.unwrap();
        assert_eq!(item.name, "Test Run");
        assert!(!item.id.is_empty());

        let bad_req = CreateBenchmarkRequest {
            name: "".into(),
            category: "Runner".into(),
            test_id: "BM-RUN-01".into(),
            mean_ms: -5.0,
            p95_ms: None,
            p99_ms: None,
            target_value: None,
            status: "FAIL".into(),
            metadata: None,
        };
        assert!(bad_req.validate_and_into().is_err());
    }
}
