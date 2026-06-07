/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Infrastructure Bus**: Global Pub/Sub notification and telemetry relay. 
 * Orchestrates cross-subsystem event propagation (swarms, logs, security alerts) and manages high-velocity pulse buffering for the UI.
 * 
 * ### 🧬 Logic Flow (Mermaid)
 * ```mermaid
 * sequenceDiagram
 *     participant S as Source Component
 *     participant EB as EventBus (Service)
 *     participant RB as Ring Buffer (Cache)
 *     participant L as Subscribed Listeners
 *     participant BC as BroadcastChannel (Cross-Tab)
 * 
 *     S->>EB: emit_log(entry)
 *     EB->>EB: Generate ID/Timestamp
 *     EB->>RB: Store in Ring Buffer (O(1))
 *     EB->>L: trigger(full_entry)
 *     EB->>BC: postMessage(EVENT_EMIT)
 *     BC-->>EB: onmessage (Deduplicate & Store)
 * ```
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Circular buffer overflow (clears oldest entry), ID cache saturation, or BroadcastChannel disconnect in non-secure browser contexts.
 * - **Telemetry Link**: Global log stream. Search for `[event_bus]` in tracing.
 * 
 * @aiContext
 * - **Dependencies**: `BroadcastChannel` (for cross-tab sync).
 * - **Side Effects**: Emits global log entries and broadcasts them to all open browser contexts.
 */

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

