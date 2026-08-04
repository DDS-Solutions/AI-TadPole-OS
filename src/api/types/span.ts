/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[span]` in observability traces.
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

// Metadata: [span]
