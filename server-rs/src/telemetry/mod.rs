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
    let (tx, _) = broadcast::channel(2000);
    tx
});

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
        let name = span.name();

        // Capture parentId if it exists
        let parent_id = span.parent().map(|p| format!("{:x}", p.id().into_u64()));

        // Try to get traceId from OTel context if it's not in attributes
        let mut trace_id = None;
        if let Some(otel_data) = span.extensions().get::<tracing_opentelemetry::OtelData>() {
            if let Some(tid) = otel_data.trace_id() {
                trace_id = Some(tid.to_string());
            }
        }

        // Basic span info
        let mut event = ::serde_json::json!({
            "type": "trace:span",
            "span": {
                "id": format!("{:x}", id.into_u64()),
                "trace_id": trace_id,
                "parent_id": parent_id,
                "name": name,
                "start_time": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
                "status": "running",
                "attributes": {}
            }
        });

        // Extract fields from attributes
        let mut visitor = FieldVisitor::new(&mut event["span"]["attributes"], &self.redactor);
        attrs.record(&mut visitor);

        // --- TRACE SYNCHRONIZATION ---
        // Override trace_id if it was explicitly provided in the attributes or captured in the span
        if let Some(attr_trace_id) = event["span"]["attributes"].get("trace_id") {
            event["span"]["trace_id"] = attr_trace_id.clone();
        }

        // Capture request_id for top-level span correlation
        if let Some(req_id) = event["span"]["attributes"].get("request_id") {
            event["span"]["request_id"] = req_id.clone();
        }

        // Try to link to a mission if mission_id was provided in attributes
        if let Some(mission_id) = event["span"]["attributes"].get("mission_id") {
            event["span"]["mission_id"] = mission_id.clone();
        }

        // Try to link to an agent if agent_id was provided
        if let Some(agent_id) = event["span"]["attributes"].get("agent_id") {
            event["span"]["agent_id"] = agent_id.clone();
        }

        // Broadcast the "start" of the span
        let _ = TELEMETRY_TX.send(event);
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if ctx.span(&id).is_none() {
            tracing::warn!("⚠️ [Telemetry] Span not found during closure: {:?}", id);
            return;
        }

        let event = ::serde_json::json!({
            "type": "trace:span_update",
            "span_id": format!("{:x}", id.into_u64()),
            "update": {
                "end_time": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
                "status": "success"
            }
        });

        let _ = TELEMETRY_TX.send(event);
    }
}

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

// Metadata: [mod]

// Metadata: [mod]
