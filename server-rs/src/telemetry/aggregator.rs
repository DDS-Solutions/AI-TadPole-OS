//! Insight Synthesis & Telemetry Aggregator
//!
//! Orchestrates the background aggregation of trace spans from the global
//! telemetry channel, calculating P-percentiles for tool execution latency.
//!
//! @docs ARCHITECTURE:TelemetryEngine
//!
//! ### AI Assist Note
//! **Insight Synthesis (Telemetry Aggregator)**: Orchestrates the
//! background synthesis of trace spans from the global telemetry
//! channel, calculating execution latency benchmarks. Features **Sliding
//! Window Aggregation**: latency metrics (p50, p95, p99) are
//! calculated over a fixed window (`window_size`), ensuring that
//! high-frequency tool calls cause older observations to be dropped
//! rapidly to reflect active system performance. Implements **Contextual
//! Metric Linking**: spans named `execute_tool` are automatically
//! correlated to provide granular performance insights for AI tool
//! orchestration (AGG-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Metric reporting skew due to anomalous latency
//!   outliers, memory pressure from large sliding windows, or
//!   broadcast channel lag causing missed spans.
//! - **Telemetry Link**: Search for `📊 [Telemetry]` or `[Metric]` in
//!   `tracing` logs for periodic aggregation reports.
//! - **Trace Scope**: `server-rs::telemetry::aggregator`

use chrono::Utc;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use tokio::sync::broadcast;
use tracing::info;

/// Aggregates span durations from the global telemetry channel.
/// Calculates p50, p95, and p99 metrics for tool execution latency.
pub struct MetricAggregator {
    durations: VecDeque<f64>,
    span_starts: HashMap<String, u128>,
    window_size: usize,
    /// Rolling count of tool spans that closed with status "error".
    error_count: u64,
}

