/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[socket_manager]` in observability traces.
 */

import { get_settings, use_settings_store } from '../../stores/settings_store';
import { WebSocketConnection } from './transport/websocket_connection';
import { ReconnectionPolicy } from './transport/reconnection_policy';
import { ConnectionMetrics } from './observability/connection_metrics';
import { LogChannel } from './channels/log_channel';
import { AgentUpdateChannel } from './channels/agent_update_channel';
import { TraceChannel } from './channels/trace_channel';
import { HealthChannel } from './channels/health_channel';
import { AudioStreamChannel } from './channels/audio_stream_channel';
import { SwarmPulseChannel } from './channels/swarm_pulse_channel';
import { HandoffChannel } from './channels/handoff_channel';
import { McpPulseChannel } from './channels/mcp_pulse_channel';
import { BaseChannel } from './channels/channel';
import type { Connection_State, State_Listener } from './types/connection_state';
import type { 
    Incoming_Socket_Message, 
    Agent_Update_Event, 
    Engine_Health_Event, 
    Handoff_Event, 
    Mcp_Pulse_Event 
} from './types/events';
import type { Swarm_Pulse } from '../../types';

// RawChannel matches all messages for debugging/raw logging
class RawChannel extends BaseChannel<Record<string, unknown>> {
    readonly name = 'raw';
    matches(): boolean { return true; }
    handle(message: Incoming_Socket_Message): void {
        this.emit(message as unknown as Record<string, unknown>);
    }
}

export class SocketManager {
    private connection: WebSocketConnection | null = null;
    private connection_references = 0;
    private disconnect_timeout: ReturnType<typeof setTimeout> | null = null;
    private settings_unsubscribe: (() => void) | null = null;
    
    private last_url = '';
    private last_key = '';

    // Channels
    private log_channel = new LogChannel();
    private agent_update_channel = new AgentUpdateChannel();
    private trace_channel = new TraceChannel();
    private health_channel = new HealthChannel();
    private audio_stream_channel = new AudioStreamChannel();
    private swarm_pulse_channel = new SwarmPulseChannel();
    private handoff_channel = new HandoffChannel();
    private mcp_pulse_channel = new McpPulseChannel();
    private raw_channel = new RawChannel();

    private status_listeners = new Set<State_Listener>();
    private metrics = new ConnectionMetrics();

    constructor() {
        // Dynamic registration for JSON channel routing
        this.channels = [
            this.log_channel,
            this.agent_update_channel,
            this.trace_channel,
            this.health_channel,
            this.handoff_channel,
            this.mcp_pulse_channel,
            this.raw_channel
        ];
    }

    private channels: Array<{
        matches(msg: Incoming_Socket_Message): boolean;
        handle(msg: Incoming_Socket_Message): void;
        clear(): void;
    }>;

    public get_connection_state(): Connection_State {
        return this.connection ? this.connection.get_state() : 'disconnected';
    }

    public get_metrics(): ConnectionMetrics {
        return this.metrics;
    }

    public subscribe(channel: 'agentUpdates', listener: (data: Agent_Update_Event) => void): () => void;
    public subscribe(channel: 'health', listener: (data: Engine_Health_Event) => void): () => void;
    public subscribe(channel: 'handoff', listener: (data: Handoff_Event) => void): () => void;
    public subscribe(channel: 'status', listener: (data: Connection_State) => void): () => void;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    public subscribe(channel: string, listener: (...args: any[]) => void): () => void;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    public subscribe(channel: string, listener: (...args: any[]) => void): () => void {
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
            case 'pulse':
                unsubscribe_func = this.subscribe_pulse(listener as (pulse: Mcp_Pulse_Event) => void);
                break;
            case 'audio_stream':
                unsubscribe_func = this.subscribe_audio_stream(listener as (chunk: ArrayBuffer) => void);
                break;
            case 'swarm_pulse':
                unsubscribe_func = this.subscribe_swarm_pulse(listener as (pulse: Swarm_Pulse) => void);
                break;
            case 'raw':
                unsubscribe_func = this.subscribe_raw(listener as (event: Record<string, unknown>) => void);
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

    public subscribe_status(listener: State_Listener): () => void {
        this.status_listeners.add(listener);
        listener(this.get_connection_state());
        return () => {
            this.status_listeners.delete(listener);
        };
    }

    public subscribe_agent_updates(listener: (update: Agent_Update_Event) => void): () => void {
        return this.agent_update_channel.subscribe(listener);
    }

    public subscribe_health(listener: (health: Engine_Health_Event) => void): () => void {
        return this.health_channel.subscribe(listener);
    }

    public subscribe_handoff(listener: (handoff: Handoff_Event) => void): () => void {
        return this.handoff_channel.subscribe(listener);
    }

    public subscribe_pulse(listener: (pulse: Mcp_Pulse_Event) => void): () => void {
        return this.mcp_pulse_channel.subscribe(listener);
    }

    public subscribe_audio_stream(listener: (chunk: ArrayBuffer) => void): () => void {
        return this.audio_stream_channel.subscribe(listener);
    }

    public subscribe_swarm_pulse(listener: (pulse: Swarm_Pulse) => void): () => void {
        return this.swarm_pulse_channel.subscribe(listener);
    }

    public subscribe_raw(listener: (event: Record<string, unknown>) => void): () => void {
        return this.raw_channel.subscribe(listener);
    }

    public send_json(data: Record<string, unknown>): boolean {
        if (this.connection) {
            return this.connection.send_json(data);
        }
        return false;
    }

    public set_agent_name_cache(agents: Array<{ id: string; name: string }>): void {
        this.log_channel.set_agent_name_cache(agents);
    }

    private get_websocket_url(): string {
        const { tadpole_os_url } = get_settings();
        const raw_url = (tadpole_os_url || 'http://localhost:8000').trim();
        const sanitized_url = raw_url.replace(/\/$/, '');
        const ws_prefix = sanitized_url.startsWith('https') ? 'wss' : 'ws';
        const protocol_replaced = sanitized_url.replace(/^https?/, ws_prefix);
        return `${protocol_replaced}/v1/engine/ws`;
    }

    private get_websocket_token(): string {
        const { tadpole_os_api_key } = get_settings();
        return tadpole_os_api_key.trim();
    }

    public connect(is_reconnect = false): void {
        if (is_reconnect && !this.connection) return;

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

                    console.debug(`[Tadpole_OS] Infrastructure settings changed. Reconnecting...`);
                    this.disconnect();
                    this.connect();
                }
            });
        }

        if (!this.connection) {
            this.connection = new WebSocketConnection(
                () => this.get_websocket_url(),
                () => this.get_websocket_token(),
                new ReconnectionPolicy(),
                this.metrics
            );

            this.connection.on_state_change((state) => {
                this.status_listeners.forEach(cb => cb(state));
            });

            this.connection.on_message((msg) => {
                this.channels.forEach(channel => {
                    if (channel.matches(msg)) {
                        channel.handle(msg);
                    }
                });
            });

            this.connection.on_binary((type, payload) => {
                if (type === 'audio') {
                    this.audio_stream_channel.handle_binary(payload);
                } else if (type === 'pulse') {
                    this.swarm_pulse_channel.handle_binary(payload);
                }
            });
        }

        this.connection.connect(is_reconnect);
    }

    public disconnect(): void {
        if (this.connection) {
            this.connection.disconnect();
        }
    }

    public destroy(): void {
        this.disconnect();
        if (this.settings_unsubscribe) {
            this.settings_unsubscribe();
            this.settings_unsubscribe = null;
        }
        if (this.disconnect_timeout) {
            clearTimeout(this.disconnect_timeout);
            this.disconnect_timeout = null;
        }
        
        this.channels.forEach(c => c.clear());
        this.audio_stream_channel.clear();
        this.swarm_pulse_channel.clear();
        this.status_listeners.clear();
        this.metrics.reset();
        
        this.connection = null;
        this.connection_references = 0;
    }
}

// Metadata: [socket_manager]
