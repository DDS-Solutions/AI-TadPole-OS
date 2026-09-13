/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / handshake
 * - **Primary Entrypoints**: `HandshakeHandler`, `Unsubscribe`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export type Unsubscribe = () => void;

export class HandshakeHandler {
    private auth_timeout_timer: ReturnType<typeof setTimeout> | null = null;
    private success_listeners = new Set<() => void>();
    private failure_listeners = new Set<(reason: string) => void>();
    private timeout_ms: number;

    constructor(timeout_ms = 10000) {
        this.timeout_ms = timeout_ms;
    }

    begin(ws: WebSocket, token: string): void {
        this.abort();

        // Send post-connect auth frame
        const auth_payload = { type: 'auth', token: token };
        ws.send(JSON.stringify(auth_payload));

        // Start auth timeout timer
        this.auth_timeout_timer = setTimeout(() => {
            this.trigger_failure('authentication timeout');
        }, this.timeout_ms);
    }

    handle_message(parsed: { type: string; message?: string }): boolean {
        if (parsed.type === 'auth_ok') {
            this.trigger_success();
            return true;
        }
        if (parsed.type === 'auth_error') {
            this.trigger_failure(parsed.message || 'Invalid credentials.');
            return true;
        }
        return false;
    }

    abort(): void {
        if (this.auth_timeout_timer) {
            clearTimeout(this.auth_timeout_timer);
            this.auth_timeout_timer = null;
        }
    }

    on_success(cb: () => void): Unsubscribe {
        this.success_listeners.add(cb);
        return () => this.success_listeners.delete(cb);
    }

    on_failure(cb: (reason: string) => void): Unsubscribe {
        this.failure_listeners.add(cb);
        return () => this.failure_listeners.delete(cb);
    }

    private trigger_success(): void {
        this.abort();
        this.success_listeners.forEach(cb => cb());
    }

    private trigger_failure(reason: string): void {
        this.abort();
        this.failure_listeners.forEach(cb => cb(reason));
    }
}
