/**
 * @file Missions.test.tsx
 * @description Suite for the Swarm Missions (Mission Management) page.
 * @module Pages/Missions
 * @testedBehavior
 * - Mission Lifecycle: Listing, tracing, and deletion of active missions.
 * - Real-time Updates: Socket event integration for mission status changes.
 * - Error Handling: Propagation of service failures to user alerts via EventBus.
 * @aiContext
 * - Mocks TadpoleOSService and TadpoleOSSocket for controlled mission state.
 */
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Missions from './Missions';
import { TadpoleOSService } from '../services/tadpoleosService';
import { EventBus } from '../services/eventBus';
import { TadpoleOSSocket } from '../services/socket';

// Mock Services
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        sendCommand: vi.fn(),
    }
}));

vi.mock('../services/eventBus', () => ({
    EventBus: {
        emit: vi.fn(),
        subscribe: vi.fn(() => () => { }),
        getHistory: vi.fn(() => []),
    }
}));

vi.mock('../services/socket', () => ({
    TadpoleOSSocket: {
        subscribeHandoff: vi.fn(() => () => { }),
        subscribeAgentUpdates: vi.fn(() => () => { }),
    }
}));

// Mock Stores
const mockWorkspaceState = {
    clusters: [
        {
            id: 'c-001',
            name: 'Alpha Cluster',
            department: 'Engineering',
            theme: 'blue',
            collaborators: ['1'],
            alphaId: '1',
            objective: 'Complete mission objective',
            isActive: true,
            pendingTasks: [
                { id: 'ho-1', description: 'Incoming handoff', status: 'pending' }
            ],
            budgetUsd: 50,
            analysisEnabled: false,
        }
    ],
    activeProposals: {
        'c-001': { reasoning: 'Proposal reasoning' }
    },
    activeCluster: {
        id: 'c-001',
        name: 'Alpha Cluster',
        department: 'Engineering',
        theme: 'blue',
        collaborators: ['1'],
        alphaId: '1',
        objective: 'Complete mission objective',
        isActive: true,
        pendingTasks: [
            { id: 'ho-1', description: 'Incoming handoff', status: 'pending' }
        ],
        budgetUsd: 50,
        analysisEnabled: false,
    },
    createCluster: vi.fn(),
    assignAgentToCluster: vi.fn(),
    unassignAgentFromCluster: vi.fn(),
    updateClusterObjective: vi.fn(),
    setAlphaNode: vi.fn(),
    deleteCluster: vi.fn(),
    toggleClusterActive: vi.fn(),
    approveBranch: vi.fn(),
    rejectBranch: vi.fn(),
    updateClusterDepartment: vi.fn(),
    updateClusterBudget: vi.fn(),
    toggleMissionAnalysis: vi.fn(),
    dismissProposal: vi.fn(),
    applyProposal: vi.fn(),
    receiveHandoff: vi.fn(),
};

vi.mock('../stores/workspaceStore', () => ({
    useWorkspaceStore: Object.assign(
        vi.fn((selector) => selector ? selector(mockWorkspaceState) : mockWorkspaceState),
        { getState: () => mockWorkspaceState }
    )
}));

const mockAgentState = {
    agents: [
        { id: '1', name: 'Agent 1', role: 'Dev', model: 'gpt-4o', modelConfig: { provider: 'openai', modelId: 'gpt-4o' } }
    ],
    fetchAgents: vi.fn(),
    updateAgent: vi.fn(),
    isLoading: false
};

vi.mock('../stores/agentStore', () => ({
    useAgentStore: vi.fn((selector) => selector ? selector(mockAgentState) : mockAgentState)
}));

vi.mock('../stores/traceStore', () => ({
    useTraceStore: {
        getState: vi.fn(() => ({ setActiveTrace: vi.fn() }))
    }
}));

// Mock Utility
vi.mock('../utils/modelUtils', () => ({
    resolveAgentModelConfig: vi.fn(() => ({ modelId: 'gpt-4o', provider: 'openai' }))
}));