/** Unified Telemetry Message Wrapper */
export interface telemetry_message {
    topic: 'LOG' | 'TRACE' | 'PULSE' | 'OVERSIGHT' | 'SYNC_REQUEST' | 'SYNC_RESPONSE';
    payload: unknown;
    timestamp: number;
    sender_id: string;
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
    private channel: BroadcastChannel | null = (typeof window !== 'undefined' && typeof BroadcastChannel !== 'undefined') ? new BroadcastChannel('tadpole-neural-hub') : null;
    private pending_sync_response: ReturnType<typeof setTimeout> | null = null;
    private trace_listeners: ((span: unknown) => void)[] = [];
    private pulse_listeners: ((pulse: unknown) => void)[] = [];
    private instance_id = (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function')
        ? Array.from(crypto.getRandomValues(new Uint32Array(2)), dec => dec.toString(36)).join('-')
        : Math.random().toString(36).substring(2, 9);
    /** Track recently processed IDs to prevent duplication from cross-tab sync. */
    private processed_ids = new Set<string>();
    private static readonly MAX_ID_CACHE = 500;

    constructor() {
        if (this.channel) {
            this.channel.onmessage = (event) => {
                try {
                    const msg = event.data as telemetry_message;
                    if (!msg || msg.sender_id === this.instance_id) return;

                    switch (msg.topic) {
                        case 'LOG':
                            this.internal_emit(msg.payload as log_entry, false);
                            break;
                        case 'TRACE':
                            this.internal_emit_trace(msg.payload, false);
                            break;
                        case 'PULSE':
                            this.internal_emit_pulse(msg.payload, false);
                            break;
                        case 'SYNC_REQUEST':
                            this.handle_sync_request();
                            break;
                        case 'SYNC_RESPONSE':
                            // Suppress pending sync response if another tab has already replied
                            if (this.pending_sync_response) {
                                clearTimeout(this.pending_sync_response);
                                this.pending_sync_response = null;
                            }
                            this.handle_sync_response(msg.payload);
                            break;
                    }
                } catch (error) {
                    console.error('[event_bus] Error handling BroadcastChannel message:', error);
                }
            };
            
            // Request initial state on startup
            setTimeout(() => this.request_sync(), 100);
        }
    }

    private request_sync(): void {
        if (this.channel) {
            try {
                this.channel.postMessage({
                    topic: 'SYNC_REQUEST',
                    payload: null,
                    timestamp: Date.now(),
                    sender_id: this.instance_id
                } as telemetry_message);
            } catch (error) {
                console.error('[event_bus] Failed to send sync request:', error);
            }
        }
    }

    private handle_sync_request(): void {
        // Only windows with history should respond
        if (this.count === 0 && this.processed_ids.size === 0) return;

        // Cancel any existing pending sync response to avoid duplicate scheduling
        if (this.pending_sync_response) {
            clearTimeout(this.pending_sync_response);
        }

        // Apply a randomized delay (50ms - 200ms) to prevent broadcast storms.
        // If another tab broadcasts a SYNC_RESPONSE during this interval, this response is cancelled.
        this.pending_sync_response = setTimeout(() => {
            if (this.channel) {
                try {
                    this.channel.postMessage({
                        topic: 'SYNC_RESPONSE',
                        payload: {
                            logs: this.get_history().slice(-100), // Limit sync response payload size to last 100 entries to prevent DoS
                        },
                        timestamp: Date.now(),
                        sender_id: this.instance_id
                    } as telemetry_message);
                } catch (error) {
                    console.error('[event_bus] Failed to send sync response:', error);
                }
            }
            this.pending_sync_response = null;
        }, 50 + Math.random() * 150);
    }

    private handle_sync_response(payload: unknown): void {
        if (!payload || typeof payload !== 'object') return;
        const p = payload as { logs?: log_entry[] };
        if (p.logs && Array.isArray(p.logs)) {
            // Limit receiver-side processing payload to last 200 logs to prevent memory/CPU pressure (EVT-010)
            const logs_to_process = p.logs.slice(-200);

            // Filter out logs that are already in our deduplication cache
            const new_logs = logs_to_process.filter((log: log_entry) => log && log.id && !this.processed_ids.has(log.id));
            if (new_logs.length === 0) return;

            // Sort logs chronologically by timestamp
            new_logs.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());

            // Write all history logs to circular buffer synchronously so get_history() is immediately populated
            new_logs.forEach((log: log_entry) => {
                // Maintain ID cache size
                if (this.processed_ids.size >= event_bus_service.MAX_ID_CACHE) {
                    const iter = this.processed_ids.values();
                    for (let i = 0; i < event_bus_service.MAX_ID_CACHE / 2; i++) {
                        const { value, done } = iter.next();
                        if (done) break;
                        this.processed_ids.delete(value);
                    }
                }
                this.processed_ids.add(log.id);

                this.ring[this.head] = log;
                this.head = (this.head + 1) % event_bus_service.BUFFER_SIZE;
                if (this.count < event_bus_service.BUFFER_SIZE) this.count++;
            });

            // Deliver notifications to listeners asynchronously in chunks to prevent event loop starvation.
            // We only deliver the last 100 logs since the UI components maintain a maximum local window of 100 logs anyway.
            const logs_to_notify = new_logs.slice(-100);
            const chunk_size = 50;
            let index = 0;
            const deliver_chunk = () => {
                const limit = Math.min(index + chunk_size, logs_to_notify.length);
                for (; index < limit; index++) {
                    const log = logs_to_notify[index];
                    this.listeners.forEach(listener => {
                        try {
                            listener(log);
                        } catch (error) {
                            console.error('[event_bus] Error in listener during sync:', error);
                        }
                    });
                }
                if (index < logs_to_notify.length) {
                    setTimeout(deliver_chunk, 0);
                }
            };
            setTimeout(deliver_chunk, 0);
        }
    }

    /** Subscribe to all future events. Returns an unsubscribe function. */
    subscribe_logs(listener: Listener): () => void {
        this.listeners.push(listener);
        return () => {
            this.listeners = this.listeners.filter(l => l !== listener);
        };
    }

    subscribe_traces(listener: (span: unknown) => void): () => void {
        this.trace_listeners.push(listener);
        return () => {
            this.trace_listeners = this.trace_listeners.filter(l => l !== listener);
        };
    }

    subscribe_pulses(listener: (pulse: unknown) => void): () => void {
        this.pulse_listeners.push(listener);
        return () => {
            this.pulse_listeners = this.pulse_listeners.filter(l => l !== listener);
        };
    }

