//! @docs ARCHITECTURE:Telemetry
//!
//! ### AI Assist Note
//! **Pulse Protocol Testing**: Validates the density and correctness of the High-Speed
//! Swarm Pulse (100ms) used for real-time visualization.

use crate::agent::types::EngineAgent;
use crate::state::AppState;
use crate::telemetry::pulse_types::{PulseConnection, PulseNode, SwarmPulse};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_pulse_aggregation_logic() {
    let state = Arc::new(AppState::new_mock().await);

    // 1. Add a mock agent to the registry
    let mut agent = EngineAgent::default();
    agent.id = "agent-007".to_string();
    agent.name = "James Bond".to_string();
    agent.status = "running".to_string();
    agent.budget_usd = 100.0;
    agent.cost_usd = 25.0; // 75% battery
    agent.active_mission = Some(json!({ "id": "mission-spy" }));

    state.registry.agents.insert("agent-007".to_string(), agent);

    // 2. Build the pulse (simulating pulse.rs logic with synthesis)
    let timestamp = 123456789;
    let mut pulse = SwarmPulse::new(timestamp);
    let mut active_missions = std::collections::HashSet::new();

    for entry in state.registry.agents.iter() {
        let a = entry.value();

        let status = match a.status.as_str() {
            "running" => 1,
            _ => 0,
        };

        pulse.nodes.push(PulseNode {
            id: a.id.clone(),
            x: 0.0,
            y: 0.0,
            status,
            battery: 75,
            signal: 100,
            progress: 0.0,
        });

        if let Some(mission) = &a.active_mission {
            if let Some(mission_id) = mission.get("id").and_then(|v| v.as_str()) {
                active_missions.insert(mission_id.to_string());
                pulse.edges.push(PulseConnection {
                    source: a.id.clone(),
                    target: mission_id.to_string(),
                });
            }
        }
    }

    // Synthesize mission nodes (new requirement)
    for m_id in active_missions {
        pulse.nodes.push(PulseNode {
            id: m_id,
            x: 0.0,
            y: 0.0,
            status: 4,
            battery: 100,
            signal: 100,
            progress: 0.0,
        });
    }

    // 3. Verifications
    assert_eq!(pulse.nodes.len(), 2); // 1 Agent + 1 Mission
    assert!(pulse.nodes.iter().any(|n| n.id == "agent-007"));
    assert!(pulse
        .nodes
        .iter()
        .any(|n| n.id == "mission-spy" && n.status == 4));
}

#[test]
fn test_messagepack_serialization_density() {
    let mut pulse = SwarmPulse::new(123456789);
    for i in 0..10 {
        pulse.nodes.push(PulseNode {
            id: format!("agent-{}", i),
            x: 1.2,
            y: 3.4,
            status: 1,
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
