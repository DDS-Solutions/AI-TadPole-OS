//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Oversight Verification (Approval Loop Tests)**: Orchestrates the
//! verification of the manual approval loop and security gating for
//! the Tadpole OS engine. Features **E2E Oversight Submission**:
//! simulates the full lifecycle of a tool call requiring oversight
//! (e.g., `delete_file`), from submission to the `oversight_resolvers`
//! queue to final disposition. Implements **UI Integration Mocking**:
//! validates that the state transition from "Pending" to
//! "Approved" or "Rejected" correctly unblocks the agent runner. AI
//! agents should run these tests to verify that the human-in-the-loop
//! (HITL) gate remains robust and deadlock-free during multi-agent
//! orchestration (GOV-04).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Deadlocks in the `oversight_resolvers`
//!   broadcast channel, mission context leaks during rejection, or
//!   incorrect state reporting in the `mission_history` table.
//! - **Trace Scope**: `server-rs::agent::test_oversight`

use crate::agent::runner::AgentRunner;
use crate::agent::types::ToolCallAudit;
use crate::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_e2e_oversight_approval_loop() {
    // 1. Setup AppState and Database
    let state = Arc::new(
        AppState::new()
            .await
            .expect("Failed to initialize state for oversight tests"),
    );
    let runner = AgentRunner::new(state.clone());

    let test_id = uuid::Uuid::new_v4().to_string();
    let agent_id = format!("test-agent-{}", test_id);
    let mission_id = format!("test-mission-{}", test_id);

    // Seed test data
    sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Oversight Test', 'security', 'Compliance', 'desc', 'idle', '{}')")
        .bind(&agent_id).execute(&state.resources.pool).await.unwrap();
    sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Oversight Verification', 'active')")
        .bind(&mission_id).bind(&agent_id).execute(&state.resources.pool).await.unwrap();

    // 2. Simulate a tool call requiring oversight (like delete_file)
    let tool_call = ToolCallAudit {
        id: "call-123".to_string(),
        agent_id: agent_id.clone(),
        mission_id: Some(mission_id.clone()),
        skill: "delete_file".to_string(),
        params: serde_json::json!({"filename": "critical_file.txt"}),
        department: "Compliance".to_string(),
        description: "Test deletion oversight".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // 3. Start the oversight submission in a task (it blocks until rx)
    let runner_clone = runner.clone();
    let mid_clone = mission_id.clone();
    let handle = tokio::spawn(async move {
        runner_clone
            .submit_oversight(tool_call, Some(mid_clone))
            .await
    });

    // 4. Wait for the resolver to be registered in the queue
    let mut entry_id = String::new();
    let mut resolved = false;
    for _ in 0..10 {
        if let Some(kv) = state.comms.oversight_resolvers.iter().next() {
            entry_id = kv.key().clone();
            resolved = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(resolved, "Oversight resolver should have been registered");

    // 5. Simulate the UI approval event
    if let Some((_, tx)) = state.comms.oversight_resolvers.remove(&entry_id) {
        let _ = tx.send(crate::agent::types::OversightResolution { approved: true, override_slot: None }); // Approve
    }

    // 6. Verify the result
    let approved = handle.await.unwrap().expect("submit_oversight failed");
    assert!(approved, "Oversight should have been approved");

    // 7. Test Rejection
    let tool_call_rej = ToolCallAudit {
        id: "call-456".to_string(),
        agent_id: agent_id.clone(),
        mission_id: Some(mission_id.clone()),
        skill: "delete_file".to_string(),
        params: serde_json::json!({"filename": "important.txt"}),
        department: "Compliance".to_string(),
        description: "Test rejection oversight".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let runner_clone2 = runner.clone();
    let handle_rej = tokio::spawn(async move {
        runner_clone2
            .submit_oversight(tool_call_rej, Some(mission_id))
            .await
    });

    let mut entry_id_rej = String::new();
    for _ in 0..10 {
        if let Some(kv) = state.comms.oversight_resolvers.iter().next() {
            entry_id_rej = kv.key().clone();
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    if let Some((_, tx)) = state.comms.oversight_resolvers.remove(&entry_id_rej) {
        let _ = tx.send(crate::agent::types::OversightResolution { approved: false, override_slot: None }); // Reject
    }

    let approved_rej = handle_rej.await.unwrap().expect("submit_oversight failed");
    assert!(!approved_rej, "Oversight should have been rejected");
}

#[tokio::test]
async fn test_model_failover_oversight() {
    // 1. Setup AppState and Database
    let state = Arc::new(
        AppState::new()
            .await
            .expect("Failed to initialize state for oversight tests"),
    );
    let runner = AgentRunner::new(state.clone());

    let test_id = uuid::Uuid::new_v4().to_string();
    let agent_id = format!("test-agent-{}", test_id);
    let mission_id = format!("test-mission-{}", test_id);

    // Seed agent with multiple model slots
    let mut agent = crate::agent::types::EngineAgent::default();
    agent.identity.id = agent_id.clone();
    agent.models.model = crate::agent::types::ModelConfig {
        provider: crate::agent::types::ModelProvider::Ollama,
        model_id: "default-model".to_string(),
        ..Default::default()
    };
    agent.models.planning_slot = Some(crate::agent::types::ModelConfig {
        provider: crate::agent::types::ModelProvider::Gemini,
        model_id: "planning-model".to_string(),
        ..Default::default()
    });
    agent.models.execution_slot = Some(crate::agent::types::ModelConfig {
        provider: crate::agent::types::ModelProvider::Groq,
        model_id: "execution-model".to_string(),
        ..Default::default()
    });
    agent.models.active_model_slot = Some("planning".to_string());
    state.registry.agents.insert(agent_id.clone(), agent);

    // Seed test data in SQLite
    sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Oversight Test', 'security', 'Compliance', 'desc', 'idle', '{}')")
        .bind(&agent_id).execute(&state.resources.pool).await.unwrap();
    sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Oversight Verification', 'active')")
        .bind(&mission_id).bind(&agent_id).execute(&state.resources.pool).await.unwrap();

    // 2. Simulate a failover requiring oversight
    let tool_call = ToolCallAudit {
        id: "failover-123".to_string(),
        agent_id: agent_id.clone(),
        mission_id: Some(mission_id.clone()),
        skill: "model_failover".to_string(),
        params: serde_json::json!({
            "failed_provider": "gemini",
            "failed_model": "planning-model",
            "proposed_fallback_provider": "groq",
            "proposed_fallback_model": "execution-model"
        }),
        department: "Compliance".to_string(),
        description: "Test failover oversight".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let runner_clone = runner.clone();
    let mid_clone = mission_id.clone();
    let handle = tokio::spawn(async move {
        runner_clone
            .submit_oversight_resolution(tool_call, Some(mid_clone))
            .await
    });

    // 3. Wait for the resolver to be registered
    let mut entry_id = String::new();
    let mut resolved = false;
    for _ in 0..10 {
        if let Some(kv) = state.comms.oversight_resolvers.iter().next() {
            entry_id = kv.key().clone();
            resolved = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(resolved, "Oversight resolver should have been registered");

    // 4. Simulate the UI response with slot override to "execution"
    if let Some((_, tx)) = state.comms.oversight_resolvers.remove(&entry_id) {
        let _ = tx.send(crate::agent::types::OversightResolution {
            approved: true,
            override_slot: Some("execution".to_string()),
        });
    }

    // 5. Verify the resolution results
    let resolution = handle.await.unwrap().expect("submit_oversight_resolution failed");
    assert!(resolution.approved);
    assert_eq!(resolution.override_slot, Some("execution".to_string()));
}

// Metadata: [test_oversight]

// Metadata: [test_oversight]
