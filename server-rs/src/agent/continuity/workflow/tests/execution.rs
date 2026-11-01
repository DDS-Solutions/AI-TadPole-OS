//! @docs ARCHITECTURE:Continuity:Workflow
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / workflow_tests_execution
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
async fn test_workflow_run_guard() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_db("sqlite::memory:").await?;
    let state = Arc::new(AppState::with_pool(pool.clone()).await);

    let run_id = "test-run-id".to_string();

    sqlx::query("INSERT INTO workflows (id, name, enabled, created_at, updated_at) VALUES ('wf-1', 'Test Workflow', 1, ?1, ?2)")
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now())
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO workflow_runs (id, workflow_id, started_at, status, current_step, context) VALUES (?1, 'wf-1', '2026-06-08', 'running', 0, '{}')")
        .bind(&run_id)
        .execute(&pool)
        .await?;

    {
        let _guard = WorkflowRunGuard::new(run_id.clone(), "default".to_string(), state);
    }

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let run_row = sqlx::query("SELECT status FROM workflow_runs WHERE id = ?1")
        .bind(&run_id)
        .fetch_one(&pool)
        .await?;
    assert_eq!(run_row.get::<String, _>("status"), "failed");

    Ok(())
}

#[test]
fn test_evaluate_condition() {
    let context = serde_json::json!({
        "auditor_step": {
            "approved": true,
            "score": 85,
            "findings": ["vuln1", "vuln2"]
        },
        "status": "ready"
    });

    let cond1 = RuleCondition {
        path: "auditor_step.approved".to_string(),
        operator: ConditionOperator::Equals,
        value: serde_json::json!(true),
    };
    assert!(evaluate_condition(&context, &cond1));

    let cond2 = RuleCondition {
        path: "status".to_string(),
        operator: ConditionOperator::Equals,
        value: serde_json::json!("ready"),
    };
    assert!(evaluate_condition(&context, &cond2));

    let cond3 = RuleCondition {
        path: "status".to_string(),
        operator: ConditionOperator::NotEquals,
        value: serde_json::json!("failed"),
    };
    assert!(evaluate_condition(&context, &cond3));

    let cond4 = RuleCondition {
        path: "auditor_step.findings".to_string(),
        operator: ConditionOperator::Contains,
        value: serde_json::json!("vuln1"),
    };
    assert!(evaluate_condition(&context, &cond4));

    let cond5 = RuleCondition {
        path: "status".to_string(),
        operator: ConditionOperator::Contains,
        value: serde_json::json!("ea"),
    };
    assert!(evaluate_condition(&context, &cond5));

    let cond6 = RuleCondition {
        path: "auditor_step.score".to_string(),
        operator: ConditionOperator::GreaterThan,
        value: serde_json::json!(80),
    };
    assert!(evaluate_condition(&context, &cond6));
    assert!(!evaluate_condition(
        &context,
        &RuleCondition {
            path: "auditor_step.score".to_string(),
            operator: ConditionOperator::GreaterThan,
            value: serde_json::json!(90),
        }
    ));

    let cond7 = RuleCondition {
        path: "auditor_step.score".to_string(),
        operator: ConditionOperator::LessThan,
        value: serde_json::json!(90),
    };
    assert!(evaluate_condition(&context, &cond7));
}

