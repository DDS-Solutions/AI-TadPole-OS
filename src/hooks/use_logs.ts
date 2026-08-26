/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_logs
 * - **Primary Entrypoints**: `useLogs`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useEffect, useRef } from 'react';
import { event_bus, type log_entry } from '../services/event_bus';

/**
 * use_logs
 * Reusable hook to subscribe to the global event_bus and manage the log history state.
 * Refactored for strict snake_case compliance and consistent prop propagation.
 */
export function useLogs() {
    const [logs, set_logs] = useState<log_entry[]>(() => event_bus.get_history());
    const logs_end_ref = useRef<HTMLDivElement>(null);
    const scroll_frame_ref = useRef<number | null>(null);

    useEffect(() => {
        // Subscribe to real-time events
        const unsubscribe_logs = event_bus.subscribe_logs((entry) => {
            set_logs(prev => [...prev, entry].slice(-100)); // Maintain local window of 100 logs
        });

        return () => {
            unsubscribe_logs();
        };
    }, []);

    // Auto-scroll logic: throttled via requestAnimationFrame to eliminate smooth-scroll layout thrashing
    useEffect(() => {
        if (!logs_end_ref.current) return;
        if (scroll_frame_ref.current !== null) {
            cancelAnimationFrame(scroll_frame_ref.current);
        }
        scroll_frame_ref.current = requestAnimationFrame(() => {
            if (logs_end_ref.current) {
                logs_end_ref.current.scrollIntoView({ behavior: 'smooth' });
            }
            scroll_frame_ref.current = null;
        });

        return () => {
            if (scroll_frame_ref.current !== null) {
                cancelAnimationFrame(scroll_frame_ref.current);
            }
        };
    }, [logs.length]);

    return { logs, logs_end_ref };
}
