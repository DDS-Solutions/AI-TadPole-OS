/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Networking**: Manages the real-time WebSocket telemetry and relay between the UI and the backend. 
 * Orchestrates event normalization for the `event_bus` and `sovereign_store`.
 * 
 * ### @aiContext
 * - **Dependencies**: `event_bus`, `use_settings_store` (URL/Auth), @msgpack/msgpack (Binary Pulse).
 * - **Side Effects**: asynchronous store hydration (`agent_store`, `sovereign_store`, `trace_store`) via reactive socket messages.
 * - **Mocking**: Mocking the `WebSocket` global is required for connection lifecycle tests.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Connection timeouts (MAX_RETRIES reached), invalid API key/URL format causing 403/404, or MessagePack decoding errors for binary pulses.
 * - **Telemetry Link**: Search for `[Tadpole_OS_Socket]` or bearer.tadpole in browser/proxy logs.
 */

import { event_bus } from './event_bus';
import { get_settings, use_settings_store } from '../stores/settings_store';
import type { Swarm_Pulse } from '../types';
import { decode } from '@msgpack/msgpack';

/** Payload for agent update/status events from the WebSocket. */
export interface Agent_Update_Event {
    type: 'agent:create' | 'agent:update' | 'agent:status' | 'engine:ui_invalidate';
    agent_id?: string;
    agentId?: string;
    status?: string;
    data?: Record<string, unknown> | Partial<import('../types').Agent>;
    resource?: 'agents' | 'missions' | 'system';
    id?: string;
    source_id?: string;
}

/** Payload for engine health broadcast events. */
export interface Engine_Health_Event {
    type: 'engine:health';
    uptime?: number;
    agent_count?: number;
    active_missions?: number;
    active_agents?: number;
    max_depth?: number;
    tpm?: number;
    recruit_count?: number;
    cpu?: number;
    memory?: number;
    latency?: number;
    [key: string]: unknown;
}

/** Payload for inter-cluster handoff events. */
export interface Handoff_Event {
    type: 'agent:handoff';
    from_cluster: string;
    to_cluster: string;
    agent_id: string;
    payload?: Record<string, unknown>;
}

/** Payload for MCP tool pulse events. */
export interface Mcp_Pulse_Event {
    type: 'engine:mcp_pulse';
    tool: string;
    status: 'success' | 'error';
    latency: number;
}

export interface Socket_Log_Event {
    type: 'log' | 'thought';
    id?: string;
    request_id?: string;
    requestId?: string;
    agent_id?: string;
    agent_name?: string;
    text?: string;
    message?: string;
    level?: string;
    message_id?: string;
    messageId?: string;
    [key: string]: unknown;
}

export interface Socket_Agent_Message_Event {
    type: 'agent:message';
    id?: string;
    message_id?: string;
    messageId?: string;
    agent_id?: string;
    agent_name?: string;
    text?: string;
    message?: string;
    [key: string]: unknown;
}

export interface Socket_Trace_Span_Event {
    type: 'trace:span';
    span: import('../types').Trace_Span;
}

export interface Socket_Trace_Span_Update_Event {
    type: 'trace:span_update';
    span_id?: string;
    spanId?: string;
    update: Partial<import('../types').Trace_Span>;
}

export interface Socket_Scheduled_Job_Complete_Event {
    type: 'engine:scheduled_job_complete';
    job_name: string;
    cost_usd?: number;
    status?: string;
}

export type Incoming_Socket_Message =
    | Socket_Log_Event
    | Socket_Agent_Message_Event
    | Agent_Update_Event
    | Engine_Health_Event
    | Handoff_Event
    | Mcp_Pulse_Event
    | Socket_Trace_Span_Event
    | Socket_Trace_Span_Update_Event
    | Socket_Scheduled_Job_Complete_Event;

/** Maximum number of reconnect attempts before giving up. */
const MAX_RETRIES = 10;
/** Initial backoff delay in ms. */
const INITIAL_BACKOFF = 2000;
/** Maximum backoff delay in ms. */
const MAX_BACKOFF = 30000;

/** Binary header constants. */
const BINARY_HEADER_AUDIO = 0x01;
const BINARY_HEADER_SWARM_PULSE = 0x02;

/** Frame size limit constants. */
const MAX_BINARY_FRAME_SIZE = 1 * 1024 * 1024; // 1MB
const MAX_TEXT_FRAME_SIZE = 5 * 1024 * 1024; // 5MB

