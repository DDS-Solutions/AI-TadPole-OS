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


/**
 * @file socket.test.ts
 * @description Suite for the Tadpole OS WebSocket client and real-time event orchestration.
 * @module Services/Socket
 * @testedBehavior
 * - Pulsing Connectivity: Verification of state transitions (connecting, connected, reconnecting).
 * - Exponential Backoff: Validation of retry logic and terminal failure thresholds.
 * - Multi-Protocol: Bearer token and pulse-v1 header injection checks.
 * - Reactive Updates: Subscription handling for agent status, health, and binary audio streams.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks global WebSocket to simulate various network failure modes and protocol handshakes.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get_tadpole_os_socket } from './socket';
import { event_bus } from './event_bus';
import { use_settings_store } from '../stores/settings_store';

vi.mock('./event_bus', () => ({
    event_bus: { 
        emit: vi.fn(),
        emit_log: vi.fn()
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
    send: ReturnType<typeof vi.fn>;
    close: ReturnType<typeof vi.fn>;
    onopen: (() => void) | null;
    onmessage: ((ev: { data: string | ArrayBuffer | Blob }) => void) | null;
    onclose: (() => void) | null;
    onerror: (() => void) | null;
}

describe('Tadpole_OS_Socket_Client', () => {
    let mock_web_socket: Mock_Web_Socket;
    let ws_constructor: ReturnType<typeof vi.fn>;
    
    beforeEach(() => {
        vi.clearAllMocks();
        vi.useFakeTimers();
        (use_settings_store as any).__reset();
        
        mock_web_socket = {
            binaryType: 'blob',
            send: vi.fn(),
            close: vi.fn(() => {
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
            constructor(url: string, protocols?: string[]) {
                (ws_constructor as unknown as (url: string, protocols?: string[]) => void)(url, protocols);
                return mock_web_socket;
            }
        }
        
        vi.stubGlobal('WebSocket', Dummy_Web_Socket);
    });

    afterEach(() => {
        vi.unstubAllGlobals();
        vi.useRealTimers();
        const socket = get_tadpole_os_socket();
        socket.disconnect();
        
        // Reset internal state for next test
        const s = socket as any;
        s.socket = null;
        s.reconnect_timer = null;
        s.state = 'disconnected';
        s.is_explicitly_closed = false;
        s.retry_count = 0;
        s.last_url = '';
        s.last_key = '';
        s.state_listeners = [];
        s.agent_update_listeners = [];
        s.health_listeners = [];
        s.handoff_listeners = [];
        s.mcp_pulse_listeners = [];
        s.audio_stream_listeners = [];
    });

    it('should initialize and connect, updating state correctly', () => {
        const socket = get_tadpole_os_socket();
        const status_listener = vi.fn();
        socket.subscribe_status(status_listener);
        
        expect(status_listener).toHaveBeenCalledWith('disconnected');
        
        socket.connect();
        expect(socket.get_connection_state()).toBe('connecting');
        // protocols includes bearer.token and tadpole-pulse-v1
        expect(ws_constructor).toHaveBeenCalledWith('ws://localhost:8000/v1/engine/ws', ['bearer.test-key', 'tadpole-pulse-v1']);
        
        if (mock_web_socket.onopen) mock_web_socket.onopen();
        expect(socket.get_connection_state()).toBe('connected');
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining('Connected') }));
    });

    it('should handle disconnects and trigger exponential backoff reconnects', () => {
        const socket = get_tadpole_os_socket();
        socket.connect();
        if (mock_web_socket.onopen) mock_web_socket.onopen();
        expect(socket.get_connection_state()).toBe('connected');
        
        if (mock_web_socket.onclose) mock_web_socket.onclose();
        expect(socket.get_connection_state()).toBe('reconnecting');
        
        vi.advanceTimersByTime(2000);
        expect(ws_constructor).toHaveBeenCalledTimes(2);
    });

    it('should handle manual disconnect and not auto-reconnect', () => {
        const socket = get_tadpole_os_socket();
        socket.connect();
        if (mock_web_socket.onopen) mock_web_socket.onopen();
        
        socket.disconnect();
        expect(socket.get_connection_state()).toBe('disconnected');
        
        vi.advanceTimersByTime(2000);
        expect(ws_constructor).toHaveBeenCalledTimes(1);
    });
    
    it('should reconnect when settings change', async () => {
        const socket = get_tadpole_os_socket();
        socket.connect();
        if (mock_web_socket.onopen) mock_web_socket.onopen();
        
        (use_settings_store as any).__triggerChange({ settings: { tadpole_os_url: 'http://new', tadpole_os_api_key: 'key2' } });
        
        expect(mock_web_socket.close).toHaveBeenCalled();
        expect(ws_constructor).toHaveBeenCalledTimes(2);
        expect(ws_constructor).toHaveBeenLastCalledWith('ws://new/v1/engine/ws', ['bearer.key2', 'tadpole-pulse-v1']);
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
        socket.connect();
        if (mock_web_socket.onopen) mock_web_socket.onopen();
        
        const agent_listener = vi.fn();
        socket.subscribe_agent_updates(agent_listener);

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

    it('should dispatch ArrayBuffer to audio stream listeners', () => {
        const socket = get_tadpole_os_socket();
        socket.connect();
        if (mock_web_socket.onopen) mock_web_socket.onopen();

        const audio_listener = vi.fn();
        socket.subscribe_audio_stream(audio_listener);
        
        const packet = new Uint8Array(9);
        packet[0] = 0x01; // Audio header
        if (mock_web_socket.onmessage) mock_web_socket.onmessage({ data: packet.buffer });
        // The service slices the first byte
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
            constructor() { throw new Error('Network fail'); }
        }
        vi.stubGlobal('WebSocket', Fails_Web_Socket);
        const socket = get_tadpole_os_socket();
        socket.connect();
        expect(socket.get_connection_state()).toBe('reconnecting');
    });

    it('should hit MAX_RETRIES limit safely', () => {
        const socket = get_tadpole_os_socket();
        socket.connect();
        if (mock_web_socket.onopen) mock_web_socket.onopen();
        vi.mocked(event_bus.emit_log).mockClear();

        for(let i=0; i<10; i++) {
           if (mock_web_socket.onclose) mock_web_socket.onclose();
           vi.advanceTimersByTime(31000); 
        }
        
        if (mock_web_socket.onclose) mock_web_socket.onclose();

        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ severity: 'error', text: expect.stringContaining('10 attempts') }));
    });
});


// Metadata: [socket_test]

// Metadata: [socket_test]