    /** Emit an event to all subscribers. `id` and `timestamp` are auto-filled if not provided. */
    emit_log(entry: Omit<log_entry, 'id' | 'timestamp'> & { id?: string; timestamp?: Date }): void {
        const valid_sources: log_source[] = ['User', 'System', 'Agent'];
        const valid_severities: log_severity[] = ['info', 'success', 'warning', 'error'];

        if (!entry || typeof entry !== 'object') {
            throw new TypeError('[event_bus] emit_log requires a valid entry object.');
        }
        if (!valid_sources.includes(entry.source)) {
            throw new TypeError(`[event_bus] Invalid log source: "${entry.source}". Must be one of: ${valid_sources.join(', ')}`);
        }
        if (!valid_severities.includes(entry.severity)) {
            throw new TypeError(`[event_bus] Invalid log severity: "${entry.severity}". Must be one of: ${valid_severities.join(', ')}`);
        }
        if (typeof entry.text !== 'string') {
            throw new TypeError('[event_bus] Log text must be a string.');
        }

        // PERFORMANCE: Truncate extremely large text payloads in the core bus to prevent memory pressure
        // Max 50k characters for the bus (UI components can truncate further if needed)
        const text = (entry.text && entry.text.length > 50000) 
            ? entry.text.substring(0, 50000) + '... [BUS TRUNCATED]'
            : entry.text;

        // Truncate large strings in metadata as well
        const metadata = entry.metadata ? this.truncate_large_metadata(entry.metadata) as Record<string, unknown> : undefined;

        const full_entry: log_entry = {
            id: entry.id || this.generate_secure_id(),
            timestamp: entry.timestamp || new Date(),
            source: entry.source,
            text,
            severity: entry.severity,
            agent_id: entry.agent_id,
            agent_name: entry.agent_name,
            mission_id: entry.mission_id,
            type_id: entry.type_id,
            metadata
        };
        this.internal_emit(full_entry, true);
    }

    emit_trace(span: unknown): void {
        this.internal_emit_trace(span, true);
    }

    emit_pulse(pulse: unknown): void {
        this.internal_emit_pulse(pulse, true);
    }

    private generate_secure_id(): string {
        if (typeof crypto !== 'undefined') {
            if (typeof crypto.randomUUID === 'function') {
                return crypto.randomUUID();
            }
            if (typeof crypto.getRandomValues === 'function') {
                const array = new Uint32Array(4);
                crypto.getRandomValues(array);
                return Array.from(array, dec => dec.toString(36)).join('-');
            }
        }
        return Date.now().toString(36) + '-' + Math.random().toString(36).substring(2, 9);
    }

    /** 
     * Recursively truncates large strings in metadata objects to prevent memory bloat,
     * while preserving native constructors (Date, RegExp), handling arrays, and preventing prototype pollution.
     */
    private truncate_large_metadata(value: unknown, depth = 0): unknown {
        if (depth > 3) return '[MAX_DEPTH_REACHED]';
        const MAX_STRING_LEN = 10000;

        if (typeof value === 'string') {
            if (value.length > MAX_STRING_LEN) {
                return value.substring(0, MAX_STRING_LEN) + '... [TRUNCATED]';
            }
            return value;
        }

        if (Array.isArray(value)) {
            return value.map(item => this.truncate_large_metadata(item, depth + 1));
        }

        if (typeof value === 'object' && value !== null) {
            if (value instanceof Date) {
                return new Date(value.getTime());
            }
            if (value instanceof RegExp) {
                return new RegExp(value.source, value.flags);
            }

            const result = Object.create(null) as Record<string, unknown>;
            for (const [key, val] of Object.entries(value)) {
                if (key === '__proto__' || key === 'constructor' || key === 'prototype') {
                    continue;
                }
                result[key] = this.truncate_large_metadata(val, depth + 1);
            }
            return { ...result };
        }

        return value;
    }