/** Connection states for the socket. */
export type Connection_State = 'connecting' | 'authenticating' | 'connected' | 'disconnected' | 'reconnecting' | 'error';

type State_Listener = (state: Connection_State) => void;

/** Recursive key sanitization helper to strip __proto__, prototype, and constructor keys */
export function sanitize_object(val: unknown): unknown {
    if (val === null || typeof val !== 'object') {
        return val;
    }
    if (Array.isArray(val)) {
        return val.map(sanitize_object);
    }
    const clean = Object.create(null);
    for (const key of Object.keys(val)) {
        if (key === '__proto__' || key === 'constructor' || key === 'prototype') {
            continue;
        }
        clean[key] = sanitize_object((val as Record<string, unknown>)[key]);
    }
    return clean;
}

/**
 * ProtocolCodec
 * Handles encoding and decoding of WebSocket payload framing.
 */
export class ProtocolCodec {
    static decode_binary(data: ArrayBuffer): { type: 'audio' | 'pulse' | 'unknown'; payload: ArrayBuffer | Swarm_Pulse } {
        const view = new Uint8Array(data);
        if (view.length === 0) {
            return { type: 'unknown', payload: data };
        }
        const header = view[0];
        const payload = data.slice(1);
        if (header === BINARY_HEADER_AUDIO) {
            return { type: 'audio', payload };
        } else if (header === BINARY_HEADER_SWARM_PULSE) {
            try {
                const pulse = decode(payload, {
                    maxStrLength: 1024 * 1024,
                    maxBinLength: 1024 * 1024,
                    maxArrayLength: 10000,
                    maxMapLength: 10000
                }) as Swarm_Pulse;
                return { type: 'pulse', payload: pulse };
            } catch (e) {
                throw new Error(`MessagePack decode failed: ${e instanceof Error ? e.message : String(e)}`, { cause: e });
            }
        }
        return { type: 'unknown', payload: data };
    }

    static decode_json(data: string): Incoming_Socket_Message {
        return JSON.parse(data) as Incoming_Socket_Message;
    }

    static encode_json(data: Record<string, unknown>): string {
        return JSON.stringify(data);
    }
}

/**
 * ReconnectionPolicy
 * Implements exponential backoff retry delays.
 */
export class ReconnectionPolicy {
    private readonly initial_backoff: number;
    private readonly max_backoff: number;
    private readonly max_retries: number;

    constructor(
        initial_backoff = INITIAL_BACKOFF,
        max_backoff = MAX_BACKOFF,
        max_retries = MAX_RETRIES
    ) {
        this.initial_backoff = initial_backoff;
        this.max_backoff = max_backoff;
        this.max_retries = max_retries;
    }

    get_delay(retry_count: number): number {
        return Math.min(this.initial_backoff * Math.pow(2, retry_count), this.max_backoff);
    }

    should_retry(retry_count: number): boolean {
        return retry_count < this.max_retries;
    }
}

/**
 * is_allowed_origin
 * Validates the socket target URL to prevent token exfiltration.
 */
export function is_allowed_origin(url_string: string, allowed_origins?: string[]): boolean {
    try {
        const url = new URL(url_string);
        const hostname = url.hostname.toLowerCase();
        
        // Always allow local loopback
        if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '[::1]') {
            return true;
        }

        // Also allow same-origin if running in browser
        if (typeof window !== 'undefined' && window.location && window.location.hostname.toLowerCase() === hostname) {
            return true;
        }
        
        // Build-time allowed origins
        const env_allowed = (typeof import.meta !== 'undefined' && import.meta.env?.VITE_ALLOWED_ORIGINS) || '';
        const env_origins = env_allowed
            ? env_allowed.split(',').map((x: string) => x.trim().toLowerCase())
            : [];
        
        const runtime_allowed = allowed_origins 
            ? allowed_origins.map(x => x.trim().toLowerCase()) 
            : [];
            
        const all_allowed = [...env_origins, ...runtime_allowed];
        
        return all_allowed.some(allowed => {
            if (!allowed) return false;
            if (allowed.includes('://')) {
                try {
                    const allowed_url = new URL(allowed);
                    return allowed_url.hostname === hostname;
                } catch {
                    return false;
                }
            }
            return allowed === hostname;
        });
    } catch {
        return false;
    }
}

/**
 * SocketRouter
 * Manages event listeners and dispatch routing for WebSocket events.
 */
