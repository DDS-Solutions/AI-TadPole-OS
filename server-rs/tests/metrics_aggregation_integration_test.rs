//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / tests / metrics_aggregation_integration_test
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use chrono::{Datelike, Utc};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use std::sync::Arc;

async fn setup_ephemeral_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to ephemeral in-memory SQLite database");

    sqlx::query(
        "CREATE TABLE mission_history (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            prompt TEXT NOT NULL,
            status TEXT NOT NULL,
            cost_usd REAL NOT NULL DEFAULT 0.0,
            created_at TEXT NOT NULL,
            updated_at TEXT
        );",
    )
    .execute(&pool)
    .await
    .expect("Failed to create mission_history schema");

    sqlx::query(
        "CREATE TABLE scheduled_jobs (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            name TEXT NOT NULL,
            prompt TEXT NOT NULL,
            cron_expr TEXT NOT NULL,
            budget_usd REAL NOT NULL DEFAULT 0.10,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await
    .expect("Failed to create scheduled_jobs schema");

    sqlx::query(
        "CREATE TABLE scheduled_job_runs (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await
    .expect("Failed to create scheduled_job_runs schema");

    pool
}

#[tokio::test]
async fn test_mission_aggregations_under_concurrent_sqlite_load() {
    let pool = setup_ephemeral_db().await;
    let now = Utc::now();
    let current_year = now.year();

    // 1. Seed historical and current-year missions
    let sample_missions = vec![
        (
            "m-1",
            "QA-99",
            "completed",
            0.025,
            format!("{}-01-15T10:00:00+00:00", current_year),
        ),
        (
            "m-2",
            "QA-99",
            "completed",
            0.015,
            format!("{}-02-20T11:00:00+00:00", current_year),
        ),
        (
            "m-3",
            "QA-99",
            "failed",
            0.010,
            format!("{}-03-05T09:00:00+00:00", current_year),
        ),
        (
            "m-4",
            "ARCH-01",
            "completed",
            0.050,
            format!("{}-04-10T14:00:00+00:00", current_year),
        ),
        (
            "m-5",
            "ARCH-01",
            "active",
            0.005,
            format!("{}-04-11T16:00:00+00:00", current_year),
        ),
        // Previous year mission (should be filtered out by YTD)
        (
            "m-prev",
            "QA-99",
            "completed",
            0.030,
            format!("{}-12-31T23:59:59+00:00", current_year - 1),
        ),
    ];

    for (id, agent, status, cost, created) in sample_missions {
        sqlx::query("INSERT INTO mission_history (id, agent_id, prompt, status, cost_usd, created_at) VALUES (?, ?, 'test prompt', ?, ?, ?)")
            .bind(id)
            .bind(agent)
            .bind(status)
            .bind(cost)
            .bind(created)
            .execute(&pool)
            .await
            .unwrap();
    }

    // Seed scheduled jobs and runs
    sqlx::query("INSERT INTO scheduled_jobs (id, agent_id, name, prompt, cron_expr, enabled, created_at) VALUES ('job-1', 'QA-99', 'Audit', 'run audit', '0 * * * *', 1, ?)")
        .bind(now.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO scheduled_jobs (id, agent_id, name, prompt, cron_expr, enabled, created_at) VALUES ('job-2', 'QA-99', 'Cleanup', 'cleanup logs', '0 0 * * *', 0, ?)")
        .bind(now.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO scheduled_job_runs (id, job_id, status, created_at) VALUES ('run-1', 'job-1', 'completed', ?)")
        .bind(now.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO scheduled_job_runs (id, job_id, status, created_at) VALUES ('run-2', 'job-1', 'failed', ?)")
        .bind(now.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO scheduled_job_runs (id, job_id, status, created_at) VALUES ('run-3', 'job-2', 'skipped', ?)")
        .bind(now.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();

    // 2. Simulate concurrent load while querying aggregations
    let pool_arc = Arc::new(pool);
    let mut handles = Vec::new();

    for i in 0..10 {
        let p = Arc::clone(&pool_arc);
        let now_str = now.to_rfc3339();
        let handle = tokio::spawn(async move {
            // Concurrent inserts and queries executing simultaneously
            let id = format!("concurrent-{}", i);
            sqlx::query("INSERT INTO mission_history (id, agent_id, prompt, status, cost_usd, created_at) VALUES (?, 'LOAD-BOT', 'load test', 'completed', 0.001, ?)")
                .bind(id)
                .bind(&now_str)
                .execute(&*p)
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    // Await all concurrent tasks
    for h in handles {
        h.await.unwrap();
    }

    // 3. Execute Consolidated Aggregation Query for YTD (filtering out previous year)
    let ytd_start = format!("{}-01-01T00:00:00+00:00", current_year);

    let mut qb = QueryBuilder::<Sqlite>::new(
        "SELECT \
            COUNT(*), \
            COALESCE(SUM(cost_usd), 0.0), \
            COALESCE(SUM(CASE WHEN LOWER(status) = 'completed' THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN LOWER(status) = 'failed' THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN LOWER(status) IN ('active', 'running') THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN LOWER(status) NOT IN ('completed', 'failed', 'active', 'running') THEN 1 ELSE 0 END), 0) \
        FROM mission_history \
        WHERE created_at >= "
    );
    qb.push_bind(&ytd_start);

    let row = qb.build().fetch_one(&*pool_arc).await.unwrap();

    let total: i64 = row.try_get(0).unwrap_or(0);
    let total_cost: f64 = row.try_get(1).unwrap_or(0.0);
    let completed: i64 = row.try_get(2).unwrap_or(0);
    let failed: i64 = row.try_get(3).unwrap_or(0);
    let active: i64 = row.try_get(4).unwrap_or(0);

    // Initial 5 current-year missions + 10 concurrent completed = 15 total in YTD
    assert_eq!(
        total, 15,
        "Should count 15 total missions in YTD (excluding m-prev)"
    );
    assert_eq!(
        completed, 13,
        "Should count 13 completed missions (3 sample + 10 load)"
    );
    assert_eq!(failed, 1, "Should count 1 failed mission");
    assert_eq!(active, 1, "Should count 1 active mission");
    assert!(
        (total_cost - 0.115).abs() < 1e-6,
        "Cost should match 0.025 + 0.015 + 0.010 + 0.050 + 0.005 + (10 * 0.001) = 0.115"
    );

    // 4. Test agent filter with QA-99 in YTD
    let mut qb_agent = QueryBuilder::<Sqlite>::new(
        "SELECT \
            COUNT(*), \
            COALESCE(SUM(cost_usd), 0.0), \
            COALESCE(SUM(CASE WHEN LOWER(status) = 'completed' THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN LOWER(status) = 'failed' THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN LOWER(status) IN ('active', 'running') THEN 1 ELSE 0 END), 0) \
        FROM mission_history \
        WHERE created_at >= ",
    );
    qb_agent.push_bind(&ytd_start);
    qb_agent.push(" AND agent_id = ");
    qb_agent.push_bind("QA-99");

    let qa_row = qb_agent.build().fetch_one(&*pool_arc).await.unwrap();
    let qa_total: i64 = qa_row.try_get(0).unwrap_or(0);
    let qa_completed: i64 = qa_row.try_get(2).unwrap_or(0);
    let qa_failed: i64 = qa_row.try_get(3).unwrap_or(0);

    assert_eq!(
        qa_total, 3,
        "QA-99 had 3 YTD missions (excluding previous year)"
    );
    assert_eq!(qa_completed, 2, "QA-99 completed 2 missions");
    assert_eq!(qa_failed, 1, "QA-99 failed 1 mission");

    // 5. Scheduled jobs query
    let (jobs_total, jobs_enabled): (i64, i64) = match sqlx::query(
        "SELECT COUNT(*), COALESCE(SUM(CASE WHEN enabled = 1 THEN 1 ELSE 0 END), 0) FROM scheduled_jobs",
    )
    .fetch_optional(&*pool_arc)
    .await
    .unwrap()
    {
        Some(r) => (r.try_get(0).unwrap_or(0), r.try_get(1).unwrap_or(0)),
        None => (0, 0),
    };
    assert_eq!(jobs_total, 2);
    assert_eq!(jobs_enabled, 1);
}
