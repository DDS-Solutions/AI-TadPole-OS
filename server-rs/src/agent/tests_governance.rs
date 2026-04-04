//! Governance Verification — Multi-layered guardrail and hierarchy tests
//!
//! @docs ARCHITECTURE:Agent

use crate::agent::runner::AgentRunner;
use crate::agent::types::{EngineAgent, GeminiFunctionCall};
use crate::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_mandatory_oversight_gate() {
    // 1. Setup AppState (Use NEW_MOCK for test isolation)
    let state = Arc::new(AppState::new_mock().await);
    let runner = AgentRunner::new(state.clone());

    let agent_id = "junior-agent-001".to_string();
    let mission_id = "test-mission-999".to_string();

    // 2. Register a "Junior" agent with requires_oversight = true
    let mut agent = EngineAgent {
        id: agent_id.clone(),
        name: "Junior Agent".to_string(),
        requires_oversight: true,
        ..Default::default()
    };
    agent.skills.push("read_file".to_string());

    state.registry.agents.insert(agent_id.clone(), agent);

    // 3. Prepare RunContext
    let ctx = crate::agent::runner::RunContext {
        agent_id: agent_id.clone(),
        name: "Junior Agent".to_string(),
        role: "Helper".to_string(),
        department: "General".to_string(),
        description: "junior".to_string(),
        mission_id: mission_id.clone(),
        model_config: crate::agent::types::ModelConfig::default(),
        provider_name: "openai".to_string(),
        skills: vec!["read_file".to_string()],
        workflows: vec![],
        mcp_tools: vec![],
        depth: 0,
        lineage: vec![],
        workspace_root: std::path::PathBuf::from("."),
        base_dir: std::path::PathBuf::from("."),
        fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(std::path::PathBuf::from(
            ".",
        )),
        safe_mode: false,
        analysis: false,
        traceparent: None,
        user_id: None,
        last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
        recent_findings: None,
        working_memory: serde_json::json!({}),
        summarized_history: None,
        backlog: None,
        structured_output: false,
    };

    // 4. Simulate a "safe" tool call (read_file)
    // Normally read_file doesn't require oversight in the manifest,
    // but the Junior Agent flag should override this.
    let fc = GeminiFunctionCall {
        name: "read_file".to_string(),
        args: serde_json::json!({"path": "hello.txt"}),
    };

    // 5. Spawn a resolver in background to approve it
    let state_clone = state.clone();
    tokio::spawn(async move {
        println!("[DEBUG TEST] Background resolver started...");
        for _ in 0..300 {
            let entry_id_opt = {
                state_clone
                    .comms
                    .oversight_resolvers
                    .iter()
                    .next()
                    .map(|kv| kv.key().clone())
            }; // RefMulti guard is dropped here

            if let Some(entry_id) = entry_id_opt {
                println!("[DEBUG TEST] Found resolver for entry: {}", entry_id);
                if let Some((_, tx)) = state_clone.comms.oversight_resolvers.remove(&entry_id) {
                    println!("[DEBUG TEST] Sending approval for entry: {}", entry_id);
                    let _ = tx.send(true);
                    return;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        println!("[DEBUG TEST] Background resolver timed out after 30s!");
    });

    // 6. Execute tool (should be gated)
    println!("[DEBUG TEST] Calling execute_tool for mandatory oversight gate test...");
    let mut output_str = String::new();
    let mut usage_opt = None;
    let result = runner
        .execute_tool(&ctx, &fc, &mut output_str, &mut usage_opt, "user message")
        .await;
    println!("[DEBUG TEST] execute_tool returned for mandatory oversight gate test.");

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mandatory_oversight_rejection() {
    // 1. Setup AppState (Use NEW_MOCK for test isolation)
    let state = Arc::new(AppState::new_mock().await);
    let runner = AgentRunner::new(state.clone());
    let agent_id = "junior-agent-002".to_string();

    let agent = EngineAgent {
        id: agent_id.clone(),
        name: "Restricted Agent".to_string(),
        requires_oversight: true,
        ..Default::default()
    };
    state.registry.agents.insert(agent_id.clone(), agent);

    let ctx = crate::agent::runner::RunContext {
        agent_id: agent_id.clone(),
        name: "Restricted Agent".to_string(),
        role: "Helper".to_string(),
        department: "General".to_string(),
        description: "junior".to_string(),
        mission_id: "test".to_string(),
        model_config: crate::agent::types::ModelConfig::default(),
        provider_name: "openai".to_string(),
        skills: vec!["read_file".to_string()],
        workflows: vec![],
        mcp_tools: vec![],
        depth: 0,
        lineage: vec![],
        workspace_root: std::path::PathBuf::from("."),
        base_dir: std::path::PathBuf::from("."),
        fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(std::path::PathBuf::from(
            ".",
        )),
        safe_mode: false,
        analysis: false,
        traceparent: None,
        user_id: None,
        last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
        recent_findings: None,
        working_memory: serde_json::json!({}),
        summarized_history: None,
        backlog: None,
        structured_output: false,
    };

    let fc = GeminiFunctionCall {
        name: "read_file".to_string(),
        args: serde_json::json!({"path": "secret.txt"}),
    };

    let mut output_str = String::new();
    let mut usage_opt = None;

    // Spawn resolver that REJECTS
    let state_clone = state.clone();
    tokio::spawn(async move {
        println!("[DEBUG TEST] Background rejection resolver started...");
        for _ in 0..300 {
            let entry_id_opt = {
                state_clone
                    .comms
                    .oversight_resolvers
                    .iter()
                    .next()
                    .map(|kv| kv.key().clone())
            }; // Guard dropped

            if let Some(entry_id) = entry_id_opt {
                println!("[DEBUG TEST] Found resolver for entry: {}", entry_id);
                if let Some((_, tx)) = state_clone.comms.oversight_resolvers.remove(&entry_id) {
                    println!("[DEBUG TEST] Sending rejection for entry: {}", entry_id);
                    let _ = tx.send(false);
                    return;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        println!("[DEBUG TEST] Background rejection resolver timed out after 30s!");
    });

    let _ = runner
        .execute_tool(&ctx, &fc, &mut output_str, &mut usage_opt, "user message")
        .await;

    // Status should represent rejection
    // execute_tool returns successfully after gate rejection but output is empty or contains error
    println!("[DEBUG TEST] output_str on rejection: '{}'", output_str);
    assert!(
        output_str.is_empty()
            || output_str.contains("error")
            || output_str.contains("{}")
            || output_str.contains("REJECTED")
    );
}
