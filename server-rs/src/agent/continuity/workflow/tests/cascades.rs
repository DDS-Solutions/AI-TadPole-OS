//! @docs ARCHITECTURE:Continuity:Workflow
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / workflow_tests_cascades
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - [Structural] Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::super::engine::WorkflowEngine;
use super::super::helpers::*;
use super::super::types::*;
use crate::db::init_db;
use crate::error::AppError;
use crate::state::AppState;
use sqlx::Row;
use std::collections::HashSet;
use std::sync::Arc;

#[test]
fn test_backoff_math_safety() {
    let factor = 2_i64;
    let attempt = 100;
    let capped_attempt = attempt.min(10);
    let delay_secs = factor.max(1) * 2_i64.saturating_pow(capped_attempt as u32 - 1);
    assert_eq!(delay_secs, 1024);
}

#[test]
fn test_downstream_dependency_resets() {
    let step_a = WorkflowStep {
        id: "A".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 1,
        name: "Step A".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec![],
    };
    let step_b = WorkflowStep {
        id: "B".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 2,
        name: "Step B".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["A".to_string()],
    };
    let step_c = WorkflowStep {
        id: "C".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 3,
        name: "Step C".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["B".to_string()],
    };
    let step_d = WorkflowStep {
        id: "D".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 2,
        name: "Step D".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["A".to_string()],
    };
    let step_e = WorkflowStep {
        id: "E".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 1,
        name: "Step E".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec![],
    };

    let steps = vec![step_a, step_b, step_c, step_d, step_e];
    let reset_targets = vec!["A".to_string()];
    let downstream = get_all_downstream(&reset_targets, &steps);

    assert!(downstream.contains("B"));
    assert!(downstream.contains("C"));
    assert!(downstream.contains("D"));
    assert!(!downstream.contains("E"));
    assert!(!downstream.contains("A"));
}

#[test]
fn test_dag_cycle_detection() {
    let step_a = WorkflowStep {
        id: "A".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 1,
        name: "Step A".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["C".to_string()],
    };
    let step_b = WorkflowStep {
        id: "B".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 2,
        name: "Step B".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["A".to_string()],
    };
    let step_c = WorkflowStep {
        id: "C".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 3,
        name: "Step C".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["B".to_string()],
    };
    let steps = vec![step_a, step_b, step_c];
    let result = detect_dependency_cycle(&steps);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cycle detected"));
}

#[tokio::test]
async fn test_workflow_cancellation() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow("default", "Cancel Test".to_string(), None)
        .await?;
    engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Step 1".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;

    let run_id = "test-run-123";
    sqlx::query("INSERT INTO workflow_runs (id, workflow_id, tenant_id, started_at, status, current_step, context) VALUES (?1, ?2, 'default', ?3, 'running', 0, '')")
        .bind(run_id)
        .bind(&wf.id)
        .bind(chrono::Utc::now())
        .execute(&state.resources.pool)
        .await?;

    let token = tokio_util::sync::CancellationToken::new();
    engine.active_runs.insert(run_id.to_string(), token.clone());

    assert!(!token.is_cancelled());
    {
        let _guard = WorkflowRunGuard::new(
            run_id.to_string(),
            "default".to_string(),
            Arc::clone(&state),
        );
        engine.cancel_run("default", run_id).await?;
        assert!(token.is_cancelled());
    }

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let status: String = sqlx::query_scalar("SELECT status FROM workflow_runs WHERE id = ?1")
        .bind(run_id)
        .fetch_one(&state.resources.pool)
        .await?;
    assert_eq!(status, "cancelled");

    Ok(())
}

