//! @docs ARCHITECTURE:TelemetryEngine
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / aggregator
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Telemetry]`
//! - **Witness Tests**: none declared

use chrono::Utc;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use tokio::sync::broadcast;
use tracing::info;

/// Aggregates span durations from the global telemetry channel.
/// Calculates p50, p95, and p99 metrics for tool execution latency.
pub struct MetricAggregator {
    durations: VecDeque<f64>,
    window_errors: VecDeque<bool>,
    span_starts: HashMap<String, u128>,
    window_size: usize,
    /// Rolling count of tool spans in the current window that closed with status "error".
    window_error_count: u64,
    /// Total lifetime count of tool errors since process start.
    lifetime_errors: u64,
    /// Count of span updates received without a recorded start (e.g., due to lag or restart).
    orphaned_updates: u64,
}

impl MetricAggregator {
    /// Creates a new MetricAggregator with a fixed sliding window size (minimum 1).
    pub fn new(window_size: usize) -> Self {
        let size = window_size.max(1);
        Self {
            durations: VecDeque::with_capacity(size),
            window_errors: VecDeque::with_capacity(size),
            span_starts: HashMap::new(),
            window_size: size,
            window_error_count: 0,
            lifetime_errors: 0,
            orphaned_updates: 0,
        }
    }

    /// Primary execution loop for the aggregator with optional graceful shutdown watch.
    pub async fn run_with_shutdown(
        mut self,
        mut rx: broadcast::Receiver<Value>,
        mut shutdown_rx: Option<tokio::sync::watch::Receiver<bool>>,
    ) {
        info!(
            "🔭 [Telemetry] MetricAggregator started (Window: {}).",
            self.window_size
        );
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            tokio::select! {
                result = async {
                    if let Some(ref mut s_rx) = shutdown_rx {
                        s_rx.changed().await
                    } else {
                        std::future::pending().await
                    }
                } => {
                    if result.is_err() || shutdown_rx.as_ref().map(|s| *s.borrow()).unwrap_or(false) {
                        tracing::info!("🛑 MetricAggregator shutting down gracefully.");
                        break;
                    }
                }
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

    /// Primary execution loop for the aggregator.
    /// Listens for trace spans and periodically broadcasts aggregated metrics.
    pub async fn run(self, rx: broadcast::Receiver<Value>) {
        self.run_with_shutdown(rx, None).await;
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
                        let is_error = msg["update"]["status"].as_str() == Some("error");
                        // Calculate duration on span closure
                        if let Some(start) = self.span_starts.remove(id) {
                            let duration = (end as u128).saturating_sub(start) as f64;
                            self.durations.push_back(duration);
                            self.window_errors.push_back(is_error);

                            if is_error {
                                self.window_error_count = self.window_error_count.saturating_add(1);
                                self.lifetime_errors = self.lifetime_errors.saturating_add(1);
                            }

                            // Maintain sliding window (O(1) with VecDeque)
                            if self.durations.len() > self.window_size {
                                self.durations.pop_front();
                                if let Some(popped_error) = self.window_errors.pop_front() {
                                    if popped_error {
                                        self.window_error_count =
                                            self.window_error_count.saturating_sub(1);
                                    }
                                }
                            }
                        } else {
                            self.orphaned_updates = self.orphaned_updates.saturating_add(1);
                            if is_error {
                                self.lifetime_errors = self.lifetime_errors.saturating_add(1);
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

        info!("📊 [Telemetry] Tool Execution Metrics (window_n={}, window_errors={}, lifetime_errors={}): p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms",
            len, self.window_error_count, self.lifetime_errors, p50, p95, p99);

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
                "window_error_count": self.window_error_count,
                "lifetime_errors": self.lifetime_errors,
                "orphaned_updates": self.orphaned_updates,
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
        assert_eq!(agg.window_error_count, 1);
        assert_eq!(agg.lifetime_errors, 1);
    }

    #[test]
    fn test_error_count_unchanged_on_success() {
        let mut agg = MetricAggregator::new(100);
        agg.process_msg(make_span_msg("d", "execute_tool", 1000));
        agg.process_msg(make_update_msg("d", 1100, "success"));
        assert_eq!(agg.window_error_count, 0);
        assert_eq!(agg.lifetime_errors, 0);
    }

    #[test]
    fn test_sliding_window_eviction() {
        let mut agg = MetricAggregator::new(3);
        for i in 0u64..5 {
            let status = if i == 0 { "error" } else { "success" };
            agg.process_msg(make_span_msg(&i.to_string(), "tool_call", i * 1000));
            agg.process_msg(make_update_msg(&i.to_string(), i * 1000 + 100, status));
        }
        assert_eq!(agg.durations.len(), 3, "window should cap at 3");
        // Sample 0 (which had an error) was evicted from the 3-element window:
        assert_eq!(agg.window_error_count, 0);
        assert_eq!(agg.lifetime_errors, 1);
    }
}
