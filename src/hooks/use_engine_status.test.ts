/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_engine_status.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { renderHook, act } from '@testing-library/react';
import { useEngineStatus } from './use_engine_status';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the socket service
const mock_socket_instance = {
    get_connection_state: vi.fn(() => 'connected'),
    subscribe: vi.fn((channel: string, cb: any) => {
        if (channel === 'status') {
            return mock_socket_instance.subscribe_status(cb);
        } else if (channel === 'health') {
            return mock_socket_instance.subscribe_health(cb);
        }
        return vi.fn();
    }),
    subscribe_status: vi.fn(() => vi.fn()),
    subscribe_health: vi.fn(() => vi.fn()),
    subscribe_swarm_pulse: vi.fn(() => vi.fn()),
};

vi.mock('../services/socket', () => ({
    get_tadpole_os_socket: () => mock_socket_instance,
}));

describe('useEngineStatus', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('initializes with the current socket state', () => {
        mock_socket_instance.get_connection_state.mockReturnValue('connected');
        const { result } = renderHook(() => useEngineStatus());
        
        expect(result.current.status).toBe('connected');
        expect(result.current.is_online).toBe(true);
    });

    it('updates when socket status changes', () => {
        let captured_callback: (state: string) => void = () => {};
        mock_socket_instance.subscribe_status.mockImplementation((cb: any) => {
            captured_callback = cb;
            return vi.fn();
        });

        const { result } = renderHook(() => useEngineStatus());
        
        act(() => {
            captured_callback('disconnected');
        });

        expect(result.current.status).toBe('disconnected');
        expect(result.current.is_online).toBe(false);
    });

    it('maps health events to metrics', () => {
        let captured_callback: (health: any) => void = () => {};
        mock_socket_instance.subscribe_health.mockImplementation((cb: any) => {
            captured_callback = cb;
            return vi.fn();
        });

        const { result } = renderHook(() => useEngineStatus());
        
        const mockHealth = {
            cpu: 45,
            memory: 1024,
            latency: 12,
            active_agents: 5,
            max_depth: 3,
            tpm: 150,
            recruit_count: 10
        };

        act(() => {
            captured_callback(mockHealth);
        });

        expect(result.current.cpu).toBe(45);
        expect(result.current.memory).toBe(1024);
        expect(result.current.active_agents).toBe(5);
        expect(result.current.health).toEqual(mockHealth);
    });

    it('updates agent count from swarm pulse', () => {
        let captured_callback: (pulse: any) => void = () => {};
        mock_socket_instance.subscribe_swarm_pulse.mockImplementation((cb: any) => {
            captured_callback = cb;
            return vi.fn();
        });

        const { result } = renderHook(() => useEngineStatus());
        
        act(() => {
            captured_callback({ nodes: [1, 2, 3] });
        });

        expect(result.current.active_agents).toBe(3);
    });
});
