import { useState, useRef, useCallback, useEffect } from 'react';

/**
 * A hook that provides a throttled update mechanism using requestAnimationFrame.
 * Essential for high-frequency telemetry data to keep the UI smooth (60fps).
 */
export function useThrottledStatus<T>(initialState: T, throttleMs: number = 100) {
    const [state, setState] = useState<T>(initialState);
    const lastUpdate = useRef<number>(0);
    const pendingState = useRef<T | null>(null);
    const rafId = useRef<number | null>(null);

    const updateThrottled = useCallback((newState: T) => {
        const now = Date.now();
        pendingState.current = newState;

        if (now - lastUpdate.current >= throttleMs) {
            // Immediate update if past throttle window
            setState(newState);
            lastUpdate.current = now;
            pendingState.current = null;
        } else if (rafId.current === null) {
            // Schedule an update for the next frame if not already scheduled
            rafId.current = requestAnimationFrame(() => {
                if (pendingState.current !== null) {
                    setState(pendingState.current);
                    lastUpdate.current = Date.now();
                    pendingState.current = null;
                }
                rafId.current = null;
            });
        }
    }, [throttleMs]);

    useEffect(() => {
        return () => {
            if (rafId.current !== null) {
                cancelAnimationFrame(rafId.current);
            }
        };
    }, []);

    return [state, updateThrottled] as const;
}