class SocketRouter {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    private listeners: Map<string, Set<(...args: any[]) => void>> = new Map();

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    add(channel: string, listener: (...args: any[]) => void): () => void {
        if (!this.listeners.has(channel)) {
            this.listeners.set(channel, new Set());
        }
        this.listeners.get(channel)!.add(listener);
        return () => {
            const set = this.listeners.get(channel);
            if (set) {
                set.delete(listener);
                if (set.size === 0) {
                    this.listeners.delete(channel);
                }
            }
        };
    }

    emit(channel: string, payload: unknown): void {
        const set = this.listeners.get(channel);
        if (set) {
            set.forEach(listener => {
                try {
                    listener(payload);
                } catch (err) {
                    console.error(`[Tadpole_OS] Error in listener for channel ${channel}:`, err);
                }
            });
        }
    }

    clear(): void {
        this.listeners.clear();
    }
}

/**
 * Tadpole_OS_Socket_Client
 * WebSocket client for streaming real-time logs from TadpoleOS.
 * Reads the connection URL from the centralized settings store.
 * Refactored for strict snake_case compliance for backend parity.
 */
export class Tadpole_OS_Socket_Client {
    private socket: WebSocket | null = null;
    private reconnect_timer: ReturnType<typeof setTimeout> | null = null;
    private is_explicitly_closed = false;
    private retry_count = 0;
    private last_url = '';
    private last_key = '';

    // Agent name cache — O(1) lookups instead of O(n) per message
    private agent_name_cache: Map<string, string> = new Map();

    // State Management
    private state: Connection_State = 'disconnected';
    private connection_references = 0;

    private settings_unsubscribe: (() => void) | null = null;
    private disconnect_timeout: ReturnType<typeof setTimeout> | null = null;
    private auth_timeout_timer: ReturnType<typeof setTimeout> | null = null;
    private send_queue: string[] = [];
    private reconnection_policy = new ReconnectionPolicy();
    private router = new SocketRouter();

    /** Resets the global singleton instance state, closing sockets and clearing listeners. */
    static reset(): void {
        if (instance) {
            instance.destroy();
            instance = null;
        }
    }

    /** Cleans up the instance resources and active subscriptions. */
    destroy(): void {
        this.disconnect();
        if (this.settings_unsubscribe) {
            this.settings_unsubscribe();
            this.settings_unsubscribe = null;
        }
        if (this.disconnect_timeout) {
            clearTimeout(this.disconnect_timeout);
            this.disconnect_timeout = null;
        }
        if (this.auth_timeout_timer) {
            clearTimeout(this.auth_timeout_timer);
            this.auth_timeout_timer = null;
        }
        this.agent_name_cache.clear();
        this.router.clear();
        this.connection_references = 0;
        this.send_queue = [];
    }

    /** Unified dynamic subscription method with reference-counting lifecycle. */
    subscribe(channel: 'agentUpdates', listener: (data: Agent_Update_Event) => void): () => void;
    subscribe(channel: 'health', listener: (data: Engine_Health_Event) => void): () => void;
    subscribe(channel: 'handoff', listener: (data: Handoff_Event) => void): () => void;
    subscribe(channel: 'status', listener: (data: Connection_State) => void): () => void;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    subscribe(channel: string, listener: (...args: any[]) => void): () => void;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    subscribe(channel: string, listener: (...args: any[]) => void): () => void {
        let unsubscribe_func: () => void;

        switch (channel) {
            case 'agentUpdates':
                unsubscribe_func = this.subscribe_agent_updates(listener as (update: Agent_Update_Event) => void);
                break;
            case 'health':
                unsubscribe_func = this.subscribe_health(listener as (health: Engine_Health_Event) => void);
                break;
            case 'handoff':
                unsubscribe_func = this.subscribe_handoff(listener as (handoff: Handoff_Event) => void);
                break;
            case 'status':
                unsubscribe_func = this.subscribe_status(listener as State_Listener);
                break;
            default:
                unsubscribe_func = () => {};
                break;
        }

        if (this.disconnect_timeout) {
            clearTimeout(this.disconnect_timeout);
            this.disconnect_timeout = null;
        }

        this.connect();
        this.connection_references++;

        return () => {
            unsubscribe_func();
            this.connection_references--;

            if (this.connection_references <= 0) {
                // Schedule disconnect after a grace period to avoid socket thrashing on route changes
                if (this.disconnect_timeout) {
                    clearTimeout(this.disconnect_timeout);
                }
                this.disconnect_timeout = setTimeout(() => {
                    this.disconnect_timeout = null;
                    if (this.connection_references <= 0) {
                        this.disconnect();
                    }
                }, 1000);
            }
        };
    }

