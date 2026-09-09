/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / websocket_connection
 * - **Primary Entrypoints**: `WebSocketConnection`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Tadpole_OS]`
 * - **Witness Tests**: none declared
 */

import { is_allowed_origin } from '../security/origin_guard';
import { ConnectionMetrics } from '../observability/connection_metrics';
import { ReconnectionPolicy } from './reconnection_policy';
import { HandshakeHandler } from './handshake';
import { ProtocolCodec } from '../codec/protocol_codec';
import { MAX_BINARY_FRAME_SIZE, MAX_TEXT_FRAME_SIZE } from '../codec/frame_limits';
import { sanitize_object } from '../codec/sanitizer';
import type { Connection_State, State_Listener } from '../types/connection_state';
import type { Incoming_Socket_Message } from '../types/events';
import { event_bus } from '../../event_bus';

export class WebSocketConnection {
    private socket: WebSocket | null = null;
    private reconnect_timer: ReturnType<typeof setTimeout> | null = null;
    private ping_timer: ReturnType<typeof setInterval> | null = null;
    private idle_timer: ReturnType<typeof setInterval> | null = null;
    
    private state: Connection_State = 'disconnected';
    private is_explicitly_closed = false;
    private retry_count = 0;
    private generation = 0;
    private last_activity_time = 0;
    
    private send_queue: string[] = [];
    private state_listeners = new Set<State_Listener>();
    private message_listeners = new Set<(msg: Incoming_Socket_Message) => void>();
    private binary_listeners = new Set<(type: 'audio' | 'pulse', payload: unknown) => void>();

    private readonly url_provider: () => string;
    private readonly token_provider: () => string;
    private readonly reconnection_policy: ReconnectionPolicy;
    private readonly handshake_handler: HandshakeHandler;
    private readonly metrics: ConnectionMetrics;

    private readonly heartbeat_interval_ms: number;
    private readonly idle_timeout_ms: number;

    constructor(
        url_provider: () => string,
        token_provider: () => string,
        reconnection_policy = new ReconnectionPolicy(),
        metrics = new ConnectionMetrics(),
        heartbeat_interval_ms = 30000,
        idle_timeout_ms = 120000
    ) {
        this.url_provider = url_provider;
        this.token_provider = token_provider;
        this.reconnection_policy = reconnection_policy;
        this.metrics = metrics;
        this.handshake_handler = new HandshakeHandler();
        
        this.heartbeat_interval_ms = heartbeat_interval_ms;
        this.idle_timeout_ms = idle_timeout_ms;

        // Hook up handshake listeners (stale-generation guarded)
        this.handshake_handler.on_success(() => {
            if (this.state !== 'authenticating' || !this.socket) {
                return;
            }
            this.set_state('connected');
            event_bus.emit_log({
                source: 'System',
                text: 'Connected to TadpoleOS Log Stream.',
                severity: 'success'
            });
            this.start_heartbeat();
            this.flush_queue();
        });

        this.handshake_handler.on_failure((reason) => {
            if (this.state !== 'authenticating' || !this.socket) {
                return;
            }
            this.metrics.auth_timeouts++;
            this.set_state('error');
            event_bus.emit_log({
                source: 'System',
                text: `Tadpole_OS: Authentication failed. ${reason}`,
                severity: 'error'
            });
            this.disconnect();
        });
    }

    public get_state(): Connection_State {
        return this.state;
    }

    public on_state_change(cb: State_Listener): () => void {
        this.state_listeners.add(cb);
        cb(this.state);
        return () => this.state_listeners.delete(cb);
    }

    public on_message(cb: (msg: Incoming_Socket_Message) => void): () => void {
        this.message_listeners.add(cb);
        return () => this.message_listeners.delete(cb);
    }

    public on_binary(cb: (type: 'audio' | 'pulse', payload: unknown) => void): () => void {
        this.binary_listeners.add(cb);
        return () => this.binary_listeners.delete(cb);
    }

    private set_state(new_state: Connection_State): void {
        if (this.state !== new_state) {
            this.state = new_state;
            this.state_listeners.forEach(cb => cb(new_state));
        }
    }

    public connect(is_reconnect = false): void {
        if (is_reconnect && this.is_explicitly_closed) {
            return;
        }

        if (this.socket || this.reconnect_timer || this.state === 'connected' || this.state === 'authenticating') {
            return;
        }

        if (!is_reconnect) {
            this.is_explicitly_closed = false;
            this.retry_count = 0;
        }

        this.set_state('connecting');

        const token = this.token_provider().trim();
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
            ws_url = this.url_provider();
            if (!is_allowed_origin(ws_url)) {
                this.set_state('error');
                event_bus.emit_log({
                    source: 'System',
                    text: 'Tadpole_OS: Connection to origin refused. Host is not in allowed origins list.',
                    severity: 'error'
                });
                return;
            }
        } catch {
            this.set_state('error');
            event_bus.emit_log({
                source: 'System',
                text: 'Tadpole_OS: Connection failed due to invalid URL format.',
                severity: 'error'
            });
            return;
        }

        const gen = ++this.generation;

        try {
            const ws = new WebSocket(ws_url, ['tadpole-pulse-v1']);
            ws.binaryType = 'arraybuffer';
            this.socket = ws;

            ws.onopen = () => {
                if (this.socket !== ws || this.generation !== gen) return;
                this.retry_count = 0;
                this.last_activity_time = Date.now();
                this.set_state('authenticating');
                this.handshake_handler.begin(ws, token);
            };

            ws.onmessage = (event) => {
                if (this.socket !== ws || this.generation !== gen) return;
                this.last_activity_time = Date.now();

                if (event.data instanceof ArrayBuffer) {
                    if (event.data.byteLength > MAX_BINARY_FRAME_SIZE) {
                        this.metrics.oversized_frames_dropped++;
                        console.warn(`[Tadpole_OS] Rejected binary frame exceeding maximum size of ${MAX_BINARY_FRAME_SIZE} bytes: ${event.data.byteLength}`);
                        return;
                    }
                    if (this.state !== 'connected') {
                        // Drop binary frames before authentication is complete
                        return;
                    }
                    try {
                        const decoded = ProtocolCodec.decode_binary(event.data);
                        this.metrics.messages_received++;
                        const frame_type = decoded.type;
                        if (frame_type === 'audio' || frame_type === 'pulse') {
                            this.binary_listeners.forEach(cb => cb(frame_type, decoded.payload));
                        }
                    } catch (e) {
                        this.metrics.decode_errors++;
                        console.error('[Tadpole_OS] Binary decode failed:', e);
                    }
                    return;
                }

                if (typeof event.data === 'string') {
                    if (event.data.length > MAX_TEXT_FRAME_SIZE) {
                        this.metrics.oversized_frames_dropped++;
                        console.warn(`[Tadpole_OS] Rejected text frame exceeding maximum size of ${MAX_TEXT_FRAME_SIZE} characters: ${event.data.length}`);
                        return;
                    }
                    try {
                        const parsed = ProtocolCodec.decode_json(event.data);
                        
                        // Let handshake handler handle it first if we are authenticating
                        if (this.state === 'authenticating') {
                            const handled = this.handshake_handler.handle_message(parsed);
                            if (handled) return;
                        }

                        // Guard against unauthenticated messages
                        if (this.state !== 'connected') {
                            return;
                        }

                        // App-level heartbeats/pongs maintain connectivity without bubbling to UI channels
                        if (parsed.type === 'heartbeat' || parsed.type === 'pong') {
                            return;
                        }

                        this.metrics.messages_received++;
                        const sanitized = sanitize_object(parsed) as Incoming_Socket_Message;
                        this.message_listeners.forEach(cb => cb(sanitized));
                    } catch (e) {
                        this.metrics.decode_errors++;
                        const preview = event.data.length > 200 ? `${event.data.slice(0, 200)}...` : event.data;
                        console.error('[Tadpole_OS] JSON parsing failed, received corrupted stream segment:', e, preview);
                    }
                }
            };

            ws.onclose = (ev) => {
                if (this.socket === ws && this.generation === gen) {
                    this.cleanup_connection();
                    if (ev && (ev.code === 4401 || ev.code === 4001)) {
                        this.set_state('error');
                        event_bus.emit_log({
                            source: 'System',
                            text: `Tadpole_OS: Connection closed due to invalid credentials (code ${ev.code}).`,
                            severity: 'error'
                        });
                        this.disconnect();
                        return;
                    }
                    if (ev && ev.code === 4408) {
                        event_bus.emit_log({
                            source: 'System',
                            text: 'Tadpole_OS: Authentication handshake timed out (code 4408). Retrying...',
                            severity: 'warning'
                        });
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
                if (this.socket === ws && this.generation === gen) {
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

    private cleanup_connection(): void {
        this.socket = null;
        this.handshake_handler.abort();
        this.stop_heartbeat();
    }

    private start_heartbeat(): void {
        this.stop_heartbeat();

        // Send app-level ping frame periodically to keep server side alive
        this.ping_timer = setInterval(() => {
            this.send_json({ type: 'ping' });
        }, this.heartbeat_interval_ms);

        // Decoupled idle check interval: runs at minimum 5s or 1/4th idle timeout
        const idle_check_interval = Math.min(this.heartbeat_interval_ms, Math.max(5000, Math.floor(this.idle_timeout_ms / 4)));
        this.idle_timer = setInterval(() => {
            const elapsed = Date.now() - this.last_activity_time;
            if (elapsed > this.idle_timeout_ms) {
                console.warn(`[Tadpole_OS] Telemetry connection idle for ${Math.round(elapsed / 1000)}s. Reconnecting...`);
                this.socket?.close();
            }
        }, idle_check_interval);
    }

    private stop_heartbeat(): void {
        if (this.ping_timer) {
            clearInterval(this.ping_timer);
            this.ping_timer = null;
        }
        if (this.idle_timer) {
            clearInterval(this.idle_timer);
            this.idle_timer = null;
        }
    }

    private schedule_reconnect(): void {
        if (!this.reconnection_policy.should_retry(this.retry_count)) {
            this.set_state('error');
            event_bus.emit_log({
                source: 'System',
                text: `Tadpole_OS: Connection failed after ${this.retry_count} attempts. Verify URL in Settings.`,
                severity: 'error'
            });
            return;
        }

        this.metrics.reconnects++;
        const delay = this.reconnection_policy.get_delay(this.retry_count, true);
        this.retry_count++;

        this.set_state('reconnecting');
        if (this.reconnect_timer) clearTimeout(this.reconnect_timer);
        this.reconnect_timer = setTimeout(() => {
            this.reconnect_timer = null;
            this.connect(true);
        }, delay);
    }

    public send_json(data: Record<string, unknown>): boolean {
        try {
            const payload = ProtocolCodec.encode_json(data);
            if (this.socket && this.socket.readyState === WebSocket.OPEN && this.state === 'connected') {
                this.socket.send(payload);
                return true;
            }
            if (this.state === 'connecting' || this.state === 'authenticating' || this.state === 'reconnecting') {
                if (this.send_queue.length >= 100) {
                    this.metrics.queue_drops++;
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

    private flush_queue(): void {
        while (this.send_queue.length > 0) {
            const msg = this.send_queue.shift();
            try {
                if (msg && this.socket && this.socket.readyState === WebSocket.OPEN) {
                    this.socket.send(msg);
                }
            } catch (error) {
                console.error('[Tadpole_OS] Queue flush failed for message:', error);
                break;
            }
        }
    }

    public disconnect(): void {
        this.is_explicitly_closed = true;
        this.retry_count = 0;
        this.send_queue = [];
        const target_state = this.state === 'error' ? 'error' : 'disconnected';
        this.set_state(target_state);
        
        if (this.reconnect_timer) {
            clearTimeout(this.reconnect_timer);
            this.reconnect_timer = null;
        }
        
        const active_socket = this.socket;
        this.cleanup_connection();
        
        if (active_socket) {
            active_socket.close();
        }
    }
}