    private internal_emit(full_entry: log_entry, broadcast: boolean): void {
        // Deduplication: prevent identical IDs from being re-processed
        if (this.processed_ids.has(full_entry.id)) {
            return;
        }

        // Maintain ID cache size
        if (this.processed_ids.size >= event_bus_service.MAX_ID_CACHE) {
            // Use iterator to delete the oldest half without O(n) array allocation
            const iter = this.processed_ids.values();
            for (let i = 0; i < event_bus_service.MAX_ID_CACHE / 2; i++) {
                const { value, done } = iter.next();
                if (done) break;
                this.processed_ids.delete(value);
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
            try {
                this.channel.postMessage({ 
                    topic: 'LOG', 
                    payload: full_entry, 
                    timestamp: Date.now(),
                    sender_id: this.instance_id
                } as telemetry_message);
            } catch (error) {
                console.error('[event_bus] Failed to broadcast log over channel, attempting fallback:', error);
                try {
                    // Fallback: strip metadata to prevent DataCloneError
                    const fallback_entry = { ...full_entry, metadata: undefined };
                    this.channel.postMessage({
                        topic: 'LOG',
                        payload: fallback_entry,
                        timestamp: Date.now(),
                        sender_id: this.instance_id
                    } as telemetry_message);
                } catch (fallback_error) {
                    console.error('[event_bus] Critical broadcast fallback failure:', fallback_error);
                }
            }
        }
    }

    private internal_emit_trace(span: unknown, broadcast: boolean): void {
        this.trace_listeners.forEach(l => {
            try {
                l(span);
            } catch (error) {
                console.error('[event_bus] Error in trace listener:', error);
            }
        });
        if (broadcast && this.channel) {
            try {
                this.channel.postMessage({ 
                    topic: 'TRACE', 
                    payload: span, 
                    timestamp: Date.now(),
                    sender_id: this.instance_id
                } as telemetry_message);
            } catch (error) {
                console.error('[event_bus] Failed to broadcast trace:', error);
            }
        }
    }

    private internal_emit_pulse(pulse: unknown, broadcast: boolean): void {
        this.pulse_listeners.forEach(l => {
            try {
                l(pulse);
            } catch (error) {
                console.error('[event_bus] Error in pulse listener:', error);
            }
        });
        if (broadcast && this.channel) {
            try {
                this.channel.postMessage({ 
                    topic: 'PULSE', 
                    payload: pulse, 
                    timestamp: Date.now(),
                    sender_id: this.instance_id
                } as telemetry_message);
            } catch (error) {
                console.error('[event_bus] Failed to broadcast pulse:', error);
            }
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
    clear_history(): void {
        this.ring = new Array(event_bus_service.BUFFER_SIZE).fill(null);
        this.head = 0;
        this.count = 0;
        this.processed_ids.clear();
        if (this.pending_sync_response) {
            clearTimeout(this.pending_sync_response);
            this.pending_sync_response = null;
        }
    }

    /** Full teardown: clears history AND removes all subscribers. Use on unmount. */
    destroy(): void {
        this.clear_history();
        this.listeners = [];
        this.trace_listeners = [];
        this.pulse_listeners = [];
        if (this.channel) {
            try {
                this.channel.close();
            } catch (error) {
                console.error('[event_bus] Error closing channel during destroy:', error);
            }
            this.channel = null;
        }
    }

    /** Reset state and re-initialize transport channel. Primarily used for test suite isolation. */
    reset(): void {
        this.clear_history();
        this.listeners = [];
        this.trace_listeners = [];
        this.pulse_listeners = [];
        if (this.channel) {
            try {
                this.channel.close();
            } catch (error) {
                console.error('[event_bus] Error closing channel during reset:', error);
            }
        }
        // Re-initialize channel for subsequent test/lifecycle usage
        this.channel = (typeof window !== 'undefined' && typeof BroadcastChannel !== 'undefined')
            ? new BroadcastChannel('tadpole-neural-hub')
            : null;
        
        // Re-initialize listener handler on message for the new channel
        if (this.channel) {
            this.channel.onmessage = (event) => {
                try {
                    const msg = event.data as telemetry_message;
                    if (!msg || msg.sender_id === this.instance_id) return;

                    switch (msg.topic) {
                        case 'LOG':
                            this.internal_emit(msg.payload as log_entry, false);
                            break;
                        case 'TRACE':
                            this.internal_emit_trace(msg.payload, false);
                            break;
                        case 'PULSE':
                            this.internal_emit_pulse(msg.payload, false);
                            break;
                        case 'SYNC_REQUEST':
                            this.handle_sync_request();
                            break;
                        case 'SYNC_RESPONSE':
                            if (this.pending_sync_response) {
                                clearTimeout(this.pending_sync_response);
                                this.pending_sync_response = null;
                            }
                            this.handle_sync_response(msg.payload);
                            break;
                    }
                } catch (error) {
                    console.error('[event_bus] Error handling BroadcastChannel message:', error);
                }
            };
        }
    }
}

/** Singleton instance shared across the entire application. */
export const event_bus = new event_bus_service();
