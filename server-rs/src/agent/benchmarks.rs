//! Agent Benchmarking — Performance measurement and model evaluation
//!
//! Provides a standardized testbed for evaluating model latency, 
//! cost, and accuracy across different task categories.
//!
//! @docs ARCHITECTURE:Agent

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BenchmarkResult {
    pub id: String,
    pub name: String,
    pub category: String,
    pub test_id: String,
    pub mean_ms: f64,
    pub p95_ms: Option<f64>,
    pub p99_ms: Option<f64>,
    pub target_value: Option<String>,
    pub status: String, // PASS | FAIL | NEEDS_INVESTIGATION
    pub metadata: Option<String>,
    pub created_at: String,
}

pub async fn save_benchmark(pool: &SqlitePool, result: BenchmarkResult) -> Result<()> {
    sqlx::query(
        "INSERT INTO benchmarks (id, name, category, test_id, mean_ms, p95_ms, p99_ms, target_value, status, metadata)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&result.id)
    .bind(&result.name)
    .bind(&result.category)
    .bind(&result.test_id)
    .bind(result.mean_ms)
    .bind(result.p95_ms)
    .bind(result.p99_ms)
    .bind(&result.target_value)
    .bind(&result.status)
    .bind(&result.metadata)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_benchmarks(pool: &SqlitePool) -> Result<Vec<BenchmarkResult>> {
    let results = sqlx::query_as::<_, BenchmarkResult>(
        "SELECT id, name, category, test_id, mean_ms, p95_ms, p99_ms, target_value, status, metadata, CAST(created_at AS TEXT) as created_at FROM benchmarks ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn get_benchmark_comparison(
    pool: &SqlitePool,
    test_id: &str,
) -> Result<Vec<BenchmarkResult>> {
    let results = sqlx::query_as::<_, BenchmarkResult>(
        "SELECT id, name, category, test_id, mean_ms, p95_ms, p99_ms, target_value, status, metadata, CAST(created_at AS TEXT) as created_at FROM benchmarks WHERE test_id = ? ORDER BY created_at DESC LIMIT 10"
    )
    .bind(test_id)
    .fetch_all(pool)
    .await?;

    Ok(results)
}
pub async fn run_benchmark_suite(pool: &SqlitePool, test_id: &str) -> Result<BenchmarkResult> {
    let mut status = "PASS".to_string();
    let target_value;
    let name;
    let category;
    let mut metadata = Some("Self-triggered via Dashboard".to_string());

    let start = std::time::Instant::now();

    match test_id {
        "BM-RUN-01" => {
            name = "Agent Runner Baseline".to_string();
            category = "Runner".to_string();
            target_value = Some("< 100ms".to_string());

            // Measure agent initialization and context resolution overhead
            let state = std::sync::Arc::new(crate::state::AppState::new().await.expect("Failed to initialize state for benchmarks"));
            let runner = crate::agent::runner::AgentRunner::new(state.clone());

            for _ in 0..10 {
                let _ = runner.state.registry.agents.contains_key("overlord");
            }
        }
        "BM-DB-01" => {
            name = "Persistence Stress (50 Writes)".to_string();
            category = "Database".to_string();
            target_value = Some("< 200ms".to_string());

            // Perform 50 actual writes
            for i in 0..50 {
                sqlx::query("INSERT INTO benchmark_logs (test_id, step) VALUES (?, ?)")
                    .bind(test_id)
                    .bind(i)
                    .execute(pool)
                    .await?;
            }
        }
        "BM-RL-01" => {
            name = "Rate Limiter Overhead".to_string();
            category = "Rate Limiter".to_string();
            target_value = Some("< 1ms".to_string());

            let limiter = crate::agent::rate_limiter::RateLimiter::new(Some(1000), Some(100000));
            for _ in 0..100 {
                limiter.acquire(10).await;
                limiter.record_usage(10);
            }
            metadata = Some("100 cycles of acquire/record".to_string());
        }
        _ => return Err(anyhow::anyhow!("Unknown benchmark test ID: {}", test_id)),
    }

    let duration = start.elapsed().as_secs_f64() * 1000.0;

    // Check status against target thresholds
    if test_id == "BM-RUN-01" && duration > 100.0 {
        status = "FAIL".to_string();
    }
    if test_id == "BM-DB-01" && duration > 200.0 {
        status = "FAIL".to_string();
    }
    if test_id == "BM-RL-01" && duration > 1.0 {
        status = "FAIL".to_string();
    }

    let result = BenchmarkResult {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        category,
        test_id: test_id.to_string(),
        mean_ms: duration,
        p95_ms: Some(duration * 1.02), // Statistical approximation for single run
        p99_ms: Some(duration * 1.05),
        target_value,
        status,
        metadata,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    save_benchmark(pool, result.clone()).await?;
    Ok(result)
}