impl MetricAggregator {
    /// Creates a new MetricAggregator with a fixed sliding window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            durations: VecDeque::with_capacity(window_size),
            span_starts: HashMap::new(),
            window_size,
            error_count: 0,
        }
    }

    /// Primary execution loop for the aggregator.
    /// Listens for trace spans and periodically broadcasts aggregated metrics.
    pub async fn run(mut self, mut rx: broadcast::Receiver<Value>) {
        info!(
            "🔭 [Telemetry] MetricAggregator started (Window: {}).",
            self.window_size
        );
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(msg) => self.process_msg(msg),
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("⚠️ [Telemetry] Aggregator lagged by {} messages.", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                _ = ticker.tick() => {
                    self.report_metrics();
                    self.cleanup_zombie_spans();
                }
            }
        }
    }

    fn process_msg(&mut self, msg: Value) {
        if let Some(msg_type) = msg["type"].as_str() {
            match msg_type {
                "trace:span" => {
                    if let Some(span) = msg.get("span") {
                        // TEL-04 fix: match any span whose name contains "tool"
                        // (covers execute_tool, run_tool, tool_call, etc.)
                        let span_name = span["name"].as_str().unwrap_or("");
                        if span_name.contains("tool") {
                            if let (Some(id), Some(start)) =
                                (span["id"].as_str(), span["start_time"].as_u64())
                            {
                                self.span_starts.insert(id.to_string(), start as u128);
                            }
                        }
                    }
                }
                "trace:span_update" => {
                    if let (Some(id), Some(end)) =
                        (msg["span_id"].as_str(), msg["update"]["end_time"].as_u64())
                    {
                        // Calculate duration on span closure
                        if let Some(start) = self.span_starts.remove(id) {
                            let duration = (end as u128).saturating_sub(start) as f64;
                            self.durations.push_back(duration);

                            // Maintain sliding window (O(1) with VecDeque)
                            if self.durations.len() > self.window_size {
                                self.durations.pop_front();
                            }

                            // Track error outcomes for error-rate reporting
                            if msg["update"]["status"].as_str() == Some("error") {
                                self.error_count = self.error_count.saturating_add(1);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn report_metrics(&self) {
        if self.durations.is_empty() {
            return;
        }

        let mut sorted: Vec<f64> = self.durations.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = sorted.len();
        let p50 = sorted[len / 2];
        let p95 = sorted[((len as f64 * 0.95) as usize).min(len - 1)];
        let p99 = sorted[((len as f64 * 0.99) as usize).min(len - 1)];

        info!("📊 [Telemetry] Tool Execution Metrics (n={}, errors={}): p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms",
            len, self.error_count, p50, p95, p99);

        // Update Prometheus metrics
        super::TOOL_LATENCY_P50.set(p50);
        super::TOOL_LATENCY_P95.set(p95);
        super::TOOL_LATENCY_P99.set(p99);
        super::TOOL_LATENCY_SAMPLE_COUNT.set(len as f64);

        let event = serde_json::json!({
            "type": "telemetry:metrics",
            "metrics": {
                "tool_latency_p50": p50,
                "tool_latency_p95": p95,
                "tool_latency_p99": p99,
                "sample_count": len,
                "error_count": self.error_count,
                "timestamp": Utc::now().to_rfc3339()
            }
        });

        // Broadcast metrics via the same global bridge for UI visualization
        let _ = super::TELEMETRY_TX.send(event);
    }

    /// Periodically cleans up span starts that did not receive a closure update
    /// (e.g., due to cancellation, network drop, or crash) to prevent a memory leak.
    fn cleanup_zombie_spans(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let ttl_ms = 120_000; // 2 minutes
        let initial_len = self.span_starts.len();
        self.span_starts
            .retain(|_, start_time| now.saturating_sub(*start_time) < ttl_ms);
        let cleaned = initial_len.saturating_sub(self.span_starts.len());
        if cleaned > 0 {
            tracing::warn!(
                "🧹 [Telemetry] Cleaned up {} zombie tool execution spans.",
                cleaned
            );
        }
    }
}

// Metadata: [aggregator]

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_span_msg(id: &str, name: &str, start: u64) -> serde_json::Value {
        json!({ "type": "trace:span", "span": { "id": id, "name": name, "start_time": start } })
    }

    fn make_update_msg(id: &str, end: u64, status: &str) -> serde_json::Value {
        json!({ "type": "trace:span_update", "span_id": id, "update": { "end_time": end, "status": status } })
    }

    #[test]
    fn test_tool_span_captured_by_contains_filter() {
        let mut agg = MetricAggregator::new(100);
        // "run_tool" contains "tool" — should be tracked
        agg.process_msg(make_span_msg("a", "run_tool", 1000));
        agg.process_msg(make_update_msg("a", 1050, "success"));
        assert_eq!(agg.durations.len(), 1);
        assert_eq!(agg.durations[0], 50.0);
    }

    #[test]
    fn test_non_tool_span_not_captured() {
        let mut agg = MetricAggregator::new(100);
        agg.process_msg(make_span_msg("b", "fetch_data", 1000));
        agg.process_msg(make_update_msg("b", 2000, "success"));
        assert!(
            agg.durations.is_empty(),
            "non-tool span should not be tracked"
        );
    }

    #[test]
    fn test_error_count_increments_on_error_status() {
        let mut agg = MetricAggregator::new(100);
        agg.process_msg(make_span_msg("c", "execute_tool", 1000));
        agg.process_msg(make_update_msg("c", 1100, "error"));
        assert_eq!(agg.error_count, 1);
    }

    #[test]
    fn test_error_count_unchanged_on_success() {
        let mut agg = MetricAggregator::new(100);
        agg.process_msg(make_span_msg("d", "execute_tool", 1000));
        agg.process_msg(make_update_msg("d", 1100, "success"));
        assert_eq!(agg.error_count, 0);
    }

    #[test]
    fn test_sliding_window_eviction() {
        let mut agg = MetricAggregator::new(3);
        for i in 0u64..5 {
            agg.process_msg(make_span_msg(&i.to_string(), "tool_call", i * 1000));
            agg.process_msg(make_update_msg(&i.to_string(), i * 1000 + 100, "success"));
        }
        assert_eq!(agg.durations.len(), 3, "window should cap at 3");
    }
}