    /** Subscribe to connection state changes. */
    subscribe_status(listener: State_Listener): () => void {
        const unsub = this.router.add('status', listener);
        listener(this.state); // Immediate update
        return unsub;
    }

    private set_state(new_state: Connection_State): void {
        if (this.state !== new_state) {
            this.state = new_state;
            this.router.emit('status', new_state);
        }
    }

    subscribe_agent_updates(listener: (update: Agent_Update_Event) => void): () => void {
        return this.router.add('agentUpdates', listener);
    }

    subscribe_health(listener: (health: Engine_Health_Event) => void): () => void {
        return this.router.add('health', listener);
    }

    subscribe_handoff(listener: (handoff: Handoff_Event) => void): () => void {
        return this.router.add('handoff', listener);
    }

    subscribe_pulse(listener: (pulse: Mcp_Pulse_Event) => void): () => void {
        return this.router.add('pulse', listener);
    }

    subscribe_audio_stream(listener: (chunk: ArrayBuffer) => void): () => void {
        return this.router.add('audio_stream', listener);
    }

    subscribe_swarm_pulse(listener: (pulse: Swarm_Pulse) => void): () => void {
        return this.router.add('swarm_pulse', listener);
    }

    subscribe_raw(listener: (event: Record<string, unknown>) => void): () => void {
        return this.router.add('raw', listener);
    }

    get_connection_state(): Connection_State {
        return this.state;
    }

    send_json(data: Record<string, unknown>): boolean {
        try {
            const payload = ProtocolCodec.encode_json(data);
            if (this.socket && this.socket.readyState === WebSocket.OPEN && this.state === 'connected') {
                this.socket.send(payload);
                return true;
            }
            if (this.state === 'connecting' || this.state === 'authenticating' || this.state === 'reconnecting') {
                if (this.send_queue.length >= 100) {
                    this.send_queue.shift(); // Drop oldest
                }
                this.send_queue.push(payload);
                return true;
            }
        } catch (error) {
            console.error('[Tadpole_OS] Failed to send JSON payload:', error);
        }
        return false;
    }

    private get_websocket_url(): string {
        const { tadpole_os_url } = get_settings();
        const raw_url = (tadpole_os_url || 'http://localhost:8000').trim();
        
        // Remove trailing slash
        const sanitized_url = raw_url.replace(/\/$/, '');
        
        // Convert HTTP/HTTPS protocol to WS/WSS
        const ws_prefix = sanitized_url.startsWith('https') ? 'wss' : 'ws';
        const protocol_replaced = sanitized_url.replace(/^https?/, ws_prefix);
        
        // Construct final WebSocket endpoint URL
        return `${protocol_replaced}/v1/engine/ws`;
    }

