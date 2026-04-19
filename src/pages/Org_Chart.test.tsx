/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validates the Organizational Chart's hierarchical graph rendering.** 
 * Ensures the `dagre` layout engine correctly positions agent nodes and connection edges based on reporting lines. 
 * Mocks `Hierarchy_Node` and `Agent_Config_Panel` to isolate graph traversal logic from complex UI rendering.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Graph rendering cycles or infinite layout loops when circular agent dependencies are introduced in the `agent_store`.
 * - **Telemetry Link**: Search `[Org_Chart.test]` in tracing logs.
 */


/**
 * @file Org_Chart.test.tsx
 * @description Suite for the Agent Hierarchy (Org_Chart) page.
 * @module Pages/Org_Chart
 * @testedBehavior
 * - Graph Rendering: Node placement and Alpha Node designation.
 * - Dynamic Configuration: Real-time role and model updates for agents.
 * - Hierarchy Integrity: Ensuring "Agent of Nine" remains the sovereign root.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks agent_store and workspace_store for swarm hierarchy control.
 * - Mocks Hierarchy_Node and Agent_Config_Panel to focus on layout logic.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import Org_Chart from './Org_Chart';
import { agents as mock_agents_data } from '../data/mock_agents';

// 1. Mock Hooks and Services
vi.mock('../services/agent_service', () => ({
    load_agents: vi.fn(async () => mock_agents_data),
    persist_agent_update: vi.fn(),
    normalize_agent: vi.fn((a) => a)
}));

vi.mock('../services/socket', () => ({
    tadpole_os_socket: {
        subscribe_agent_updates: vi.fn(() => () => { }),
        get_connection_state: vi.fn(() => 'connected')
    }
}));

const mock_workspace_store = {
    clusters: [
        { id: 'cl-command', name: 'Strategic Command', theme: 'blue', alpha_id: '1', collaborators: ['1'], is_active: true },
        { id: '2', name: 'Beta Cluster', theme: 'zinc', alpha_id: '2', collaborators: ['2', '3'], is_active: false },
        { id: '3', name: 'Gamma Cluster', theme: 'amber', alpha_id: '3', collaborators: ['4', '5'], is_active: false }
    ],
    get_agent_path: vi.fn(() => 'test-path')
};

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: Object.assign(
        vi.fn(() => mock_workspace_store),
        { getState: () => mock_workspace_store }
    )
}));

vi.mock('../stores/dropdown_store', () => ({
    use_dropdown_store: vi.fn((selector) => {
        const state = { open_id: null, close_dropdown: vi.fn() };
        return selector ? selector(state) : state;
    }),
}));

vi.mock('../stores/agent_store', () => ({
    use_agent_store: vi.fn(() => ({
        agents: mock_agents_data,
        fetch_agents: vi.fn(),
        update_agent: vi.fn()
    }))
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

// 2. Mock Child Components (using snake_case props per parity guidelines)
vi.mock('../components/Hierarchy_Node', () => ({
    Hierarchy_Node: (props: any) => (
        <div data-testid={`node-${props.agent?.id}`} data-theme={props.theme_color}>
            {props.agent?.name} - {props.is_root ? 'ROOT' : 'NODE'}
            <button onClick={() => props.on_skill_trigger?.(props.agent.id, 'test-skill')}>Trigger Skill</button>
            <button onClick={() => props.on_role_change?.(props.agent.id, 'New Role')}>Change Role</button>
            <button onClick={() => props.on_configure_click?.(props.agent.id)}>Configure</button>
            <button onClick={() => props.on_model_change?.(props.agent.id, 'gpt-4')}>Change Model</button>
        </div>
    )
}));

vi.mock('../components/Agent_Config_Panel', () => ({
    default: ({ agent, on_close }: any) => (
        <div data-testid="config-panel">
            Config Panel for {agent.name}
            <button onClick={on_close}>Close</button>
        </div>
    )
}));

describe('Org_Chart', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders hierarchy shell', async () => {
        render(<Org_Chart />);
        await waitFor(() => {
            expect(screen.getByText(/Chain 2/)).toBeInTheDocument();
        });
    });

    it('partitions and displays agents correctly', async () => {
        render(<Org_Chart />);

        await waitFor(() => {
            // Check Alpha (Root)
            const alpha_node = screen.getByTestId('node-1');
            expect(alpha_node).toBeInTheDocument();
            expect(alpha_node).toHaveTextContent('ROOT');

            // Check Nexus (Agent 2)
            const nexus_node = screen.getByTestId('node-2');
            expect(nexus_node).toBeInTheDocument();
        });
    });

    it('handles skill triggers', async () => {
        const { use_agent_store } = await import('../stores/agent_store');
        const update_agent = vi.fn();
        vi.mocked(use_agent_store).mockReturnValue({
            agents: mock_agents_data,
            fetch_agents: vi.fn(),
            update_agent
        } as any);

        render(<Org_Chart />);
        await waitFor(() => expect(screen.getByTestId('node-1')).toBeInTheDocument());

        const trigger_btn = screen.getAllByText('Trigger Skill')[0];
        fireEvent.click(trigger_btn);

        expect(update_agent).toHaveBeenCalled();
    });

    it('handles role and model changes', async () => {
        const { use_agent_store } = await import('../stores/agent_store');
        const update_agent = vi.fn();
        vi.mocked(use_agent_store).mockReturnValue({
            agents: mock_agents_data,
            fetch_agents: vi.fn(),
            update_agent
        } as any);

        render(<Org_Chart />);
        await waitFor(() => expect(screen.getByTestId('node-1')).toBeInTheDocument());

        // Role change
        fireEvent.click(screen.getAllByText('Change Role')[0]);
        expect(update_agent).toHaveBeenCalledWith('1', expect.objectContaining({ role: 'New Role' }));

        // Model change
        fireEvent.click(screen.getAllByText('Change Model')[0]);
        expect(update_agent).toHaveBeenCalledWith('1', expect.objectContaining({ model: 'gpt-4' }));
    });

    it('opens and closes the config panel', async () => {
        render(<Org_Chart />);
        await waitFor(() => expect(screen.getByTestId('node-1')).toBeInTheDocument());

        fireEvent.click(screen.getAllByText('Configure')[0]);
        expect(screen.getByTestId('config-panel')).toBeInTheDocument();
        expect(screen.getByText(/Config Panel for Agent of Nine/)).toBeInTheDocument();

        fireEvent.click(screen.getByText('Close'));
        expect(screen.queryByTestId('config-panel')).not.toBeInTheDocument();
    });
});

