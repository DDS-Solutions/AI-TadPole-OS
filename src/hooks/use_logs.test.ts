/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification Suite**: Log Persistence Hook. 
 * Validates the stateful subscription to the `event_bus` and the correct hydration of log history. 
 * Ensures the UI remains synchronized with real-time log streams.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing logs on initial mount or memory pressure from excessive state updates in high-velocity swarms.
 * - **Telemetry Link**: Run `npm run test` or search `[use_logs.test]` in Vitest logs.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useLogs } from './use_logs';
import { event_bus } from '../services/event_bus';

// Mock event_bus
vi.mock('../services/event_bus', () => ({
    event_bus: {
        get_history: vi.fn().mockReturnValue([]),
        subscribe_logs: vi.fn().mockReturnValue(() => {})
    }
}));

describe('useLogs', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Mock scrollIntoView
        window.HTMLElement.prototype.scrollIntoView = vi.fn();
    });

    it('should hydrate with history on mount', () => {
        const mock_history = [{ id: '1', text: 'Old log', severity: 'info', source: 'System', timestamp: new Date() }];
        vi.mocked(event_bus.get_history).mockReturnValue(mock_history as any);

        const { result } = renderHook(() => useLogs());

        expect(result.current.logs).toEqual(mock_history);
        expect(event_bus.subscribe_logs).toHaveBeenCalled();
    });

    it('should update logs when event_bus emits', () => {
        let captured_listener: any;
        vi.mocked(event_bus.subscribe_logs).mockImplementation((listener) => {
            captured_listener = listener;
            return () => {};
        });

        const { result } = renderHook(() => useLogs());

        act(() => {
            captured_listener({ id: '2', text: 'New log', severity: 'success', source: 'Agent', timestamp: new Date() });
        });

        expect(result.current.logs.length).toBeGreaterThanOrEqual(1);
        expect(result.current.logs[result.current.logs.length - 1].text).toBe('New log');
    });

    it('should maintain a maximum window of 100 logs', () => {
        let captured_listener: any;
        vi.mocked(event_bus.subscribe_logs).mockImplementation((listener) => {
            captured_listener = listener;
            return () => {};
        });

        const { result } = renderHook(() => useLogs());

        act(() => {
            for (let i = 0; i < 110; i++) {
                captured_listener({ id: `id-${i}`, text: `Log ${i}`, severity: 'info', source: 'System', timestamp: new Date() });
            }
        });

        expect(result.current.logs.length).toBe(100);
        expect(result.current.logs[0].text).toBe('Log 10');
        expect(result.current.logs[99].text).toBe('Log 109');
    });

    it('should unsubscribe on unmount', () => {
        const unsubscribe = vi.fn();
        vi.mocked(event_bus.subscribe_logs).mockReturnValue(unsubscribe);

        const { unmount } = renderHook(() => useLogs());
        unmount();

        expect(unsubscribe).toHaveBeenCalled();
    });
});

// Metadata: [use_logs_test]

// Metadata: [use_logs_test]
