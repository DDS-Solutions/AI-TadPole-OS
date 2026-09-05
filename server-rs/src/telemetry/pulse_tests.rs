//! @docs ARCHITECTURE:Observability
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / pulse_tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::EngineAgent;
use crate::state::AppState;
use crate::telemetry::pulse::build_swarm_pulse;
use crate::telemetry::pulse_types::{PulseNode, PulseNodeStatus, SwarmPulse};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_pulse_aggregation_logic() {
    let state = Arc::new(AppState::new_minimal_mock().await);

    // 1. Add a mock agent to the registry
    let agent = EngineAgent {
        identity: crate::agent::types::AgentIdentity {
            id: "agent-007".to_string(),
            name: "James Bond".to_string(),
            ..Default::default()
        },
        health: crate::agent::types::AgentHealth {
            status: "running".to_string(),
            ..Default::default()
        },
        economics: crate::agent::types::AgentEconomics {
            budget_usd: 100.0,
            cost_usd: 25.0, // 75% battery
            ..Default::default()
        },
        state: crate::agent::types::AgentState {
            active_mission: Some(json!({ "id": "mission-spy" })),
            ..Default::default()
        },
        ..Default::default()
    };

    state.registry.agents.insert("agent-007".to_string(), agent);

    // 2. Build the pulse by calling production build_swarm_pulse directly
    let now = Utc::now();
    let pulse = build_swarm_pulse(&state.registry.agents, now);

    // 3. Verifications
    let agent_node = pulse.nodes.iter().find(|n| n.id == "agent-007");
    assert!(agent_node.is_some(), "agent-007 must exist in pulse nodes");
    let agent_node = agent_node.unwrap();
    assert_eq!(agent_node.status, PulseNodeStatus::Busy.as_u8());
    assert_eq!(agent_node.battery, 75);

    let mission_node = pulse.nodes.iter().find(|n| n.id == "mission-spy");
    assert!(
        mission_node.is_some(),
        "Synthesized mission node must exist"
    );
    let mission_node = mission_node.unwrap();
    assert_eq!(mission_node.status, PulseNodeStatus::MissionHub.as_u8());

    let edge = pulse
        .edges
        .iter()
        .find(|e| e.source == "agent-007" && e.target == "mission-spy");
    assert!(
        edge.is_some(),
        "Edge connecting agent to mission must exist"
    );
}

#[test]
fn test_messagepack_serialization_density() {
    let mut pulse = SwarmPulse::new(123456789);
    for i in 0..10 {
        pulse.nodes.push(PulseNode {
            id: format!("agent-{}", i),
            x: 1.2,
            y: 3.4,
            status: PulseNodeStatus::Busy.as_u8(),
            battery: 80,
            signal: 100,
            progress: 0.5,
        });
    }

    // Binary MessagePack serialization
    let binary = rmp_serde::to_vec(&pulse).expect("Failed to serialize to MessagePack");

    // JSON serialization for comparison
    let json_ver = serde_json::to_string(&pulse).expect("Failed to serialize to JSON");

    println!("MsgPack Size: {} bytes", binary.len());
    println!("JSON Size: {} bytes", json_ver.len());

    assert!(
        binary.len() < json_ver.len(),
        "MessagePack should be more dense than JSON"
    );
}

#[tokio::test]
async fn test_pulse_dynamic_progress_and_hierarchical_edges() {
    let state = Arc::new(AppState::new_minimal_mock().await);

    // 1. Add Lead Agent and Specialist Agent
    let lead_agent = EngineAgent {
        identity: crate::agent::types::AgentIdentity {
            id: "lead-alpha".to_string(),
            name: "Lead Alpha".to_string(),
            ..Default::default()
        },
        health: crate::agent::types::AgentHealth {
            status: "thinking".to_string(),
            ..Default::default()
        },
        state: crate::agent::types::AgentState {
            active_mission: Some(json!({ "id": "mission-alpha" })),
            current_reasoning_turn: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    let specialist_agent = EngineAgent {
        identity: crate::agent::types::AgentIdentity {
            id: "sec-auditor".to_string(),
            name: "Security Auditor".to_string(),
            ..Default::default()
        },
        health: crate::agent::types::AgentHealth {
            status: "working".to_string(),
            ..Default::default()
        },
        state: crate::agent::types::AgentState {
            active_mission: Some(json!({ "id": "mission-alpha", "parent_agent_id": "lead-alpha" })),
            current_reasoning_turn: 1,
            ..Default::default()
        },
        ..Default::default()
    };

    state
        .registry
        .agents
        .insert("lead-alpha".to_string(), lead_agent);
    state
        .registry
        .agents
        .insert("sec-auditor".to_string(), specialist_agent);

    // 2. Build pulse by calling production build_swarm_pulse directly
    let now = Utc::now();
    let pulse = build_swarm_pulse(&state.registry.agents, now);

    // 3. Verification
    let lead_node = pulse.nodes.iter().find(|n| n.id == "lead-alpha").unwrap();
    assert!(
        lead_node.progress >= 0.15 && lead_node.progress <= 0.90,
        "Thinking node should have dynamic progress"
    );

    let sec_node = pulse.nodes.iter().find(|n| n.id == "sec-auditor").unwrap();
    assert!(
        sec_node.progress >= 0.20 && sec_node.progress <= 0.95,
        "Working node should have dynamic progress"
    );

    // Check hierarchical edge
    let hierarchical_edge = pulse
        .edges
        .iter()
        .find(|e| e.source == "lead-alpha" && e.target == "sec-auditor");
    assert!(
        hierarchical_edge.is_some(),
        "Pulse must include direct hierarchical parent-to-child recruitment edge"
    );
}