// Mock Sub-components to simplify testing
vi.mock('../components/missions/ClusterSidebar', () => ({
    ClusterSidebar: (props: any) => (
        <div data-testid="sidebar">
            <button onClick={() => props.onSelectCluster('c-002')}>Select Cluster</button>
            <button onClick={() => props.onCreateCluster()}>Create Cluster</button>
            <button onClick={() => props.onDeleteCluster('c-001')}>Delete Cluster</button>
            <button onClick={() => props.onToggleActive('c-001')}>Toggle Active</button>
            <button onClick={() => props.onUpdateDepartment('c-001', 'Test Dept')}>Update Dept</button>
            <button onClick={() => props.onUpdateBudget('c-001', 999)}>Update Budget</button>
        </div>
    )
}));
vi.mock('../components/missions/MissionHeader', () => ({
    MissionHeader: ({ onRunMission, onToggleAnalysis, activeCluster }: any) => (
        <div>
            <button onClick={onRunMission}>RUN MISSION</button>
            <button onClick={() => onToggleAnalysis(activeCluster.id)}>Toggle Analysis</button>
        </div>
    )
}));
vi.mock('../components/missions/NeuralMap', () => ({
    NeuralMap: () => <div data-testid="neural-map">Neural Map</div>
}));
vi.mock('../components/missions/AgentTeamView', () => ({
    AgentTeamView: (props: any) => (
        <div data-testid="team-view">
            <button onClick={() => props.onAssign('2')}>Assign Agent</button>
            <button onClick={() => props.onUnassign('1')}>Unassign Agent</button>
            <button onClick={() => props.onSetAlpha('1')}>Set Alpha</button>
        </div>
    )
}));

