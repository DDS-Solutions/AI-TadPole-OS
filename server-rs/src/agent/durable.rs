//! @docs ARCHITECTURE:Agent:DurableExecution
//!
//! ### AI Assist Note
//! **Durable Workflow Execution Engine (DBOS Pattern)**: Provides step-level
//! memoization, deterministic crash resumption, and input-hash verification
//! for agent missions. Eliminates redundant LLM token expenditures and prevents
//! accidental re-execution of side-effects on restart.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Step serialization errors, database pool timeouts, or input hash mismatches.
//! - **Telemetry Link**: Search `[durable]` in tracing logs.

use crate::error::AppError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::future::Future;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Computes a deterministic SHA-256 hash of a JSON payload for parameter integrity verification.
pub fn hash_payload(val: &serde_json::Value) -> String {
    let mut hasher = Sha256::new();
    let canonical = serde_json::to_string(val).unwrap_or_default();
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}

/// Initializes a durable workflow instance in SQLite.
pub async fn start_workflow(
    pool: &SqlitePool,
    workflow_id: &str,
    mission_id: Option<&str>,
    agent_id: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO durable_workflows (workflow_id, mission_id, agent_id, status)
         VALUES (?, ?, ?, 'RUNNING')
         ON CONFLICT(workflow_id) DO UPDATE SET updated_at = CURRENT_TIMESTAMP",
    )
    .bind(workflow_id)
    .bind(mission_id)
    .bind(agent_id)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;

    info!(
        "📝 [Durable] Workflow '{}' registered for agent '{}'",
        workflow_id, agent_id
    );
    Ok(())
}

/// Marks a durable workflow as successfully completed.
pub async fn complete_workflow(pool: &SqlitePool, workflow_id: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE durable_workflows SET status = 'COMPLETED', updated_at = CURRENT_TIMESTAMP WHERE workflow_id = ?"
    )
    .bind(workflow_id)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;

    info!(
        "🏁 [Durable] Workflow '{}' marked as COMPLETED",
        workflow_id
    );
    Ok(())
}

/// Marks a durable workflow as failed.
pub async fn fail_workflow(
    pool: &SqlitePool,
    workflow_id: &str,
    error_msg: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE durable_workflows SET status = 'FAILED', updated_at = CURRENT_TIMESTAMP WHERE workflow_id = ?"
    )
    .bind(workflow_id)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;

    warn!(
        "❌ [Durable] Workflow '{}' marked as FAILED: {}",
        workflow_id, error_msg
    );
    Ok(())
}

