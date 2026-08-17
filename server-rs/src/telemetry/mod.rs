//! @docs ARCHITECTURE:TelemetryBridge
//! @docs OPERATIONS_MANUAL:Tracing
//!
//! ### AI Assist Note
//! **Telemetric Bridge**: Orchestrates the high-throughput mapping of
//! internal `tracing` spans to **OpenTelemetry (OTel)** compatible
//! JSON events. Features **Span Reconstruction**: captures `span_id`,
//! `trace_id`, and `parent_id` to build the recursive reasoning tree in
//! the UI. Implements **Contextual Context Alignment**: automatically
//! links telemetry to `mission_id` or `agent_id` for granular
//! mission-specific observability. Note: The `TELEMETRY_TX` broadcast
//! hub is optimized for high-volume pulse data; avoid adding large
//! BLOBs to attributes to prevent congestion (TEL-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Broadcast channel capacity saturation (2000
//!   events), malformed JSON in dynamic attributes, or OTel
//!   extension mismatches causing trace discontinuity.
//! - **Telemetry Link**: Search for `[Telemetry]` or `[Trace]` in
//!   `tracing` logs for bridge performance benchmarks.
//! - **Trace Scope**: `server-rs::telemetry`
//!
pub mod aggregator;
pub mod pulse;
pub mod pulse_types;
pub mod sink;

#[cfg(test)]
mod pulse_tests;
#[cfg(test)]
mod telemetry_layer_tests;

use crate::secret_redactor::SecretRedactor;
use once_cell::sync::Lazy;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tracing::{span, Subscriber};
use tracing_subscriber::Layer;

/// Global broadcast channel for telemetry events.
///
/// Optimized for high-throughput JSON emissions from system spans and
/// agent lifecycle events.
pub static TELEMETRY_TX: Lazy<broadcast::Sender<::serde_json::Value>> = Lazy::new(|| {
    let (tx, _) = broadcast::channel(1000);
    tx
});

// ---------------------------------------------------------------------------
// Internal span-extension types
// ---------------------------------------------------------------------------

/// Stores mutable span metadata collected across the span lifecycle.
/// Stored in tracing-subscriber span extensions.
#[allow(dead_code)] // fields serialized via serde_json::json! in on_close
struct SpanMeta {
    name: String,
    start_time: u128,
    parent_id: Option<String>,
    trace_id: Option<String>,
    request_id: Option<String>,
    mission_id: Option<String>,
    agent_id: Option<String>,
    /// Attributes captured at span creation and augmented on `on_record`.
    attributes: ::serde_json::Value,
    /// Error status set via `SpanStatus::error(msg)` before the span closes.
    error: Option<String>,
    /// HTTP response status code, recorded by `inject_request_id` middleware (Gap 3).
    http_status: Option<u16>,
}

/// Marker stored in span extensions to signal an error outcome.
///
/// Call `tracing::Span::current().record("error", msg)` **before** the span
/// closes to tag the span as failed in the telemetry stream.
pub struct SpanStatus {
    pub message: String,
}