    /** Opens the WebSocket connection and begins listening for events. */
    connect(is_reconnect = false): void {
        // Guard: Don't connect if connection was explicitly closed and we are in a reconnect attempt
        if (is_reconnect && this.is_explicitly_closed) {
            return;
        }

        // Guard: Don't connect if already connecting, authenticating, or connected
        if (this.socket || this.reconnect_timer || this.state === 'connected' || this.state === 'authenticating') {
            return;
        }

        // Reset the closure flag on manual/fresh connection attempts
        if (!is_reconnect) {
            this.is_explicitly_closed = false;
        }

        this.set_state('connecting');

        // Lazy store subscription on first connect
        if (!this.settings_unsubscribe) {
            const initial = get_settings();
            this.last_url = initial.tadpole_os_url;
            this.last_key = initial.tadpole_os_api_key;

            this.settings_unsubscribe = use_settings_store.subscribe((state) => {
                const { tadpole_os_url, tadpole_os_api_key } = state.settings;

                if (tadpole_os_url !== this.last_url || tadpole_os_api_key !== this.last_key) {
                    this.last_url = tadpole_os_url;
                    this.last_key = tadpole_os_api_key;

                    if (this.is_explicitly_closed) return;

                    console.debug(`[Tadpole_OS] Infrastructure settings changed. Reconnecting...`);
                    this.disconnect();
                    // Reset closure flag so connect() can proceed
                    this.is_explicitly_closed = false;
                    this.connect();
                }
            });
        }

        // Get URL from centralized settings, converting http/https to ws/wss
        const { tadpole_os_api_key } = get_settings();
        const token = tadpole_os_api_key.trim();
        if (!token) {
            this.set_state('error');
            event_bus.emit_log({
                source: 'System',
                text: 'Tadpole_OS: Missing API token. Add your NEURAL_TOKEN in Settings before connecting telemetry.',
                severity: 'error'
            });
            return;
        }

        let ws_url: string;
        try {
            ws_url = this.get_websocket_url();
            if (!is_allowed_origin(ws_url)) {
                this.set_state('error');
                event_bus.emit_log({
                    source: 'System',
                    text: 'Tadpole_OS: Connection to origin refused. Host is not in allowed origins list.',
                    severity: 'error'
                });
                return;
            }
        } catch (error) {
            console.error('[Tadpole_OS] Invalid URL format:', error);
            this.set_state('error');
            event_bus.emit_log({
                source: 'System',
                text: 'Tadpole_OS: Connection failed due to invalid URL format.',
                severity: 'error'
            });
            return;
        }

        try {
            const ws = new WebSocket(ws_url, ['tadpole-pulse-v1']);
            ws.binaryType = 'arraybuffer';
            this.socket = ws;

            ws.onopen = () => {
                if (this.socket !== ws) return; // Guard against stale connections
                this.retry_count = 0; // Reset on successful connection
                this.set_state('authenticating');

                // Start auth timeout timer (10 seconds)
                if (this.auth_timeout_timer) clearTimeout(this.auth_timeout_timer);
                this.auth_timeout_timer = setTimeout(() => {
                    if (this.socket === ws && this.state === 'authenticating') {
                        this.set_state('error');
                        event_bus.emit_log({
                            source: 'System',
                            text: 'Tadpole_OS: Connection closed due to authentication timeout.',
                            severity: 'error'
                        });
                        this.disconnect();
                    }
                }, 10000);

                // Send post-connect auth frame
                const auth_payload = { type: 'auth', token: token };
                ws.send(JSON.stringify(auth_payload));
            };

            this.socket.onmessage = (event) => {
                if (event.data instanceof ArrayBuffer) {
                    if (event.data.byteLength > MAX_BINARY_FRAME_SIZE) {
                        console.warn(`[Tadpole_OS] Rejected binary frame exceeding maximum size of ${MAX_BINARY_FRAME_SIZE} bytes: ${event.data.byteLength}`);
                        return;
                    }
                    if (this.state !== 'connected') {
                        // Drop binary frames received before authentication is completed
                        return;
                    }
                    try {
                        const decoded = ProtocolCodec.decode_binary(event.data);
                        if (decoded.type === 'audio') {
                            this.router.emit('audio_stream', decoded.payload as ArrayBuffer);
                        } else if (decoded.type === 'pulse') {
                            this.router.emit('swarm_pulse', decoded.payload as Swarm_Pulse);
                        }
                    } catch (e) {
                        console.error('[Tadpole_OS] Binary decode failed:', e);
                    }
                    return;
                }

                if (typeof event.data === 'string') {
                    if (event.data.length > MAX_TEXT_FRAME_SIZE) {
                        console.warn(`[Tadpole_OS] Rejected text frame exceeding maximum size of ${MAX_TEXT_FRAME_SIZE} characters: ${event.data.length}`);
                        return;
                    }
                    try {
                        const parsed = JSON.parse(event.data);
                        
                        // Check for post-connect auth responses first
                        if (parsed.type === 'auth_ok') {
                            if (this.state === 'authenticating') {
                                if (this.auth_timeout_timer) {
                                    clearTimeout(this.auth_timeout_timer);
                                    this.auth_timeout_timer = null;
                                }
                                this.set_state('connected');
                                event_bus.emit_log({
                                    source: 'System',
                                    text: 'Connected to TadpoleOS Log Stream.',
                                    severity: 'success'
                                });
                                // Flush send queue
                                while (this.send_queue.length > 0) {
                                    const msg = this.send_queue.shift();
                                    if (msg && this.socket && this.socket.readyState === WebSocket.OPEN) {
                                        this.socket.send(msg);
                                    }
                                }
                            }
                            return;
                        }
                        if (parsed.type === 'auth_error') {
                            if (this.auth_timeout_timer) {
                                clearTimeout(this.auth_timeout_timer);
                                this.auth_timeout_timer = null;
                            }
                            this.set_state('error');
                            event_bus.emit_log({
                                source: 'System',
                                text: `Tadpole_OS: Authentication failed. ${parsed.message || 'Invalid credentials.'}`,
                                severity: 'error'
                            });
                            this.disconnect();
                            return;
                        }

                        // Guard against unauthenticated messages
                        if (this.state !== 'connected') {
                            return;
                        }

                        const data = sanitize_object(parsed) as Incoming_Socket_Message;
                        void this.handle_socket_message(data).catch(err => {
                            console.error('[Tadpole_OS] Async error in handle_socket_message:', err);
                        });
                    } catch (e) {
                        console.error('[Tadpole_OS] JSON parsing failed, received corrupted stream segment:', e, event.data);
                    }
                }
            };

            ws.onclose = (ev) => {
                if (this.socket === ws) {
                    if (this.auth_timeout_timer) {
                        clearTimeout(this.auth_timeout_timer);
                        this.auth_timeout_timer = null;
                    }
                    this.socket = null;
                    if (ev && ev.code === 4001) {
                        this.set_state('error');
                        event_bus.emit_log({
                            source: 'System',
                            text: 'Tadpole_OS: Connection closed due to authentication failure or timeout (code 4001).',
                            severity: 'error'
                        });
                        this.disconnect();
                        return;
                    }
                    if (this.state !== 'error') {
                        this.set_state('disconnected');
                    }
                    if (!this.is_explicitly_closed) {
                        this.schedule_reconnect();
                    }
                }
            };

            ws.onerror = () => {
                if (this.socket === ws) {
                    if (this.auth_timeout_timer) {
                        clearTimeout(this.auth_timeout_timer);
                        this.auth_timeout_timer = null;
                    }
                    this.set_state('disconnected');
                    ws.close();
                }
            };

        } catch (error) {
            console.error('[Tadpole_OS] Connection failed:', error);
            this.set_state('disconnected');
            this.schedule_reconnect();
        }
    }

