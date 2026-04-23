/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Mission control view's orchestration of swarm clusters.** 
 * Verifies active mission tracking, priority reassignment, and neural optimization proposals across multiple agent nodes. 
 * Mocks `workspace_store` and `socket` to isolate mission state from backend handoff latencies.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: State mismatch between the `missions_store` and the UI grid during rapid mission status transitions (e.g. queueing to execution).
 * - **Telemetry Link**: Search `[Missions.test]` in tracing logs.
 */


/**
 * @file Missions.test.tsx
 * @description Suite for the Swarm Missions (Mission Management) page.
 * @module Pages/Missions
 * @testedBehavior
 * - Mission Lifecycle: Listing, tracing, and deletion of active missions.
 * - Real-time Updates: Socket event integration for mission status changes.
 * - Error Handling: Propagation of service failures to user alerts via event_bus.
 * @aiContext
 * - Mocks tadpole_os_service and tadpole_os_socket for controlled mission state.
 * - Refactored for 100% snake_case architectural parity.
 */
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Missions from './Missions';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import { tadpole_os_socket } from '../services/socket';

// Mock Services
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        send_command: vi.fn(),
        get_mission_quotas: vi.fn(() => Promise.resolve({ quotas: [] })),
        update_mission_quota: vi.fn(() => Promise.resolve()),
    }
}));

vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
        subscribe_logs: vi.fn(() => () => { }),
        get_history: vi.fn(() => []),
    }
}));

vi.mock('../services/socket', () => ({
    tadpole_os_socket: {
        subscribe_handoff: vi.fn(() => () => { }),
        subscribe_agent_updates: vi.fn(() => () => { }),
    }
}));

// Mock Stores
const mock_workspace_state = {
    clusters: [
        {
            id: 'c-001',
            name: 'Alpha Cluster',
            department: 'Engineering',
            theme: 'blue',
            collaborators: ['1'],
            alpha_id: '1',
            objective: 'Complete mission objective',
            is_active: true,
            pending_tasks: [
                { id: 'ho-1', agent_id: '1', description: 'Incoming handoff', status: 'pending', target_path: '/tmp', timestamp: Date.now() }
            ],
            budget_usd: 50,
            analysis_enabled: false,
        }
    ],
    active_proposals: {
        'c-001': { reasoning: 'Proposal reasoning' }
    },
    active_cluster: {
        id: 'c-001',
        name: 'Alpha Cluster',
        department: 'Engineering',
        theme: 'blue',
        collaborators: ['1'],
        alpha_id: '1',
        objective: 'Complete mission objective',
        is_active: true,
        pending_tasks: [
            { id: 'ho-1', agent_id: '1', description: 'Incoming handoff', status: 'pending', target_path: '/tmp', timestamp: Date.now() }
        ],
        budget_usd: 50,
        analysis_enabled: false,
    },
    create_cluster: vi.fn(),
    assign_agent_to_cluster: vi.fn(),
    unassign_agent_from_cluster: vi.fn(),
    update_cluster_objective: vi.fn(),
    set_alpha_node: vi.fn(),
    delete_cluster: vi.fn(),
    toggle_cluster_active: vi.fn(),
    approve_branch: vi.fn(),
    reject_branch: vi.fn(),
    update_cluster_department: vi.fn(),
    update_cluster_budget: vi.fn(),
    toggle_mission_analysis: vi.fn(),
    dismiss_proposal: vi.fn(),
    apply_proposal: vi.fn(),
    receive_handoff: vi.fn(),
};

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: Object.assign(
        vi.fn((selector) => selector ? selector(mock_workspace_state) : mock_workspace_state),
        { getState: () => mock_workspace_state }
    )
}));

const mock_agent_state = {
    agents: [
        { id: '1', name: 'Agent 1', role: 'Dev', model: 'gpt-4o', model_config: { provider: 'openai', model_id: 'gpt-4o' } }
    ],
    fetch_agents: vi.fn(),
    update_agent: vi.fn(),
    is_loading: false
};