#[tokio::test]
async fn test_declarative_branching_no_match() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    state
        .governance
        .null_providers_test_mode
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let pool = state.resources.pool.clone();
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow(
            "default",
            "Branch Workflow".to_string(),
            Some("Desc".to_string()),
        )
        .await?;

    let step1 = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Step 1".to_string(),
            "Prompt 1".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;

    let step2 = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Step 2".to_string(),
            "Prompt 2".to_string(),
            2,
            None,
            None,
            Some(vec![step1.id.clone()]),
        )
        .await?;

    let routing_cfg = RoutingConfig {
        default_next: None,
        rules: vec![RoutingRule {
            condition: RuleCondition {
                path: "step_1".to_string(),
                operator: ConditionOperator::Equals,
                value: serde_json::json!("nonexistent_value"),
            },
            next_step: step1.id.clone(),
            reset_steps: vec![step1.id.clone()],
        }],
        max_resets: None,
    };

    let step2_cfg = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: Some(routing_cfg),
        fan_out: None,
        tournament: None,
        safe_mode: None,
        analysis: None,
    };

    sqlx::query("UPDATE workflow_steps SET config = ?1 WHERE id = ?2")
        .bind(serde_json::to_string(&step2_cfg)?)
        .bind(&step2.id)
        .execute(&pool)
        .await?;

    let initial_context = serde_json::json!({});
    let (output, run_id) = engine
        .run_workflow("default", &wf.id, initial_context)
        .await?;

    assert!(output.contains("[DEGRADED: test_mode]"));

    let step_runs = sqlx::query("SELECT * FROM workflow_step_runs WHERE run_id = ?1")
        .bind(&run_id)
        .fetch_all(&pool)
        .await?;
    assert_eq!(step_runs.len(), 2);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_fan_out() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    state
        .governance
        .null_providers_test_mode
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let pool = state.resources.pool.clone();
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow(
            "default",
            "Fanout Workflow".to_string(),
            Some("Desc".to_string()),
        )
        .await?;

    let step = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Fanout Step".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;

    let fanout_cfg = FanOutConfig {
        array_path: "files".to_string(),
        item_placeholder: "file".to_string(),
        agent_id: "1".to_string(),
        prompt_template: "Process {{file}}".to_string(),
        fail_strategy: FanOutFailStrategy::FailFast,
    };

    let step_cfg = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: None,
        fan_out: Some(fanout_cfg),
        tournament: None,
        safe_mode: None,
        analysis: None,
    };

    sqlx::query("UPDATE workflow_steps SET config = ?1 WHERE id = ?2")
        .bind(serde_json::to_string(&step_cfg)?)
        .bind(&step.id)
        .execute(&pool)
        .await?;

    let initial_context = serde_json::json!({
        "files": ["main.rs", "lib.rs"]
    });

    let (output, run_id) = engine
        .run_workflow("default", &wf.id, initial_context)
        .await?;

    let parsed_outputs: Vec<String> = serde_json::from_str(&output)?;
    assert_eq!(parsed_outputs.len(), 2);
    assert!(parsed_outputs[0].contains("[DEGRADED: test_mode]"));

    let step_run = sqlx::query("SELECT metadata FROM workflow_step_runs WHERE run_id = ?1 LIMIT 1")
        .bind(&run_id)
        .fetch_one(&pool)
        .await?;
    let metadata_str: Option<String> = step_run.get("metadata");
    assert!(metadata_str.is_some());
    let metadata: serde_json::Value = serde_json::from_str(&metadata_str.unwrap())?;
    assert_eq!(metadata["type"], "fan_out");
    assert_eq!(metadata["items_count"], 2);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_model_tournament() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    state
        .governance
        .null_providers_test_mode
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let pool = state.resources.pool.clone();
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow(
            "default",
            "Tournament Workflow".to_string(),
            Some("Desc".to_string()),
        )
        .await?;

    let step = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Tournament Step".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;

    let tournament_cfg = TournamentConfig {
        candidates: vec![
            TournamentCandidate {
                agent_id: "1".to_string(),
                prompt_template: "Prompt A".to_string(),
            },
            TournamentCandidate {
                agent_id: "1".to_string(),
                prompt_template: "Prompt B".to_string(),
            },
        ],
        judge_agent_id: "1".to_string(),
        judge_prompt_template:
            "Compare candidate 0: {{candidate_0}} and candidate 1: {{candidate_1}}".to_string(),
        min_successful: None,
    };

    let step_cfg = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: None,
        fan_out: None,
        tournament: Some(tournament_cfg),
        safe_mode: None,
        analysis: None,
    };

    sqlx::query("UPDATE workflow_steps SET config = ?1 WHERE id = ?2")
        .bind(serde_json::to_string(&step_cfg)?)
        .bind(&step.id)
        .execute(&pool)
        .await?;

    let (output, run_id) = engine
        .run_workflow("default", &wf.id, serde_json::json!({}))
        .await?;

    assert!(output.contains("[DEGRADED: test_mode]"));

    let step_run = sqlx::query("SELECT metadata FROM workflow_step_runs WHERE run_id = ?1 LIMIT 1")
        .bind(&run_id)
        .fetch_one(&pool)
        .await?;
    let metadata_str: Option<String> = step_run.get("metadata");
    assert!(metadata_str.is_some());
    let metadata: serde_json::Value = serde_json::from_str(&metadata_str.unwrap())?;
    assert_eq!(metadata["type"], "tournament");
    assert_eq!(metadata["candidates"].as_array().unwrap().len(), 2);
    assert_eq!(metadata["judge"]["agent_id"], "1");

    Ok(())
}

#[tokio::test]
async fn test_fan_out_array_size_limit() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new_mock().await);
    state
        .governance
        .null_providers_test_mode
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let pool = state.resources.pool.clone();
    let engine = WorkflowEngine::new(Arc::clone(&state));

    let wf = engine
        .create_workflow(
            "default",
            "Fanout Limit Workflow".to_string(),
            Some("Desc".to_string()),
        )
        .await?;

    let step = engine
        .add_step(
            "default",
            &wf.id,
            "1",
            "Fanout Step".to_string(),
            "Prompt".to_string(),
            1,
            None,
            None,
            None,
        )
        .await?;

    let fanout_cfg = FanOutConfig {
        array_path: "items".to_string(),
        item_placeholder: "item".to_string(),
        agent_id: "1".to_string(),
        prompt_template: "Process {{item}}".to_string(),
        fail_strategy: FanOutFailStrategy::FailFast,
    };

    let step_cfg = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: None,
        fan_out: Some(fanout_cfg),
        tournament: None,
        safe_mode: None,
        analysis: None,
    };

    sqlx::query("UPDATE workflow_steps SET config = ?1 WHERE id = ?2")
        .bind(serde_json::to_string(&step_cfg)?)
        .bind(&step.id)
        .execute(&pool)
        .await?;

    let large_array: Vec<String> = (0..51).map(|i| format!("item_{}", i)).collect();
    let initial_context = serde_json::json!({
        "items": large_array
    });

    let result = engine
        .run_workflow("default", &wf.id, initial_context)
        .await;
    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("exceeds limit of 50"));

    Ok(())
}

