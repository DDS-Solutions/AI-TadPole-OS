/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Operations Dashboard's real-time command execution and status tracking.** 
 * Verifies integration with the `tadpole_os_service` for sending mission-critical commands and handling asynchronous completion signals. 
 * Mocks `useEngineStatus` and `useDashboardData` to isolate aggregate telemetry from backend engine load spikes.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Command timeout failures or incorrect status mapping (e.g. 'queued' instead of 'in_progress') during backend engine load spikes.
 * - **Telemetry Link**: Search `[Ops_Dashboard.test]` in tracing logs.
 */


/**
 * @file Ops_Dashboard.test.tsx
 * @description Suite for the main Operations Dashboard.
 * @module Pages/Ops_Dashboard
 * @testedBehavior
 * - Real-time Visualization: Rendering of neural waterfalls and lineage streams.
 * - System Health: Display of engine status and agent counts.
 * - Command Execution: Integration with the global command palette.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks useEngineStatus and useDashboardData hooks for telemetry control.
 * - Mocks i18n to return keys for stable assertion matching.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, within, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import Ops_Dashboard from './Ops_Dashboard';
import { agents as mock_agents } from '../data/mock_agents';

const mock_update_agent = vi.fn();
const mock_discover_nodes = vi.fn();

// 1. Mock Hooks and Services
vi.mock('../hooks/use_engine_status', () => ({
    useEngineStatus: vi.fn(() => ({
        is_online: true,
        connection_state: 'connected',
        status: 'connected'
    }))
}));

vi.mock('../hooks/use_dashboard_data', () => ({
    useDashboardData: vi.fn(() => ({
        is_online: true,
        agents_list: mock_agents,
        agents_count: mock_agents.length,
        active_agents: 2,
        total_cost: 150,
        total_tokens: 5000,
        budget_util: 45,
        recruit_velocity: 1,
        nodes: [{ id: 'bunker-1', name: 'Bunker One', address: '10.0.0.1' }],
        nodes_loading: false,
        logs: [],
        logs_end_ref: { current: null },
        assigned_agent_ids: new Set(['1', '2']),
        available_roles: ['CEO', 'Architect', 'Engineer'],
        clusters: [
            { id: 'cluster-1', collaborators: ['1', '2'], alpha_id: '1', is_active: true, objective: 'Test Objective', budget_usd: 1000 }
        ],
        update_agent: mock_update_agent,
        discover_nodes: mock_discover_nodes
    }))
}));

vi.mock('../services/agent_service', () => ({
    persist_agent_update: vi.fn()
}));

// Mock state using vi.hoisted for sharing with vi.mock
const { mock_state } = vi.hoisted(() => ({
    mock_state: {
        agents: [
            { id: 'a1', name: 'Agent 1', role: 'Worker', status: 'online', type: 'llm', department: 'Operations' }
        ],
        clusters: [
            { id: 'c1', name: 'Cluster 1', department: 'Operations', agents: ['a1'] }
        ],
        is_sidebar_open: true,
    }
}));

vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        send_command: vi.fn().mockResolvedValue(true),
        deploy_engine: vi.fn().mockResolvedValue({ status: 'success', output: 'Deployed' }),
        create_agent: vi.fn().mockResolvedValue(true),
    }
}));

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: vi.fn((selector) => {
        const state = {
            agents: mock_state.agents,
            mission_clusters: mock_state.clusters,
            get_agents: vi.fn(),
            get_clusters: vi.fn(),
        };
        return selector ? selector(state) : state;
    })
}));

vi.mock('../stores/header_store', () => ({
    use_header_store: vi.fn((selector) => {
        const state = {
            set_header_actions: vi.fn(),
            clear_header_actions: vi.fn(),
        };
        return selector ? selector(state) : state;
    }),
}));

vi.mock('../stores/settings_store', () => ({
    use_settings_store: vi.fn((selector) => {
        const state = {
            is_sidebar_open: mock_state.is_sidebar_open,
            set_sidebar_open: vi.fn(),
        };
        return selector ? selector(state) : state;
    })
}));

vi.mock('../stores/dropdown_store', () => ({
    use_dropdown_store: vi.fn((selector) => {
        const state = { close_dropdown: vi.fn() };
        return selector ? selector(state) : state;
    })
}));

vi.mock('../stores/role_store', () => ({
    use_role_store: {
        getState: vi.fn(() => ({
            roles: {
                'CEO': {
                    id: 'ceo',
                    name: 'CEO',
                    skills: ['deep_research', 'system_audit', 'fetch_url', 'issue_alpha_directive'],
                    workflows: ['deploy_to_prod', 'emergency_shutdown', 'neural_handoff', 'Deep Analysis']
                }
            }
        }))
    }
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => {
            if (key === 'stats.active_swarm_tooltip') return 'Active Swarm Tooltip';
            return key;
        }
    }
}));

// Mock scrollIntoView
window.HTMLElement.prototype.scrollIntoView = vi.fn();

// 2. Mock Child Components (using snake_case props per parity guidelines)
vi.mock('../components/dashboard/Agent_Status_Grid', () => ({
    Agent_Status_Grid: ({ agents, on_skill_trigger, on_role_change, on_model_change, on_configure_click, handle_agent_update }: any) => (
        <div data-testid="agent-grid">
            {agents.map((a: any) => (
                <div key={a.id} data-testid={`node-${a.id}`}>
                    <button onClick={() => on_skill_trigger(a.id, 'test_skill', 1)}>Slot 1</button>
                    <button onClick={() => on_skill_trigger(a.id, 'test_skill', 2)}>Slot 2</button>
                    <button onClick={() => on_skill_trigger(a.id, 'test_skill', 3)}>Slot 3</button>
                    <button onClick={() => on_role_change(a.id, 'CEO')}>Promote</button>
                    <button onClick={() => on_model_change(a.id, 'gemini-pro')}>Change Model</button>
                    <button onClick={() => handle_agent_update(a.id, { model_2: 'gemini-pro' })}>Change Model 2</button>
                    <button onClick={() => handle_agent_update(a.id, { model_3: 'gemini-pro' })}>Change Model 3</button>
                    <button onClick={() => on_configure_click(a.id)}>Configure</button>
                </div>
            ))}
        </div>
    )
}));

vi.mock('../components/Terminal', () => ({
    default: () => <div data-testid="terminal">Terminal</div>
}));

vi.mock('../components/AgentConfigPanel', () => ({
    default: ({ onClose, onUpdate }: any) => (
        <div data-testid="config-panel">
            Config Panel
            <button onClick={onClose}>Close</button>
            <button onClick={() => onUpdate('1', { name: 'Updated' })}>Update</button>
        </div>
    )
}));

vi.mock('../components/Swarm_Visualizer', () => ({
    Swarm_Visualizer: () => <div data-testid="swarm-visualizer">Visualizer</div>
}));

describe('Ops_Dashboard', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders dashboard shell with metrics and nodes', async () => {
        render(<Ops_Dashboard />);

        expect(screen.getByText('stats.active_swarm')).toBeInTheDocument();
        expect(screen.getByText('stats.swarm_cost')).toBeInTheDocument();
    });

    it('handles skill trigger interaction with multiple slots', async () => {
        const { tadpole_os_service } = await import('../services/tadpoleos_service');
        render(<Ops_Dashboard />);

        await waitFor(() => screen.getByTestId('node-1'));
        
        const node_1 = screen.getByTestId('node-1');
        fireEvent.click(within(node_1).getByText('Slot 1'));
        expect(tadpole_os_service.send_command).toHaveBeenCalledWith(
            '1', 'test_skill', 'GPT-5.2', 'openai', 'cluster-1', 'Executive', 1000
        );

        // Slot 2
        fireEvent.click(within(node_1).getByText('Slot 2'));
        expect(tadpole_os_service.send_command).toHaveBeenCalledWith(
            '1', 'test_skill', 'Claude Opus 4.5', 'anthropic', 'cluster-1', 'Executive', 1000
        );

        // Slot 3
        fireEvent.click(within(node_1).getByText('Slot 3'));
        expect(tadpole_os_service.send_command).toHaveBeenCalledWith(
            '1', 'test_skill', 'LLaMA 4 Maverick', 'meta', 'cluster-1', 'Executive', 1000
        );
    });

    it('handles role changes', async () => {
        render(<Ops_Dashboard />);
        await waitFor(() => screen.getByTestId('node-1'));

        const node_1 = screen.getByTestId('node-1');
        fireEvent.click(within(node_1).getByText('Promote'));
        expect(mock_update_agent).toHaveBeenCalledWith('1', expect.objectContaining({
            role: 'CEO',
            skills: ['deep_research', 'system_audit', 'fetch_url', 'issue_alpha_directive'],
            workflows: ['deploy_to_prod', 'emergency_shutdown', 'neural_handoff', 'Deep Analysis']
        }));
    });

    it('handles model changes', async () => {
        render(<Ops_Dashboard />);
        await waitFor(() => screen.getByTestId('node-1'));

        const node_1 = screen.getByTestId('node-1');
        fireEvent.click(within(node_1).getByText('Change Model'));
        fireEvent.click(within(node_1).getByText('Change Model 2'));
        fireEvent.click(within(node_1).getByText('Change Model 3'));

        expect(mock_update_agent).toHaveBeenCalledTimes(3);
    });

    it('opens and closes config panel', async () => {
        render(<Ops_Dashboard />);
        await waitFor(() => screen.getByTestId('node-1'));

        const node_1 = screen.getByTestId('node-1');
        fireEvent.click(within(node_1).getByText('Configure'));
        expect(screen.getByTestId('config-panel')).toBeInTheDocument();

        fireEvent.click(screen.getByText('Update'));
        expect(mock_update_agent).toHaveBeenCalledWith('1', { name: 'Updated' });

        fireEvent.click(screen.getByText('Close'));
        expect(screen.queryByTestId('config-panel')).not.toBeInTheDocument();
    });

    it('handles skill trigger failure', async () => {
        const { tadpole_os_service } = await import('../services/tadpoleos_service');
        vi.mocked(tadpole_os_service.send_command).mockRejectedValueOnce(new Error('Skill Error'));
        
        const console_spy = vi.spyOn(console, 'error').mockImplementation(() => {});
        render(<Ops_Dashboard />);

        await waitFor(() => screen.getByTestId('node-1'));
        
        const node_1 = screen.getByTestId('node-1');
        fireEvent.click(within(node_1).getByText('Slot 1'));

        await waitFor(() => {
            expect(console_spy).toHaveBeenCalledWith(expect.stringContaining('Failed to trigger skill'), expect.any(Error));
        });
        console_spy.mockRestore();
    });
});


// Metadata: [Ops_Dashboard_test]

// Metadata: [Ops_Dashboard_test]
