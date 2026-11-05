//! @docs ARCHITECTURE:Continuity:Workflow
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / workflow_tests_crud
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

#[tokio::test]
async fn test_workflow_crud() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_db("sqlite::memory:").await?;
    let state = Arc::new(AppState::with_pool(pool).await);
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow(
            "default",
            "Test Workflow".to_string(),
            Some("Desc".to_string()),
        )
        .await?;
    assert_eq!(wf.name, "Test Workflow");

    engine
        .add_step(
            "default",
            &wf.id,
            "agent-1",
            "Step 1".to_string(),
            "Do A".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;
    engine
        .add_step(
            "default",
            &wf.id,
            "agent-2",
            "Step 2".to_string(),
            "Do B".to_string(),
            2,
            None,
            None,
            None,
        )
        .await?;

    let steps = engine.get_workflow_steps("default", &wf.id).await?;
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].step_order, 1);
    assert_eq!(steps[1].step_order, 2);

    engine.delete_workflow("default", &wf.id).await?;
    let list = engine.list_workflows("default").await?;
    assert!(list.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_workflow_step_length_validation() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    let engine = WorkflowEngine::new(Arc::clone(&state));
    let wf = engine
        .create_workflow("default", "Length Test".to_string(), None)
        .await?;

    let long_name = "a".repeat(101);
    let res_name = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            long_name,
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await;
    assert!(res_name.is_err());
    assert!(res_name
        .unwrap_err()
        .to_string()
        .contains("Step name cannot exceed 100 characters"));

    let long_prompt = "a".repeat(50001);
    let res_prompt = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Step".to_string(),
            long_prompt,
            1,
            None,
            None,
            None,
        )
        .await;
    assert!(res_prompt.is_err());
    assert!(res_prompt
        .unwrap_err()
        .to_string()
        .contains("Prompt template cannot exceed 50000 characters"));

    Ok(())
}

#[tokio::test]
async fn test_add_step_eager_validation() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    let engine = WorkflowEngine::new(Arc::clone(&state));
    let wf = engine
        .create_workflow("default", "Eager Validation WF".to_string(), None)
        .await?;

    let step_1 = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Initial Step".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;
    assert_eq!(step_1.name, "Initial Step");

    // Self dependency rejected
    let self_dep_err = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Self Dep Step".to_string(),
            "Prompt".to_string(),
            2,
            None,
            None,
            Some(vec!["Self Dep Step".to_string()]),
        )
        .await;
    assert!(self_dep_err.is_err());
    assert!(self_dep_err
        .unwrap_err()
        .to_string()
        .contains("cannot depend on itself"));

    // Missing dependency rejected
    let missing_dep_err = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Second Step".to_string(),
            "Prompt".to_string(),
            2,
            None,
            None,
            Some(vec!["non_existent_step_id".to_string()]),
        )
        .await;
    assert!(missing_dep_err.is_err());
    assert!(missing_dep_err
        .unwrap_err()
        .to_string()
        .contains("which does not exist"));

    Ok(())
}

#[test]
fn test_resolve_step_target_by_name() {
    let step_1 = WorkflowStep {
        id: "uuid-1111".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 1,
        name: "Analyze Code".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec![],
    };
    let steps = vec![step_1];

    // By ID
    assert_eq!(
        resolve_step_target("uuid-1111", &steps),
        Some("uuid-1111".to_string())
    );
    // By exact Name
    assert_eq!(
        resolve_step_target("Analyze Code", &steps),
        Some("uuid-1111".to_string())
    );
    // By sanitized context key
    assert_eq!(
        resolve_step_target("analyze_code", &steps),
        Some("uuid-1111".to_string())
    );
    // Non-matching
    assert_eq!(resolve_step_target("unknown_step", &steps), None);
}

#[tokio::test]
async fn test_workflow_version_optimistic_locking() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    state
        .governance
        .null_providers_test_mode
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let _pool = state.resources.pool.clone();
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow("default", "Locking Test".to_string(), None)
        .await?;
    let step1 = engine
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
    engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Step 2".to_string(),
            "Prompt 2".to_string(),
            2,
            None,
            None,
            Some(vec![step1.id]),
        )
        .await?;

    let state_clone = Arc::clone(&state);
    let wf_id = wf.id.clone();
    let mut rx = state.comms.event_tx.subscribe();
    tokio::spawn(async move {
        while let Ok(evt) = rx.recv().await {
            if evt.get("type").and_then(|t| t.as_str()) == Some("engine:workflow_run_started") {
                let _ = sqlx::query("UPDATE workflows SET version = 99 WHERE id = ?1")
                    .bind(&wf_id)
                    .execute(&state_clone.resources.pool)
                    .await;
                break;
            }
        }
    });

    let result = engine
        .run_workflow("default", &wf.id, serde_json::json!({}))
        .await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("configuration changed"));

    Ok(())
}

#[tokio::test]
async fn test_tenant_idor_prevention() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf_a = engine
        .create_workflow(
            "tenant-a",
            "Tenant A Workflow".to_string(),
            Some("Desc".to_string()),
        )
        .await?;

    let result_step = engine
        .add_step(
            "tenant-b",
            &wf_a.id,
            "1",
            "Step".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await;
    assert!(result_step.is_err());
    assert!(matches!(result_step.unwrap_err(), AppError::Forbidden(_)));

    let result_run = engine
        .run_workflow("tenant-b", &wf_a.id, serde_json::json!({}))
        .await;
    assert!(result_run.is_err());
    assert!(matches!(result_run.unwrap_err(), AppError::NotFound(_)));

    Ok(())
}
