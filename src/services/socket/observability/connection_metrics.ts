/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / connection_metrics
 * - **Primary Entrypoints**: `ConnectionMetrics`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
}