    private _get_agent_id(data: Incoming_Socket_Message): string | undefined {
        const d = data as Record<string, unknown>;
        if ('agent_id' in d && typeof d.agent_id === 'string') return d.agent_id;
        if ('agentId' in d && typeof d.agentId === 'string') return d.agentId;
        if ('id' in d && typeof d.id === 'string') return d.id;
        return undefined;
    }

    private _get_agent_name(agent_id: string | undefined, fallback_name?: string): string {
        if (agent_id) {
            const cached = this.agent_name_cache.get(agent_id);
            if (cached) return cached;
        }
        return fallback_name || agent_id || '';
    }

    private _handle_log_message(data: Socket_Log_Event): void {
        const agent_id = this._get_agent_id(data);
        const agent_name = this._get_agent_name(agent_id, data.agent_name);

        const metadata = Object.assign(Object.create(null), data);

        event_bus.emit_log({
            id: (data.id || data.request_id || data.requestId || '') as string,
            source: agent_id ? 'Agent' : 'System',
            agent_id,
            agent_name,
            text: (data.text || data.message || JSON.stringify(data)) as string,
            severity: (data.level === 'error' ? 'error' : 'info'),
            metadata: metadata as Record<string, unknown>
        });
    }

    private _handle_agent_message(data: Socket_Agent_Message_Event): void {
        const agent_id = this._get_agent_id(data);
        const agent_name = this._get_agent_name(agent_id, data.agent_name);

        const metadata = Object.assign(Object.create(null), data);

        event_bus.emit_log({
            id: (data.id || data.message_id || data.messageId || '') as string,
            source: 'Agent',
            agent_id,
            agent_name,
            text: (data.text || 'Mission action complete.') as string,
            severity: 'info',
            metadata: metadata as Record<string, unknown>
        });
    }

