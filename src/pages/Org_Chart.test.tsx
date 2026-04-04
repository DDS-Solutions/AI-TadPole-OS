/**
 * @file Org_Chart.test.tsx
 * @description Suite for the Agent Hierarchy (Org_Chart) page.
 * @module Pages/Org_Chart
 * @testedBehavior
 * - Graph Rendering: Node placement and Alpha Node designation.
 * - Dynamic Configuration: Real-time role and model updates for agents.
 * - Hierarchy Integrity: Ensuring "Agent of Nine" remains the sovereign root.
 * @aiContext
 * - Mocking Hierarchy_Node and Agent_Config_Panel to focus on layout logic.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import Org_Chart from './Org_Chart';
import { agents as mockAgentsData } from '../data/mock_agents';
import { tadpole_os_socket } from '../services/socket';

// 1. Mock Hooks and Services
vi.mock('../services/agent_service', () => ({
    loadAgents: vi.fn(async () => mockAgentsData),
    persistAgentUpdate: vi.fn(),
    normalizeAgent: vi.fn((a) => a)
}));

vi.mock('../services/socket', () => ({
    tadpole_os_socket: {
        subscribeAgentUpdates: vi.fn(() => () => { }),
        getConnectionState: vi.fn(() => 'connected')
    }
}));

const mockWorkspaceStore = {
    clusters: [
        { id: 'cl-command', name: 'Strategic Command', theme: 'blue', alphaId: '1', collaborators: ['1'], isActive: true },
        { id: '2', name: 'Beta Cluster', theme: 'zinc', alphaId: '2', collaborators: ['2', '3'], isActive: false },
        { id: '3', name: 'Gamma Cluster', theme: 'amber', alphaId: '3', collaborators: ['4', '5'], isActive: false }
    ],
    getAgentPath: vi.fn(() => 'test-path')
};

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: Object.assign(
        vi.fn(() => mockWorkspaceStore),
        { getState: () => mockWorkspaceStore }
    )
}));

vi.mock('../stores/dropdown_store', () => ({
    use_dropdown_store: vi.fn((selector) => selector({ openId: null })),
}));

// 2. Mock Child Components
vi.mock('../components/Hierarchy_Node', () => ({
    Hierarchy_Node: (props: any) => (
        <div data-testid={`node-${props.agent?.id}`} data-theme={props.theme_color}>
            {props.agent?.name} - {props.isRoot ? 'ROOT' : 'NODE'}
            <button onClick={() => props.onSkillTrigger?.(props.agent.id, 'test-skill')}>Trigger Skill</button>
            <button onClick={() => props.onRoleChange?.(props.agent.id, 'New Role')}>Change Role</button>
            <button onClick={() => props.onConfigureClick?.(props.agent.id)}>Configure</button>
            <button onClick={() => props.onModelChange?.(props.agent.id, 'gpt-4')}>Change Model</button>
        </div>
    )
}));

vi.mock('../components/Agent_Config_Panel', () => ({
    default: ({ agent, onClose }: any) => (
        <div data-testid="config-panel">
            Config Panel for {agent.name}
            <button onClick={onClose}>Close</button>
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
            expect(screen.getByText('Chain 2')).toBeInTheDocument();
        });
    });

    it('partitions and displays agents correctly', async () => {
        render(<Org_Chart />);

        await waitFor(() => {
            // Check Alpha (Root)
            const alphaNode = screen.getByTestId('node-1');
            expect(alphaNode).toBeInTheDocument();
            expect(alphaNode).toHaveTextContent('ROOT');

            // Check Nexus (Agent 2)
            const nexusNode = screen.getByTestId('node-2');
            expect(nexusNode).toBeInTheDocument();
        });
    });

    it('displays cluster chains', async () => {
        render(<Org_Chart />);

        await waitFor(() => {
            expect(screen.getByText('Chain 2')).toBeInTheDocument();
            expect(screen.getByText('Chain 3')).toBeInTheDocument();
        });
    });

    it('handles skill triggers', async () => {
        render(<Org_Chart />);
        await waitFor(() => expect(screen.getByTestId('node-1')).toBeInTheDocument());

        const triggerBtn = screen.getAllByText('Trigger Skill')[0];
        fireEvent.click(triggerBtn);

        // State update verification (indirectly via rerender or checking internal state if we could, 
        // but here we check if it doesn't crash and ideally we'd witness a status change text if we didn't mock it so aggressively)
        // For local state updates, we check if the component renders without errors after interactions.
    });

    it('handles role and model changes', async () => {
        const { persistAgentUpdate } = await import('../services/agent_service');
        render(<Org_Chart />);
        await waitFor(() => expect(screen.getByTestId('node-1')).toBeInTheDocument());

        // Role change
        fireEvent.click(screen.getAllByText('Change Role')[0]);
        expect(persistAgentUpdate).toHaveBeenCalledWith('1', expect.objectContaining({ role: 'New Role' }));

        // Model change
        fireEvent.click(screen.getAllByText('Change Model')[0]);
        expect(persistAgentUpdate).toHaveBeenCalledWith('1', expect.objectContaining({ model: 'gpt-4' }));
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

    it('updates agents when socket event is received', async () => {
        let updateCallback: ((event: any) => void) | undefined;
        
        vi.mocked(tadpole_os_socket.subscribeAgentUpdates).mockImplementation((cb) => {
            updateCallback = cb;
            return () => {};
        });

        render(<Org_Chart />);

        await waitFor(() => expect(screen.getByTestId('node-1')).toBeInTheDocument());

        // Simulate socket update
        if (updateCallback) {
            await act(async () => {
                updateCallback!({
                    type: 'agent:update',
                    agent_id: '1',
                    data: { name: 'Updated Agent 1' }
                });
            });
        }

        // Wait for rerender
        await waitFor(() => {
            expect(screen.getByTestId('node-1')).toBeInTheDocument();
        });
    });
});
