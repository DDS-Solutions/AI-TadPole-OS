//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / finalize / tests
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none

use crate::agent::runner::service_traits::mission_state::{
    persist_agent_outcome, AgentOutcome, OutcomeHealth,
};
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::{EngineAgent, TokenUsage};
use crate::error::AppError;
use crate::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_finalize_run_metrics() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    let runner = AgentRunner::new(state.clone());
    let mut ctx = RunContext::default();
    ctx.safe_mode = true; // This fixture exercises accounting for a read-only run.
    ctx.identity.agent_id = "test-agent".to_string();

    // Setup mock agent
    let mut agent = EngineAgent::default();
    agent.identity.id = ctx.identity.agent_id.clone();
    state
        .registry
        .agents
        .insert(ctx.identity.agent_id.clone(), agent.clone());

    // Sync agent to DB first so create_mission doesn't fail
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
        .await
        .unwrap();

    // Create a mock mission in the DB to satisfy persistence calls
    let mission = crate::agent::mission::create_mission(
        &state.resources.pool,
        &ctx.identity.agent_id,
        "Test Mission",
        1.0,
    )
    .await
    .unwrap();
    ctx.mission_state.mission_id = mission.id;

    let usage = Some(TokenUsage {
        input_tokens: 100,
        output_tokens: 50,
        total_tokens: 150,
    });

    let result = runner.finalize_run(&ctx, "Final output", &usage).await;
    assert_eq!(result.unwrap(), "Final output");

    // Verify metrics
    let agent = state.registry.agents.get(&ctx.identity.agent_id).unwrap();
    assert_eq!(agent.economics.tokens_used, 150);
    assert!(agent.economics.cost_usd > 0.0);
    assert_eq!(agent.health.failure_count, 0);
}

#[tokio::test]
async fn branch_finalization_does_not_close_parent_mission() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    let runner = AgentRunner::new(state.clone());
    let mut parent = EngineAgent::default();
    parent.identity.id = "parent-agent".to_string();
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut parent)
        .await
        .unwrap();
    let mission = crate::agent::mission::create_mission(
        &state.resources.pool,
        "parent-agent",
        "Shared branch ledger",
        5.0,
    )
    .await
    .unwrap();
    let ctx = RunContext {
        identity: crate::agent::runner::AgentIdentity {
            agent_id: "child-agent".to_string(),
            ..Default::default()
        },
        mission_state: crate::agent::runner::MissionState {
            mission_id: mission.id.clone(),
            depth: 1,
            ..Default::default()
        },
        ..Default::default()
    };

    runner
        .finalize_mission_persistence(&ctx, "branch result", &None, 0.0)
        .await
        .unwrap();

    let persisted = crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        persisted.status,
        crate::agent::types::MissionStatus::Pending
    );
}

#[tokio::test]
async fn independently_owned_child_mission_reaches_a_terminal_status() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    let runner = AgentRunner::new(state.clone());
    let mut child = EngineAgent::default();
    child.identity.id = "independent-child".into();
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut child)
        .await
        .unwrap();
    let mission = crate::agent::mission::create_mission(
        &state.resources.pool,
        "independent-child",
        "Separate child ledger",
        1.0,
    )
    .await
    .unwrap();
    let ctx = RunContext {
        identity: crate::agent::runner::AgentIdentity {
            agent_id: "independent-child".into(),
            ..Default::default()
        },
        mission_state: crate::agent::runner::MissionState {
            mission_id: mission.id.clone(),
            depth: 1,
            ..Default::default()
        },
        safe_mode: true,
        ..Default::default()
    };
    runner
        .finalize_mission_persistence(&ctx, "Verified source report", &None, 0.0)
        .await
        .unwrap();
    let completed = crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        completed.status,
        crate::agent::types::MissionStatus::Completed
    );
    ctx.swarm_partial_failure
        .store(true, std::sync::atomic::Ordering::Release);
    runner
        .finalize_mission_persistence(&ctx, "Incomplete branch", &None, 0.0)
        .await
        .unwrap();
    let paused = crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(paused.status, crate::agent::types::MissionStatus::Paused);
}

