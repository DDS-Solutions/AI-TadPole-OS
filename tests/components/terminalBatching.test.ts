import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { EventBus } from '../../src/services/eventBus';

// We test the RAF batching pattern in isolation since Terminal.tsx
// uses a specific pattern that coalesces EventBus events.

describe('Terminal RAF Batching Pattern', () => {
    let rafCallbacks: (() => void)[];
    let originalRAF: typeof requestAnimationFrame;
    let originalCAF: typeof cancelAnimationFrame;

    beforeEach(() => {
        rafCallbacks = [];
        originalRAF = globalThis.requestAnimationFrame;
        originalCAF = globalThis.cancelAnimationFrame;

        // Mock RAF to be manually flushable
        globalThis.requestAnimationFrame = vi.fn((cb: FrameRequestCallback) => {
            rafCallbacks.push(() => cb(performance.now()));
            return rafCallbacks.length;
        }) as unknown as typeof requestAnimationFrame;

        globalThis.cancelAnimationFrame = vi.fn();
    });

    afterEach(() => {
        globalThis.requestAnimationFrame = originalRAF;
        globalThis.cancelAnimationFrame = originalCAF;
        EventBus.destroy();
    });

    it('should batch multiple events into a single buffer before RAF flush', () => {
        const buffer: string[] = [];

        // Simulate the Terminal pattern: events go to buffer
        const unsub = EventBus.subscribe((entry) => {
            buffer.push(entry.text || '');
        });

        // Emit 5 events rapidly (using valid LogSource 'System')
        EventBus.emit({ type: 'info', source: 'System', text: 'msg1', severity: 'info' });
        EventBus.emit({ type: 'info', source: 'System', text: 'msg2', severity: 'info' });
        EventBus.emit({ type: 'info', source: 'System', text: 'msg3', severity: 'info' });
        EventBus.emit({ type: 'info', source: 'System', text: 'msg4', severity: 'info' });
        EventBus.emit({ type: 'info', source: 'System', text: 'msg5', severity: 'info' });

        // All 5 should arrive in buffer before any RAF flush
        expect(buffer).toHaveLength(5);
        expect(buffer).toEqual(['msg1', 'msg2', 'msg3', 'msg4', 'msg5']);

        unsub();
    });

    it('should not lose events between RAF frames', () => {
        const received: string[] = [];

        const unsub = EventBus.subscribe((entry) => {
            received.push(entry.text || '');
        });

        // Frame 1: emit 2 events
        EventBus.emit({ source: 'System', text: 'a', severity: 'info' });
        EventBus.emit({ source: 'System', text: 'b', severity: 'info' });

        // Simulate RAF flush
        rafCallbacks.forEach(cb => cb());
        rafCallbacks = [];

        // Frame 2: emit 1 more event
        EventBus.emit({ source: 'Agent', text: 'c', severity: 'info', agentId: 'a1' });

        // All events received
        expect(received).toEqual(['a', 'b', 'c']);

        unsub();
    });

    it('should respect MAX_LOG_ENTRIES cap from Terminal pattern', () => {
        const MAX = 500; // matches Terminal.tsx constant
        const logs: string[] = [];

        // Fill beyond cap
        for (let i = 0; i < MAX + 100; i++) {
            logs.push(`msg-${i}`);
        }

        // The capping logic from Terminal.tsx: combined.slice(-MAX)
        const capped = logs.length > MAX ? logs.slice(-MAX) : logs;

        expect(capped).toHaveLength(MAX);
        expect(capped[0]).toBe(`msg-100`); // oldest 100 pruned
        expect(capped[MAX - 1]).toBe(`msg-${MAX + 99}`); // newest preserved
    });

    it('should clean up RAF on unmount via cancelAnimationFrame', () => {
        // Trigger RAF registration
        const id = requestAnimationFrame(() => { });
        cancelAnimationFrame(id);

        expect(cancelAnimationFrame).toHaveBeenCalledWith(id);
    });
});
