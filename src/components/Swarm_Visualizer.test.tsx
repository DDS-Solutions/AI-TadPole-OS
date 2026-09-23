/**
 * @docs ARCHITECTURE:Telemetry
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Swarm_Visualizer.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { Swarm_Visualizer } from './Swarm_Visualizer';
import { agent_service } from '../services/agent_service';


// Mock react-force-graph-2d
vi.mock('react-force-graph-2d', () => ({
    default: ({ graphData }: any) => (
        <div data-testid="mock-force-graph">
            <div data-testid="node-count">{graphData?.nodes?.length ?? 0}</div>
            <div data-testid="link-count">{graphData?.links?.length ?? 0}</div>
        </div>
    ),
}));

// Mock agent_service
vi.mock('../services/agent_service', () => ({
    agent_service: {
        get_swarm_graph: vi.fn(),
    },
}));

// Mock socket client
const mockSubscribeSwarmPulse = vi.fn().mockReturnValue(() => {});
const mockSubscribeRaw = vi.fn().mockReturnValue(() => {});
vi.mock('../services/socket', () => ({
    get_tadpole_os_socket: () => ({
        subscribe_swarm_pulse: mockSubscribeSwarmPulse,
        subscribe_raw: mockSubscribeRaw,
    }),
}));

describe('Swarm_Visualizer Component', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        (agent_service.get_swarm_graph as any).mockResolvedValue({
            nodes: [
                { id: 'agent-1', label: 'Architect Agent', status: 'active' },
                { id: 'agent-2', label: 'Security Auditor', status: 'idle' },
            ],
            edges: [
                { source: 'agent-1', target: 'agent-2' },
            ],
        });
    });

    it('hydrates graph nodes and edges via REST baseline on mount', async () => {
        render(<Swarm_Visualizer />);

        expect(agent_service.get_swarm_graph).toHaveBeenCalled();
        expect(mockSubscribeSwarmPulse).toHaveBeenCalled();

        await waitFor(() => {
            expect(screen.getByTestId('node-count').textContent).toBe('2');
            expect(screen.getByTestId('link-count').textContent).toBe('1');
        });
    });

    it('toggles between swarm telemetry and reasoning trace view modes', async () => {
        render(<Swarm_Visualizer />);

        const traceBtn = screen.getByRole('button', { name: /reasoning trace/i });
        expect(traceBtn).toBeDefined();

        fireEvent.click(traceBtn);
        // After switching to trace mode, logic_data is displayed
        await waitFor(() => {
            expect(screen.getByTestId('mock-force-graph')).toBeDefined();
        });

        const swarmBtn = screen.getByRole('button', { name: /^swarm$/i });
        fireEvent.click(swarmBtn);
        await waitFor(() => {
            expect(screen.getByTestId('node-count').textContent).toBe('2');
        });
    });
});
