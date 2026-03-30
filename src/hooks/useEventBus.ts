import { useState, useEffect, useCallback, useRef } from 'react';
import { EventBus, type LogEntry } from '../services/eventBus';

/**
 * Subscribes to the EventBus with automatic cleanup.
 * Optionally filters events by type and throttles high-frequency updates.
 *
 * @param filter   - Optional event type filter (e.g., "agent:message")
 * @param maxBuffer - Maximum buffer size (defaults to 200)
 */
export function useEventBus(
    filter?: string | ((entry: LogEntry) => boolean),
    maxBuffer = 200
) {
    const [entries, setEntries] = useState<LogEntry[]>([]);
    const bufferRef = useRef<LogEntry[]>([]);
    const rafRef = useRef<number | null>(null);

    const flush = useCallback(() => {
        if (bufferRef.current.length > 0) {
            setEntries(prev => {
                const merged = [...prev, ...bufferRef.current];
                bufferRef.current = [];
                return merged.length > maxBuffer ? merged.slice(-maxBuffer) : merged;
            });
        }
        rafRef.current = null;
    }, [maxBuffer]);

    useEffect(() => {
        const predicate = typeof filter === 'string'
            ? (e: LogEntry) => e.source === filter || e.text?.includes(filter)
            : filter;

        const unsubscribe = EventBus.subscribe((entry: LogEntry) => {
            if (predicate && !predicate(entry)) return;
            bufferRef.current.push(entry);
            if (!rafRef.current) {
                rafRef.current = requestAnimationFrame(flush);
            }
        });

        return () => {
            unsubscribe();
            if (rafRef.current) cancelAnimationFrame(rafRef.current);
        };
    }, [filter, flush]);

    const clear = useCallback(() => {
        setEntries([]);
        bufferRef.current = [];
    }, []);

    return { entries, clear, latest: entries[entries.length - 1] ?? null };
}
