/**
 * @module EventBus
 * Central pub/sub service that synchronizes the Terminal, Voice Comms,
 * and WebSocket log stream into a single unified event timeline.
 */

/** Origin of a log entry. */
type LogSource = 'User' | 'System' | 'Agent';

/** Visual severity used for color-coding in the Terminal UI. */
type LogSeverity = 'info' | 'success' | 'warning' | 'error';

/** A single event in the unified command timeline. */
export interface LogEntry {
    /** Unique identifier (auto-generated). */
    id: string;
    /** When the event occurred (auto-generated). */
    timestamp: Date;
    /** Who produced this entry. */
    source: LogSource;
    /** Human-readable message content. */
    text: string;
    /** Severity level for UI color-coding. */
    severity: LogSeverity;
    /** The originating agent's ID, if `source` is `'Agent'`. */
    agentId?: string;
    /** The originating agent's friendly name, if available. */
    agentName?: string;
    /** The associated mission (cluster) ID, if applicable. */
    missionId?: string;
    /** RFC 9457 Error URI for machine-readable error handling. */
    typeId?: string;
    /** Flexible metadata for extended diagnostic display. */
    metadata?: Record<string, unknown>;
}

type Listener = (entry: LogEntry) => void;

/**
 * Lightweight pub/sub event bus.
 * Components subscribe to receive {@link LogEntry} objects in real time.
 * History uses a true circular buffer (no array reallocation).
 */
class EventBusService {
    private listeners: Listener[] = [];

    /** Circular buffer for history — avoids array reallocation on overflow. */
    private static readonly BUFFER_SIZE = 1000;
    private ring: (LogEntry | null)[] = new Array(EventBusService.BUFFER_SIZE).fill(null);
    private head = 0;   // write pointer
    private count = 0;  // number of entries currently stored
    private channel: BroadcastChannel | null = typeof window !== 'undefined' ? new BroadcastChannel('tadpole-event-bus-sync') : null;
    /** Track recently processed IDs to prevent duplication from cross-tab sync. */
    private processedIds = new Set<string>();
    private static readonly MAX_ID_CACHE = 500;

    constructor() {
        if (this.channel) {
            this.channel.onmessage = (event) => {
                if (event.data?.type === 'EVENT_EMIT' && event.data.payload) {
                    this.internalEmit(event.data.payload, false);
                }
            };
        }
    }

    /** Subscribe to all future events. Returns an unsubscribe function. */
    subscribe(listener: Listener): () => void {
        this.listeners.push(listener);
        return () => {
            this.listeners = this.listeners.filter(l => l !== listener);
        };
    }

    /** Emit an event to all subscribers. `id` and `timestamp` are auto-filled if not provided. */
    emit(entry: Omit<LogEntry, 'id' | 'timestamp'> & { id?: string; timestamp?: Date }) {
        const fullEntry: LogEntry = {
            id: entry.id || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : Math.random().toString(36).substring(2)),
            timestamp: entry.timestamp || new Date(),
            source: entry.source,
            text: entry.text,
            severity: entry.severity,
            agentId: entry.agentId,
            agentName: entry.agentName,
            missionId: entry.missionId,
            typeId: entry.typeId,
            metadata: entry.metadata
        };
        this.internalEmit(fullEntry, true);
    }

    private internalEmit(fullEntry: LogEntry, broadcast: boolean) {
        // Deduplication: prevent identical IDs from being re-processed
        if (this.processedIds.has(fullEntry.id)) {
            return;
        }

        // Maintain ID cache size
        if (this.processedIds.size >= EventBusService.MAX_ID_CACHE) {
            // Simple approach: clear half the cache when full to avoid frequent churn
            const idsArray = Array.from(this.processedIds);
            for (let i = 0; i < EventBusService.MAX_ID_CACHE / 2; i++) {
                this.processedIds.delete(idsArray[i]);
            }
        }
        this.processedIds.add(fullEntry.id);

        // Write to circular buffer (O(1), no allocation)
        this.ring[this.head] = fullEntry;
        this.head = (this.head + 1) % EventBusService.BUFFER_SIZE;
        if (this.count < EventBusService.BUFFER_SIZE) this.count++;

        this.listeners.forEach(listener => {
            try {
                listener(fullEntry);
            } catch (error) {
                console.error('[EventBus] Error in listener:', error);
            }
        });

        if (broadcast && this.channel) {
            this.channel.postMessage({ type: 'EVENT_EMIT', payload: fullEntry });
        }
    }

    /** Returns a chronologically ordered copy of all stored history. */
    getHistory(): LogEntry[] {
        if (this.count === 0) return [];
        const result: LogEntry[] = [];
        const start = this.count < EventBusService.BUFFER_SIZE
            ? 0
            : this.head; // oldest entry is at head when buffer is full
        for (let i = 0; i < this.count; i++) {
            const idx = (start + i) % EventBusService.BUFFER_SIZE;
            if (this.ring[idx]) result.push(this.ring[idx]!);
        }
        return result;
    }

    /** Clears event history but keeps all subscribers intact. Safe for `/clear`. */
    clearHistory() {
        this.ring = new Array(EventBusService.BUFFER_SIZE).fill(null);
        this.head = 0;
        this.count = 0;
        this.processedIds.clear();
    }

    /** Full teardown: clears history AND removes all subscribers. Use on unmount. */
    destroy() {
        this.clearHistory();
        this.listeners = [];
    }
}

/** Singleton instance shared across the entire application. */
export const EventBus = new EventBusService();
