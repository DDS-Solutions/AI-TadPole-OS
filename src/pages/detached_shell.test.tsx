/**
 * @docs ARCHITECTURE:UI-Pages
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:TestSuites**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[detached_shell_test]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:TestSuites
 *
 * Verifies detached windows only subscribe to dashboard-wide data when their
 * content actually needs agent registry state.
 */

import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import Detached_Shell from './detached_shell';

const { mock_use_dashboard_data } = vi.hoisted(() => ({
    mock_use_dashboard_data: vi.fn(),
}));

vi.mock('../hooks/use_dashboard_data', () => ({
    useDashboardData: mock_use_dashboard_data,
}));

vi.mock('../components/dashboard/System_Log', () => ({
    System_Log: () => <div data-testid="system-log" />,
}));

vi.mock('../components/Neural_Waterfall', () => ({
    Neural_Waterfall: () => <div data-testid="trace-stream" />,
}));

vi.mock('../components/Lineage_Stream', () => ({
    Lineage_Stream: () => <div data-testid="lineage-stream" />,
}));

vi.mock('../components/SovereignChat', () => ({
    SovereignChat: () => <div data-testid="sovereign-chat" />,
}));

vi.mock('../components/dashboard/Agent_Status_Grid', () => ({
    Agent_Status_Grid: () => <div data-testid="agent-status-grid" />,
}));

vi.mock('../components/AgentConfigPanel', () => ({
    default: () => <div data-testid="agent-config-panel" />,
}));

vi.mock('../components/layout/Observability_Sidebar', () => ({
    Observability_Sidebar: () => <div data-testid="observability-sidebar" />,
}));

describe('Detached_Shell subscription ownership', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mock_use_dashboard_data.mockReturnValue({
            agents_list: [{ id: 'agent-1', name: 'Agent 1', model: 'gpt-4o', department: 'Operations' }],
            assigned_agent_ids: new Set(['agent-1']),
            available_roles: ['Operator'],
            clusters: [],
            toggle_cluster_active: vi.fn(),
            update_agent: vi.fn(),
        });
    });

    it('does not initialize dashboard data for log-only detached windows', async () => {
        render(
            <MemoryRouter initialEntries={['/detached-view?type=system-log']}>
                <Detached_Shell />
            </MemoryRouter>
        );

        expect(await screen.findByTestId('system-log')).toBeInTheDocument();
        expect(mock_use_dashboard_data).not.toHaveBeenCalled();
    });

    it('initializes dashboard data for agent registry detached windows', async () => {
        render(
            <MemoryRouter initialEntries={['/detached-view?type=agent-status']}>
                <Detached_Shell />
            </MemoryRouter>
        );

        expect(await screen.findByTestId('agent-status-grid')).toBeInTheDocument();
        expect(mock_use_dashboard_data).toHaveBeenCalled();
    });
});

// Metadata: [detached_shell_test]
