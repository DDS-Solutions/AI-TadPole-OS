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
vi.mock('../services/socket', () => ({
    tadpole_os_socket: {
        connect: vi.fn(),
        subscribe_agent_updates: vi.fn(() => vi.fn()),
        subscribe_health: vi.fn(() => vi.fn()),
        subscribe_handoff: vi.fn(() => vi.fn()),
        subscribe_status: vi.fn(() => vi.fn()),
    },
}));

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
});

// Metadata: [use_web_socket_event_test]