#[tokio::test]
async fn partial_root_result_pauses_for_human_review() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    let runner = AgentRunner::new(state.clone());
    let mut parent = EngineAgent::default();
    parent.identity.id = "parent-agent".to_string();
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut parent)
        .await
        .unwrap();
    let mission = crate::agent::mission::create_mission(
        &state.resources.pool,
        "parent-agent",
        "Human review ledger",
        5.0,
    )
    .await
    .unwrap();
    let ctx = RunContext {
        identity: crate::agent::runner::AgentIdentity {
            agent_id: "parent-agent".to_string(),
            ..Default::default()
        },
        mission_state: crate::agent::runner::MissionState {
            mission_id: mission.id.clone(),
            ..Default::default()
        },
        swarm_partial_failure: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        ..Default::default()
    };

    runner
        .finalize_mission_persistence(&ctx, "partial result", &None, 0.0)
        .await
        .unwrap();

    let persisted = crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted.status, crate::agent::types::MissionStatus::Paused);
}

#[tokio::test]
async fn test_fail_mission_without_registry_agent() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    let runner = AgentRunner::new(state.clone());
    let agent_id = "missing-agent";

    let mut agent = EngineAgent::default();
    agent.identity.id = agent_id.to_string();
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
        .await
        .unwrap();

    let mission = crate::agent::mission::create_mission(
        &state.resources.pool,
        agent_id,
        "Ghost Mission",
        1.0,
    )
    .await
    .unwrap();

    // Ensure missing from in-memory registry
    state.registry.agents.remove(agent_id);

    let ctx = RunContext {
        identity: crate::agent::runner::AgentIdentity {
            agent_id: agent_id.to_string(),
            ..Default::default()
        },
        mission_state: crate::agent::runner::MissionState {
            mission_id: mission.id.clone(),
            ..Default::default()
        },
        ..Default::default()
    };

    let err = AppError::BadRequest("Execution failed".to_string());
    let result = runner.fail_mission(&ctx, &err, &None).await;
    assert!(
        result.is_ok(),
        "fail_mission must succeed even if agent is missing from registry"
    );

    // Verify mission status transitioned to Failed
    let persisted = crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted.status, crate::agent::types::MissionStatus::Failed);

    // Verify error step logged
    let steps = crate::agent::mission::get_mission_logs(&state.resources.pool, &mission.id)
        .await
        .unwrap();
    assert!(steps.iter().any(|s| s.severity == "error"));
}

#[tokio::test]
async fn test_branch_error_step_causes_parent_to_pause() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    let runner = AgentRunner::new(state.clone());
    let mut parent = EngineAgent::default();
    parent.identity.id = "root-agent".to_string();
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut parent)
        .await
        .unwrap();

    let mut child = EngineAgent::default();
    child.identity.id = "child-worker".to_string();
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut child)
        .await
        .unwrap();

    let mission = crate::agent::mission::create_mission(
        &state.resources.pool,
        "root-agent",
        "Swarm Tree with Branch Failure",
        5.0,
    )
    .await
    .unwrap();

    // Execute fail_mission on a child context (depth: 1)
    let child_ctx = RunContext {
        identity: crate::agent::runner::AgentIdentity {
            agent_id: "child-worker".to_string(),
            ..Default::default()
        },
        mission_state: crate::agent::runner::MissionState {
            mission_id: mission.id.clone(),
            depth: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let child_err = AppError::BadRequest("Child worker crashed".to_string());
    runner
        .fail_mission(&child_ctx, &child_err, &None)
        .await
        .unwrap();

    // Now root finalizes
    let root_ctx = RunContext {
        identity: crate::agent::runner::AgentIdentity {
            agent_id: "root-agent".to_string(),
            ..Default::default()
        },
        mission_state: crate::agent::runner::MissionState {
            mission_id: mission.id.clone(),
            depth: 0,
            ..Default::default()
        },
        ..Default::default()
    };

    runner
        .finalize_mission_persistence(&root_ctx, "root result", &None, 0.0)
        .await
        .unwrap();

    let persisted = crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted.status, crate::agent::types::MissionStatus::Paused);
}

