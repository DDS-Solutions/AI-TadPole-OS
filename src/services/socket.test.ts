/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / socket.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { get_tadpole_os_socket, Tadpole_OS_Socket_Client } from './socket';
import { use_settings_store } from '../stores/settings_store';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Create a robust MockWebSocket to spy on standard events
class MockWebSocket {
    static CONNECTING = 0;
    static OPEN = 1;
    static CLOSING = 2;
    static CLOSED = 3;

    static instances: MockWebSocket[] = [];
    
    url: string;
    readyState: number = 0; // CONNECTING
    onopen: (() => void) | null = null;
    onclose: ((event: any) => void) | null = null;
    onmessage: ((event: any) => void) | null = null;
    onerror: ((event: any) => void) | null = null;
    
    sent_messages: string[] = [];
    closed = false;

    constructor(url: string) {
        this.url = url;
        MockWebSocket.instances.push(this);
        setTimeout(() => {
            this.readyState = 1; // OPEN
            if (this.onopen) this.onopen();
        }, 10);
    }

    send(data: string) {
        this.sent_messages.push(data);
    }

    close() {
        this.closed = true;
        this.readyState = 3; // CLOSED
        if (this.onclose) this.onclose({ code: 1000, reason: 'Normal closure' });
    }
}

describe('SocketManager / Tadpole_OS_Socket_Client', () => {
    beforeEach(() => {
        vi.stubGlobal('WebSocket', MockWebSocket);
        MockWebSocket.instances = [];
        Tadpole_OS_Socket_Client.reset();
        
        // Setup default settings
        use_settings_store.setState({
            settings: {
                tadpole_os_url: 'http://localhost:8000',
                tadpole_os_api_key: 'test-api-key',
            } as any
        });
    });

    afterEach(() => {
        Tadpole_OS_Socket_Client.reset();
        vi.unstubAllGlobals();
    });

    it('initializes to disconnected state', () => {
        const client = get_tadpole_os_socket();
        expect(client.get_connection_state()).toBe('disconnected');
    });

    it('connects when a channel is subscribed and disconnects on unsubscribe', async () => {
        const client = get_tadpole_os_socket();
        const listener = vi.fn();
        
        // Enable fake timers BEFORE subscribing so that disconnect timeout uses fake timers
        vi.useFakeTimers();
        
        // Subscribe to trace channel (which should trigger connect)
        const unsubscribe = client.subscribe('raw', listener);
        
        // Wait for MockWebSocket constructor open timeout (10ms)
        await vi.advanceTimersByTimeAsync(20);
        
        expect(MockWebSocket.instances).toHaveLength(1);
        expect(client.get_connection_state()).toBe('authenticating');
        
        // Simulate auth success handshake response from server
        const socketInstance = MockWebSocket.instances[0];
        socketInstance.onmessage?.({
            data: JSON.stringify({ type: 'auth_ok' })
        });
        
        expect(client.get_connection_state()).toBe('connected');

        // Unsubscribe
        unsubscribe();
        
        // Fast forward 1100ms to trigger the disconnect timeout
        await vi.advanceTimersByTimeAsync(1100);
        
        expect(socketInstance.closed).toBe(true);
        expect(client.get_connection_state()).toBe('disconnected');
        
        vi.useRealTimers();
    });

    it('dispatches incoming messages to appropriate channels', async () => {
        const client = get_tadpole_os_socket();
        const listener = vi.fn();
        
        client.subscribe('health', listener);
        await new Promise(resolve => setTimeout(resolve, 20));
        
        const socketInstance = MockWebSocket.instances[0];
        
        // Complete handshake first
        socketInstance.onmessage?.({
            data: JSON.stringify({ type: 'auth_ok' })
        });
        expect(client.get_connection_state()).toBe('connected');

        // Simulate engine health event
        socketInstance.onmessage?.({
            data: JSON.stringify({
                type: 'engine:health',
                cpu: 10,
                memory: 1048576
            })
        });

        expect(listener).toHaveBeenCalledWith({
            type: 'engine:health',
            cpu: 10,
            memory: 1048576
        });
    });

    it('sends JSON messages over active socket', async () => {
        const client = get_tadpole_os_socket();
        client.subscribe('raw', vi.fn());
        await new Promise(resolve => setTimeout(resolve, 20));

        const socketInstance = MockWebSocket.instances[0];
        
        // Complete handshake first
        socketInstance.onmessage?.({
            data: JSON.stringify({ type: 'auth_ok' })
        });
        expect(client.get_connection_state()).toBe('connected');

        const success = client.send_json({ action: 'ping' });
        expect(success).toBe(true);
        
        expect(socketInstance.sent_messages).toHaveLength(2);
        
        const first_sent = JSON.parse(socketInstance.sent_messages[0]);
        expect(first_sent.type).toBe('auth');
        
        const second_sent = JSON.parse(socketInstance.sent_messages[1]);
        expect(second_sent.action).toBe('ping');
    });

    it('reconnects when settings change', async () => {
        const client = get_tadpole_os_socket();
        client.subscribe('raw', vi.fn());
        await new Promise(resolve => setTimeout(resolve, 20));

        expect(MockWebSocket.instances).toHaveLength(1);
        const originalInstance = MockWebSocket.instances[0];

        // Trigger settings update directly
        use_settings_store.setState({
            settings: {
                tadpole_os_url: 'http://localhost:9000',
                tadpole_os_api_key: 'new-api-key',
            } as any
        });

        // Verify the old websocket was closed and a new one was instantiated
        expect(originalInstance.closed).toBe(true);
        expect(MockWebSocket.instances).toHaveLength(2);
        expect(MockWebSocket.instances[1].url).toBe('ws://localhost:9000/v1/engine/ws');
    });

    it('dispatches binary audio streams to audio subscription', async () => {
        const client = get_tadpole_os_socket();
        const listener = vi.fn();

        client.subscribe('audio_stream', listener);
        await new Promise(resolve => setTimeout(resolve, 20));

        const socketInstance = MockWebSocket.instances[0];
        
        // Handshake
        socketInstance.onmessage?.({
            data: JSON.stringify({ type: 'auth_ok' })
        });

        // Create buffer with header 0x01 (audio) and payload [10, 20, 30]
        const buffer = new Uint8Array([0x01, 10, 20, 30]).buffer;
        
        socketInstance.onmessage?.({
            data: buffer
        });

        expect(listener).toHaveBeenCalled();
        const calledArg = listener.mock.calls[0][0];
        expect(new Uint8Array(calledArg)).toEqual(new Uint8Array([10, 20, 30]));
    });
});