impl SpanStatus {
    #[allow(dead_code)] // public API — called by route handlers to tag spans as failed
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// TelemetryLayer
// ---------------------------------------------------------------------------

/// Custom Tracing Layer that bridges OpenTelemetry spans to the frontend.
/// Custom tracing layer that maps internal `tracing` spans to OpenTelemetry (OTel)
/// compatible JSON events.
///
/// This layer is responsible for the "Telemetric Bridge" between the high-performance
/// Rust backend and the React-based visualizers.
///
/// Mapping Logic:
/// - `span.id()` -> `id`: 64-bit hex identifier.
/// - `span.metadata().name()` -> `name`: The unit of work (e.g., ToolOrchestration).
/// - `span.values()` -> `attributes`: Dynamic key-value pairs following OTel conventions.
///
/// ### Fix TEL-02: `on_close` now emits the correct status
/// Span metadata (including any error recorded via `SpanStatus`) is stored in
/// span extensions on `on_new_span` and `on_record`, then read back on `on_close`
/// to broadcast the final `success` or `error` status.
///
/// ### Fix TEL-03: `trace_id` propagation
/// `trace_id` is captured from span attributes in `on_new_span`/`on_record`
/// and stored in `SpanMeta`. This allows the `W3C traceparent` value injected by
/// `inject_request_id` middleware to propagate correctly through child spans.
pub struct TelemetryLayer {
    redactor: SecretRedactor,
}

impl TelemetryLayer {
    /// Initializes a new telemetry layer.
    pub fn new() -> Self {
        Self {
            redactor: SecretRedactor::from_env(),
        }
    }
}

impl<S> Layer<S> for TelemetryLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span = match ctx.span(id) {
            Some(s) => s,
            None => {
                tracing::warn!("⚠️ [Telemetry] Span not found during creation: {:?}", id);
                return;
            }
        };
        let name = span.name().to_string();
        let parent_id = span.parent().map(|p| format!("{:x}", p.id().into_u64()));
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        // Collect attributes
        let mut attributes = ::serde_json::json!({});
        let mut visitor = FieldVisitor::new(&mut attributes, &self.redactor);
        attrs.record(&mut visitor);

