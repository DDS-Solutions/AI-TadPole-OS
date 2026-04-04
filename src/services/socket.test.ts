/**
 * @file socket.test.ts
 * @description Suite for the Tadpole OS WebSocket client and real-time event orchestration.
 * @module Services/Socket
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { getTadpoleOSSocket } from './socket';
import { event_bus } from './event_bus';
import { use_settings_store } from '../stores/settings_store';

vi.mock('./event_bus', () => ({
    event_bus: { emit: vi.fn() },
}));

vi.mock('../stores/settings_store', () => {
    let listeners: ((state: { settings?: { TadpoleOSUrl?: string; TadpoleOSApiKey?: string } }) => void)[] = [];
    let currentSettings = { TadpoleOSUrl: 'http://localhost:8000', TadpoleOSApiKey: 'test-key' };
    return {
        getSettings: vi.fn(() => currentSettings),
        use_settings_store: {
            subscribe: vi.fn((fn: (state: { settings?: { TadpoleOSUrl?: string; TadpoleOSApiKey?: string } }) => void) => {
                listeners.push(fn);
                return () => { listeners = listeners.filter(l => l !== fn); };
            }),
            __triggerChange: (state: { settings?: { TadpoleOSUrl?: string; TadpoleOSApiKey?: string } }) => {
                if (state.settings) {
                    currentSettings = { ...currentSettings, ...state.settings };
                }
                listeners.forEach(l => l(state));
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
            activeScope: 'global',
            addMessage: vi.fn(),
            updateMessage: vi.fn(),
            appendMessagePart: vi.fn(() => false),
            getMessageById: vi.fn()
        }))
    }
}));

interface MockWebSocket {
    binaryType: string;
    send: ReturnType<typeof vi.fn>;
    close: ReturnType<typeof vi.fn>;
    onopen: (() => void) | null;
    onmessage: ((ev: { data: string | ArrayBuffer | Blob }) => void) | null;
    onclose: (() => void) | null;
    onerror: (() => void) | null;
}

describe('TadpoleOSSocketClient', () => {
    let mockWebSocket: MockWebSocket;
    let wsConstruct: ReturnType<typeof vi.fn>;
    
    beforeEach(() => {
        vi.clearAllMocks();
        vi.useFakeTimers();
        
        mockWebSocket = {
            binaryType: 'blob',
            send: vi.fn(),
            close: vi.fn(() => {
                if (typeof mockWebSocket.onclose === 'function') {
                    mockWebSocket.onclose();
                }
            }),
            onopen: null,
            onmessage: null,
            onclose: null,
            onerror: null,
        };
        
        wsConstruct = vi.fn();
        class DummyWebSocket {
            constructor(url: string, protocols?: string[]) {
                (wsConstruct as unknown as (url: string, protocols?: string[]) => void)(url, protocols);
                return mockWebSocket;
            }
        }
        
        vi.stubGlobal('WebSocket', DummyWebSocket);
    });

    afterEach(() => {
        vi.unstubAllGlobals();
        vi.useRealTimers();
        const socket = getTadpoleOSSocket();
        socket.disconnect();
        
        const s = socket as unknown as Record<string, unknown>;
        s.socket = null;
        s.reconnectTimer = null;
        s.state = 'disconnected';
        s.isExplicitlyClosed = false;
        s.retryCount = 0;
        s.lastUrl = '';
        s.lastKey = '';
        s.stateListeners = [];
        s.agentUpdateListeners = [];
        s.healthListeners = [];
        s.handoffListeners = [];
        s.mcpPulseListeners = [];
        s.audioStreamListeners = [];
    });

    it('should initialize and connect, updating state correctly', () => {
        const socket = getTadpoleOSSocket();
        const stateListener = vi.fn();
        socket.subscribeStatus(stateListener);
        
        expect(stateListener).toHaveBeenCalledWith('disconnected');
        
        socket.connect();
        expect(socket.getConnectionState()).toBe('connecting');
        expect(wsConstruct).toHaveBeenCalledWith('ws://localhost:8000/v1/engine/ws', ['bearer.test-key']);
        
        if (mockWebSocket.onopen) mockWebSocket.onopen();
        expect(socket.getConnectionState()).toBe('connected');
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining('Connected') }));
    });

    it('should handle disconnects and trigger exponential backoff reconnects', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        if (mockWebSocket.onopen) mockWebSocket.onopen();
        expect(socket.getConnectionState()).toBe('connected');
        
        if (mockWebSocket.onclose) mockWebSocket.onclose();
        expect(socket.getConnectionState()).toBe('reconnecting');
        
        vi.advanceTimersByTime(2000);
        expect(wsConstruct).toHaveBeenCalledTimes(2);
    });

    it('should handle manual disconnect and not auto-reconnect', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        if (mockWebSocket.onopen) mockWebSocket.onopen();
        
        socket.disconnect();
        expect(socket.getConnectionState()).toBe('disconnected');
        
        vi.advanceTimersByTime(2000);
        expect(wsConstruct).toHaveBeenCalledTimes(1);
    });
    
    it('should reconnect when settings change', async () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        if (mockWebSocket.onopen) mockWebSocket.onopen();
        
        (use_settings_store as unknown as { __triggerChange: (state: unknown) => void }).__triggerChange({ settings: { TadpoleOSUrl: 'http://new', TadpoleOSApiKey: 'key2' } });
        
        expect(mockWebSocket.close).toHaveBeenCalled();
        expect(wsConstruct).toHaveBeenCalledTimes(2);
        expect(wsConstruct).toHaveBeenLastCalledWith('ws://new/v1/engine/ws', ['bearer.key2']);
    });

    it('should deserialize messages correctly and emit events', async () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        if (mockWebSocket.onopen) mockWebSocket.onopen();
        
        const agentListener = vi.fn();
        socket.subscribeAgentUpdates(agentListener);

        if (mockWebSocket.onmessage) mockWebSocket.onmessage({ data: JSON.stringify({ type: 'log', message: 'hello', level: 'info', agent_id: '1' }) });
        
        await vi.waitFor(() => {
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ 
                text: 'hello', 
                severity: 'info',
                agent_name: 'Agent of Nine'
            }));
        });
        
        if (mockWebSocket.onmessage) mockWebSocket.onmessage({ data: JSON.stringify({ type: 'agent:status', status: 'thinking', agent_id: '1' }) });
        
        await vi.waitFor(() => {
            expect(agentListener).toHaveBeenCalledWith(expect.objectContaining({ 
                type: 'agent:update', 
                data: { status: 'thinking' } 
            }));
        });
    });

    it('should dispatch ArrayBuffer to audio stream listeners', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        if (mockWebSocket.onopen) mockWebSocket.onopen();

        const audioListener = vi.fn();
        socket.subscribeAudioStream(audioListener);
        
        const buffer = new ArrayBuffer(8);
        if (mockWebSocket.onmessage) mockWebSocket.onmessage({ data: buffer });
        expect(audioListener).toHaveBeenCalledWith(buffer);
    });

    it('should handle websocket error event', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        if (mockWebSocket.onerror) mockWebSocket.onerror();
        expect(socket.getConnectionState()).toBe('reconnecting');
    });

    it('should handle connection failure gracefully', () => {
        class FailsWebSocket {
            constructor() { throw new Error('Network fail'); }
        }
        vi.stubGlobal('WebSocket', FailsWebSocket);
        const socket = getTadpoleOSSocket();
        socket.connect();
        expect(socket.getConnectionState()).toBe('reconnecting');
    });

    it('should hit MAX_RETRIES limit safely', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        if (mockWebSocket.onopen) mockWebSocket.onopen();
        vi.mocked(event_bus.emit_log).mockClear();

        for(let i=0; i<10; i++) {
           if (mockWebSocket.onclose) mockWebSocket.onclose();
           vi.advanceTimersByTime(31000); 
        }
        
        if (mockWebSocket.onclose) mockWebSocket.onclose();

        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ severity: 'error', text: expect.stringContaining('10 attempts') }));
    });
});

