import { useState, useEffect, useRef } from 'react';
import { event_bus, type log_entry } from '../services/event_bus';

/**
 * use_logs
 * Reusable hook to subscribe to the global event_bus and manage the log history state.
 * Refactored for strict snake_case compliance and consistent prop propagation.
 */
export function use_logs() {
    const [logs, set_logs] = useState<log_entry[]>(() => event_bus.get_history());
    const logs_end_ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        // Subscribe to real-time events
        const unsubscribe_logs = event_bus.subscribe_logs((entry) => {
            set_logs(prev => [...prev, entry].slice(-100)); // Maintain local window of 100 logs
        });

        return () => {
            unsubscribe_logs();
        };
    }, []);

    // Auto-scroll logic: triggered whenever logs change
    useEffect(() => {
        if (logs_end_ref.current) {
            logs_end_ref.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [logs]);

    return { logs, logs_end_ref };
}
