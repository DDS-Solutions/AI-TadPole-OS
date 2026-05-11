/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification Suite**: Infrastructure Event Bus. 
 * Validates the core Pub/Sub logic, circular buffer management (1,000 log limit), and cross-tab synchronization via `BroadcastChannel`. 
 * Ensures high-velocity telemetry pulses do not block the UI thread.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memory leak if subscriptions aren't cleaned up or BroadcastChannel deadlock in multi-tab environments.
 * - **Telemetry Link**: Run `npm run test` or check for `[event_bus.test]` in Vitest traces.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// ✅ FIX: Stub global BEFORE importing the service
vi.hoisted(() => {
    class MockBroadcastChannel {
        onmessage: any = null;
        postMessage() {} // Placeholder for spying
        close() {}
    }
    vi.stubGlobal('BroadcastChannel', MockBroadcastChannel);
});

import { event_bus } from './event_bus';

describe('event_bus', () => {
    let mockPostMessage: any;

    beforeEach(() => {
        event_bus.destroy();
        vi.clearAllMocks();
        // Spy on the prototype of our stubbed global
        mockPostMessage = vi.spyOn(BroadcastChannel.prototype, 'postMessage');
    });

    it('emits and stores logs', () => {
        const listener = vi.fn();
        event_bus.subscribe_logs(listener);

        event_bus.emit_log({
            source: 'System',
            text: 'Test message',
            severity: 'info'
        });

        expect(listener).toHaveBeenCalled();
        const history = event_bus.get_history();
        expect(history.length).toBe(1);
        expect(history[0].text).toBe('Test message');
    });

    it('manages subscriptions and cleanup', () => {
        const listener = vi.fn();
        const unsubscribe = event_bus.subscribe_logs(listener);

        event_bus.emit_log({ source: 'System', text: 'Msg 1', severity: 'info' });
        expect(listener).toHaveBeenCalledTimes(1);

        unsubscribe();
        event_bus.emit_log({ source: 'System', text: 'Msg 2', severity: 'info' });
        expect(listener).toHaveBeenCalledTimes(1);
    });

    it('maintains a circular buffer', () => {
        for (let i = 0; i < 1100; i++) {
            event_bus.emit_log({
                source: 'System',
                text: `Message ${i}`,
                severity: 'info',
                id: `id-${i}`
            });
        }

        const history = event_bus.get_history();
        expect(history.length).toBe(1000);
        expect(history[0].text).toBe('Message 100');
        expect(history[999].text).toBe('Message 1099');
    });

    it('deduplicates events with the same ID', () => {
        const listener = vi.fn();
        event_bus.subscribe_logs(listener);

        event_bus.emit_log({ id: 'dup-1', source: 'System', text: 'Msg', severity: 'info' });
        event_bus.emit_log({ id: 'dup-1', source: 'System', text: 'Msg', severity: 'info' });

        expect(listener).toHaveBeenCalledTimes(1);
        expect(event_bus.get_history().length).toBe(1);
    });

    it('broadcasts events to other tabs', () => {
        event_bus.emit_log({ source: 'System', text: 'Sync this', severity: 'info' });
        expect(mockPostMessage).toHaveBeenCalledWith(expect.objectContaining({
            type: 'EVENT_EMIT',
            payload: expect.objectContaining({ text: 'Sync this' })
        }));
    });

    it('clears history correctly', () => {
        event_bus.emit_log({ source: 'System', text: 'Clear me', severity: 'info' });
        expect(event_bus.get_history().length).toBe(1);

        event_bus.clear_history();
        expect(event_bus.get_history().length).toBe(0);
    });

    it('handles listener errors gracefully', () => {
        const console_spy = vi.spyOn(console, 'error').mockImplementation(() => {});
        event_bus.subscribe_logs(() => { throw new Error('Boom'); });
        event_bus.emit_log({ source: 'System', text: 'Safe', severity: 'info' });
        expect(console_spy).toHaveBeenCalled();
        console_spy.mockRestore();
    });
});

// Metadata: [event_bus_test]

// Metadata: [event_bus_test]
