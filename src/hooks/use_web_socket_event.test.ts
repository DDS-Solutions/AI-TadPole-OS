/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_web_socket_event.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { renderHook } from '@testing-library/react';
import { useWebSocketEvent } from './use_web_socket_event';
import { get_tadpole_os_socket } from '../services/socket';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the socket service
const mock_socket_instance = {
    connect: vi.fn(),
    subscribe_agent_updates: vi.fn(() => vi.fn()),
    subscribe_health: vi.fn(() => vi.fn()),
    subscribe_handoff: vi.fn(() => vi.fn()),
    subscribe_status: vi.fn(() => vi.fn()),
    subscribe: vi.fn(function(channel: string, listener: any) {
        switch (channel) {
            case 'agentUpdates':
                return mock_socket_instance.subscribe_agent_updates(listener);
            case 'health':
                return mock_socket_instance.subscribe_health(listener);
            case 'handoff':
                return mock_socket_instance.subscribe_handoff(listener);
            case 'status':
                return mock_socket_instance.subscribe_status(listener);
            default:
                return vi.fn();
        }
    })
};

vi.mock('../services/socket', () => {
    return {
        get_tadpole_os_socket: () => mock_socket_instance
    };
});

describe('useWebSocketEvent', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('subscribes to agentUpdates channel on mount', () => {
        const handler = vi.fn();
        renderHook(() => useWebSocketEvent('agentUpdates', handler));

        expect(get_tadpole_os_socket().connect).toHaveBeenCalled();
        expect(mock_socket_instance.subscribe_agent_updates).toHaveBeenCalled();
    });

    it('subscribes to health channel on mount', () => {
        const handler = vi.fn();
        renderHook(() => useWebSocketEvent('health', handler));

        expect(mock_socket_instance.subscribe_health).toHaveBeenCalled();
    });

    it('unsubscribes on unmount', () => {
        const unsubscribe = vi.fn();
        mock_socket_instance.subscribe_agent_updates.mockReturnValue(unsubscribe);

        const { unmount } = renderHook(() => useWebSocketEvent('agentUpdates', vi.fn()));
        unmount();

        expect(unsubscribe).toHaveBeenCalled();
    });

    it('throttles events when throttle_ms is provided', () => {
        vi.useFakeTimers();
        const handler = vi.fn();
        let captured_handler: (data: any) => void = () => {};
        
        mock_socket_instance.subscribe_agent_updates.mockImplementation((h: any) => {
            captured_handler = h;
            return vi.fn();
        });

        renderHook(() => useWebSocketEvent('agentUpdates', handler, 1000));

        // First call - immediate
        captured_handler({ id: 1 });
        expect(handler).toHaveBeenCalledTimes(1);

        // Second call - within throttle period
        captured_handler({ id: 2 });
        expect(handler).toHaveBeenCalledTimes(1);

        // Advance time
        vi.advanceTimersByTime(1100);

        // Third call - after throttle period
        captured_handler({ id: 3 });
        expect(handler).toHaveBeenCalledTimes(2);

        vi.useRealTimers();
    });

    it('executes trailing-edge throttle event at the end of throttle period', () => {
        vi.useFakeTimers();
        const handler = vi.fn();
        let captured_handler: (data: any) => void = () => {};
        
        mock_socket_instance.subscribe_agent_updates.mockImplementation((h: any) => {
            captured_handler = h;
            return vi.fn();
        });

        renderHook(() => useWebSocketEvent('agentUpdates', handler, 1000));

        // First call - immediate (leading edge)
        captured_handler({ val: 'first' });
        expect(handler).toHaveBeenLastCalledWith({ val: 'first' });
        expect(handler).toHaveBeenCalledTimes(1);

        // Second call - throttled
        captured_handler({ val: 'second' });
        expect(handler).toHaveBeenCalledTimes(1);

        // Third call - throttled, should overwrite second
        captured_handler({ val: 'third' });
        expect(handler).toHaveBeenCalledTimes(1);

        // Advance time partially (500ms) - nothing yet
        vi.advanceTimersByTime(500);
        expect(handler).toHaveBeenCalledTimes(1);

        // Advance the rest of the throttle period (500ms)
        vi.advanceTimersByTime(500);
        expect(handler).toHaveBeenCalledTimes(2);
        expect(handler).toHaveBeenLastCalledWith({ val: 'third' });

        vi.useRealTimers();
    });
});
