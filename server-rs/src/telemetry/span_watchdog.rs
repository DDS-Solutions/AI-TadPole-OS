//! @docs ARCHITECTURE:TelemetryBridge
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / span_watchdog
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[SpanWatchdog]`
//! - **Witness Tests**: none declared

use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

/// Default inactivity thresholds
pub const DEFAULT_CLOUD_TTL_SECS: u64 = 60;
pub const DEFAULT_LOCAL_TTL_SECS: u64 = 300;
pub const WATCHDOG_SWEEP_INTERVAL_SECS: u64 = 10;

#[derive(Debug, Clone)]
pub struct ActiveSpanMetadata {
    pub span_id: String,
    pub trace_id: Option<String>,
    pub parent_id: Option<String>,
    pub name: String,
    pub agent_id: Option<String>,
    pub mission_id: Option<String>,
    pub start_epoch_ms: u128,
    pub start_instant: Instant,
    pub last_activity_instant: Instant,
    pub timeout_seconds: u64,
    pub provider: Option<String>,
    pub slot: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct RegisterSpanParams {
    pub span_id: String,
    pub name: String,
    pub agent_id: Option<String>,
    pub mission_id: Option<String>,
    pub trace_id: Option<String>,
    pub parent_id: Option<String>,
    pub provider: Option<String>,
    pub slot: Option<u8>,
    pub timeout_override_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReapedSpanUpdate {
    pub name: String,
    pub status: String,
    pub start_time: u128,
    pub end_time: u128,
    pub duration_ms: u128,
    pub error: String,
    #[serde(rename = "error.reason")]
    pub error_reason: String,
    pub reaped_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mission_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReapedSpanEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub span_id: String,
    pub update: ReapedSpanUpdate,
}

/// Global registry of in-flight trace spans actively monitored by the watchdog.
pub static ACTIVE_SPANS: Lazy<DashMap<String, ActiveSpanMetadata>> = Lazy::new(DashMap::new);

/// Helper to get current epoch millis for absolute timestamp serialization.
pub fn current_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_else(|e| {
            tracing::warn!("⚠️ [SpanWatchdog] System time before UNIX_EPOCH: {:?}", e);
            0
        })
}

pub struct SpanWatchdog;

impl SpanWatchdog {
    /// Registers a new active span with provider-aware dynamic TTL and monotonic tracking.
    #[allow(dead_code)]
    pub fn register_span(
        span_id: String,
        name: String,
        agent_id: Option<String>,
        mission_id: Option<String>,
        trace_id: Option<String>,
        parent_id: Option<String>,
        provider: Option<String>,
        slot: Option<u8>,
        timeout_override_secs: Option<u64>,
    ) {
        Self::register_span_with_params(RegisterSpanParams {
            span_id,
            name,
            agent_id,
            mission_id,
            trace_id,
            parent_id,
            provider,
            slot,
            timeout_override_secs,
        });
    }

    /// Registers a new active span using a structured parameters object.
    #[allow(dead_code)]
    pub fn register_span_with_params(params: RegisterSpanParams) {
        let epoch_now = current_epoch_ms();
        let instant_now = Instant::now();

        let provider_lower = params.provider.as_ref().map(|p| p.to_lowercase());
        let timeout = if let Some(override_secs) = params.timeout_override_secs {
            override_secs.max(1)
        } else if let Some(ref p) = provider_lower {
            if p.contains("ollama") || p.contains("local") {
                DEFAULT_LOCAL_TTL_SECS
            } else {
                DEFAULT_CLOUD_TTL_SECS
            }
        } else {
            DEFAULT_CLOUD_TTL_SECS
        };

        let meta = ActiveSpanMetadata {
            span_id: params.span_id.clone(),
            trace_id: params.trace_id,
            parent_id: params.parent_id,
            name: params.name,
            agent_id: params.agent_id,
            mission_id: params.mission_id,
            start_epoch_ms: epoch_now,
            start_instant: instant_now,
            last_activity_instant: instant_now,
            timeout_seconds: timeout,
            provider: params.provider,
            slot: params.slot,
        };

        ACTIVE_SPANS.insert(params.span_id, meta);
    }

    /// Resets the inactivity timer when tokens or stream chunks are received (drift-proof).
    #[allow(dead_code)]
    pub fn heartbeat(span_id: &str) {
        if let Some(mut span) = ACTIVE_SPANS.get_mut(span_id) {
            span.last_activity_instant = Instant::now();
        }
    }

    /// Closes and unregisters a span upon normal completion.
    #[allow(dead_code)]
    pub fn close_span(span_id: &str) {
        ACTIVE_SPANS.remove(span_id);
    }

    /// Sweeps all active spans using monotonic elapsed checks and returns strongly-typed reaped events.
    pub fn sweep() -> Vec<ReapedSpanEvent> {
        let mut reaped_events = Vec::new();
        let mut to_remove = Vec::new();
        let epoch_now = current_epoch_ms();

        for entry in ACTIVE_SPANS.iter() {
            let meta = entry.value();
            let elapsed_inactivity = meta.last_activity_instant.elapsed();
            let threshold = Duration::from_secs(meta.timeout_seconds);

            if elapsed_inactivity >= threshold {
                to_remove.push(meta.span_id.clone());

                let total_duration_ms = meta.start_instant.elapsed().as_millis();
                let end_epoch_ms = epoch_now;

                let event = ReapedSpanEvent {
                    event_type: "trace:span_update".to_string(),
                    span_id: meta.span_id.clone(),
                    update: ReapedSpanUpdate {
                        name: meta.name.clone(),
                        status: "error".to_string(),
                        start_time: meta.start_epoch_ms,
                        end_time: end_epoch_ms,
                        duration_ms: total_duration_ms,
                        error: "SPAN_TIMEOUT_REAPED".to_string(),
                        error_reason: format!(
                            "Span exceeded {}s inactivity lifecycle limit without closure",
                            meta.timeout_seconds
                        ),
                        reaped_by: "SpanLifecycleWatchdog".to_string(),
                        trace_id: meta.trace_id.clone(),
                        parent_id: meta.parent_id.clone(),
                        agent_id: meta.agent_id.clone(),
                        mission_id: meta.mission_id.clone(),
                        provider: meta.provider.clone(),
                        slot: meta.slot,
                    },
                };

                reaped_events.push(event);
            }
        }

        for id in to_remove {
            ACTIVE_SPANS.remove(&id);
        }

        reaped_events
    }

    /// Background task that executes periodic watchdog sweeps and broadcasts type-safe events.
    pub async fn start_background_loop() {
        tracing::info!(
            "⏱️ [SpanWatchdog] Adaptive Span Lifecycle Watchdog launched (Interval: {}s).",
            WATCHDOG_SWEEP_INTERVAL_SECS
        );

        loop {
            sleep(Duration::from_secs(WATCHDOG_SWEEP_INTERVAL_SECS)).await;

            let reaped = Self::sweep();

            for event in reaped {
                tracing::warn!(
                    "🧹 [SpanWatchdog] Reaped inactive trace span: {} (elapsed > {}s)",
                    event.span_id,
                    event.update.duration_ms / 1000
                );

                match serde_json::to_value(&event) {
                    Ok(val) => {
                        if let Err(e) = crate::telemetry::TELEMETRY_TX.send(val) {
                            tracing::warn!(
                                "⚠️ [SpanWatchdog] Failed to broadcast reaped span event (receivers dropped or buffer full): {:?}",
                                e
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "🚨 [SpanWatchdog] Failed to serialize reaped span event: {:?}",
                            e
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_vs_local_span_ttl() {
        let span_cloud = "span-cloud-1".to_string();
        let span_local = "span-local-1".to_string();

        SpanWatchdog::register_span(
            span_cloud.clone(),
            "CloudInference".to_string(),
            Some("agent-1".to_string()),
            None,
            None,
            None,
            Some("openai".to_string()),
            Some(1),
            None,
        );

        SpanWatchdog::register_span(
            span_local.clone(),
            "LocalReasoning".to_string(),
            Some("agent-2".to_string()),
            None,
            None,
            None,
            Some("ollama".to_string()),
            Some(3),
            None,
        );

        let cloud_timeout = ACTIVE_SPANS
            .get(&span_cloud)
            .map(|m| m.timeout_seconds)
            .unwrap();
        assert_eq!(cloud_timeout, DEFAULT_CLOUD_TTL_SECS);

        let local_timeout = ACTIVE_SPANS
            .get(&span_local)
            .map(|m| m.timeout_seconds)
            .unwrap();
        assert_eq!(local_timeout, DEFAULT_LOCAL_TTL_SECS);

        SpanWatchdog::close_span(&span_cloud);
        SpanWatchdog::close_span(&span_local);
    }

    #[test]
    fn test_heartbeat_prevents_premature_reaping() {
        let span_id = "span-hb-test".to_string();
        let epoch_now = current_epoch_ms();
        let now = Instant::now();

        ACTIVE_SPANS.insert(
            span_id.clone(),
            ActiveSpanMetadata {
                span_id: span_id.clone(),
                trace_id: None,
                parent_id: None,
                name: "LongRunningLocalInference".to_string(),
                agent_id: Some("99".to_string()),
                mission_id: None,
                start_epoch_ms: epoch_now,
                start_instant: now,
                // Simulate that the span started and had activity recently (0s ago with a 60s timeout)
                last_activity_instant: now,
                timeout_seconds: 60,
                provider: Some("ollama".to_string()),
                slot: Some(2),
            },
        );

        // Immediate sweep should NOT reap active span
        let reaped = SpanWatchdog::sweep();
        assert!(reaped.is_empty(), "Fresh span should not be reaped!");

        // Simulate inactivity exceeding threshold
        if let Some(mut span) = ACTIVE_SPANS.get_mut(&span_id) {
            span.last_activity_instant = now.checked_sub(Duration::from_secs(75)).unwrap_or(now);
        }

        // Now sweep should detect the inactive span
        let reaped_after = SpanWatchdog::sweep();
        assert_eq!(reaped_after.len(), 1, "Inactive span should be reaped!");
        assert_eq!(reaped_after[0].update.status, "error");
        assert_eq!(reaped_after[0].update.error, "SPAN_TIMEOUT_REAPED");
        assert_eq!(reaped_after[0].event_type, "trace:span_update");
        assert_eq!(reaped_after[0].span_id, span_id);
    }
}