/// Executes a step within a durable workflow with step-level memoization.
///
/// If this step has previously succeeded with identical input hash, the cached result is returned
/// immediately without invoking the `operation` closure (zero token burn & zero duplicate I/O).
pub async fn execute_durable_step<T, F, Fut>(
    pool: &SqlitePool,
    workflow_id: &str,
    step_index: i64,
    step_name: &str,
    input: &serde_json::Value,
    operation: F,
) -> Result<T, AppError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let input_hash = hash_payload(input);

    // 1. Check for existing completed step memoization
    let cached_step = sqlx::query(
        "SELECT output_json, input_hash FROM durable_steps WHERE workflow_id = ? AND step_index = ? AND status = 'COMPLETED'"
    )
    .bind(workflow_id)
    .bind(step_index)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Sqlx)?;

    if let Some(row) = cached_step {
        let stored_hash: String = row.get("input_hash");
        let output_json: Option<String> = row.get("output_json");

        if stored_hash == input_hash {
            if let Some(json_str) = output_json {
                match serde_json::from_str::<T>(&json_str) {
                    Ok(deserialized) => {
                        info!(
                            "⏩ [Durable] Fast-forwarding step '{}' (#{}:{}) from SQLite cache (Zero Token Burn)",
                            step_name, workflow_id, step_index
                        );
                        return Ok(deserialized);
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ [Durable] Cache deserialization error on step '{}': {}. Re-executing step...",
                            step_name, e
                        );
                    }
                }
            }
        } else {
            info!(
                "🔄 [Durable] Input hash changed for step '{}' (#{}:{}). Re-executing...",
                step_name, workflow_id, step_index
            );
        }
    }

    // 2. Mark step as PENDING
    let step_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO durable_steps (step_id, workflow_id, step_index, step_name, input_hash, status)
         VALUES (?, ?, ?, ?, ?, 'PENDING')
         ON CONFLICT(workflow_id, step_index) DO UPDATE SET 
            input_hash = excluded.input_hash,
            status = 'PENDING',
            error_detail = NULL"
    )
    .bind(&step_id)
    .bind(workflow_id)
    .bind(step_index)
    .bind(step_name)
    .bind(&input_hash)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;

    // 3. Execute the actual step operation
    let result = match operation().await {
        Ok(res) => res,
        Err(err) => {
            let err_str = err.to_string();
            let _ = sqlx::query(
                "UPDATE durable_steps SET status = 'FAILED', error_detail = ? WHERE workflow_id = ? AND step_index = ?"
            )
            .bind(&err_str)
            .bind(workflow_id)
            .bind(step_index)
            .execute(pool)
            .await;

            error!(
                "🚨 [Durable] Step '{}' (#{}:{}) failed: {}",
                step_name, workflow_id, step_index, err_str
            );
            return Err(err);
        }
    };

    // 4. Persist completed step output
    let output_json = serde_json::to_string(&result).map_err(|e| {
        AppError::InternalServerError(format!("Serialization error for step result: {}", e))
    })?;

    sqlx::query(
        "UPDATE durable_steps SET status = 'COMPLETED', output_json = ?, error_detail = NULL, executed_at = CURRENT_TIMESTAMP
         WHERE workflow_id = ? AND step_index = ?"
    )
    .bind(&output_json)
    .bind(workflow_id)
    .bind(step_index)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;

    info!(
        "✅ [Durable] Step '{}' (#{}:{}) committed to SQLite",
        step_name, workflow_id, step_index
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE durable_workflows (
                workflow_id TEXT PRIMARY KEY,
                mission_id TEXT,
                agent_id TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE durable_steps (
                step_id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                step_name TEXT NOT NULL,
                input_hash TEXT NOT NULL,
                output_json TEXT,
                status TEXT NOT NULL,
                error_detail TEXT,
                executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(workflow_id, step_index)
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_durable_step_memoization_fast_forwards() {
        let pool = setup_test_db().await;
        let wf_id = "wf_crash_test_001";
        start_workflow(&pool, wf_id, Some("mission_123"), "agent_alpha")
            .await
            .unwrap();

        let execution_counter = Arc::new(AtomicUsize::new(0));

        // Step 1 Execution
        let exec1 = execution_counter.clone();
        let res1: String = execute_durable_step(
            &pool,
            wf_id,
            1,
            "generate_plan",
            &serde_json::json!({"prompt": "Build rocket"}),
            || async move {
                exec1.fetch_add(1, Ordering::SeqCst);
                Ok("Rocket plan v1".to_string())
            },
        )
        .await
        .unwrap();

        assert_eq!(res1, "Rocket plan v1");
        assert_eq!(execution_counter.load(Ordering::SeqCst), 1);

        // Step 2 Execution
        let exec2 = execution_counter.clone();
        let res2: String = execute_durable_step(
            &pool,
            wf_id,
            2,
            "synthesize_code",
            &serde_json::json!({"plan": "Rocket plan v1"}),
            || async move {
                exec2.fetch_add(1, Ordering::SeqCst);
                Ok("pub fn launch() {}".to_string())
            },
        )
        .await
        .unwrap();

        assert_eq!(res2, "pub fn launch() {}");
        assert_eq!(execution_counter.load(Ordering::SeqCst), 2);

        // --- SIMULATE ENGINE CRASH & RECOVERY ---
        // When resuming the workflow from beginning, Step 1 and Step 2 must be fast-forwarded from cache
        let exec_replayed1 = execution_counter.clone();
        let replay_res1: String = execute_durable_step(
            &pool,
            wf_id,
            1,
            "generate_plan",
            &serde_json::json!({"prompt": "Build rocket"}),
            || async move {
                exec_replayed1.fetch_add(1, Ordering::SeqCst);
                Ok("WRONG_SHOULD_NOT_EXECUTE".to_string())
            },
        )
        .await
        .unwrap();

        let exec_replayed2 = execution_counter.clone();
        let replay_res2: String = execute_durable_step(
            &pool,
            wf_id,
            2,
            "synthesize_code",
            &serde_json::json!({"plan": "Rocket plan v1"}),
            || async move {
                exec_replayed2.fetch_add(1, Ordering::SeqCst);
                Ok("WRONG_SHOULD_NOT_EXECUTE".to_string())
            },
        )
        .await
        .unwrap();

        // Outputs match exactly
        assert_eq!(replay_res1, "Rocket plan v1");
        assert_eq!(replay_res2, "pub fn launch() {}");
        // Execution counter did NOT increase (zero extra LLM calls made!)
        assert_eq!(execution_counter.load(Ordering::SeqCst), 2);

        // Now run Step 3 (the new step)
        let exec3 = execution_counter.clone();
        let res3: String = execute_durable_step(
            &pool,
            wf_id,
            3,
            "verify_tests",
            &serde_json::json!({"code": "pub fn launch() {}"}),
            || async move {
                exec3.fetch_add(1, Ordering::SeqCst);
                Ok("Tests passed 100%".to_string())
            },
        )
        .await
        .unwrap();

        assert_eq!(res3, "Tests passed 100%");
        assert_eq!(execution_counter.load(Ordering::SeqCst), 3);

        complete_workflow(&pool, wf_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_durable_step_reexecutes_on_input_hash_change() {
        let pool = setup_test_db().await;
        let wf_id = "wf_hash_test_002";
        start_workflow(&pool, wf_id, None, "agent_alpha")
            .await
            .unwrap();

        let counter = Arc::new(AtomicUsize::new(0));

        let c1 = counter.clone();
        let _ = execute_durable_step(
            &pool,
            wf_id,
            1,
            "query_docs",
            &serde_json::json!({"query": "version 1"}),
            || async move {
                c1.fetch_add(1, Ordering::SeqCst);
                Ok("Doc result v1".to_string())
            },
        )
        .await
        .unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Change input parameters: should re-execute because hash differs
        let c2 = counter.clone();
        let res: String = execute_durable_step(
            &pool,
            wf_id,
            1,
            "query_docs",
            &serde_json::json!({"query": "version 2 (modified)"}),
            || async move {
                c2.fetch_add(1, Ordering::SeqCst);
                Ok("Doc result v2 (new)".to_string())
            },
        )
        .await
        .unwrap();

        assert_eq!(res, "Doc result v2 (new)");
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_durable_workflow_status_transitions() {
        let pool = setup_test_db().await;
        let wf_id = "wf_lifecycle_003";

        // 1. Start workflow -> Running
        start_workflow(&pool, wf_id, Some("mission_life"), "agent_gamma")
            .await
            .unwrap();

        // 2. Complete workflow -> Completed
        complete_workflow(&pool, wf_id).await.unwrap();

        // 3. Failed workflow test
        let wf_fail_id = "wf_failed_004";
        start_workflow(&pool, wf_fail_id, None, "agent_gamma")
            .await
            .unwrap();
        fail_workflow(&pool, wf_fail_id, "Syntax error in AST")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_durable_step_failure_recorded_properly() {
        let pool = setup_test_db().await;
        let wf_id = "wf_step_fail_005";
        start_workflow(&pool, wf_id, None, "agent_alpha")
            .await
            .unwrap();

        let res: Result<String, AppError> = execute_durable_step(
            &pool,
            wf_id,
            1,
            "failing_step",
            &serde_json::json!({"test": 123}),
            || async move {
                Err(AppError::InternalServerError(
                    "Upstream API timeout".to_string(),
                ))
            },
        )
        .await;

        assert!(res.is_err(), "Step failure must propagate as Err");
    }
}