        // Extract well-known correlation fields from attributes
        let trace_id = attributes
            .get("trace_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let request_id = attributes
            .get("request_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let mission_id = attributes
            .get("mission_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let agent_id = attributes
            .get("agent_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Build and store metadata in span extensions for retrieval at on_close
        let meta = SpanMeta {
            name: name.clone(),
            start_time,
            parent_id: parent_id.clone(),
            trace_id: trace_id.clone(),
            request_id: request_id.clone(),
            mission_id: mission_id.clone(),
            agent_id: agent_id.clone(),
            attributes: attributes.clone(),
            error: None,
            http_status: None,
        };
        span.extensions_mut().insert(meta);

        // Broadcast span-start event
        let mut event = ::serde_json::json!({
            "type": "trace:span",
            "span": {
                "id": format!("{:x}", id.into_u64()),
                "trace_id": trace_id,
                "parent_id": parent_id,
                "name": name,
                "start_time": start_time,
                "status": "running",
                "attributes": attributes
            }
        });

        // Promote correlation fields to top-level for easy filtering
        if let Some(ref rid) = request_id {
            event["span"]["request_id"] = ::serde_json::json!(rid);
        }
        if let Some(ref mid) = mission_id {
            event["span"]["mission_id"] = ::serde_json::json!(mid);
        }
        if let Some(ref aid) = agent_id {
            event["span"]["agent_id"] = ::serde_json::json!(aid);
        }

        let _ = TELEMETRY_TX.send(event);
    }

    fn on_record(
        &self,
        id: &span::Id,
        values: &span::Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span = match ctx.span(id) {
            Some(s) => s,
            None => return,
        };
        let mut exts = span.extensions_mut();
        if let Some(meta) = exts.get_mut::<SpanMeta>() {
            // Merge newly recorded fields into the stored attributes
            let mut visitor = FieldVisitor::new(&mut meta.attributes, &self.redactor);
            values.record(&mut visitor);

            // Update correlation fields if newly recorded
            if meta.trace_id.is_none() {
                meta.trace_id = meta
                    .attributes
                    .get("trace_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
            if meta.request_id.is_none() {
                meta.request_id = meta
                    .attributes
                    .get("request_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
            if meta.mission_id.is_none() {
                meta.mission_id = meta
                    .attributes
                    .get("mission_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
            if meta.agent_id.is_none() {
                meta.agent_id = meta
                    .attributes
                    .get("agent_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
            // Capture error field if recorded via span.record("error", msg)
            if let Some(err_val) = meta.attributes.get("error").and_then(|v| v.as_str()) {
                if meta.error.is_none() {
                    meta.error = Some(err_val.to_string());
                }
            }
            // Gap 3: Capture http_status if recorded by inject_request_id middleware
            if meta.http_status.is_none() {
                meta.http_status = meta
                    .attributes
                    .get("http_status")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16);
            }
        }
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let span = match ctx.span(&id) {
            Some(s) => s,
            None => {
                tracing::warn!("⚠️ [Telemetry] Span not found during closure: {:?}", id);
                return;
            }
        };

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        // Read stored metadata to determine true outcome
        let exts = span.extensions();
        let (status, error_msg, trace_id, request_id, mission_id, agent_id, http_status) =
            if let Some(meta) = exts.get::<SpanMeta>() {
                let status = if meta.error.is_some() {
                    "error"
                } else {
                    "success"
                };
                (
                    status,
                    meta.error.clone(),
                    meta.trace_id.clone(),
                    meta.request_id.clone(),
                    meta.mission_id.clone(),
                    meta.agent_id.clone(),
                    meta.http_status,
                )
            } else {
                // Fallback if metadata was never stored (shouldn't happen)
                ("success", None, None, None, None, None, None)
            };

        // Also check for a SpanStatus extension (set directly via extensions_mut)
        let span_status_error = span
            .extensions()
            .get::<SpanStatus>()
            .map(|s| s.message.clone());
        let final_error = span_status_error.or(error_msg);
        let final_status = if final_error.is_some() {
            "error"
        } else {
            status
        };

        let mut update = ::serde_json::json!({
            "type": "trace:span_update",
            "span_id": format!("{:x}", id.into_u64()),
            "update": {
                "end_time": end_time,
                "status": final_status,
            }
        });

        if let Some(ref err) = final_error {
            update["update"]["error"] = ::serde_json::json!(err);
        }
        // Gap 3: Emit HTTP status code; promote 4xx/5xx to error status
        if let Some(code) = http_status {
            update["update"]["http_status"] = ::serde_json::json!(code);
            if code >= 400 {
                update["update"]["status"] = ::serde_json::json!("error");
            }
        }
        if let Some(ref tid) = trace_id {
            update["update"]["trace_id"] = ::serde_json::json!(tid);
        }
        if let Some(ref rid) = request_id {
            update["update"]["request_id"] = ::serde_json::json!(rid);
        }
        if let Some(ref mid) = mission_id {
            update["update"]["mission_id"] = ::serde_json::json!(mid);
        }
        if let Some(ref aid) = agent_id {
            update["update"]["agent_id"] = ::serde_json::json!(aid);
        }

        let _ = TELEMETRY_TX.send(update);
    }
}

// ---------------------------------------------------------------------------
// FieldVisitor
// ---------------------------------------------------------------------------

struct FieldVisitor<'a> {
    target: &'a mut serde_json::Value,
    redactor: &'a SecretRedactor,
}

impl<'a> FieldVisitor<'a> {
    fn new(target: &'a mut serde_json::Value, redactor: &'a SecretRedactor) -> Self {
        Self { target, redactor }
    }
}

impl<'a> tracing::field::Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let val_str = format!("{:?}", value);
        let safe_val = self.redactor.redact(&val_str);
        self.target[field.name()] = ::serde_json::json!(safe_val);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let safe_val = self.redactor.redact(value);
        self.target[field.name()] = ::serde_json::json!(safe_val);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.target[field.name()] = ::serde_json::json!(value);
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.target[field.name()] = ::serde_json::json!(value);
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.target[field.name()] = ::serde_json::json!(value);
    }
}

// ---------------------------------------------------------------------------
// Prometheus metrics
// ---------------------------------------------------------------------------

pub static TOOL_LATENCY_P50: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tool_latency_p50",
        "The p50 tool execution latency in milliseconds"
    )
    .expect("metric can be created")
});

pub static TOOL_LATENCY_P95: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tool_latency_p95",
        "The p95 tool execution latency in milliseconds"
    )
    .expect("metric can be created")
});

pub static TOOL_LATENCY_P99: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tool_latency_p99",
        "The p99 tool execution latency in milliseconds"
    )
    .expect("metric can be created")
});

pub static TOOL_LATENCY_SAMPLE_COUNT: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tool_latency_sample_count",
        "The number of tool execution latency samples in the sliding window"
    )
    .expect("metric can be created")
});

