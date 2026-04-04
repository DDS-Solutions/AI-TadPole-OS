/**
 * @module event_bus
 * Central pub/sub service that synchronizes the Terminal, Voice Comms,
 * and WebSocket log stream into a single unified event timeline.
 */

/** Origin of a log entry. */
type log_source = 'User' | 'System' | 'Agent';

/** Visual severity used for color-coding in the Terminal UI. */
type log_severity = 'info' | 'success' | 'warning' | 'error';

/** A single event in the unified command timeline. */
export interface log_entry {
    /** Unique identifier (auto-generated). */
    id: string;
    /** When the event occurred (auto-generated). */
    timestamp: Date;
    /** Who produced this entry. */
    source: log_source;
    /** Human-readable message content. */
    text: string;
    /** Severity level for UI color-coding. */
    severity: log_severity;
    /** The originating agent's ID, if `source` is `'Agent'`. */
    agent_id?: string;
    /** The originating agent's friendly name, if available. */
    agent_name?: string;
    /** The associated mission (cluster) ID, if applicable. */
    mission_id?: string;
    /** RFC 9457 Error URI for machine-readable error handling. */
    type_id?: string;
    /** Flexible metadata for extended diagnostic display. */
    metadata?: Record<string, unknown>;
}

type Listener = (entry: log_entry) => void;

/**
 * Lightweight pub/sub event bus.
 * Components subscribe to receive {@link log_entry} objects in real time.
 * History uses a true circular buffer (no array reallocation).
 */
class event_bus_service {
    private listeners: Listener[] = [];

    /** Circular buffer for history — avoids array reallocation on overflow. */
    private static readonly BUFFER_SIZE = 1000;
    private ring: (log_entry | null)[] = new Array(event_bus_service.BUFFER_SIZE).fill(null);
    private head = 0;   // write pointer
    private count = 0;  // number of entries currently stored
    private channel: BroadcastChannel | null = typeof window !== 'undefined' ? new BroadcastChannel('tadpole-event-bus-sync') : null;
    /** Track recently processed IDs to prevent duplication from cross-tab sync. */
    private processed_ids = new Set<string>();
    private static readonly MAX_ID_CACHE = 500;

    constructor() {
        if (this.channel) {
            this.channel.onmessage = (event) => {
                if (event.data?.type === 'EVENT_EMIT' && event.data.payload) {
                    this.internal_emit(event.data.payload, false);
                }
            };
        }
    }

    /** Subscribe to all future events. Returns an unsubscribe function. */
    subscribe_logs(listener: Listener): () => void {
        this.listeners.push(listener);
        return () => {
            this.listeners = this.listeners.filter(l => l !== listener);
        };
    }

    /** Emit an event to all subscribers. `id` and `timestamp` are auto-filled if not provided. */
    emit_log(entry: Omit<log_entry, 'id' | 'timestamp'> & { id?: string; timestamp?: Date }) {
        const full_entry: log_entry = {
            id: entry.id || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : Math.random().toString(36).substring(2)),
            timestamp: entry.timestamp || new Date(),
            source: entry.source,
            text: entry.text,
            severity: entry.severity,
            agent_id: entry.agent_id,
            agent_name: entry.agent_name,
            mission_id: entry.mission_id,
            type_id: entry.type_id,
            metadata: entry.metadata
        };
        this.internal_emit(full_entry, true);
    }

    private internal_emit(full_entry: log_entry, broadcast: boolean) {
        // Deduplication: prevent identical IDs from being re-processed
        if (this.processed_ids.has(full_entry.id)) {
            return;
        }

        // Maintain ID cache size
        if (this.processed_ids.size >= event_bus_service.MAX_ID_CACHE) {
            // Simple approach: clear half the cache when full to avoid frequent churn
            const idsArray = Array.from(this.processed_ids);
            for (let i = 0; i < event_bus_service.MAX_ID_CACHE / 2; i++) {
                this.processed_ids.delete(idsArray[i]);
            }
        }
        this.processed_ids.add(full_entry.id);

        // Write to circular buffer (O(1), no allocation)
        this.ring[this.head] = full_entry;
        this.head = (this.head + 1) % event_bus_service.BUFFER_SIZE;
        if (this.count < event_bus_service.BUFFER_SIZE) this.count++;

        this.listeners.forEach(listener => {
            try {
                listener(full_entry);
            } catch (error) {
                console.error('[event_bus] Error in listener:', error);
            }
        });

        if (broadcast && this.channel) {
            this.channel.postMessage({ type: 'EVENT_EMIT', payload: full_entry });
        }
    }

    /** Returns a chronologically ordered copy of all stored history. */
    get_history(): log_entry[] {
        if (this.count === 0) return [];
        const result: log_entry[] = [];
        const start = this.count < event_bus_service.BUFFER_SIZE
            ? 0
            : this.head; // oldest entry is at head when buffer is full
        for (let i = 0; i < this.count; i++) {
            const idx = (start + i) % event_bus_service.BUFFER_SIZE;
            if (this.ring[idx]) result.push(this.ring[idx]!);
        }
        return result;
    }

    /** Clears event history but keeps all subscribers intact. Safe for `/clear`. */
    clear_history() {
        this.ring = new Array(event_bus_service.BUFFER_SIZE).fill(null);
        this.head = 0;
        this.count = 0;
        this.processed_ids.clear();
    }

    /** Full teardown: clears history AND removes all subscribers. Use on unmount. */
    destroy() {
        this.clear_history();
        this.listeners = [];
    }
}

/** Singleton instance shared across the entire application. */
export const event_bus = new event_bus_service();