#[tokio::test]
async fn test_effective_model_id_pricing() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    let runner = AgentRunner::new(state.clone());

    let mut ctx = RunContext::default();
    ctx.identity.agent_id = "pricing-agent".to_string();
    // Static configuration points to expensive Opus model
    ctx.model_config.model_id = "claude-4.8-opus".to_string();

    assert_eq!(runner.effective_model_id(&ctx), "claude-4.8-opus");

    // Resolved model is set to gpt-4o-mini (e.g. redirected or dynamically resolved)
    *ctx.resolved_model_id.lock() = Some("gpt-4o-mini".to_string());
    assert_eq!(runner.effective_model_id(&ctx), "gpt-4o-mini");

    let usage = Some(TokenUsage {
        input_tokens: 1000,
        output_tokens: 1000,
        total_tokens: 2000,
    });

    let mut agent = EngineAgent::default();
    agent.identity.id = ctx.identity.agent_id.clone();
    state
        .registry
        .agents
        .insert(ctx.identity.agent_id.clone(), agent.clone());
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
        .await
        .unwrap();

    let mission = crate::agent::mission::create_mission(
        &state.resources.pool,
        &ctx.identity.agent_id,
        "Pricing Mission",
        1.0,
    )
    .await
    .unwrap();
    ctx.mission_state.mission_id = mission.id;

    runner.finalize_run(&ctx, "output", &usage).await.unwrap();

    let updated_agent = state.registry.agents.get(&ctx.identity.agent_id).unwrap();
    assert_eq!(updated_agent.economics.tokens_used, 2000);
    // gpt-4o-mini rate: 1000 * 0.00015/1k + 1000 * 0.0006/1k = 0.00075 USD
    // (Opus rate would have been 0.090 USD)
    assert!((updated_agent.economics.cost_usd - 0.00075).abs() < 1e-6);
    assert!((updated_agent.economics.cost_usd - 0.090).abs() > 0.01);
}

#[tokio::test]
async fn test_finalize_run_resilient_to_missing_agent() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    let runner = AgentRunner::new(state.clone());
    let agent_id = "missing-agent-success";

    let mut agent = EngineAgent::default();
    agent.identity.id = agent_id.to_string();
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
        .await
        .unwrap();

    let mission = crate::agent::mission::create_mission(
        &state.resources.pool,
        agent_id,
        "Success Mission",
        1.0,
    )
    .await
    .unwrap();

    // Ensure missing from in-memory registry
    state.registry.agents.remove(agent_id);

    let ctx = RunContext {
        identity: crate::agent::runner::AgentIdentity {
            agent_id: agent_id.to_string(),
            ..Default::default()
        },
        mission_state: crate::agent::runner::MissionState {
            mission_id: mission.id.clone(),
            ..Default::default()
        },
        ..Default::default()
    };

    let result = runner.finalize_run(&ctx, "Success output", &None).await;
    assert!(
        result.is_ok(),
        "finalize_run must succeed and broadcast even if agent is missing from registry"
    );
    assert!(result.unwrap().starts_with("MISSION_REVIEW_REQUIRED"));

    let persisted = crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted.status, crate::agent::types::MissionStatus::Paused);
}

#[test]
fn test_scrub_mythos_tags_extended() {
    let input = "Hello <thinking depth=\"2\">internal</thinking> <thought>reasoning</thought> world!<|im_end|><|thought|><|endoftext|><halting_signal/>";
    let cleaned = AgentRunner::scrub_mythos_tags(input);
    assert_eq!(cleaned, "Hello internal  world!");
}