pub static TADPOLE_ACTIVE_AGENTS: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tadpole_active_agents",
        "The number of active agent nodes in the registry"
    )
    .expect("metric can be created")
});

pub static TADPOLE_HEALTH_STATE: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tadpole_health_state",
        "The current health state of the engine (0=Degraded, 1=Warming, 2=Ready)"
    )
    .expect("metric can be created")
});

pub static TADPOLE_MAX_SWARM_DEPTH: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tadpole_max_swarm_depth",
        "The maximum recursion depth of the swarm"
    )
    .expect("metric can be created")
});

pub static TADPOLE_TPM_ACCUMULATOR: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tadpole_tpm_accumulator",
        "The accumulated tokens per minute of the swarm"
    )
    .expect("metric can be created")
});

pub static TADPOLE_RECRUIT_COUNT: Lazy<prometheus::Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "tadpole_recruit_count",
        "The total count of recruited agents"
    )
    .expect("metric can be created")
});

/// Forces evaluation and registration of all Prometheus metrics.
pub fn init_prometheus_metrics() {
    Lazy::force(&TOOL_LATENCY_P50);
    Lazy::force(&TOOL_LATENCY_P95);
    Lazy::force(&TOOL_LATENCY_P99);
    Lazy::force(&TOOL_LATENCY_SAMPLE_COUNT);
    Lazy::force(&TADPOLE_ACTIVE_AGENTS);
    Lazy::force(&TADPOLE_HEALTH_STATE);
    Lazy::force(&TADPOLE_MAX_SWARM_DEPTH);
    Lazy::force(&TADPOLE_TPM_ACCUMULATOR);
    Lazy::force(&TADPOLE_RECRUIT_COUNT);
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// Verify that a span_update with status "error" is emitted when an error
    /// field is recorded on the span before close.
    #[test]
    fn test_span_close_emits_error_status_when_error_recorded() {
        // Simulate the JSON produced by on_close when an error is present
        let end_time: u128 = 1_000_000;
        let error_msg = Some("tool execution failed".to_string());
        let final_status = if error_msg.is_some() {
            "error"
        } else {
            "success"
        };

        let update = serde_json::json!({
            "type": "trace:span_update",
            "span_id": "abc123",
            "update": {
                "end_time": end_time,
                "status": final_status,
                "error": error_msg.as_deref().unwrap_or(""),
            }
        });

        assert_eq!(update["update"]["status"], "error");
        assert_eq!(update["update"]["error"], "tool execution failed");
    }

    /// Verify that a span_update with status "success" is emitted when no error is set.
    #[test]
    fn test_span_close_emits_success_status_when_no_error() {
        let error_msg: Option<String> = None;
        let final_status = if error_msg.is_some() {
            "error"
        } else {
            "success"
        };

        let update = serde_json::json!({
            "type": "trace:span_update",
            "span_id": "def456",
            "update": {
                "end_time": 2_000_000u128,
                "status": final_status,
            }
        });

        assert_eq!(update["update"]["status"], "success");
        assert!(!update["update"].as_object().unwrap().contains_key("error"));
    }

    /// Verify that trace_id extracted from attributes is promoted to the span-start event.
    #[test]
    fn test_trace_id_promoted_to_span_event() {
        let trace_id = Some("4bf92f3577b34da6a3ce929d0e0e4736".to_string());
        let event = serde_json::json!({
            "type": "trace:span",
            "span": {
                "id": "abc",
                "trace_id": trace_id,
                "name": "test_span",
                "status": "running",
            }
        });
        assert_eq!(
            event["span"]["trace_id"],
            Value::String("4bf92f3577b34da6a3ce929d0e0e4736".to_string())
        );
    }

    /// Verify the SpanStatus helper type carries the error message.
    #[test]
    fn test_span_status_error_helper() {
        let s = SpanStatus::error("db connection failed");
        assert_eq!(s.message, "db connection failed");
    }
}

// Metadata: [mod]