#[test]
fn test_workflow_dependency_cycle_duplicates() {
    let step_a = WorkflowStep {
        id: "A".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 1,
        name: "Step A".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec![],
    };
    let step_b = WorkflowStep {
        id: "B".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 2,
        name: "Step B".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["A".to_string(), "A".to_string()],
    };
    let steps = vec![step_a, step_b];
    let result = detect_dependency_cycle(&steps);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_workflow_step_failure_event() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    state
        .governance
        .null_providers_test_mode
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let mut rx = state.comms.event_tx.subscribe();

    let engine = WorkflowEngine::new(Arc::clone(&state));
    let wf = engine
        .create_workflow("default", "Failure Event Test".to_string(), None)
        .await?;
    let step = engine
        .add_step(
            "default",
            &wf.id,
            "non-existent-agent-id",
            "Failing Step".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;

    let result = engine
        .run_workflow("default", &wf.id, serde_json::json!({}))
        .await;
    assert!(result.is_err());

    let mut found_failure_event = false;
    while let Ok(event) = rx.try_recv() {
        if event["type"] == "engine:workflow_step_failed" {
            assert_eq!(event["data"]["step_id"], step.id);
            found_failure_event = true;
            break;
        }
    }
    assert!(
        found_failure_event,
        "Should have emitted engine:workflow_step_failed event"
    );

    Ok(())
}

#[tokio::test]
async fn test_step_run_failed_on_config_error() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    let pool = state.resources.pool.clone();
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow("default", "Config Error Test".to_string(), None)
        .await?;
    let step = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Failing Step Config".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;

    let invalid_cfg = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: Some(RoutingConfig {
            default_next: None,
            rules: vec![],
            max_resets: None,
        }),
        fan_out: Some(FanOutConfig {
            array_path: "".to_string(),
            item_placeholder: "".to_string(),
            agent_id: "".to_string(),
            prompt_template: "".to_string(),
            fail_strategy: FanOutFailStrategy::FailFast,
        }),
        tournament: None,
        safe_mode: None,
        analysis: None,
    };

    sqlx::query("UPDATE workflow_steps SET config = ?1 WHERE id = ?2")
        .bind(serde_json::to_string(&invalid_cfg)?)
        .bind(&step.id)
        .execute(&pool)
        .await?;

    let result = engine
        .run_workflow("default", &wf.id, serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("cannot mix routing, fan_out, or tournament modes"));

    Ok(())
}

#[tokio::test]
async fn test_cancel_run_does_not_overwrite_failed_step() -> Result<(), Box<dyn std::error::Error>>
{
    let state = Arc::new(AppState::new_mock().await);
    let pool = state.resources.pool.clone();
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow("default", "Overwriting Test".to_string(), None)
        .await?;
    let step = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Step 1".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;

    let run_id = "test-cancel-overwrite-run";
    sqlx::query("INSERT INTO workflow_runs (id, workflow_id, tenant_id, started_at, status, current_step, context) VALUES (?1, ?2, 'default', ?3, 'running', 0, '')")
        .bind(run_id)
        .bind(&wf.id)
        .bind(chrono::Utc::now())
        .execute(&pool)
        .await?;

    let step_run_id = "test-cancel-overwrite-step-run";
    sqlx::query("INSERT INTO workflow_step_runs (id, run_id, step_id, started_at, status, output_text) VALUES (?1, ?2, ?3, ?4, 'failed', 'some error')")
        .bind(step_run_id)
        .bind(run_id)
        .bind(&step.id)
        .bind(chrono::Utc::now())
        .execute(&pool)
        .await?;

    let token = tokio_util::sync::CancellationToken::new();
    engine.active_runs.insert(run_id.to_string(), token.clone());

    engine.cancel_run("default", run_id).await?;

    let step_run_status: String =
        sqlx::query_scalar("SELECT status FROM workflow_step_runs WHERE id = ?1")
            .bind(step_run_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(step_run_status, "failed");

    Ok(())
}

#[tokio::test]
async fn test_failure_metadata_preservation() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    let pool = state.resources.pool.clone();
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow("default", "Forensics Test".to_string(), None)
        .await?;
    let step = engine
        .add_step(
            "default",
            &wf.id,
            "non-existent-agent-id",
            "Failing Forensics Step".to_string(),
            "Prompt".to_string(),
            1,
            Some(0), // 0 retries = 1 attempt
            None,
            None,
        )
        .await?;

    let result = engine
        .run_workflow("default", &wf.id, serde_json::json!({}))
        .await;
    assert!(result.is_err());

    let step_run = sqlx::query(
        "SELECT metadata, status, output_text FROM workflow_step_runs WHERE step_id = ?1",
    )
    .bind(&step.id)
    .fetch_one(&pool)
    .await?;

    let status: String = step_run.get("status");
    assert_eq!(status, "failed");

    let metadata_str: String = step_run.get("metadata");
    assert_ne!(metadata_str, "{}");
    let metadata: serde_json::Value = serde_json::from_str(&metadata_str)?;
    assert_eq!(metadata["type"], "standard");
    assert_eq!(metadata["status"], "failed");
    assert!(metadata["attempts"].as_array().is_some());
    assert!(!metadata["attempts"].as_array().unwrap().is_empty());

    Ok(())
}
