//! Oversight Verification — Security gate and manual approval tests
//!
//! @docs ARCHITECTURE:Agent

use crate::agent::runner::AgentRunner;
use crate::agent::types::ToolCall;
use crate::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_e2e_oversight_approval_loop() {
    // 1. Setup AppState and Database
    let state = Arc::new(AppState::new().await.expect("Failed to initialize state for oversight tests"));
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
    let tool_call = ToolCall {
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
        let _ = tx.send(true); // Approve
    }

    // 6. Verify the result
    let approved = handle.await.unwrap();
    assert!(approved, "Oversight should have been approved");

    // 7. Test Rejection
    let tool_call_rej = ToolCall {
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
        let _ = tx.send(false); // Reject
    }

    let approved_rej = handle_rej.await.unwrap();
    assert!(!approved_rej, "Oversight should have been rejected");
}
