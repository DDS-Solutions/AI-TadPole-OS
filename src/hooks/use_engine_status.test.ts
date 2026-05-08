/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[use_engine_status_test]` in observability traces.
 */

import { renderHook, act } from '@testing-library/react';
import { useEngineStatus } from './use_engine_status';
import { tadpole_os_socket } from '../services/socket';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the socket service
vi.mock('../services/socket', () => ({
    tadpole_os_socket: {
        get_connection_state: vi.fn(() => 'connected'),
        subscribe_status: vi.fn(() => vi.fn()),
        subscribe_health: vi.fn(() => vi.fn()),
        subscribe_swarm_pulse: vi.fn(() => vi.fn()),
    },
}));

describe('useEngineStatus', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('initializes with the current socket state', () => {
        (tadpole_os_socket.get_connection_state as any).mockReturnValue('connected');
        const { result } = renderHook(() => useEngineStatus());
        
        expect(result.current.status).toBe('connected');
        expect(result.current.is_online).toBe(true);
    });

    it('updates when socket status changes', () => {
        let captured_callback: (state: string) => void = () => {};
        (tadpole_os_socket.subscribe_status as any).mockImplementation((cb: any) => {
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
        (tadpole_os_socket.subscribe_health as any).mockImplementation((cb: any) => {
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
        (tadpole_os_socket.subscribe_swarm_pulse as any).mockImplementation((cb: any) => {
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

// Metadata: [use_engine_status_test]
