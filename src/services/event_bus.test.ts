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
        event_bus.reset();
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
            topic: 'LOG',
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

    it('safely ignores null or invalid payloads in handle_sync_response', () => {
        const console_spy = vi.spyOn(console, 'error').mockImplementation(() => {});
        (event_bus as any).handle_sync_response(null);
        (event_bus as any).handle_sync_response(undefined);
        (event_bus as any).handle_sync_response("invalid");
        (event_bus as any).handle_sync_response({ logs: "not-an-array" });
        expect(console_spy).not.toHaveBeenCalled();
        console_spy.mockRestore();
    });

    it('correctly truncates metadata and preserves native dates/regexes and arrays', () => {
        const date = new Date(1717777777777);
        const regex = /abc/g;
        const metadata = {
            nested: {
                str: 'a'.repeat(20000),
                arr: ['item1', 'item2'],
                date,
                regex,
                __proto__: { polluted: true }
            }
        };

        const result = (event_bus as any).truncate_large_metadata(metadata);
        expect(result.nested.str.length).toBe(10000 + '... [TRUNCATED]'.length);
        expect(result.nested.arr).toEqual(['item1', 'item2']);
        expect(result.nested.date).toBeInstanceOf(Date);
        expect(result.nested.date.getTime()).toBe(date.getTime());
        expect(result.nested.regex).toBeInstanceOf(RegExp);
        expect(result.nested.regex.source).toBe('abc');
        expect((result.nested as any).polluted).toBeUndefined();
    });

    it('handles DataCloneError in postMessage gracefully and falls back to stripping metadata', () => {
        const console_spy = vi.spyOn(console, 'error').mockImplementation(() => {});
        mockPostMessage.mockImplementationOnce(() => {
            throw new DOMException('DataCloneError', 'DataCloneError');
        });

        event_bus.emit_log({
            source: 'System',
            text: 'Trigger clone error',
            severity: 'info',
            metadata: { nonCloneable: () => {} }
        });

        expect(console_spy).toHaveBeenCalled();
        expect(mockPostMessage).toHaveBeenCalledTimes(2);
        expect(mockPostMessage.mock.calls[1][0].payload.metadata).toBeUndefined();
        console_spy.mockRestore();
    });

    it('applies backoff and suppression to SYNC_REQUEST responses', async () => {
        vi.useFakeTimers();
        const postMessageSpy = vi.spyOn(BroadcastChannel.prototype, 'postMessage');
        
        event_bus.emit_log({ source: 'System', text: 'History log', severity: 'info' });
        postMessageSpy.mockClear();

        (event_bus as any).handle_sync_request();
        expect(postMessageSpy).not.toHaveBeenCalled();

        const onmessage_handler = (event_bus as any).channel.onmessage;
        onmessage_handler({
            data: {
                topic: 'SYNC_RESPONSE',
                payload: { logs: [] },
                timestamp: Date.now(),
                sender_id: 'other-tab'
            }
        });

        vi.runAllTimers();
        expect(postMessageSpy).not.toHaveBeenCalled();
        vi.useRealTimers();
    });

    it('enforces input validation in emit_log', () => {
        expect(() => event_bus.emit_log(null as any)).toThrow(TypeError);
        expect(() => event_bus.emit_log({ source: 'Invalid' as any, text: 'hi', severity: 'info' })).toThrow(TypeError);
        expect(() => event_bus.emit_log({ source: 'System', text: 'hi', severity: 'invalid' as any })).toThrow(TypeError);
        expect(() => event_bus.emit_log({ source: 'System', text: 123 as any, severity: 'info' })).toThrow(TypeError);
    });

    it('clears pending timeouts and closes channel on destroy', () => {
        vi.useFakeTimers();
        event_bus.emit_log({ source: 'System', text: 'Log', severity: 'info' });
        (event_bus as any).handle_sync_request();
        expect((event_bus as any).pending_sync_response).not.toBeNull();

        const closeSpy = vi.spyOn((event_bus as any).channel, 'close');
        event_bus.destroy();

        expect((event_bus as any).pending_sync_response).toBeNull();
        expect(closeSpy).toHaveBeenCalled();
        expect((event_bus as any).channel).toBeNull();
        vi.useRealTimers();
    });

    it('generates secure IDs using fallback mechanisms when crypto.randomUUID is missing', () => {
        const originalUUID = crypto.randomUUID;
        // @ts-expect-error: mocking missing randomUUID
        crypto.randomUUID = undefined;

        const getValuesSpy = vi.spyOn(crypto, 'getRandomValues');

        const id = (event_bus as any).generate_secure_id();
        expect(id).toBeDefined();
        expect(typeof id).toBe('string');
        expect(getValuesSpy).toHaveBeenCalled();

        // Restore
        // @ts-expect-error: restoring original randomUUID
        crypto.randomUUID = originalUUID;
        getValuesSpy.mockRestore();
    });

    it('caps receiver-side sync processing payload to the last 200 logs (EVT-010)', () => {
        const incoming_logs: log_entry[] = [];
        for (let i = 0; i < 300; i++) {
            incoming_logs.push({
                id: `sync-cap-${i}`,
                timestamp: new Date(1717777777000 + i * 1000),
                source: 'System',
                text: `Log ${i}`,
                severity: 'info'
            });
        }

        (event_bus as any).handle_sync_response({ logs: incoming_logs });

        const history = event_bus.get_history();
        expect(history.length).toBe(200);
        expect(history[0].text).toBe('Log 100');
        expect(history[199].text).toBe('Log 299');
    });
});

// Metadata: [event_bus_test]
