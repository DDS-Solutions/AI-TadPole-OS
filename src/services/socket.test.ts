/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the WebSocket communication layer for real-time engine synchronization.** 
 * Tests the automatic reconnection logic, heartbeats, and payload serialization for agent status, health signals, and binary audio streams. 
 * Mocks global `WebSocket` to simulate various network failure modes and protocol handshakes (bearer tokens/tadpole-pulse-v1).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Race conditions in state updates during rapid connect/disconnect cycles or failure to handle binary payloads in the handoff channel.
 * - **Telemetry Link**: Search `[socket.test]` in tracing logs.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get_tadpole_os_socket, Tadpole_OS_Socket_Client, is_allowed_origin, ProtocolCodec, ReconnectionPolicy } from './socket';
import { event_bus } from './event_bus';
import { use_settings_store } from '../stores/settings_store';
import { encode } from '@msgpack/msgpack';

vi.mock('./event_bus', () => ({
    event_bus: { 
        emit: vi.fn(),
        emit_log: vi.fn(),
        subscribe_traces: vi.fn(() => vi.fn()),
        emit_trace: vi.fn(),
    },
}));

vi.mock('../stores/settings_store', () => {
    let listeners: ((state: { settings: { tadpole_os_url: string; tadpole_os_api_key: string } }) => void)[] = [];
    const default_settings = { tadpole_os_url: 'http://localhost:8000', tadpole_os_api_key: 'test-key' };
    let current_settings = { ...default_settings };
    return {
        get_settings: vi.fn(() => current_settings),
        use_settings_store: {
            subscribe: vi.fn((fn: (state: { settings: { tadpole_os_url: string; tadpole_os_api_key: string } }) => void) => {
                listeners.push(fn);
                return () => { listeners = listeners.filter(l => l !== fn); };
            }),
            getState: vi.fn(() => ({ settings: current_settings })),
            __triggerChange: (state: { settings: { tadpole_os_url: string; tadpole_os_api_key: string } }) => {
                if (state.settings) {
                    current_settings = { ...current_settings, ...state.settings };
                }
                listeners.forEach(l => l({ settings: current_settings } as any));
            },
            __reset: () => {
                current_settings = { ...default_settings };
            },
        }
    };
});

vi.mock('../stores/agent_store', () => ({
    use_agent_store: {
        getState: vi.fn(() => ({
            agents: [
                { id: '1', name: 'Agent of Nine' },
                { id: '2', name: 'Tadpole Alpha' }
            ]
        }))
    }
}));

vi.mock('../stores/sovereign_store', () => ({
    use_sovereign_store: {
        getState: vi.fn(() => ({
            messages: [],
            active_scope: 'global',
            add_message: vi.fn(),
            update_message: vi.fn(),
            append_message_part: vi.fn(() => false),
            get_message_by_id: vi.fn()
        }))
    }
}));

interface Mock_Web_Socket {
    binaryType: string;
    readyState: number;
    send: ReturnType<typeof vi.fn>;
    close: ReturnType<typeof vi.fn>;
    onopen: (() => void) | null;
    onmessage: ((ev: { data: string | ArrayBuffer | Blob }) => void) | null;
    onclose: ((ev?: { code: number }) => void) | null;
    onerror: (() => void) | null;
}

