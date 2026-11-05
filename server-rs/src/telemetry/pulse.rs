//! @docs ARCHITECTURE:TelemetryBridge
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / pulse
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Telemetry]`
//! - **Witness Tests**: none declared

use crate::agent::types::EngineAgent;
use crate::state::AppState;
use crate::telemetry::pulse_types::{PulseConnection, PulseNode, PulseNodeStatus, SwarmPulse};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Builds a complete SwarmPulse snapshot from the current agent registry.
pub fn build_swarm_pulse(agents: &DashMap<String, EngineAgent>, now: DateTime<Utc>) -> SwarmPulse {
    let timestamp = now.timestamp_millis() as u64;
    let mut pulse = SwarmPulse::new(timestamp);
    let mut active_missions = HashSet::new();

    // 1. Map Agents to Nodes
    for entry in agents.iter() {
        let agent = entry.value();

        // Map status string to PulseNodeStatus
        let status = match agent.health.status.as_str() {
            "running" | "active" | "thinking" | "working" => PulseNodeStatus::Busy.as_u8(),
            "failed" => PulseNodeStatus::Error.as_u8(),
            "throttled" => PulseNodeStatus::Degraded.as_u8(),
            _ => PulseNodeStatus::Idle.as_u8(),
        };

        // Calculate battery as remaining percentage of allocated budget
        let battery = if agent.economics.budget_usd > 0.0 {
            let remaining = (agent.economics.budget_usd - agent.economics.cost_usd).max(0.0);
            ((remaining / agent.economics.budget_usd) * 100.0).clamp(0.0, 100.0) as u8
        } else {
            100
        };

        // Calculate signal based on heartbeat recency
        let signal = if let Some(last_heartbeat) = agent.health.heartbeat_at {
            let latency = (now - last_heartbeat).num_seconds();
            if latency < 5 {
                100
            } else if latency < 15 {
                70
            } else if latency < 30 {
                40
            } else {
                10
            }
        } else {
            100
        };

        // Calculate progress dynamically based on agent reasoning status and turn depth
        let progress = match agent.health.status.as_str() {
            "thinking" => {
                let turn = agent.state.current_reasoning_turn.max(1) as f32;
                (turn / 4.0).clamp(0.15, 0.90)
            }
            "working" | "running" | "active" => {
                if agent.state.current_reasoning_turn > 0 {
                    (agent.state.current_reasoning_turn as f32 / 3.0).clamp(0.20, 0.95)
                } else {
                    0.50
                }
            }
            "completed" | "done" => 1.0,
            _ => 0.0,
        };

        pulse.nodes.push(PulseNode {
            id: agent.identity.id.clone(),
            x: 0.0, // Layout handled by frontend ForceGraph
            y: 0.0,
            status,
            battery,
            signal,
            progress,
        });

        // 2. Map Connections (Hierarchical Swarm + Active Mission Relationships)
        if let Some(mission) = &agent.state.active_mission {
            let parent_id_opt = mission
                .get("parent_agent_id")
                .or_else(|| mission.get("lead_agent_id"))
                .or_else(|| agent.metadata.get("parent_agent_id"))
                .and_then(|v: &serde_json::Value| v.as_str());

            let mut linked_to_parent = false;
            if let Some(parent_id) = parent_id_opt {
                if parent_id != agent.identity.id && agents.contains_key(parent_id) {
                    pulse.edges.push(PulseConnection {
                        source: parent_id.to_string(),
                        target: agent.identity.id.clone(),
                    });
                    linked_to_parent = true;
                }
            }

            if !linked_to_parent {
                if let Some(mission_id) = mission
                    .get("id")
                    .and_then(|v: &serde_json::Value| v.as_str())
                {
                    active_missions.insert(mission_id.to_string());
                    pulse.edges.push(PulseConnection {
                        source: agent.identity.id.clone(),
                        target: mission_id.to_string(),
                    });
                }
            }
        }
    }

    // 3. Synthesize Mission Nodes (Central Anchors)
    for mission_id in active_missions {
        pulse.nodes.push(PulseNode {
            id: mission_id,
            x: 0.0,
            y: 0.0,
            status: PulseNodeStatus::MissionHub.as_u8(),
            battery: 100,
            signal: 100,
            progress: 0.0,
        });
    }

    pulse
}

/// Launches the high-speed pulse loop (100ms interval).
pub async fn spawn_pulse_loop(
    state: Arc<AppState>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let mut interval = interval(Duration::from_millis(100));

    tracing::info!("💓 [Telemetry] Swarm Pulse Loop (MsgPack) started (100ms interval)");

    loop {
        tokio::select! {
            result = shutdown_rx.changed() => {
                if result.is_err() || *shutdown_rx.borrow() {
                    tracing::info!("🛑 Swarm Pulse Loop shutting down gracefully.");
                    break;
                }
            }
            _ = interval.tick() => {
                let now = Utc::now();
                let pulse = build_swarm_pulse(&state.registry.agents, now);

                let node_count = pulse.nodes.len();
                let edge_count = pulse.edges.len();

                // Periodic diagnostic logging (every ~5 seconds at 10Hz)
                static TICK_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let ticks = TICK_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                if ticks.is_multiple_of(50) {
                    tracing::info!(
                        "💓 [Telemetry] Swarm Pulse: {} nodes, {} edges broadcast",
                        node_count,
                        edge_count
                    );
                }

                let _ = state.comms.pulse_tx.send(Arc::new(pulse));
            }
        }
    }
}
