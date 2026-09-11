/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / websocket_connection.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { WebSocketConnection } from './websocket_connection';
import { ReconnectionPolicy } from './reconnection_policy';
import { ConnectionMetrics } from '../observability/connection_metrics';

class MockWebSocket {
    static OPEN = 1;
    static CLOSED = 3;
    readyState = MockWebSocket.OPEN;
    binaryType = 'arraybuffer';
    url: string;
    onopen: (() => void) | null = null;
    onmessage: ((event: { data: any }) => void) | null = null;
    onclose: ((event: any) => void) | null = null;
    onerror: (() => void) | null = null;
    sent_data: any[] = [];

    constructor(url: string) {
        this.url = url;
    }

    send(data: any) {
        this.sent_data.push(data);
    }

    close(code = 1000) {
        this.readyState = MockWebSocket.CLOSED;
        if (this.onclose) this.onclose({ code });
    }
}

describe('WebSocketConnection', () => {
    let original_ws: any;
    let created_sockets: MockWebSocket[] = [];

    beforeEach(() => {
        vi.useFakeTimers();
        created_sockets = [];
        original_ws = global.WebSocket;
        (global as any).WebSocket = class extends MockWebSocket {
            constructor(url: string) {
                super(url);
                created_sockets.push(this);
            }
        };
        (global.WebSocket as any).OPEN = 1;
        (global.WebSocket as any).CLOSED = 3;
    });

    afterEach(() => {
        vi.useRealTimers();
        global.WebSocket = original_ws;
    });

    it('connects to target URL and initiates authentication', () => {
        const conn = new WebSocketConnection(
            () => 'ws://localhost:8000/v1/engine/ws',
            () => 'secret-token'
        );

        conn.connect();
        expect(created_sockets).toHaveLength(1);
        const ws = created_sockets[0];
        expect(ws.url).toBe('ws://localhost:8000/v1/engine/ws');

        // Trigger socket open
        ws.onopen?.();
        expect(conn.get_state()).toBe('authenticating');

        // Check handshake payload was sent
        expect(ws.sent_data).toContainEqual(JSON.stringify({ type: 'auth', token: 'secret-token' }));
    });

    it('transitions to connected and starts heartbeat upon auth_ok', () => {
        const conn = new WebSocketConnection(
            () => 'ws://localhost:8000/v1/engine/ws',
            () => 'secret-token',
            new ReconnectionPolicy(),
            new ConnectionMetrics(),
            10000 // 10s heartbeat
        );

        conn.connect();
        const ws = created_sockets[0];
        ws.onopen?.();

        // Simulate auth_ok from server
        ws.onmessage?.({ data: JSON.stringify({ type: 'auth_ok' }) });
        expect(conn.get_state()).toBe('connected');

        // Advance 10s -> sends heartbeat ping frame
        vi.advanceTimersByTime(10000);
        expect(ws.sent_data).toContainEqual(JSON.stringify({ type: 'ping' }));

        conn.disconnect();
    });

    it('queues messages when not connected and flushes on auth_ok', () => {
        const conn = new WebSocketConnection(
            () => 'ws://localhost:8000/v1/engine/ws',
            () => 'secret-token'
        );

        conn.connect();
        // Socket opened but not authenticated yet
        const ws = created_sockets[0];
        ws.onopen?.();

        const queued = conn.send_json({ action: 'do_task' });
        expect(queued).toBe(true);

        // Simulate auth_ok
        ws.onmessage?.({ data: JSON.stringify({ type: 'auth_ok' }) });

        expect(ws.sent_data).toContainEqual(JSON.stringify({ action: 'do_task' }));
        conn.disconnect();
    });

    it('stops reconnecting and transitions to error on 4401 invalid credentials', () => {
        const conn = new WebSocketConnection(
            () => 'ws://localhost:8000/v1/engine/ws',
            () => 'bad-token'
        );

        conn.connect();
        const ws = created_sockets[0];
        ws.onopen?.();

        // Simulate auth rejection close frame
        ws.close(4401);

        expect(conn.get_state()).toBe('error');

        // Advance timers - should NOT attempt reconnect
        vi.advanceTimersByTime(30000);
        expect(created_sockets).toHaveLength(1);
    });

    it('receives and sanitizes incoming messages', () => {
        const conn = new WebSocketConnection(
            () => 'ws://localhost:8000/v1/engine/ws',
            () => 'secret-token'
        );

        const messages: any[] = [];
        conn.on_message((msg) => messages.push(msg));

        conn.connect();
        const ws = created_sockets[0];
        ws.onopen?.();
        ws.onmessage?.({ data: JSON.stringify({ type: 'auth_ok' }) });

        // Normal log message
        ws.onmessage?.({ data: JSON.stringify({ type: 'log', text: 'System ready' }) });
        expect(messages).toHaveLength(1);
        expect(messages[0].text).toBe('System ready');

        conn.disconnect();
    });
});
