/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Networking**: Manages the real-time WebSocket telemetry and relay between the UI and the backend. 
 * Orchestrates event normalization for the `event_bus` and `sovereign_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Connection timeouts (MAX_RETRIES reached), invalid API key/URL format causing 403/404, or MessagePack decoding errors for binary pulses.
 * - **Telemetry Link**: Search for `[Tadpole_OS_Socket]` or `bearer.tadpole` in browser/proxy logs.
 */

import { event_bus } from './event_bus';
import { get_settings, use_settings_store } from '../stores/settings_store';
import type { Message_Part, Swarm_Pulse } from '../types';
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

/** Maximum number of reconnect attempts before giving up. */
const MAX_RETRIES = 10;
/** Initial backoff delay in ms. */
const INITIAL_BACKOFF = 2000;
/** Maximum backoff delay in ms. */
const MAX_BACKOFF = 30000;

/** Connection states for the socket. */
export type Connection_State = 'connecting' | 'connected' | 'disconnected' | 'reconnecting' | 'error';

type State_Listener = (state: Connection_State) => void;

type Store_Bindings = {
    use_agent_store: typeof import('../stores/agent_store').use_agent_store;
    use_sovereign_store: typeof import('../stores/sovereign_store').use_sovereign_store;
    use_trace_store: typeof import('../stores/trace_store').use_trace_store;
};

/**
 * Tadpole_OS_Socket_Client
 * WebSocket client for streaming real-time logs from TadpoleOS.
 * Reads the connection URL from the centralized settings store.
 * Refactored for strict snake_case compliance for backend parity.
 */
class Tadpole_OS_Socket_Client {
    private socket: WebSocket | null = null;
    private reconnect_timer: ReturnType<typeof setTimeout> | null = null;
    private is_explicitly_closed = false;
    private retry_count = 0;
    private last_url = '';
    private last_key = '';
    private store_bindings_promise: Promise<Store_Bindings> | null = null;

    // State Management
    private state: Connection_State = 'disconnected';
    private state_listeners: State_Listener[] = [];

    /** Subscribe to connection state changes. */
    subscribe_status(listener: State_Listener): () => void {
        this.state_listeners.push(listener);
        listener(this.state); // Immediate update
        return () => {
            this.state_listeners = this.state_listeners.filter(l => l !== listener);
        };
    }

    constructor() {
        // REACTIVE RECONNECTION: Watch for infrastructure changes
        // Initialize trackers
        const initial = get_settings();
        this.last_url = initial.tadpole_os_url;
        this.last_key = initial.tadpole_os_api_key;

        use_settings_store.subscribe((state) => {
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

    private set_state(new_state: Connection_State) {
        if (this.state !== new_state) {
            this.state = new_state;
            this.state_listeners.forEach(l => l(new_state));
        }
    }

    // Agent Update Management
    private agent_update_listeners: ((update: Agent_Update_Event) => void)[] = [];

    subscribe_agent_updates(listener: (update: Agent_Update_Event) => void): () => void {
        this.agent_update_listeners.push(listener);
        return () => {
            this.agent_update_listeners = this.agent_update_listeners.filter(l => l !== listener);
        };
    }

    // Health Management
    private health_listeners: ((health: Engine_Health_Event) => void)[] = [];

    subscribe_health(listener: (health: Engine_Health_Event) => void): () => void {
        this.health_listeners.push(listener);
        return () => {
            this.health_listeners = this.health_listeners.filter(l => l !== listener);
        };
    }

    // Handoff Management
    private handoff_listeners: ((handoff: Handoff_Event) => void)[] = [];

    subscribe_handoff(listener: (handoff: Handoff_Event) => void): () => void {
        this.handoff_listeners.push(listener);
        return () => {
            this.handoff_listeners = this.handoff_listeners.filter(l => l !== listener);
        };
    }

    // MCP Pulse Management
    private mcp_pulse_listeners: ((pulse: Mcp_Pulse_Event) => void)[] = [];

    subscribe_pulse(listener: (pulse: Mcp_Pulse_Event) => void): () => void {
        this.mcp_pulse_listeners.push(listener);
        return () => {
            this.mcp_pulse_listeners = this.mcp_pulse_listeners.filter(l => l !== listener);
        };
    }

    // Audio Stream Management
    private audio_stream_listeners: ((chunk: ArrayBuffer) => void)[] = [];

    subscribe_audio_stream(listener: (chunk: ArrayBuffer) => void): () => void {
        this.audio_stream_listeners.push(listener);
        return () => {
            this.audio_stream_listeners = this.audio_stream_listeners.filter(l => l !== listener);
        };
    }

    // Swarm Pulse Management
    private pulse_listeners: ((pulse: Swarm_Pulse) => void)[] = [];

    subscribe_swarm_pulse(listener: (pulse: Swarm_Pulse) => void): () => void {
        this.pulse_listeners.push(listener);
        return () => {
            this.pulse_listeners = this.pulse_listeners.filter(l => l !== listener);
        };
    }

    get_connection_state(): Connection_State {
        return this.state;
    }

    private get_store_bindings(): Promise<Store_Bindings> {
        if (!this.store_bindings_promise) {
            this.store_bindings_promise = Promise.all([
                import('../stores/agent_store'),
                import('../stores/sovereign_store'),
                import('../stores/trace_store')
            ]).then(([agent_store, sovereign_store, trace_store]) => ({
                use_agent_store: agent_store.use_agent_store,
                use_sovereign_store: sovereign_store.use_sovereign_store,
                use_trace_store: trace_store.use_trace_store
            }));
        }
        return this.store_bindings_promise;
    }

    /** Opens the WebSocket connection and begins listening for events. */
    connect() {
        // Guard: Don't connect if already connecting or connected
        if (this.socket || this.reconnect_timer || this.state === 'connected') {
            return;
        }

        this.is_explicitly_closed = false;
        this.set_state('connecting');

        // Get URL from centralized settings, converting http/https to ws/wss
        const { tadpole_os_url, tadpole_os_api_key } = get_settings();
        // Remove trailing slash if present, then replace http with ws
        const base_url = (tadpole_os_url || 'http://localhost:8000').trim().replace(/\/$/, '').replace(/^http/, 'ws');
        // Centralized token retrieval from SettingsStore
        const token = tadpole_os_api_key || 'tadpole-os-sidecar-default-2026';
        // SEC-01: Token sent via Sec-WebSocket-Protocol header, not URL query params.
        const ws_url = `${base_url}/v1/engine/ws`;

        try {
            const ws = new WebSocket(ws_url, [`bearer.${token}`, 'tadpole-pulse-v1']);
            ws.binaryType = 'arraybuffer';
            this.socket = ws;

            ws.onopen = () => {
                if (this.socket !== ws) return; // Guard against stale connections
                this.retry_count = 0; // Reset on successful connection
                this.set_state('connected');

                event_bus.emit_log({
                    source: 'System',
                    text: 'Connected to TadpoleOS Log Stream.',
                    severity: 'success'
                });
            };

            this.socket.onmessage = (event) => {
                if (event.data instanceof ArrayBuffer) {
                    const view = new Uint8Array(event.data);
                    const header = view[0];
                    const payload = event.data.slice(1);

                    if (header === 0x01) {
                        // Audio Stream
                        this.audio_stream_listeners.forEach(l => l(payload));
                    } else if (header === 0x02) {
                        // Swarm Pulse (MessagePack)
                        try {
                            const pulse = decode(payload) as Swarm_Pulse;
                            this.pulse_listeners.forEach(l => l(pulse));
                        } catch (e) {
                            console.error('[Tadpole_OS] Failed to decode swarm pulse:', e);
                        }
                    }
                    return;
                }

                try {
                    const data = JSON.parse(event.data);
                    this.handle_socket_message(data);
                } catch {
                    // Ignore malformed packets silently
                }
            };

            ws.onclose = () => {
                if (this.socket === ws) {
                    this.socket = null;
                    this.set_state('disconnected');
                    if (!this.is_explicitly_closed) {
                        this.schedule_reconnect();
                    }
                }
            };

            ws.onerror = () => {
                if (this.socket === ws) {
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

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    private async handle_socket_message(data: any) {
        const { use_agent_store, use_sovereign_store, use_trace_store } = await this.get_store_bindings();
        const agent_store = use_agent_store.getState();
        const sovereign_store = use_sovereign_store.getState();

        if (data.type === 'log' || data.type === 'thought') {
            const agent_name = agent_store.agents.find(a => String(a.id) === String(data.agent_id))?.name || data.agent_name;

            event_bus.emit_log({
                id: data.id || data.request_id || data.requestId, 
                source: data.agent_id ? 'Agent' : 'System',
                agent_id: data.agent_id,
                agent_name,
                text: data.text || data.message || JSON.stringify(data),
                severity: (data.level === 'error' ? 'error' : 'info'),
                metadata: data
            });

            // Bridge agent logs/thoughts to Sovereign_Chat
            if (data.agent_id && (data.type === 'thought' || (data.type === 'log' && data.level !== 'debug'))) {
                const agent = agent_store.agents.find(a => String(a.id) === String(data.agent_id));
                const resolved_name = agent?.name || data.agent_name || data.agent_id;

                const message_id = data.message_id || data.messageId || data.request_id || data.requestId || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : Date.now().toString());

                const new_part: Message_Part = data.type === 'thought'
                    ? { type: 'thought', content: data.message || data.text || '', status: 'done' }
                    : { type: 'text', content: data.message || data.text || '' };

                const did_update = sovereign_store.append_message_part(message_id, new_part);
                if (!did_update) {
                    sovereign_store.add_message({
                        id: message_id,
                        sender_id: String(data.agent_id),
                        sender_name: resolved_name as string,
                        agent_id: String(data.agent_id),
                        text: data.message || data.text || '',
                        parts: [new_part],
                        scope: sovereign_store.active_scope
                    });
                }
            }
        } else if (data.type === 'agent:message') {
            const agent_name = agent_store.agents.find(a => String(a.id) === String(data.agent_id))?.name || data.agent_name;

            event_bus.emit_log({
                id: data.id || data.message_id || data.messageId, 
                source: 'Agent',
                agent_id: data.agent_id,
                agent_name,
                text: data.text || 'Mission action complete.',
                severity: 'info',
                metadata: data
            });
            // Bridge to Sovereign_Chat
            const agent = agent_store.agents.find(a => a.id === data.agent_id);

            const resolved_name = agent?.name || data.agent_name || data.agent_id || 'Agent';
            const message_id = data.message_id || data.messageId || data.id || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : Date.now().toString());

            const new_part: Message_Part = { type: 'text' as const, content: data.text || data.message || 'Mission action complete.' };

            const did_update = sovereign_store.append_message_part(message_id as string, new_part);
            if (!did_update) {
                sovereign_store.add_message({
                    id: message_id as string,
                    sender_id: data.agent_id || 'system',
                    sender_name: resolved_name as string,
                    agent_id: data.agent_id, 
                    target_node: resolved_name as string,
                    text: data.text || data.message || 'Mission action complete.',
                    parts: [new_part],
                    scope: sovereign_store.active_scope
                });
            }
        } else if (data.type === 'agent:create' || data.type === 'agent:update' || data.type === 'agent:status' || data.type === 'engine:ui_invalidate') {
            const normalized_agent_id = data.agent_id ?? data.agentId ?? data.id;
            const normalized_data = data.type === 'agent:status'
                ? { ...data, type: 'agent:update', agent_id: normalized_agent_id, data: { status: data.status } }
                : { ...data, agent_id: normalized_agent_id };

            if (data.type === 'engine:ui_invalidate') {
                event_bus.emit_log({
                    source: 'System',
                    text: `UI Invalidated: ${data.resource}${data.id ? ` (#${data.id})` : ''}`,
                    severity: 'info'
                });
            }

            this.agent_update_listeners.forEach(l => l(normalized_data));
        } else if (data.type === 'engine:health') {
            this.health_listeners.forEach(l => l(data));
        } else if (data.type === 'agent:handoff') {
            this.handoff_listeners.forEach(l => l(data));
        } else if (data.type === 'engine:mcp_pulse') {
            this.mcp_pulse_listeners.forEach(l => l(data));
        } else if (data.type === 'trace:span') {
            use_trace_store.getState().add_span(data.span);
        } else if (data.type === 'trace:span_update') {
            use_trace_store.getState().update_span(data.span_id || data.spanId, data.update);
        } else if (data.type === 'engine:scheduled_job_complete') {
            event_bus.emit_log({
                source: 'System',
                text: `Scheduled Job '${data.job_name}' completed. Cost: $${data.cost_usd?.toFixed(4)}`,
                severity: data.status === 'failed' ? 'error' : 'success'
            });
        }
    }

    private schedule_reconnect() {
        if (this.retry_count >= MAX_RETRIES) {
            event_bus.emit_log({
                source: 'System',
                text: `Tadpole_OS: Connection failed after ${MAX_RETRIES} attempts. Verify URL in Settings.`,
                severity: 'error'
            });
            return;
        }

        const delay = Math.min(INITIAL_BACKOFF * Math.pow(2, this.retry_count), MAX_BACKOFF);
        this.retry_count++;

        this.set_state('reconnecting');
        if (this.reconnect_timer) clearTimeout(this.reconnect_timer);
        this.reconnect_timer = setTimeout(() => {
            this.reconnect_timer = null;
            this.connect();
        }, delay);
    }

    disconnect() {
        this.is_explicitly_closed = true;
        this.retry_count = 0;
        this.set_state('disconnected');
        if (this.reconnect_timer) clearTimeout(this.reconnect_timer);
        this.socket?.close();
    }
}

/** 
 * Lazy constructor to avoid side-effects at import time.
 */
let instance: Tadpole_OS_Socket_Client | null = null;
export const get_tadpole_os_socket = () => {
    if (!instance) {
        instance = new Tadpole_OS_Socket_Client();
    }
    return instance;
};

/** 
 * Type-safe Proxy for the Tadpole_OS_Socket_Client.
 */
export const tadpole_os_socket = new Proxy({} as Tadpole_OS_Socket_Client, {
    get(_, prop) {
        const inst = get_tadpole_os_socket();
        const value = inst[prop as keyof Tadpole_OS_Socket_Client];
        if (typeof value === 'function') {
            return (value as (...args: unknown[]) => unknown).bind(inst);
        }
        return value;
    },
    set(_, prop, value) {
        const inst = get_tadpole_os_socket();
        const key = prop as keyof Tadpole_OS_Socket_Client;
        // Type-safe property setting
        (inst[key] as typeof value) = value;
        return true;
    }
});

