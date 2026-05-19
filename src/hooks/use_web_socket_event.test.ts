/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[use_web_socket_event_test]` in observability traces.
 */

import { renderHook } from '@testing-library/react';
import { useWebSocketEvent } from './use_web_socket_event';
import { tadpole_os_socket } from '../services/socket';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the socket service
vi.mock('../services/socket', () => {
    const mock_socket = {
        connect: vi.fn(),
        subscribe_agent_updates: vi.fn(() => vi.fn()),
        subscribe_health: vi.fn(() => vi.fn()),
        subscribe_handoff: vi.fn(() => vi.fn()),
        subscribe_status: vi.fn(() => vi.fn()),
        subscribe: vi.fn((channel: string, listener: any) => {
            switch (channel) {
                case 'agentUpdates':
                    return mock_socket.subscribe_agent_updates(listener);
                case 'health':
                    return mock_socket.subscribe_health(listener);
                case 'handoff':
                    return mock_socket.subscribe_handoff(listener);
                case 'status':
                    return mock_socket.subscribe_status(listener);
                default:
                    return vi.fn();
            }
        })
    };
    return {
        tadpole_os_socket: mock_socket
    };
});

describe('useWebSocketEvent', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('subscribes to agentUpdates channel on mount', () => {
        const handler = vi.fn();
        renderHook(() => useWebSocketEvent('agentUpdates', handler));

        expect(tadpole_os_socket.connect).toHaveBeenCalled();
        expect(tadpole_os_socket.subscribe_agent_updates).toHaveBeenCalled();
    });

    it('subscribes to health channel on mount', () => {
        const handler = vi.fn();
        renderHook(() => useWebSocketEvent('health', handler));

        expect(tadpole_os_socket.subscribe_health).toHaveBeenCalled();
    });

    it('unsubscribes on unmount', () => {
        const unsubscribe = vi.fn();
        (tadpole_os_socket.subscribe_agent_updates as any).mockReturnValue(unsubscribe);

        const { unmount } = renderHook(() => useWebSocketEvent('agentUpdates', vi.fn()));
        unmount();

        expect(unsubscribe).toHaveBeenCalled();
    });

    it('throttles events when throttle_ms is provided', () => {
        vi.useFakeTimers();
        const handler = vi.fn();
        let captured_handler: (data: any) => void = () => {};
        
        (tadpole_os_socket.subscribe_agent_updates as any).mockImplementation((h: any) => {
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
        
        (tadpole_os_socket.subscribe_agent_updates as any).mockImplementation((h: any) => {
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

// Metadata: [use_web_socket_event_test]
