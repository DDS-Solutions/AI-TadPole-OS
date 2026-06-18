/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Lifecycle Guardian**: Standardized management for high-frequency component mounts/unmounts. 
 * Orchestrates `AbortController` propagation to prevent memory leaks and "Update on Unmounted Component" errors during rapid navigation.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Abort signal leakage (failing to pass `getSignal()` to fetch), or `isMounted` state desync on concurrent effect execution.
 * - **Telemetry Link**: Search for `COMPONENT_UNMOUNTED` or `[useSafeLifecycle]` in trace logs.
 */

import { useEffect, useRef, useCallback, useMemo } from 'react';

/**
 * Standardized lifecycle management for high-frequency components.
 * Provides an AbortController and safety guards for async operations.
 * 
 * USE CASE: Neural Waterfall, Lineage Stream, and Telemetry Charts
 * to prevent memory leaks during rapid context switching or detachment.
 */
export function useSafeLifecycle() {
    const isMounted = useRef(true);
    const abortControllerRef = useRef<AbortController | null>(null);

    useEffect(() => {
        isMounted.current = true;
        abortControllerRef.current = new AbortController();

        return () => {
            isMounted.current = false;
            if (abortControllerRef.current) {
                abortControllerRef.current.abort('COMPONENT_UNMOUNTED');
            }
        };
    }, []);

    /**
     * Executes a task only if the component is still mounted.
     */
    const safeRun = useCallback(<T>(task: () => T | Promise<T>): T | Promise<T> | void => {
        if (!isMounted.current) return;
        return task();
    }, []);

    /**
     * Returns a fresh AbortSignal for this component's current mount lifecycle.
     */
    const getSignal = useCallback(() => {
        if (!abortControllerRef.current) {
            abortControllerRef.current = new AbortController();
        }
        return abortControllerRef.current.signal;
    }, []);

    /**
     * Safe wrapper for setTimeouts that automatically clear on unmount.
     */
    const safeTimeout = useCallback((callback: () => void, delay: number) => {
        const id = setTimeout(() => {
            if (isMounted.current) callback();
        }, delay);
        return () => clearTimeout(id);
    }, []);

    const getIsMounted = useCallback(() => isMounted.current, []);

    return useMemo(() => ({
        isMounted: getIsMounted,
        safeRun,
        getSignal,
        safeTimeout
    }), [getIsMounted, safeRun, getSignal, safeTimeout]);
}


// Metadata: [use_safe_lifecycle]

// Metadata: [use_safe_lifecycle]
