import { useEffect, useRef } from 'react';

/**
 * Declarative setInterval hook with automatic cleanup.
 * Handles dynamic delay changes and pausing (delay = null).
 *
 * @param callback - Function to call on each interval tick
 * @param delay    - Interval in ms, or null to pause
 */
export function useInterval(callback: () => void, delay: number | null) {
    const savedCallback = useRef(callback);

    // Remember the latest callback
    useEffect(() => {
        savedCallback.current = callback;
    }, [callback]);

    useEffect(() => {
        if (delay === null) return;

        const id = setInterval(() => savedCallback.current(), delay);
        return () => clearInterval(id);
    }, [delay]);
}
