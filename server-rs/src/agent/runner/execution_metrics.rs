//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / execution_metrics
//! - **Primary Entrypoints**: `ExecutionMetrics`, `ExecutionMetricsSnapshot`
//!
//! ### AI Assist Note
//! - Implements AI-Tadpole-OS-inspired live per-mission execution metrics rollup.
//! - Thread-safe, lock-free atomic counters for tokens, costs, tool calls, and summarizations.
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Lock-free atomic recording on hot execution paths.
//! - `[Behavioral]` High summarization frequency (>= 3) triggers an operational warning indicator.
//!   - enforced_by: `test_execution_metrics_lifecycle`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_execution_metrics_lifecycle`

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

/// Per-mission aggregate live execution metrics.
#[derive(Debug)]
pub struct ExecutionMetrics {
    total_turns: AtomicU32,
    tool_calls_attempted: AtomicU32,
    tool_calls_failed: AtomicU32,
    cached_reads_hit: AtomicU32,
    files_modified_count: AtomicU32,
    total_input_tokens: AtomicU64,
    total_output_tokens: AtomicU64,
    peak_context_tokens: AtomicU64,
    total_cost_micro_usd: AtomicU64,
    total_summarizations: AtomicU32,
    total_sub_agents: AtomicU32,
    start_time: parking_lot::Mutex<Instant>,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            total_turns: AtomicU32::new(0),
            tool_calls_attempted: AtomicU32::new(0),
            tool_calls_failed: AtomicU32::new(0),
            cached_reads_hit: AtomicU32::new(0),
            files_modified_count: AtomicU32::new(0),
            total_input_tokens: AtomicU64::new(0),
            total_output_tokens: AtomicU64::new(0),
            peak_context_tokens: AtomicU64::new(0),
            total_cost_micro_usd: AtomicU64::new(0),
            total_summarizations: AtomicU32::new(0),
            total_sub_agents: AtomicU32::new(0),
            start_time: parking_lot::Mutex::new(Instant::now()),
        }
    }
}

/// Serializable snapshot of aggregate execution metrics for telemetry, UI, and audit logs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionMetricsSnapshot {
    pub mission_id: String,
    pub agent_id: String,
    pub total_turns: u32,
    pub tool_calls_attempted: u32,
    pub tool_calls_failed: u32,
    pub cached_reads_hit: u32,
    pub files_modified_count: u32,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub peak_context_tokens: u64,
    pub total_cost_usd: f64,
    pub total_summarizations: u32,
    pub total_sub_agents: u32,
    pub elapsed_ms: u64,
    pub excessive_summarization_warning: bool,
}

impl ExecutionMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn record_turn(&self) {
        self.total_turns.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_tool_attempt(&self) {
        self.tool_calls_attempted.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_tool_failure(&self) {
        self.tool_calls_failed.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_cached_read(&self) {
        self.cached_reads_hit.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_file_modified(&self) {
        self.files_modified_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_tokens(&self, input: u64, output: u64, context_size: u64) {
        self.total_input_tokens.fetch_add(input, Ordering::Relaxed);
        self.total_output_tokens
            .fetch_add(output, Ordering::Relaxed);

        // Update peak context tokens using CAS loop
        let mut current_peak = self.peak_context_tokens.load(Ordering::Relaxed);
        while context_size > current_peak {
            match self.peak_context_tokens.compare_exchange_weak(
                current_peak,
                context_size,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    #[inline]
    pub fn record_cost(&self, cost_usd: f64) {
        if cost_usd > 0.0 {
            let micro_usd = (cost_usd * 1_000_000.0).round() as u64;
            self.total_cost_micro_usd
                .fetch_add(micro_usd, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn record_summarization(&self) {
        self.total_summarizations.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_sub_agent(&self) {
        self.total_sub_agents.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn should_warn_excessive_summarization(&self) -> bool {
        self.total_summarizations.load(Ordering::Relaxed) >= 3
    }

    pub fn snapshot(&self, mission_id: &str, agent_id: &str) -> ExecutionMetricsSnapshot {
        let elapsed_ms = self.start_time.lock().elapsed().as_millis() as u64;
        let total_summarizations = self.total_summarizations.load(Ordering::Relaxed);
        let total_cost_micro = self.total_cost_micro_usd.load(Ordering::Relaxed);

        ExecutionMetricsSnapshot {
            mission_id: mission_id.to_string(),
            agent_id: agent_id.to_string(),
            total_turns: self.total_turns.load(Ordering::Relaxed),
            tool_calls_attempted: self.tool_calls_attempted.load(Ordering::Relaxed),
            tool_calls_failed: self.tool_calls_failed.load(Ordering::Relaxed),
            cached_reads_hit: self.cached_reads_hit.load(Ordering::Relaxed),
            files_modified_count: self.files_modified_count.load(Ordering::Relaxed),
            total_input_tokens: self.total_input_tokens.load(Ordering::Relaxed),
            total_output_tokens: self.total_output_tokens.load(Ordering::Relaxed),
            peak_context_tokens: self.peak_context_tokens.load(Ordering::Relaxed),
            total_cost_usd: total_cost_micro as f64 / 1_000_000.0,
            total_summarizations,
            total_sub_agents: self.total_sub_agents.load(Ordering::Relaxed),
            elapsed_ms,
            excessive_summarization_warning: total_summarizations >= 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_metrics_lifecycle() {
        let metrics = ExecutionMetrics::new();
        metrics.record_turn();
        metrics.record_turn();
        assert_eq!(metrics.total_turns.load(Ordering::Relaxed), 2);

        metrics.record_tool_attempt();
        metrics.record_tool_attempt();
        metrics.record_tool_failure();
        assert_eq!(metrics.tool_calls_attempted.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.tool_calls_failed.load(Ordering::Relaxed), 1);

        metrics.record_tokens(500, 150, 650);
        metrics.record_tokens(300, 100, 1200);
        assert_eq!(metrics.total_input_tokens.load(Ordering::Relaxed), 800);
        assert_eq!(metrics.total_output_tokens.load(Ordering::Relaxed), 250);
        assert_eq!(metrics.peak_context_tokens.load(Ordering::Relaxed), 1200);

        metrics.record_cost(0.045);
        metrics.record_cost(0.015);
        let snap = metrics.snapshot("test-mission", "test-agent");
        assert!((snap.total_cost_usd - 0.06).abs() < 1e-4);
        assert!(!snap.excessive_summarization_warning);

        metrics.record_summarization();
        metrics.record_summarization();
        metrics.record_summarization();
        assert!(metrics.should_warn_excessive_summarization());
        let snap2 = metrics.snapshot("test-mission", "test-agent");
        assert!(snap2.excessive_summarization_warning);
        assert_eq!(snap2.total_summarizations, 3);
    }
}
