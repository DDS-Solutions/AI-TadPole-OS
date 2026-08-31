/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / socket_manager.test
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
import { SocketManager } from './socket_manager';

describe('SocketManager', () => {
    let manager: SocketManager;

    beforeEach(() => {
        vi.useFakeTimers();
        manager = new SocketManager();
    });

    afterEach(() => {
        manager.disconnect();
        vi.useRealTimers();
    });

    it('initializes with disconnected state', () => {
        expect(manager.get_connection_state()).toBe('disconnected');
    });

    it('tracks status listeners and notifies current state on subscribe', () => {
        const listener = vi.fn();
        const unsub = manager.subscribe_status(listener);

        expect(listener).toHaveBeenCalledWith('disconnected');
        unsub();
    });

    it('handles reference counting on subscribe / unsubscribe', () => {
        const dummy_listener = vi.fn();
        const unsub1 = manager.subscribe('health', dummy_listener);
        const unsub2 = manager.subscribe('agentUpdates', dummy_listener);

        unsub1();
        unsub2();

        // Disconnect timer should trigger after all references unsubscribe
        vi.advanceTimersByTime(1500);
        expect(manager.get_connection_state()).toBe('disconnected');
    });

    it('safely handles send_json when disconnected', () => {
        const sent = manager.send_json({ command: 'ping' });
        expect(sent).toBe(false);
    });

    it('sets agent name cache on log channel', () => {
        expect(() => {
            manager.set_agent_name_cache([
                { id: 'agent-1', name: 'Lead Agent' }
            ]);
        }).not.toThrow();
    });
});