    private _handle_agent_update(data: Agent_Update_Event): void {
        const normalized_agent_id = this._get_agent_id(data) || '';
        const normalized_data = data.type === 'agent:status'
            ? { ...data, type: 'agent:update' as const, agent_id: normalized_agent_id, data: { status: data.status } }
            : { ...data, agent_id: normalized_agent_id };

        if (data.type === 'engine:ui_invalidate') {
            event_bus.emit_log({
                source: 'System',
                text: `UI Invalidated: ${data.resource}${data.id ? ` (#${data.id})` : ''}`,
                severity: 'info'
            });
        }

        this.router.emit('agentUpdates', normalized_data as Agent_Update_Event);
    }

    private _handle_health(data: Engine_Health_Event): void {
        this.router.emit('health', data);
    }

    private _handle_handoff(data: Handoff_Event): void {
        this.router.emit('handoff', data);
    }

    private _handle_mcp_pulse(data: Mcp_Pulse_Event): void {
        this.router.emit('pulse', data);
    }

    private _handle_trace_span(data: Socket_Trace_Span_Event): void {
        event_bus.emit_trace(data.span);
    }

    private _handle_trace_span_update(data: Socket_Trace_Span_Update_Event): void {
        event_bus.emit_trace({
            id: (data.span_id || data.spanId) as string,
            ...data.update
        });
    }

    private _handle_scheduled_job_complete(data: Socket_Scheduled_Job_Complete_Event): void {
        event_bus.emit_log({
            source: 'System',
            text: `Scheduled Job '${data.job_name}' completed. Cost: $${(data.cost_usd || 0).toFixed(4)}`,
            severity: data.status === 'failed' ? 'error' : 'success'
        });
    }

    /** Updates the socket's internal agent friendly-name cache. */
    public set_agent_name_cache(agents: Array<{ id: string; name: string }>): void {
        this.agent_name_cache.clear();
        for (const a of agents) {
            if (a && a.id && a.name) {
                this.agent_name_cache.set(a.id, a.name);
            }
        }
    }

    private async handle_socket_message(data: Incoming_Socket_Message): Promise<void> {
        try {
            // Notify raw listeners first
            this.router.emit('raw', data as unknown as Record<string, unknown>);

            switch (data.type) {
                case 'log':
                case 'thought':
                    this._handle_log_message(data);
                    break;
                case 'agent:message':
                    this._handle_agent_message(data);
                    break;
                case 'agent:create':
                case 'agent:update':
                case 'agent:status':
                case 'engine:ui_invalidate':
                    this._handle_agent_update(data);
                    break;
                case 'engine:health':
                    this._handle_health(data);
                    break;
                case 'agent:handoff':
                    this._handle_handoff(data);
                    break;
                case 'engine:mcp_pulse':
                    this._handle_mcp_pulse(data);
                    break;
                case 'trace:span':
                    this._handle_trace_span(data);
                    break;
                case 'trace:span_update':
                    this._handle_trace_span_update(data);
                    break;
                case 'engine:scheduled_job_complete':
                    this._handle_scheduled_job_complete(data);
                    break;
                default:
                    console.warn(`[Tadpole_OS] Received event with unrecognized type: ${(data as { type?: string }).type}`);
            }
        } catch (error) {
            console.error('[Tadpole_OS] Error in handle_socket_message:', error, data);
        }
    }

    private schedule_reconnect(): void {
        if (!this.reconnection_policy.should_retry(this.retry_count)) {
            event_bus.emit_log({
                source: 'System',
                text: `Tadpole_OS: Connection failed after ${MAX_RETRIES} attempts. Verify URL in Settings.`,
                severity: 'error'
            });
            return;
        }

        const delay = this.reconnection_policy.get_delay(this.retry_count);
        this.retry_count++;

        this.set_state('reconnecting');
        if (this.reconnect_timer) clearTimeout(this.reconnect_timer);
        this.reconnect_timer = setTimeout(() => {
            this.reconnect_timer = null;
            this.connect(true);
        }, delay);
    }

    disconnect(): void {
        this.is_explicitly_closed = true;
        this.retry_count = 0;
        this.send_queue = [];
        const target_state = this.state === 'error' ? 'error' : 'disconnected';
        this.set_state(target_state);
        if (this.reconnect_timer) clearTimeout(this.reconnect_timer);
        if (this.auth_timeout_timer) {
            clearTimeout(this.auth_timeout_timer);
            this.auth_timeout_timer = null;
        }
        this.socket?.close();
        this.socket = null;
    }
}

/** 
 * Lazy constructor to avoid side-effects at import time.
 */
let instance: Tadpole_OS_Socket_Client | null = null;
export const get_tadpole_os_socket = (): Tadpole_OS_Socket_Client => {
    if (!instance) {
        instance = new Tadpole_OS_Socket_Client();
    }
    return instance;
};

// Metadata: [socket]
