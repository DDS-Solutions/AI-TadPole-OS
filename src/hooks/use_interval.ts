/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Lifecycle Utility**: Declarative `setInterval` wrapper with automatic cleanup and dynamic delay updates. 
 * Essential for low-frequency polling (e.g., heartbeat checks, UI refresh timers) without manual `useEffect` boilerplate.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Callback closure staleness (if `savedCallback` isn't updated), or interval drift during heavy main-thread load.
 * - **Telemetry Link**: Search for `[useInterval]` in browser performance profiles.
 */

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

