//! Telemetry Bridge & Observability Layer - The Telemetric Bridge
//!
//! Orchestrates the high-throughput mapping of internal `tracing` spans to 
//! OpenTelemetry (OTel) compatible JSON events. Bridges the Rust backend 
//! to the React-based visualizers via the `TELEMETRY_TX` broadcast hub.
//!
//! @docs ARCHITECTURE:TelemetryBridge
//! @docs OPERATIONS_MANUAL:Tracing
//!
//! ### AI Assist Note
//! **Telemetric Bridge**: This layer captures `spanId`, `traceId`, and 
//! `parentId` to reconstruct the swarm's recursive reasoning tree in the UI. 
//! It automatically links spans to `mission_id` or `agent_id` if those fields 
//! are present in the `tracing` attributes. Avoid adding large BLOBs to 
//! attributes to prevent broadcast congestion.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Broadcast channel capacity reached (2000 events), malformed JSON in attributes, or OTel extension mismatch.
//! - **Telemetry Link**: Search for `[Telemetry]` or `[Trace]` in `tracing` logs.
//! - **Trace Scope**: `server-rs::telemetry`
//!
pub mod aggregator;
pub mod pulse;
pub mod pulse_types;

#[cfg(test)]
mod pulse_tests;
use once_cell::sync::Lazy;
use opentelemetry::trace::TraceContextExt;
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
/// - `span.id()` -> `spanId`: 64-bit hex identifier.
/// - `span.metadata().name()` -> `name`: The unit of work (e.g., ToolOrchestration).
/// - `span.values()` -> `attributes`: Dynamic key-value pairs following OTel conventions.
pub struct TelemetryLayer;

impl TelemetryLayer {
    /// Initializes a new telemetry layer.
    pub fn new() -> Self {
        Self
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
        let span = ctx.span(id).expect("Span not found");
        let name = span.name();

        // Capture parentId if it exists
        let parent_id = ctx
            .span(id)
            .and_then(|s| s.parent())
            .map(|p| format!("{:x}", p.id().into_u64()));

        // Try to get traceId from OTel context if it's not in attributes
        let mut trace_id = None;
        if let Some(otel_data) = span.extensions().get::<tracing_opentelemetry::OtelData>() {
            let otel_span = otel_data.parent_cx.span();
            let span_context = otel_span.span_context();
            if span_context.is_valid() {
                trace_id = Some(span_context.trace_id().to_string());
            }
        }

        // Basic span info
        let mut event = ::serde_json::json!({
            "type": "trace:span",
            "span": {
                "id": format!("{:x}", id.into_u64()),
                "traceId": trace_id,
                "parentId": parent_id,
                "name": name,
                "startTime": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
                "status": "running",
                "attributes": {}
            }
        });

        // Extract fields from attributes
        let mut visitor = FieldVisitor::new(&mut event["span"]["attributes"]);
        attrs.record(&mut visitor);

        // Override traceId if it was explicitly provided in the attributes
        if let Some(attr_trace_id) = event["span"]["attributes"].get("trace_id") {
            event["span"]["traceId"] = attr_trace_id.clone();
        }

        // Try to link to a mission if mission_id was provided in attributes
        if let Some(mission_id) = event["span"]["attributes"].get("mission_id") {
            event["span"]["missionId"] = mission_id.clone();
        }

        // Try to link to an agent if agent_id was provided
        if let Some(agent_id) = event["span"]["attributes"].get("agent_id") {
            event["span"]["agentId"] = agent_id.clone();
        }

        // Broadcast the "start" of the span
        let _ = TELEMETRY_TX.send(event);
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let _span = ctx.span(&id).expect("Span not found");

        let event = ::serde_json::json!({
            "type": "trace:span_update",
            "spanId": format!("{:x}", id.into_u64()),
            "update": {
                "endTime": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
                "status": "success"
            }
        });

        let _ = TELEMETRY_TX.send(event);
    }
}

struct FieldVisitor<'a> {
    target: &'a mut serde_json::Value,
}

impl<'a> FieldVisitor<'a> {
    fn new(target: &'a mut serde_json::Value) -> Self {
        Self { target }
    }
}

impl<'a> tracing::field::Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.target[field.name()] = ::serde_json::json!(format!("{:?}", value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.target[field.name()] = ::serde_json::json!(value);
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
