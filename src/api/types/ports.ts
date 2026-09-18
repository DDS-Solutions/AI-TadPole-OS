/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / ports
 * - **Primary Entrypoints**: `HttpClientAdapter`, `TelemetryPort`, `SettingsPort`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
