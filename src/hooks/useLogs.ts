import { useState, useEffect, useRef } from 'react';
import { EventBus, type LogEntry } from '../services/eventBus';

/**
 * useLogs
 * Reusable hook to subscribe to the global EventBus and manage the log history state.
 * Includes auto-scroll target reference for log views.
 */
export function useLogs() {
    const [logs, setLogs] = useState<LogEntry[]>(() => EventBus.getHistory());
    const logsEndRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        // Subscribe to real-time events
        const unsubscribeLogs = EventBus.subscribe((entry) => {
            setLogs(prev => [...prev, entry].slice(-100)); // Maintain local window of 100 logs
        });

        // NOTE: Manual history sync removed to satisfy 'react-hooks/set-state-in-effect'
        // Initial state is already populated via the useState initializer.

        return () => {
            unsubscribeLogs();
        };
    }, []);

    // Auto-scroll logic: triggered whenever logs change
    useEffect(() => {
        if (logsEndRef.current) {
            logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [logs]);

    return { logs, logsEndRef };
}