vi.mock('../stores/agent_store', () => ({
    use_agent_store: vi.fn((selector) => selector ? selector(mock_agent_state) : mock_agent_state)
}));

vi.mock('../stores/trace_store', () => ({
    use_trace_store: {
        getState: vi.fn(() => ({ set_active_trace: vi.fn() }))
    }
}));

// Mock Utility
vi.mock('../utils/model_utils', () => ({
    resolve_agent_model_config: vi.fn(() => ({ model_id: 'gpt-4o', provider: 'openai' }))
}));

// Mock Sub-components to simplify testing
vi.mock('../components/missions/Cluster_Sidebar', () => ({
    Cluster_Sidebar: (props: any) => (
        <div data-testid="sidebar">
            <button onClick={() => props.on_select_cluster('c-002')}>Select Cluster</button>
            <button onClick={() => props.on_create_cluster()}>Create Cluster</button>
            <button onClick={() => props.on_delete_cluster('c-001')}>Delete Cluster</button>
            <button onClick={() => props.on_toggle_active('c-001')}>Toggle Active</button>
            <button onClick={() => props.on_update_department('c-001', 'Test Dept')}>Update Dept</button>
            <button onClick={() => props.on_update_budget('c-001', 999)}>Update Budget</button>
        </div>
    )
}));

vi.mock('../components/missions/Mission_Header', () => ({
    Mission_Header: ({ on_run_mission, on_toggle_analysis, active_cluster }: any) => (
        <div>
            <button onClick={on_run_mission}>RUN MISSION</button>
            <button onClick={() => on_toggle_analysis(active_cluster.id)}>Toggle Analysis</button>
        </div>
    )
}));

vi.mock('../components/missions/Neural_Map', () => ({
    Neural_Map: () => <div data-testid="neural-map">Neural Map</div>
}));

