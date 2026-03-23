import { describe, it, expect, beforeEach } from 'vitest';

// Mock crypto.randomUUID
let uuidCounter = 0;
Object.defineProperty(globalThis, 'crypto', {
    value: { randomUUID: () => `uuid-${++uuidCounter}` },
    writable: true,
});

import { EventBus } from '../src/services/eventBus';

describe('EventBus', () => {
    beforeEach(() => {
        EventBus.destroy();
        uuidCounter = 0;
    });

    it('emits events to subscribers', () => {
        const received: string[] = [];
        EventBus.subscribe((event) => received.push(event.text));

        EventBus.emit({ source: 'System', text: 'hello', severity: 'info' });

        expect(received).toContain('hello');
    });

    it('clearHistory preserves subscribers', () => {
        const received: string[] = [];
        EventBus.subscribe((event) => received.push(event.text));

        EventBus.emit({ source: 'System', text: 'before-clear', severity: 'info' });
        EventBus.clearHistory();

        // After clearHistory, the subscriber should still work
        EventBus.emit({ source: 'System', text: 'after-clear', severity: 'info' });

        expect(received).toContain('before-clear');
        expect(received).toContain('after-clear');
    });

    it('destroy removes subscribers', () => {
        const received: string[] = [];
        EventBus.subscribe((event) => received.push(event.text));

        EventBus.destroy();

        EventBus.emit({ source: 'System', text: 'should-not-arrive', severity: 'info' });
        expect(received).not.toContain('should-not-arrive');
    });

    it('tracks event history', () => {
        EventBus.emit({ source: 'System', text: 'event1', severity: 'info' });
        EventBus.emit({ source: 'System', text: 'event2', severity: 'info' });

        const history = EventBus.getHistory();
        expect(history.length).toBe(2);
        expect(history[0].text).toBe('event1');
        expect(history[1].text).toBe('event2');
    });

    it('clears history but keeps it accessible after', () => {
        EventBus.emit({ source: 'System', text: 'old', severity: 'info' });
        EventBus.clearHistory();

        const history = EventBus.getHistory();
        expect(history.length).toBe(0);

        EventBus.emit({ source: 'System', text: 'new', severity: 'info' });
        expect(EventBus.getHistory().length).toBe(1);
    });

    it('assigns unique IDs to events', () => {
        EventBus.emit({ source: 'System', text: 'a', severity: 'info' });
        EventBus.emit({ source: 'System', text: 'b', severity: 'info' });

        const history = EventBus.getHistory();
        expect(history[0].id).not.toBe(history[1].id);
    });

    it('caps history at 1000 entries via circular buffer', () => {
        // Emit 1001 events — circular buffer should retain the last 1000
        for (let i = 0; i < 1001; i++) {
            EventBus.emit({ source: 'System', text: `event-${i}`, severity: 'info' });
        }

        const history = EventBus.getHistory();
        // Circular buffer: exactly 1000 entries retained (oldest dropped)
        expect(history.length).toBe(1000);
        // The oldest retained entry should be event-1 (event-0 was overwritten)
        expect(history[0].text).toBe('event-1');
        // The most recent event should be preserved
        expect(history[history.length - 1].text).toBe('event-1000');
    });
});
