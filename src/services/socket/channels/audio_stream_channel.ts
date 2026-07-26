/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[audio_stream_channel]` in observability traces.
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

// Metadata: [audio_stream_channel]
