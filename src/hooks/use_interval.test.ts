/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification Suite**: Declarative Interval Hook. 
 * Validates the lifecycle of the `useInterval` hook, ensuring timers are correctly cleared on unmount and dynamic delays are handled without stale closure issues.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Timer leak (setInterval not cleared) or incorrect execution frequency in high-latency React renders.
 * - **Telemetry Link**: Run `npm run test` or check `[use_interval.test]` in Vitest output.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useInterval } from './use_interval';

describe('useInterval', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
        vi.clearAllTimers();
    });

    it('should call the callback after the delay', () => {
        const callback = vi.fn();
        renderHook(() => useInterval(callback, 1000));

        expect(callback).not.toBeCalled();

        vi.advanceTimersByTime(1000);
        expect(callback).toHaveBeenCalledTimes(1);

        vi.advanceTimersByTime(1000);
        expect(callback).toHaveBeenCalledTimes(2);
    });

    it('should not call the callback if delay is null', () => {
        const callback = vi.fn();
        renderHook(() => useInterval(callback, null));

        vi.advanceTimersByTime(5000);
        expect(callback).not.toBeCalled();
    });

    it('should update the interval when delay changes', () => {
        const callback = vi.fn();
        const { rerender } = renderHook(
            ({ delay }) => useInterval(callback, delay),
            { initialProps: { delay: 1000 } }
        );

        vi.advanceTimersByTime(1000);
        expect(callback).toHaveBeenCalledTimes(1);

        rerender({ delay: 2000 });
        vi.advanceTimersByTime(1000); // Should not trigger yet
        expect(callback).toHaveBeenCalledTimes(1);

        vi.advanceTimersByTime(1000); // Total 2000 since rerender
        expect(callback).toHaveBeenCalledTimes(2);
    });

    it('should clear interval on unmount', () => {
        const callback = vi.fn();
        const { unmount } = renderHook(() => useInterval(callback, 1000));

        unmount();
        vi.advanceTimersByTime(1000);
        expect(callback).not.toBeCalled();
    });
});

// Metadata: [use_interval_test]

// Metadata: [use_interval_test]
