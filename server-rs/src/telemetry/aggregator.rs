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
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

/// Aggregates span durations from the global telemetry channel.
/// Calculates p50, p95, and p99 metrics for tool execution latency.
pub struct MetricAggregator {
    durations: Vec<f64>,
    span_starts: HashMap<String, u128>,
    window_size: usize,
}

impl MetricAggregator {
    /// Creates a new MetricAggregator with a fixed sliding window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            durations: Vec::with_capacity(window_size),
            span_starts: HashMap::new(),
            window_size,
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
                }
            }
        }
    }

    fn process_msg(&mut self, msg: Value) {
        if let Some(msg_type) = msg["type"].as_str() {
            match msg_type {
                "trace:span" => {
                    if let Some(span) = msg.get("span") {
                        // Capture start time for 'execute_tool' spans
                        if span["name"] == "execute_tool" {
                            if let (Some(id), Some(start)) =
                                (span["id"].as_str(), span["startTime"].as_u64())
                            {
                                self.span_starts.insert(id.to_string(), start as u128);
                            }
                        }
                    }
                }
                "trace:span_update" => {
                    if let (Some(id), Some(end)) =
                        (msg["spanId"].as_str(), msg["update"]["endTime"].as_u64())
                    {
                        // Calculate duration on span closure
                        if let Some(start) = self.span_starts.remove(id) {
                            let duration = (end as u128).saturating_sub(start) as f64;
                            self.durations.push(duration);

                            // Maintain sliding window
                            if self.durations.len() > self.window_size {
                                self.durations.remove(0);
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

        let mut sorted = self.durations.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("Invalid duration found"));

        let len = sorted.len();
        let p50 = sorted[len / 2];
        let p95 = sorted[((len as f64 * 0.95) as usize).min(len - 1)];
        let p99 = sorted[((len as f64 * 0.99) as usize).min(len - 1)];

        info!("📊 [Telemetry] Tool Execution Metrics (n={}): p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms", 
            len, p50, p95, p99);

        let event = serde_json::json!({
            "type": "telemetry:metrics",
            "metrics": {
                "tool_latency_p50": p50,
                "tool_latency_p95": p95,
                "tool_latency_p99": p99,
                "sample_count": len,
                "timestamp": Utc::now().to_rfc3339()
            }
        });

        // Broadcast metrics via the same global bridge for UI visualization
        let _ = super::TELEMETRY_TX.send(event);
    }
}