vi.mock('../components/missions/Agent_Team_View', () => ({
    Agent_Team_View: (props: any) => (
        <div data-testid="team-view">
            <button onClick={() => props.on_assign('2')}>Assign Agent</button>
            <button onClick={() => props.on_unassign('1')}>Unassign Agent</button>
            <button onClick={() => props.on_set_alpha('1')}>Set Alpha</button>
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
        const original_alpha = mock_workspace_state.clusters[0].alpha_id;
        mock_workspace_state.clusters[0].alpha_id = undefined as any;
        
        render(<Missions />);
        fireEvent.click(screen.getByText('RUN MISSION'));
        
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ 
            severity: 'error',
            text: expect.stringContaining('No Alpha Node designated')
        }));
        
        mock_workspace_state.clusters[0].alpha_id = original_alpha;
    });

    it('shows error if no objective is set', async () => {
        const original_objective = mock_workspace_state.clusters[0].objective;
        mock_workspace_state.clusters[0].objective = '';
        
        render(<Missions />);
        fireEvent.click(screen.getByText('RUN MISSION'));
        
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ 
            severity: 'error',
            text: expect.stringContaining('Objective not defined')
        }));
        
        mock_workspace_state.clusters[0].objective = original_objective;
    });

    it('handles command rejection from engine', async () => {
        (tadpole_os_service.send_command as any).mockResolvedValue(false);
        render(<Missions />);

        fireEvent.click(screen.getByText('RUN MISSION'));

        await waitFor(() => {
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ 
                severity: 'error',
                text: expect.stringContaining('Engine rejected')
            }));
        });
    });

    it('handles send_command throw exception', async () => {
        (tadpole_os_service.send_command as any).mockRejectedValue(new Error('Network Crash'));
        render(<Missions />);

        fireEvent.click(screen.getByText('RUN MISSION'));

        await waitFor(() => {
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ 
                severity: 'error',
                text: expect.stringContaining('Mission Launch Failed: Network Crash')
            }));
        });
    });

    it('launches a mission correctly', async () => {
        (tadpole_os_service.send_command as any).mockResolvedValue(true);
        render(<Missions />);

        fireEvent.click(screen.getByText('RUN MISSION'));

        await waitFor(() => {
            expect(tadpole_os_service.send_command).toHaveBeenCalled();
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ severity: 'success' }));
        });
    });

    it('handles neural optimization proposal', () => {
        render(<Missions />);

        expect(screen.getByText('Proposal reasoning')).toBeInTheDocument();

        fireEvent.click(screen.getByText('Dismiss'));
        expect(mock_workspace_state.dismiss_proposal).toHaveBeenCalledWith('c-001');

        fireEvent.click(screen.getByText(/Authorize Sync/i));
        expect(mock_workspace_state.apply_proposal).toHaveBeenCalledWith('c-001');
    });

    it('handles incoming handoffs (approve and reject)', () => {
        render(<Missions />);

        expect(screen.getByText('Incoming handoff')).toBeInTheDocument();

        // approve_branch
        fireEvent.click(screen.getByText('Incoming handoff').closest('div')?.nextSibling?.firstChild as Element);
        expect(mock_workspace_state.approve_branch).toHaveBeenCalled();

        // reject_branch
        fireEvent.click(screen.getByText('Incoming handoff').closest('div')?.nextSibling?.lastChild as Element);
        expect(mock_workspace_state.reject_branch).toHaveBeenCalled();
    });

    it('updates objective on textarea change', () => {
        render(<Missions />);
        const textarea = screen.getByPlaceholderText(/Describe the cluster/i);
        fireEvent.change(textarea, { target: { value: 'New Objective' } });
        expect(mock_workspace_state.update_cluster_objective).toHaveBeenCalledWith('c-001', 'New Objective');
    });

    it('handles cluster management actions from sidebar', () => {
        render(<Missions />);
        
        fireEvent.click(screen.getByText('Create Cluster'));
        expect(mock_workspace_state.create_cluster).toHaveBeenCalled();

        fireEvent.click(screen.getByText('Delete Cluster'));
        expect(mock_workspace_state.delete_cluster).toHaveBeenCalledWith('c-001');

        fireEvent.click(screen.getByText('Toggle Active'));
        expect(mock_workspace_state.toggle_cluster_active).toHaveBeenCalledWith('c-001');

        fireEvent.click(screen.getByText('Update Dept'));
        expect(mock_workspace_state.update_cluster_department).toHaveBeenCalledWith('c-001', 'Test Dept');

        fireEvent.click(screen.getByText('Update Budget'));
        expect(mock_workspace_state.update_cluster_budget).toHaveBeenCalledWith('c-001', 999);
    });

    it('handles agent team management actions', () => {
        render(<Missions />);
        
        fireEvent.click(screen.getByText('Assign Agent'));
        expect(mock_workspace_state.assign_agent_to_cluster).toHaveBeenCalledWith('2', 'c-001');

        fireEvent.click(screen.getByText('Unassign Agent'));
        expect(mock_workspace_state.unassign_agent_from_cluster).toHaveBeenCalledWith('1', 'c-001');

        fireEvent.click(screen.getByText('Set Alpha'));
        expect(mock_workspace_state.set_alpha_node).toHaveBeenCalledWith('c-001', '1');
    });

    it('toggles mission analysis', () => {
        render(<Missions />);
        fireEvent.click(screen.getByText('Toggle Analysis'));
        expect(mock_workspace_state.toggle_mission_analysis).toHaveBeenCalledWith('c-001');
    });

    it('calls receive_handoff when socket event arrives', () => {
        let handoff_cb: any;
        (tadpole_os_socket.subscribe_handoff as any).mockImplementation((cb: any) => {
            handoff_cb = cb;
            return () => {};
        });

        render(<Missions />);

        if (handoff_cb) {
            handoff_cb({
                from_cluster: 'src',
                to_cluster: 'c-001',
                payload: { description: 'New Handoff via Socket' }
            });
        }

        expect(mock_workspace_state.receive_handoff).toHaveBeenCalled();
    });
});


// Metadata: [Missions_test]

// Metadata: [Missions_test]
