import { describe, it, expect, beforeEach } from 'vitest';

// Mock crypto.randomUUID
let uuidCounter = 0;
Object.defineProperty(globalThis, 'crypto', {
    value: { randomUUID: () => `uuid-${++uuidCounter}` },
    writable: true,
});

import { event_bus } from '../src/services/event_bus';

describe('event_bus', () => {
    beforeEach(() => {
        event_bus.destroy();
        uuidCounter = 0;
    });

    it('emits events to subscribers', () => {
        const received: string[] = [];
        event_bus.subscribe_logs((event) => received.push(event.text));

        event_bus.emit_log({ source: 'System', text: 'hello', severity: 'info' });

        expect(received).toContain('hello');
    });

    it('clearHistory preserves subscribers', () => {
        const received: string[] = [];
        event_bus.subscribe_logs((event) => received.push(event.text));

        event_bus.emit_log({ source: 'System', text: 'before-clear', severity: 'info' });
        event_bus.clear_history();

        // After clearHistory, the subscriber should still work
        event_bus.emit_log({ source: 'System', text: 'after-clear', severity: 'info' });

        expect(received).toContain('before-clear');
        expect(received).toContain('after-clear');
    });

    it('destroy removes subscribers', () => {
        const received: string[] = [];
        event_bus.subscribe_logs((event) => received.push(event.text));

        event_bus.destroy();

        event_bus.emit_log({ source: 'System', text: 'should-not-arrive', severity: 'info' });
        expect(received).not.toContain('should-not-arrive');
    });

    it('tracks event history', () => {
        event_bus.emit_log({ source: 'System', text: 'event1', severity: 'info' });
        event_bus.emit_log({ source: 'System', text: 'event2', severity: 'info' });

        const history = event_bus.get_history();
        expect(history.length).toBe(2);
        expect(history[0].text).toBe('event1');
        expect(history[1].text).toBe('event2');
    });

    it('clears history but keeps it accessible after', () => {
        event_bus.emit_log({ source: 'System', text: 'old', severity: 'info' });
        event_bus.clear_history();

        const history = event_bus.get_history();
        expect(history.length).toBe(0);

        event_bus.emit_log({ source: 'System', text: 'new', severity: 'info' });
        expect(event_bus.get_history().length).toBe(1);
    });

    it('assigns unique IDs to events', () => {
        event_bus.emit_log({ source: 'System', text: 'a', severity: 'info' });
        event_bus.emit_log({ source: 'System', text: 'b', severity: 'info' });

        const history = event_bus.get_history();
        expect(history[0].id).not.toBe(history[1].id);
    });

    it('caps history at 1000 entries via circular buffer', () => {
        // Emit 1001 events — circular buffer should retain the last 1000
        for (let i = 0; i < 1001; i++) {
            event_bus.emit_log({ source: 'System', text: `event-${i}`, severity: 'info' });
        }

        const history = event_bus.get_history();
        // Circular buffer: exactly 1000 entries retained (oldest dropped)
        expect(history.length).toBe(1000);
        // The oldest retained entry should be event-1 (event-0 was overwritten)
        expect(history[0].text).toBe('event-1');
        // The most recent event should be preserved
        expect(history[history.length - 1].text).toBe('event-1000');
    });
});



