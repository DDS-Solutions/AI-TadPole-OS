/**
 * @file socket.test.ts
 * @description Suite for the Tadpole OS WebSocket client and real-time event orchestration.
 * @module Services/Socket
 * @testedBehavior
 * - Lifecycle Management: Connection initialization, exponential backoff, and manual disconnection.
 * - Reactive Sync: Automatic reconnection triggered by settings changes (API URL/Key).
 * - Message Deserialization: Multi-modal message handling (JSON logs vs binary audio streams).
 * - Resilience: Handling of network failures and max retry exhaustion.
 * @aiContext
 * - Stubs global.WebSocket with a mockable DummyWebSocket class.
 * - Uses vitest fake timers to validate backoff timings and retry logic.
 * - Orchestrates simulated store changes via __triggerChange to test reactive connection logic.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { getTadpoleOSSocket } from './socket';
import { EventBus } from './eventBus';
import { useSettingsStore } from '../stores/settingsStore';

vi.mock('./eventBus', () => ({
    EventBus: { emit: vi.fn() },
}));

vi.mock('../stores/settingsStore', () => {
    let listeners: any[] = [];
    let currentSettings = { TadpoleOSUrl: 'http://localhost:8000', TadpoleOSApiKey: 'test-key' };
    return {
        getSettings: vi.fn(() => currentSettings),
        useSettingsStore: {
            subscribe: vi.fn((fn) => {
                listeners.push(fn);
                return () => { listeners = listeners.filter(l => l !== fn); };
            }),
            __triggerChange: (state: any) => {
                if (state.settings) {
                    currentSettings = { ...currentSettings, ...state.settings };
                }
                listeners.forEach(l => l(state));
            },
        }
    };
});

describe('TadpoleOSSocketClient', () => {
    let mockWebSocket: any;
    let wsConstruct: any;
    
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
                wsConstruct(url, protocols);
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
        
        const s = socket as any;
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
        
        mockWebSocket.onopen();
        expect(socket.getConnectionState()).toBe('connected');
        expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining('Connected') }));
    });

    it('should handle disconnects and trigger exponential backoff reconnects', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        mockWebSocket.onopen();
        expect(socket.getConnectionState()).toBe('connected');
        
        mockWebSocket.onclose();
        expect(socket.getConnectionState()).toBe('connecting');
        
        vi.advanceTimersByTime(2000);
        expect(wsConstruct).toHaveBeenCalledTimes(2);
    });

    it('should handle manual disconnect and not auto-reconnect', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        mockWebSocket.onopen();
        
        socket.disconnect();
        expect(socket.getConnectionState()).toBe('disconnected');
        
        vi.advanceTimersByTime(2000);
        expect(wsConstruct).toHaveBeenCalledTimes(1);
    });
    
    it('should reconnect when settings change', async () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        mockWebSocket.onopen();
        
        (useSettingsStore as any).__triggerChange({ settings: { TadpoleOSUrl: 'http://new', TadpoleOSApiKey: 'key2' } });
        
        expect(mockWebSocket.close).toHaveBeenCalled();
        expect(wsConstruct).toHaveBeenCalledTimes(2);
        expect(wsConstruct).toHaveBeenLastCalledWith('ws://new/v1/engine/ws', ['bearer.key2']);
    });

    it('should deserialize messages correctly and emit events', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        mockWebSocket.onopen();
        
        const agentListener = vi.fn();
        socket.subscribeAgentUpdates(agentListener);

        mockWebSocket.onmessage({ data: JSON.stringify({ type: 'log', message: 'hello', level: 'info' }) });
        expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ text: 'hello', severity: 'info' }));
        
        mockWebSocket.onmessage({ data: JSON.stringify({ type: 'agent:status', status: 'thinking', agentId: '1' }) });
        expect(agentListener).toHaveBeenCalledWith(expect.objectContaining({ 
            type: 'agent:update', 
            data: { status: 'thinking' } 
        }));
        // We no longer emit to EventBus for status updates to prevent duplication
        expect(EventBus.emit).toHaveBeenCalledTimes(2);
    });

    it('should dispatch ArrayBuffer to audio stream listeners', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        mockWebSocket.onopen();

        const audioListener = vi.fn();
        socket.subscribeAudioStream(audioListener);
        
        const buffer = new ArrayBuffer(8);
        mockWebSocket.onmessage({ data: buffer });
        expect(audioListener).toHaveBeenCalledWith(buffer);
    });

    it('should handle websocket error event', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        mockWebSocket.onerror();
        expect(socket.getConnectionState()).toBe('connecting');
    });

    it('should handle connection failure gracefully', () => {
        class FailsWebSocket {
            constructor() { throw new Error('Network fail'); }
        }
        vi.stubGlobal('WebSocket', FailsWebSocket);
        const socket = getTadpoleOSSocket();
        socket.connect();
        expect(socket.getConnectionState()).toBe('connecting');
    });

    it('should hit MAX_RETRIES limit safely', () => {
        const socket = getTadpoleOSSocket();
        socket.connect();
        mockWebSocket.onopen();
        vi.mocked(EventBus.emit).mockClear();

        for(let i=0; i<10; i++) {
           mockWebSocket.onclose();
           vi.advanceTimersByTime(31000); 
        }
        
        mockWebSocket.onclose();

        expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ severity: 'error', text: expect.stringContaining('10 attempts') }));
    });
});
