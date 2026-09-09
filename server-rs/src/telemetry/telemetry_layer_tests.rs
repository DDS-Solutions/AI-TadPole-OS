//! @docs ARCHITECTURE:Observability
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / telemetry_layer_tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::telemetry::{TelemetryLayer, TELEMETRY_TX};
use std::time::Duration;
use tokio::time::timeout;
use tracing::span;
use tracing_subscriber::prelude::*;

#[tokio::test]
async fn test_telemetry_layer_snake_case_parity() {
    // 1. Setup subscriber with TelemetryLayer
    let layer = TelemetryLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);

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
    });

    // 3. Verify the broadcasted event within timeout
    let event = timeout(Duration::from_secs(3), async {
        while let Ok(e) = rx.recv().await {
            if e["type"] == "trace:span" && e["span"]["name"] == "TestMissionCoordination" {
                return Some(e);
            }
        }
        None
    })
    .await
    .expect("Timed out waiting for matching trace:span event")
    .expect("Failed to receive matching telemetry event");

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
    assert!(
        span_data.get("id").is_some(),
        "Span JSON root must contain hex id"
    );

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
        }
        // Span dropped here, on_close triggers
    });

    // Receive the specific update event for "ClosingSpan"
    let update_event = timeout(Duration::from_secs(3), async {
        while let Ok(e) = rx.recv().await {
            if e["type"] == "trace:span_update" && e["update"]["name"] == "ClosingSpan" {
                return Some(e);
            }
        }
        None
    })
    .await
    .expect("Timed out waiting for matching trace:span_update event")
    .expect("Failed to receive span update event");

    assert_eq!(update_event["type"], "trace:span_update");
    assert!(
        update_event.get("span_id").is_some(),
        "Missing span_id in update event"
    );
    assert_eq!(update_event["update"]["name"], "ClosingSpan");
    assert!(update_event["update"].get("end_time").is_some());
    assert_eq!(update_event["update"]["status"], "success");
}
