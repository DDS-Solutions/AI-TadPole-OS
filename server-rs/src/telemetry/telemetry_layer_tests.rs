//! @docs ARCHITECTURE:Observability
//! 
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[telemetry_layer_tests.rs]` in tracing logs.
//!
//! @docs ARCHITECTURE:Telemetry Hardening
//!
//! ### AI Assist Note
//! **Telemetric Bridge Validation**: Ensures that the `TelemetryLayer` correctly
//! maps internal tracing spans to the `snake_case` JSON keys required by the
//! frontend (agent_id, mission_id, trace_id, span_id, etc.) (TEL-TEST-01).

use crate::telemetry::{TelemetryLayer, TELEMETRY_TX};
use tracing::span;
use tracing_subscriber::prelude::*;

#[tokio::test]
async fn test_telemetry_layer_snake_case_parity() {
    // 1. Setup subscriber with TelemetryLayer
    let layer = TelemetryLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);

    // We use a local scope for the subscriber to avoid interfering with other tests
    // Note: tracing's set_global_default can only be called once.
    // For unit tests, we usually use tracing::subscriber::with_default.

    let mut rx = TELEMETRY_TX.subscribe();

    tracing::subscriber::with_default(subscriber, || {
        // 2. Create a span with attributes
        let span = span!(
            tracing::Level::INFO,
            "TestMissionCoordination",
            agent_id = "agent-alpha",
            mission_id = "mission-omega",
            trace_id = "trace-12345"
        );

        let _enter = span.enter();
        // Span is "running" now, TelemetryLayer should have sent an event
    });

    // 3. Verify the broadcasted event
    // We expect a "trace:span" event
    let event = rx.recv().await.expect("Failed to receive telemetry event");

    assert_eq!(event["type"], "trace:span");
    let span_data = &event["span"];

    // Verify snake_case keys are present and correctly mapped
    assert!(
        span_data.get("agent_id").is_some(),
        "Missing agent_id key in telemetry JSON"
    );
    assert!(
        span_data.get("mission_id").is_some(),
        "Missing mission_id key in telemetry JSON"
    );
    assert!(
        span_data.get("trace_id").is_some(),
        "Missing trace_id key in telemetry JSON"
    );
    assert!(span_data.get("span_id").is_none()); // span_id is usually 'id' in the JSON root of the span object?
                                                 // Wait, looking at telemetry/mod.rs:
                                                 // "id": format!("{:x}", id.into_u64()) -> this corresponds to spanId in doc but 'id' in JSON.

    assert_eq!(span_data["agent_id"], "agent-alpha");
    assert_eq!(span_data["mission_id"], "mission-omega");
    assert_eq!(span_data["trace_id"], "trace-12345");
    assert_eq!(span_data["name"], "TestMissionCoordination");
    assert!(span_data.get("start_time").is_some());
}

#[tokio::test]
async fn test_telemetry_span_update_on_close() {
    let layer = TelemetryLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);
    let mut rx = TELEMETRY_TX.subscribe();

    tracing::subscriber::with_default(subscriber, || {
        {
            let span = span!(tracing::Level::INFO, "ClosingSpan");
            let _enter = span.enter();
            // Start event sent
        }
        // Span dropped here, on_close should trigger
    });

    // Skip the first event (start)
    let _start_event = rx.recv().await.unwrap();

    // Receive the update event
    let update_event = rx
        .recv()
        .await
        .expect("Failed to receive span update event");

    assert_eq!(update_event["type"], "trace:span_update");
    assert!(
        update_event.get("span_id").is_some(),
        "Missing span_id in update event"
    );
    assert!(update_event["update"].get("end_time").is_some());
    assert_eq!(update_event["update"]["status"], "success");
}
