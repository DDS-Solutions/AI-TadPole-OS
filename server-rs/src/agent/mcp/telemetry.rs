//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Telemetry & Metrics
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Non-sentinel rolling latency calculation accurately tracking 0 ms durations.
//! - `[Structural]` Unified event schema for engine tool pulse and observability.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

use super::gate::sanitize_reflected;
use super::types::McpToolStats;

/// Real-time metrics collector and pulse emitter for MCP and native tool executions.
#[derive(Clone)]
pub struct McpTelemetry {
    stats: Arc<DashMap<String, McpToolStats>>,
    event_tx: broadcast::Sender<serde_json::Value>,
}

impl McpTelemetry {
    pub fn new(event_tx: broadcast::Sender<serde_json::Value>) -> Self {
        Self {
            stats: Arc::new(DashMap::new()),
            event_tx,
        }
    }

    pub fn stats_map(&self) -> &Arc<DashMap<String, McpToolStats>> {
        &self.stats
    }

    pub fn get_tool_stats(&self, tool_name: &str) -> Option<McpToolStats> {
        self.stats.get(tool_name).map(|s| s.clone())
    }

    /// Updates execution count, error count, and rolling average latency without 0 ms sentinel bugs.
    pub fn update_stats(&self, tool_name: &str, is_success: bool, latency_ms: u64) {
        let mut entry = self.stats.entry(tool_name.to_string()).or_default();
        entry.invocations += 1;
        if is_success {
            entry.success_count += 1;
        } else {
            entry.failure_count += 1;
        }

        if entry.invocations == 1 {
            entry.avg_latency_ms = latency_ms;
        } else {
            let prev_count = (entry.invocations - 1) as u128;
            let current_latency = latency_ms as u128;
            let avg = entry.avg_latency_ms as u128;
            let new_avg = ((avg * prev_count) + current_latency) / (entry.invocations as u128);
            entry.avg_latency_ms = new_avg as u64;
        }
    }

    /// Records invocation stats and emits a telemetry pulse over broadcast channel.
    pub fn record_invocation(&self, tool_name: &str, is_success: bool, latency_ms: u64) {
        self.update_stats(tool_name, is_success, latency_ms);
        self.emit_pulse(tool_name, is_success, latency_ms);
    }

    /// Emits a structured telemetry event over the system broadcast bus.
    pub fn emit_pulse(&self, tool_name: &str, is_success: bool, latency_ms: u64) {
        let pulse = serde_json::json!({
            "type": "engine:mcp_pulse",
            "tool": sanitize_reflected(tool_name, 128),
            "status": if is_success { "success" } else { "error" },
            "latency": latency_ms
        });
        let _ = self.event_tx.send(pulse);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_avg_latency_with_zero_latency() {
        let (tx, _) = broadcast::channel(16);
        let telemetry = McpTelemetry::new(tx);

        // Invoc 1: 0 ms
        telemetry.update_stats("test_tool", true, 0);
        assert_eq!(
            telemetry
                .get_tool_stats("test_tool")
                .unwrap()
                .avg_latency_ms,
            0
        );
        assert_eq!(
            telemetry.get_tool_stats("test_tool").unwrap().invocations,
            1
        );
        assert_eq!(
            telemetry.get_tool_stats("test_tool").unwrap().success_count,
            1
        );
        assert_eq!(
            telemetry.get_tool_stats("test_tool").unwrap().failure_count,
            0
        );

        // Invoc 2: 20 ms -> expected (0 * 1 + 20) / 2 = 10 ms
        telemetry.update_stats("test_tool", true, 20);
        assert_eq!(
            telemetry
                .get_tool_stats("test_tool")
                .unwrap()
                .avg_latency_ms,
            10
        );
        assert_eq!(
            telemetry.get_tool_stats("test_tool").unwrap().invocations,
            2
        );

        // Invoc 3: error
        telemetry.update_stats("test_tool", false, 30);
        assert_eq!(
            telemetry.get_tool_stats("test_tool").unwrap().failure_count,
            1
        );
        assert_eq!(
            telemetry.get_tool_stats("test_tool").unwrap().success_count,
            2
        );
        assert_eq!(
            telemetry.get_tool_stats("test_tool").unwrap().invocations,
            3
        );
    }
}