describe('Missions Page', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders the mission management shell', async () => {
        render(<Missions />);
        expect(screen.getByTestId('sidebar')).toBeInTheDocument();
        expect(screen.getByTestId('neural-map')).toBeInTheDocument();
    });

    it('shows error if no alpha node is assigned', async () => {
        // Temporarily change mock state to have no alpha
        const originalAlpha = mockWorkspaceState.clusters[0].alphaId;
        mockWorkspaceState.clusters[0].alphaId = undefined as any;
        
        render(<Missions />);
        fireEvent.click(screen.getByText('RUN MISSION'));
        
        expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ 
            severity: 'error',
            text: expect.stringContaining('No Alpha Node designated')
        }));
        
        mockWorkspaceState.clusters[0].alphaId = originalAlpha;
    });

    it('shows error if no objective is set', async () => {
        const originalObjective = mockWorkspaceState.clusters[0].objective;
        mockWorkspaceState.clusters[0].objective = '';
        
        render(<Missions />);
        fireEvent.click(screen.getByText('RUN MISSION'));
        
        expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ 
            severity: 'error',
            text: expect.stringContaining('Objective not defined')
        }));
        
        mockWorkspaceState.clusters[0].objective = originalObjective;
    });

    it('handles command rejection from engine', async () => {
        (TadpoleOSService.sendCommand as any).mockResolvedValue(false);
        render(<Missions />);

        fireEvent.click(screen.getByText('RUN MISSION'));

        await waitFor(() => {
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ 
                severity: 'error',
                text: expect.stringContaining('Engine rejected')
            }));
        });
    });

    it('handles sendCommand throw exception', async () => {
        (TadpoleOSService.sendCommand as any).mockRejectedValue(new Error('Network Crash'));
        render(<Missions />);

        fireEvent.click(screen.getByText('RUN MISSION'));

        await waitFor(() => {
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ 
                severity: 'error',
                text: expect.stringContaining('Mission Launch Failed: Network Crash')
            }));
        });
    });

    it('launches a mission correctly', async () => {
        (TadpoleOSService.sendCommand as any).mockResolvedValue(true);
        render(<Missions />);

        fireEvent.click(screen.getByText('RUN MISSION'));

        await waitFor(() => {
            expect(TadpoleOSService.sendCommand).toHaveBeenCalled();
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ severity: 'success' }));
        });
    });

    it('handles neural optimization proposal', () => {
        // activeCluster and activeProposals are now set in mockWorkspaceState by default
        render(<Missions />);

        expect(screen.getByText('Proposal reasoning')).toBeInTheDocument();

        fireEvent.click(screen.getByText('Dismiss'));
        expect(mockWorkspaceState.dismissProposal).toHaveBeenCalledWith('c-001');

        fireEvent.click(screen.getByText(/Authorize Sync/i));
        expect(mockWorkspaceState.applyProposal).toHaveBeenCalledWith('c-001');
    });

    it('handles incoming handoffs (approve and reject)', () => {
        render(<Missions />);

        expect(screen.getByText('Incoming handoff')).toBeInTheDocument();

        // approveBranch
        fireEvent.click(screen.getByText('Incoming handoff').closest('div')?.nextSibling?.firstChild as Element);
        expect(mockWorkspaceState.approveBranch).toHaveBeenCalled();

        // rejectBranch
        fireEvent.click(screen.getByText('Incoming handoff').closest('div')?.nextSibling?.lastChild as Element);
        expect(mockWorkspaceState.rejectBranch).toHaveBeenCalled();
    });

    it('updates objective on textarea change', () => {
        render(<Missions />);
        const textarea = screen.getByPlaceholderText(/Describe the cluster/i);
        fireEvent.change(textarea, { target: { value: 'New Objective' } });
        expect(mockWorkspaceState.updateClusterObjective).toHaveBeenCalledWith('c-001', 'New Objective');
    });

    it('handles cluster management actions from sidebar', () => {
        render(<Missions />);
        
        fireEvent.click(screen.getByText('Create Cluster'));
        expect(mockWorkspaceState.createCluster).toHaveBeenCalled();

        fireEvent.click(screen.getByText('Delete Cluster'));
        expect(mockWorkspaceState.deleteCluster).toHaveBeenCalledWith('c-001');

        fireEvent.click(screen.getByText('Toggle Active'));
        expect(mockWorkspaceState.toggleClusterActive).toHaveBeenCalledWith('c-001');

        fireEvent.click(screen.getByText('Update Dept'));
        expect(mockWorkspaceState.updateClusterDepartment).toHaveBeenCalledWith('c-001', 'Test Dept');

        fireEvent.click(screen.getByText('Update Budget'));
        expect(mockWorkspaceState.updateClusterBudget).toHaveBeenCalledWith('c-001', 999);
    });

    it('handles agent team management actions', () => {
        render(<Missions />);
        
        fireEvent.click(screen.getByText('Assign Agent'));
        expect(mockWorkspaceState.assignAgentToCluster).toHaveBeenCalledWith('2', 'c-001');

        fireEvent.click(screen.getByText('Unassign Agent'));
        expect(mockWorkspaceState.unassignAgentFromCluster).toHaveBeenCalledWith('1', 'c-001');

        fireEvent.click(screen.getByText('Set Alpha'));
        expect(mockWorkspaceState.setAlphaNode).toHaveBeenCalledWith('c-001', '1');
    });

    it('toggles mission analysis', () => {
        render(<Missions />);
        fireEvent.click(screen.getByText('Toggle Analysis'));
        expect(mockWorkspaceState.toggleMissionAnalysis).toHaveBeenCalledWith('c-001');
    });

    it('calls receiveHandoff when socket event arrives', () => {
        let handoffCb: any;
        (TadpoleOSSocket.subscribeHandoff as any).mockImplementation((cb: any) => {
            handoffCb = cb;
            return () => {};
        });

        render(<Missions />);

        if (handoffCb) {
            handoffCb({
                fromCluster: 'src',
                toCluster: 'c-001',
                payload: { description: 'New Handoff via Socket' }
            });
        }

        expect(mockWorkspaceState.receiveHandoff).toHaveBeenCalled();
    });
});