#[tokio::test]
async fn test_terminal_step_output_resolution() {
    let step_a = WorkflowStep {
        id: "step-a-id".to_string(),
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
        id: "step-b-id".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 2,
        name: "Step B".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["step-a-id".to_string()],
    };
    let step_c = WorkflowStep {
        id: "step-c-id".to_string(),
        workflow_id: "wf".to_string(),
        agent_id: "1".to_string(),
        step_order: 3,
        name: "Step C".to_string(),
        prompt_template: "".to_string(),
        config: None,
        max_retries: 3,
        backoff_factor_secs: 2,
        depends_on: vec!["step-a-id".to_string()],
    };

    let steps = vec![step_a, step_b, step_c];
    let terminals = get_terminal_steps(&steps);
    // B and C are terminals (nothing depends on them), A is not
    assert_eq!(terminals.len(), 2);
    let terminal_ids: HashSet<String> = terminals.iter().map(|s| s.id.clone()).collect();
    assert!(terminal_ids.contains("step-b-id"));
    assert!(terminal_ids.contains("step-c-id"));
    assert!(!terminal_ids.contains("step-a-id"));
}

#[tokio::test]
async fn test_routing_plus_tournament_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let step_cfg = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: Some(RoutingConfig {
            default_next: None,
            rules: vec![],
            max_resets: None,
        }),
        fan_out: None,
        tournament: Some(TournamentConfig {
            candidates: vec![],
            judge_agent_id: "1".to_string(),
            judge_prompt_template: "".to_string(),
            min_successful: None,
        }),
        safe_mode: None,
        analysis: None,
    };
    assert!(step_cfg.validate().is_err());
    Ok(())
}

#[tokio::test]
async fn test_tournament_candidate_cap_and_placeholder_validation() {
    // 0 candidates rejected
    let empty_tourney = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: None,
        fan_out: None,
        tournament: Some(TournamentConfig {
            candidates: vec![],
            judge_agent_id: "1".to_string(),
            judge_prompt_template: "".to_string(),
            min_successful: None,
        }),
        safe_mode: None,
        analysis: None,
    };
    assert!(empty_tourney.validate().is_err());

    // >20 candidates rejected
    let mut too_many = Vec::new();
    for i in 0..21 {
        too_many.push(TournamentCandidate {
            agent_id: "1".to_string(),
            prompt_template: format!("Candidate {}", i),
        });
    }
    let overflow_tourney = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: None,
        fan_out: None,
        tournament: Some(TournamentConfig {
            candidates: too_many,
            judge_agent_id: "1".to_string(),
            judge_prompt_template: "".to_string(),
            min_successful: None,
        }),
        safe_mode: None,
        analysis: None,
    };
    assert!(overflow_tourney.validate().is_err());

    // Out-of-bounds placeholder rejected (e.g. {{candidate_5}} when only 2 candidates exist)
    let bad_placeholder = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: None,
        fan_out: None,
        tournament: Some(TournamentConfig {
            candidates: vec![
                TournamentCandidate {
                    agent_id: "1".to_string(),
                    prompt_template: "Prompt 0".to_string(),
                },
                TournamentCandidate {
                    agent_id: "1".to_string(),
                    prompt_template: "Prompt 1".to_string(),
                },
            ],
            judge_agent_id: "1".to_string(),
            judge_prompt_template: "Judge {{candidate_0}} vs {{candidate_5}}".to_string(),
            min_successful: None,
        }),
        safe_mode: None,
        analysis: None,
    };
    assert!(bad_placeholder.validate().is_err());

    // Valid placeholders pass
    let valid_tourney = StepConfig {
        context_keys: None,
        context_json_path: None,
        context_max_chars: None,
        routing: None,
        fan_out: None,
        tournament: Some(TournamentConfig {
            candidates: vec![
                TournamentCandidate {
                    agent_id: "1".to_string(),
                    prompt_template: "Prompt 0".to_string(),
                },
                TournamentCandidate {
                    agent_id: "1".to_string(),
                    prompt_template: "Prompt 1".to_string(),
                },
            ],
            judge_agent_id: "1".to_string(),
            judge_prompt_template: "Judge {{candidate_0}} vs {{candidate_1}}".to_string(),
            min_successful: Some(1),
        }),
        safe_mode: None,
        analysis: None,
    };
    assert!(valid_tourney.validate().is_ok());
}
