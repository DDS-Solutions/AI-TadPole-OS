/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[ports]` in observability traces.
 */

import type { SpanData, TelemetrySpanUpdate } from './span';

export interface HttpClientAdapter {
    fetch: typeof fetch;
    crypto: typeof crypto;
}

export interface TelemetryPort {
    addSpan: (span: SpanData) => void;
    updateSpan: (id: string, updates: TelemetrySpanUpdate) => void;
}

export interface SettingsPort {
    getSettings: () => { tadpole_os_url?: string; tadpole_os_api_key?: string };
}

// Metadata: [ports]
