/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / handshake.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { HandshakeHandler } from './handshake';

describe('HandshakeHandler', () => {
    let mock_ws: any;

    beforeEach(() => {
        vi.useFakeTimers();
        mock_ws = {
            send: vi.fn()
        };
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('begin sends auth frame and starts timeout', () => {
        const handler = new HandshakeHandler(5000);
        const on_failure = vi.fn();
        handler.on_failure(on_failure);

        handler.begin(mock_ws, 'test-token');

        expect(mock_ws.send).toHaveBeenCalledWith(JSON.stringify({ type: 'auth', token: 'test-token' }));

        vi.advanceTimersByTime(5000);
        expect(on_failure).toHaveBeenCalledWith('authentication timeout');
    });

    it('handle_message triggers on_success on auth_ok', () => {
        const handler = new HandshakeHandler(5000);
        const on_success = vi.fn();
        handler.on_success(on_success);

        handler.begin(mock_ws, 'test-token');
        const handled = handler.handle_message({ type: 'auth_ok' });

        expect(handled).toBe(true);
        expect(on_success).toHaveBeenCalledTimes(1);

        // Advance timers - failure should NOT be called because timer was aborted
        const on_failure = vi.fn();
        handler.on_failure(on_failure);
        vi.advanceTimersByTime(5000);
        expect(on_failure).not.toHaveBeenCalled();
    });

    it('handle_message triggers on_failure on auth_error', () => {
        const handler = new HandshakeHandler(5000);
        const on_failure = vi.fn();
        handler.on_failure(on_failure);

        handler.begin(mock_ws, 'test-token');
        const handled = handler.handle_message({ type: 'auth_error', message: 'Token expired' });

        expect(handled).toBe(true);
        expect(on_failure).toHaveBeenCalledWith('Token expired');
    });

    it('handle_message returns false on non-auth messages', () => {
        const handler = new HandshakeHandler(5000);
        const handled = handler.handle_message({ type: 'log' });
        expect(handled).toBe(false);
    });

    it('abort cancels the timer', () => {
        const handler = new HandshakeHandler(5000);
        const on_failure = vi.fn();
        handler.on_failure(on_failure);

        handler.begin(mock_ws, 'test-token');
        handler.abort();

        vi.advanceTimersByTime(5000);
        expect(on_failure).not.toHaveBeenCalled();
    });
});
