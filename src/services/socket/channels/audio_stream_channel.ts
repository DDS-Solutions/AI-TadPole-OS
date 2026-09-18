/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / audio_stream_channel
 * - **Primary Entrypoints**: `AudioStreamChannel`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { BaseChannel } from './channel';

export class AudioStreamChannel extends BaseChannel<ArrayBuffer> {
    readonly name = 'audio_stream';

    matches(): boolean {
        return false; // Binary packets are routed explicitly, not via JSON handle
    }

    handle(): void {
        // No-op for JSON messages
    }

    handle_binary(data: ArrayBuffer): void {
        this.emit(data);
    }
}
