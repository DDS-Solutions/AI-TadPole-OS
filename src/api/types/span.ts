/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / span
 * - **Primary Entrypoints**: `SpanData`, `TelemetrySpanUpdate`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export interface SpanData {
    id: string;
    trace_id: string;
    name: string;
    agent_id: string;
    mission_id: string;
    start_time: number;
    status: 'running' | 'success' | 'error';
    attributes: Record<string, string | number | boolean>;
}

export interface TelemetrySpanUpdate {
    end_time: number;
    status: 'success' | 'error';
    attributes?: Record<string, string | number | boolean>;
}