describe('Tadpole_OS_Socket_Client', () => {
    let mock_web_socket: Mock_Web_Socket;
    let ws_constructor: ReturnType<typeof vi.fn>;
    
    beforeEach(() => {
        vi.clearAllMocks();
        vi.useFakeTimers();
        (use_settings_store as any).__reset();
        
        get_tadpole_os_socket().set_agent_name_cache([
            { id: '1', name: 'Agent of Nine' },
            { id: '2', name: 'Tadpole Alpha' }
        ]);
        
        mock_web_socket = {
            binaryType: 'blob',
            readyState: 0, // CONNECTING
            send: vi.fn(),
            close: vi.fn(() => {
                mock_web_socket.readyState = 3; // CLOSED
                if (typeof mock_web_socket.onclose === 'function') {
                    mock_web_socket.onclose();
                }
            }),
            onopen: null,
            onmessage: null,
            onclose: null,
            onerror: null,
        };
        
        ws_constructor = vi.fn();
        class Dummy_Web_Socket {
            static CONNECTING = 0;
            static OPEN = 1;
            static CLOSING = 2;
            static CLOSED = 3;

            CONNECTING = 0;
            OPEN = 1;
            CLOSING = 2;
            CLOSED = 3;

            constructor(url: string, protocols?: string[]) {
                (ws_constructor as unknown as (url: string, protocols?: string[]) => void)(url, protocols);
                return mock_web_socket as any;
            }
        }
        
        vi.stubGlobal('WebSocket', Dummy_Web_Socket);
    });

    afterEach(() => {
        vi.unstubAllGlobals();
        vi.useRealTimers();
        Tadpole_OS_Socket_Client.reset();
    });

    function fully_connect(socket: Tadpole_OS_Socket_Client) {
        socket.connect();
        if (mock_web_socket.onopen) {
            mock_web_socket.readyState = 1;
            mock_web_socket.onopen();
        }
        if (mock_web_socket.onmessage) {
            mock_web_socket.onmessage({ data: JSON.stringify({ type: 'auth_ok' }) });
        }
    }

    it('should initialize and connect, updating state correctly', () => {
        const socket = get_tadpole_os_socket();
        const status_listener = vi.fn();
        socket.subscribe_status(status_listener);
        
        expect(status_listener).toHaveBeenCalledWith('disconnected');
        
        socket.connect();
        expect(socket.get_connection_state()).toBe('connecting');
        expect(ws_constructor).toHaveBeenCalledWith('ws://localhost:8000/v1/engine/ws', ['tadpole-pulse-v1']);
        
        if (mock_web_socket.onopen) {
            mock_web_socket.readyState = 1; // OPEN
            mock_web_socket.onopen();
        }
        expect(socket.get_connection_state()).toBe('authenticating');
        expect(mock_web_socket.send).toHaveBeenCalledWith(JSON.stringify({ type: 'auth', token: 'test-key' }));

        if (mock_web_socket.onmessage) {
            mock_web_socket.onmessage({ data: JSON.stringify({ type: 'auth_ok' }) });
        }
        expect(socket.get_connection_state()).toBe('connected');
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining('Connected') }));
    });

    it('should handle auth_error correctly, transition to error state, and disconnect', () => {
        const socket = get_tadpole_os_socket();
        socket.connect();
        if (mock_web_socket.onopen) {
            mock_web_socket.readyState = 1;
            mock_web_socket.onopen();
        }
        expect(socket.get_connection_state()).toBe('authenticating');
        
        if (mock_web_socket.onmessage) {
            mock_web_socket.onmessage({ data: JSON.stringify({ type: 'auth_error', message: 'Invalid token' }) });
        }
        expect(socket.get_connection_state()).toBe('error');
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Authentication failed')
        }));
    });

    it('should handle close code 4001 as a terminal auth failure', () => {
        const socket = get_tadpole_os_socket();
        socket.connect();
        if (mock_web_socket.onopen) {
            mock_web_socket.readyState = 1;
            mock_web_socket.onopen();
        }
        
        if (mock_web_socket.onclose) {
            mock_web_socket.onclose({ code: 4001 } as any);
        }
        expect(socket.get_connection_state()).toBe('error');
    });

    it('should handle disconnects and trigger exponential backoff reconnects', () => {
        const socket = get_tadpole_os_socket();
        fully_connect(socket);
        expect(socket.get_connection_state()).toBe('connected');
        
        if (mock_web_socket.onclose) mock_web_socket.onclose({ code: 1000 } as any);
        expect(socket.get_connection_state()).toBe('reconnecting');
        
        vi.advanceTimersByTime(2000);
        expect(ws_constructor).toHaveBeenCalledTimes(2);
    });

    it('should handle manual disconnect and not auto-reconnect', () => {
        const socket = get_tadpole_os_socket();
        fully_connect(socket);
        
        socket.disconnect();
        expect(socket.get_connection_state()).toBe('disconnected');
        
        vi.advanceTimersByTime(2000);
        expect(ws_constructor).toHaveBeenCalledTimes(1);
    });
    
    it('should reconnect when settings change', async () => {
        const socket = get_tadpole_os_socket();
        fully_connect(socket);
        expect(socket.get_connection_state()).toBe('connected');
        
        // Use localhost to pass allowed origins check
        (use_settings_store as any).__triggerChange({ settings: { tadpole_os_url: 'http://localhost:9000', tadpole_os_api_key: 'key2' } });
        
        expect(mock_web_socket.close).toHaveBeenCalled();
        expect(ws_constructor).toHaveBeenCalledTimes(2);
        expect(ws_constructor).toHaveBeenLastCalledWith('ws://localhost:9000/v1/engine/ws', ['tadpole-pulse-v1']);
    });

    it('should refuse to connect when the API token is missing', () => {
        (use_settings_store as any).__triggerChange({ settings: { tadpole_os_api_key: '' } });

        const socket = get_tadpole_os_socket();
        socket.connect();

        expect(socket.get_connection_state()).toBe('error');
        expect(ws_constructor).not.toHaveBeenCalled();
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Missing API token')
        }));
    });

    it('should deserialize messages correctly and emit events', async () => {
        const socket = get_tadpole_os_socket();
        fully_connect(socket);
        
        const agent_listener = vi.fn();
        socket.subscribe('agentUpdates', agent_listener);

        if (mock_web_socket.onmessage) mock_web_socket.onmessage({ data: JSON.stringify({ type: 'log', text: 'hello', level: 'info', agent_id: '1' }) });
        
        await vi.waitFor(() => {
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ 
                text: 'hello', 
                severity: 'info',
                agent_name: 'Agent of Nine'
            }));
        });
        
        if (mock_web_socket.onmessage) mock_web_socket.onmessage({ data: JSON.stringify({ type: 'agent:status', status: 'thinking', agent_id: '1' }) });
        
        await vi.waitFor(() => {
            expect(agent_listener).toHaveBeenCalledWith(expect.objectContaining({ 
                type: 'agent:update', 
                data: { status: 'thinking' } 
            }));
        });
    });

    it('should dispatch ArrayBuffer to audio stream listeners only after connected', () => {
        const socket = get_tadpole_os_socket();
        
        const audio_listener = vi.fn();
        socket.subscribe_audio_stream(audio_listener);

        const packet = new Uint8Array(9);
        packet[0] = 0x01; // Audio header

        // If not connected yet (in authenticating state), binary frames are dropped
        socket.connect();
        if (mock_web_socket.onopen) {
            mock_web_socket.readyState = 1;
            mock_web_socket.onopen();
        }
        if (mock_web_socket.onmessage) mock_web_socket.onmessage({ data: packet.buffer });
        expect(audio_listener).not.toHaveBeenCalled();

        // Send auth_ok to connect fully
        if (mock_web_socket.onmessage) mock_web_socket.onmessage({ data: JSON.stringify({ type: 'auth_ok' }) });
        if (mock_web_socket.onmessage) mock_web_socket.onmessage({ data: packet.buffer });
        expect(audio_listener).toHaveBeenCalled();
    });

    it('should handle websocket error event', () => {
        const socket = get_tadpole_os_socket();
        socket.connect();
        if (mock_web_socket.onerror) mock_web_socket.onerror();
        expect(socket.get_connection_state()).toBe('reconnecting');
    });

    it('should handle connection failure gracefully', () => {
        class Fails_Web_Socket {
            static CONNECTING = 0;
            static OPEN = 1;
            static CLOSING = 2;
            static CLOSED = 3;
            constructor() { throw new Error('Network fail'); }
        }
        vi.stubGlobal('WebSocket', Fails_Web_Socket);
        const socket = get_tadpole_os_socket();
        socket.connect();
        expect(socket.get_connection_state()).toBe('reconnecting');
    });

    it('should hit MAX_RETRIES limit safely', () => {
        const socket = get_tadpole_os_socket();
        fully_connect(socket);
        vi.mocked(event_bus.emit_log).mockClear();

        for(let i=0; i<10; i++) {
           if (mock_web_socket.onclose) mock_web_socket.onclose({ code: 1000 } as any);
           vi.advanceTimersByTime(31000); 
        }
        
        if (mock_web_socket.onclose) mock_web_socket.onclose({ code: 1000 } as any);

        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ severity: 'error', text: expect.stringContaining('10 attempts') }));
    });

    it('should manage connection via reference counting and clean up after delay', () => {
        const socket = get_tadpole_os_socket();
        
        const unsub1 = socket.subscribe('agentUpdates', vi.fn());
        expect(socket.get_connection_state()).toBe('connecting');
        
        if (mock_web_socket.onopen) {
            mock_web_socket.readyState = 1;
            mock_web_socket.onopen();
        }
        if (mock_web_socket.onmessage) {
            mock_web_socket.onmessage({ data: JSON.stringify({ type: 'auth_ok' }) });
        }
        expect(socket.get_connection_state()).toBe('connected');
        
        const unsub2 = socket.subscribe('health', vi.fn());
        
        // Unsubscribing 1 should not disconnect
        unsub1();
        vi.advanceTimersByTime(1100);
        expect(socket.get_connection_state()).toBe('connected');
        
        // Unsubscribing 2 should trigger disconnect after 1000ms delay
        unsub2();
        expect(socket.get_connection_state()).toBe('connected');
        
        vi.advanceTimersByTime(1100);
        expect(socket.get_connection_state()).toBe('disconnected');
    });

    describe('Origin Allowlist Validation', () => {
        it('should allow loopback/localhost connections by default', () => {
            expect(is_allowed_origin('ws://localhost:8000/ws')).toBe(true);
            expect(is_allowed_origin('ws://127.0.0.1:8000/ws')).toBe(true);
            expect(is_allowed_origin('ws://[::1]:8000/ws')).toBe(true);
        });

        it('should block arbitrary external domains by default', () => {
            expect(is_allowed_origin('ws://malicious.com/ws')).toBe(false);
            expect(is_allowed_origin('ws://attacker-controlled-server.net/ws')).toBe(false);
        });

        it('should allow connection if hostname is configured in build-time environment allowlist', () => {
            vi.stubEnv('VITE_ALLOWED_ORIGINS', 'trusted-subdomain.com,another-site.org');
            expect(is_allowed_origin('ws://trusted-subdomain.com/ws')).toBe(true);
            expect(is_allowed_origin('ws://another-site.org/ws')).toBe(true);
            expect(is_allowed_origin('ws://malicious.com/ws')).toBe(false);
            vi.unstubAllEnvs();
        });

        it('should support checking against explicitly passed runtime origins list', () => {
            const runtime_list = ['my-domain.com'];
            expect(is_allowed_origin('ws://my-domain.com/ws', runtime_list)).toBe(true);
            expect(is_allowed_origin('ws://evil.com/ws', runtime_list)).toBe(false);
        });
    });

    describe('Send Queueing', () => {
        it('should queue messages sent while connecting and flush them on open', () => {
            const socket = get_tadpole_os_socket();
            socket.connect();
            
            expect(socket.get_connection_state()).toBe('connecting');
            
            const payload = { event: 'mission:abort' };
            const queued = socket.send_json(payload);
            expect(queued).toBe(true);
            expect(mock_web_socket.send).not.toHaveBeenCalled();

            if (mock_web_socket.onopen) {
                mock_web_socket.readyState = 1;
                mock_web_socket.onopen();
            }
            // Transition to authenticating and send auth frame
            expect(mock_web_socket.send).toHaveBeenCalledWith(JSON.stringify({ type: 'auth', token: 'test-key' }));
            
            // Server responds with auth_ok
            if (mock_web_socket.onmessage) {
                mock_web_socket.onmessage({ data: JSON.stringify({ type: 'auth_ok' }) });
            }
            
            expect(mock_web_socket.send).toHaveBeenCalledWith(JSON.stringify(payload));
        });

        it('should enforce MAX_QUEUE_SIZE of 100 messages and drop the oldest', () => {
            const socket = get_tadpole_os_socket();
            socket.connect();
            
            // Send 105 messages
            for (let i = 0; i < 105; i++) {
                socket.send_json({ id: i });
            }
            
            // Connect and authenticate
            if (mock_web_socket.onopen) {
                mock_web_socket.readyState = 1;
                mock_web_socket.onopen();
            }
            if (mock_web_socket.onmessage) {
                mock_web_socket.onmessage({ data: JSON.stringify({ type: 'auth_ok' }) });
            }

            // Verify that only the last 100 messages are sent (0 to 4 are dropped)
            expect(mock_web_socket.send).toHaveBeenCalledTimes(101); // 1 auth frame + 100 queued
            expect(mock_web_socket.send).not.toHaveBeenCalledWith(JSON.stringify({ id: 0 }));
            expect(mock_web_socket.send).not.toHaveBeenCalledWith(JSON.stringify({ id: 4 }));
            expect(mock_web_socket.send).toHaveBeenCalledWith(JSON.stringify({ id: 5 }));
            expect(mock_web_socket.send).toHaveBeenCalledWith(JSON.stringify({ id: 104 }));
        });

        it('should clear queued messages upon calling disconnect()', () => {
            const socket = get_tadpole_os_socket();
            socket.connect();
            socket.send_json({ event: 'discard' });
            
            socket.disconnect();
            expect(socket.get_connection_state()).toBe('disconnected');
            
            // Reconnect
            fully_connect(socket);
            
            expect(mock_web_socket.send).not.toHaveBeenCalledWith(JSON.stringify({ event: 'discard' }));
        });
    });

    describe('Max Frame Size Enforcement', () => {
        it('should reject and drop binary frames exceeding 1MB', () => {
            const socket = get_tadpole_os_socket();
            fully_connect(socket);

            const audio_listener = vi.fn();
            socket.subscribe_audio_stream(audio_listener);

            const console_warn_spy = vi.spyOn(console, 'warn').mockImplementation(() => {});
            
            // 2MB buffer containing audio header
            const oversized_buf = new ArrayBuffer(2 * 1024 * 1024);
            const view = new Uint8Array(oversized_buf);
            view[0] = 0x01; // Audio header

            if (mock_web_socket.onmessage) {
                mock_web_socket.onmessage({ data: oversized_buf });
            }

            expect(audio_listener).not.toHaveBeenCalled();
            expect(console_warn_spy).toHaveBeenCalledWith(expect.stringContaining('Rejected binary frame'));
            console_warn_spy.mockRestore();
        });

        it('should reject and drop text frames exceeding 5MB', () => {
            const socket = get_tadpole_os_socket();
            fully_connect(socket);

            const raw_listener = vi.fn();
            socket.subscribe_raw(raw_listener);

            const console_warn_spy = vi.spyOn(console, 'warn').mockImplementation(() => {});
            
            const oversized_text = 'a'.repeat(6 * 1024 * 1024); // 6MB string

            if (mock_web_socket.onmessage) {
                mock_web_socket.onmessage({ data: oversized_text });
            }

            expect(raw_listener).not.toHaveBeenCalled();
            expect(console_warn_spy).toHaveBeenCalledWith(expect.stringContaining('Rejected text frame'));
            console_warn_spy.mockRestore();
        });
    });

    describe('Store Unsubscription & Cleanup', () => {
        it('should lazily subscribe on first connect and unsubscribe from settings store when destroyed', () => {
            const unsub_spy = vi.fn();
            vi.mocked(use_settings_store.subscribe).mockReturnValue(unsub_spy);

            const client = new Tadpole_OS_Socket_Client();
            expect(use_settings_store.subscribe).not.toHaveBeenCalled();

            client.connect();
            expect(use_settings_store.subscribe).toHaveBeenCalled();

            client.destroy();
            expect(unsub_spy).toHaveBeenCalled();
        });
    });

    describe('ProtocolCodec and ReconnectionPolicy unit tests', () => {
        it('ProtocolCodec should decode valid MessagePack swarm pulse frames', () => {
            const mock_pulse = { type: 'pulse', nodes: [{ id: '1', status: 1, battery: 100, signal: 100, progress: 0 }] };
            const encoded = encode(mock_pulse);
            const frame = new Uint8Array(encoded.byteLength + 1);
            frame[0] = 0x02; // BINARY_HEADER_SWARM_PULSE
            frame.set(new Uint8Array(encoded), 1);

            const decoded = ProtocolCodec.decode_binary(frame.buffer);
            expect(decoded.type).toBe('pulse');
            expect((decoded.payload as any).type).toBe('pulse');
        });

        it('ReconnectionPolicy should return correct exponential delay', () => {
            const policy = new ReconnectionPolicy(1000, 10000, 5);
            expect(policy.get_delay(0)).toBe(1000);
            expect(policy.get_delay(1)).toBe(2000);
            expect(policy.get_delay(2)).toBe(4000);
            expect(policy.get_delay(3)).toBe(8000);
            expect(policy.get_delay(4)).toBe(10000); // capped at max
            
            expect(policy.should_retry(4)).toBe(true);
            expect(policy.should_retry(5)).toBe(false);
        });
    });

    describe('Prototype Pollution Sanitization', () => {
        it('should strip __proto__, prototype, and constructor keys recursively', () => {
            const socket = get_tadpole_os_socket();
            fully_connect(socket);

            const evil_payload = JSON.parse(JSON.stringify({
                type: 'log',
                text: 'injection payload',
                __proto__: { polluted: true },
                nested: {
                    constructor: { polluted: true },
                    prototype: { polluted: true },
                    safe: 123
                }
            }));

            if (mock_web_socket.onmessage) {
                mock_web_socket.onmessage({ data: JSON.stringify(evil_payload) });
            }

            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: 'injection payload',
                metadata: expect.any(Object)
            }));

            const args = vi.mocked(event_bus.emit_log).mock.calls;
            const lastCallMetadata = args[args.length - 1][0].metadata as any;
            
            expect(lastCallMetadata.__proto__).toBeUndefined();
            expect(lastCallMetadata.constructor).toBeUndefined();
            expect(lastCallMetadata.prototype).toBeUndefined();
            expect(lastCallMetadata.nested.constructor).toBeUndefined();
            expect(lastCallMetadata.nested.prototype).toBeUndefined();
            expect(lastCallMetadata.nested.safe).toBe(123);
        });
    });

    describe('Authentication Timeout', () => {
        it('should trigger connection timeout, transition to error state, and disconnect after 10s', () => {
            const socket = get_tadpole_os_socket();
            socket.connect();
            
            if (mock_web_socket.onopen) {
                mock_web_socket.readyState = 1;
                mock_web_socket.onopen();
            }
            expect(socket.get_connection_state()).toBe('authenticating');

            // Advance time by 10 seconds (10000ms)
            vi.advanceTimersByTime(10000);

            expect(socket.get_connection_state()).toBe('error');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('authentication timeout')
            }));
            expect(mock_web_socket.close).toHaveBeenCalled();
        });
    });
});

// Metadata: [socket_test]
