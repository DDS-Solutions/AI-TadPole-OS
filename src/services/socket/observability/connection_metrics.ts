/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[connection_metrics]` in observability traces.
 */

export class ConnectionMetrics {
    public messages_received = 0;
    public decode_errors = 0;
    public reconnects = 0;
    public queue_drops = 0;
    public auth_timeouts = 0;
    public oversized_frames_dropped = 0;

    reset(): void {
        this.messages_received = 0;
        this.decode_errors = 0;
        this.reconnects = 0;
        this.queue_drops = 0;
        this.auth_timeouts = 0;
        this.oversized_frames_dropped = 0;
    }

    get_snapshot() {
        return {
            messages_received: this.messages_received,
            decode_errors: this.decode_errors,
            reconnects: this.reconnects,
            queue_drops: this.queue_drops,
            auth_timeouts: this.auth_timeouts,
            oversized_frames_dropped: this.oversized_frames_dropped,
        };
    }
}

// Metadata: [connection_metrics]
