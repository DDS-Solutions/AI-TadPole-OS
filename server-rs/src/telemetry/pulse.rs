//! @docs ARCHITECTURE:TelemetryBridge
//!
//! ### AI Assist Note
//! **Swarm Pulse (High-Speed Telemetry)**: Orchestrates the
//! real-time aggregation of agent states into a **MessagePack (MsgPack)**
//! binary pulse for sub-millisecond visualization. Features a
//! **100ms Pulse Loop**: periodically broadcasts `SwarmPulse` objects
//! containing node status, battery (budget), and signal (heartbeat)
//! metrics. Implements **Binary Pulse Consolidation**: encoding
//! happens in the WebSocket handler to minimize double-encoding
//! overhead across multiple concurrent client visualizers (PULS-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: High CPU/Memory overhead during pulse
//!   generation for large swarms (>100 agents), MessagePack
//!   deserialization failures in the frontend, or stale signal
//!   metrics due to heartbeat latency spikes.
//! - **Trace Scope**: `server-rs::telemetry::pulse`

use crate::state::AppState;
use crate::telemetry::pulse_types::{PulseConnection, PulseNode, SwarmPulse};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Launches the high-speed pulse loop (100ms interval).
pub async fn spawn_pulse_loop(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_millis(100));

    tracing::info!("💓 [Telemetry] Swarm Pulse Loop (MsgPack) started (100ms interval)");

    loop {
        interval.tick().await;

        // 1. Build the pulse from the current registry state
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        let mut pulse = SwarmPulse::new(timestamp);
        let mut active_missions = HashSet::new();

        // 2. Map Agents to Nodes
        for entry in state.registry.agents.iter() {
            let agent = entry.value();

            // Map status string to u8 for the pulse
            // Agent status: (idle | running | throttled | failed)
            // Pulse status: 0: idle, 1: busy, 2: error, 3: degraded
            let status = match agent.status.as_str() {
                "running" => 1,
                "failed" => 2,
                "throttled" => 3,
                _ => 0, // idle
            };

            // Map battery (budget used percentage)
            let battery = if agent.budget_usd > 0.0 {
                let remaining = (agent.budget_usd - agent.cost_usd).max(0.0);
                ((remaining / agent.budget_usd) * 100.0) as u8
            } else {
                100
            };

            // Calculate signal based on heartbeat latency
            let signal = if let Some(last_heartbeat) = agent.heartbeat_at {
                let latency = (chrono::Utc::now() - last_heartbeat).num_seconds();
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

            pulse.nodes.push(PulseNode {
                id: agent.id.clone(),
                x: 0.0, // Layout handled by frontend ForceGraph
                y: 0.0,
                status,
                battery,
                signal,
                progress: 0.0,
            });

            // 3. Map Connections (Active Mission Relationships)
            if let Some(mission) = &agent.active_mission {
                if let Some(mission_id) = mission.get("id").and_then(|v| v.as_str()) {
                    active_missions.insert(mission_id.to_string());
                    pulse.edges.push(PulseConnection {
                        source: agent.id.clone(),
                        target: mission_id.to_string(),
                    });
                }
            }
        }

        // 4. Synthesize Mission Nodes (Central Anchors)
        // This ensures the UI visualizer has valid targets for mission edges.
        for mission_id in active_missions {
            pulse.nodes.push(PulseNode {
                id: mission_id,
                x: 0.0,
                y: 0.0,
                status: 4, // Status 4 designated for 'Mission Hub'
                battery: 100,
                signal: 100,
                progress: 0.0,
            });
        }

        // 5. Broadcast the pulse
        // Note: Broadcasts the Arc<SwarmPulse>. Encoding happens in the WS handler
        // to avoid double-encoding overhead if multiple clients are listening.
        let _ = state.comms.pulse_tx.send(Arc::new(pulse));
    }
}
