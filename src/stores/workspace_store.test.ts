/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Project Workspace and Cluster isolation store.** 
 * Verifies the mapping of mission clusters to physical filesystem paths, the management of project-level budget quotas, and the orchestration of agent-to-cluster assignments. 
 * Tests the generation and application of AI-driven architectural refinements (proposals) and inter-cluster task delegation (handoffs). 
 * Mocks `proposal_service` and `settings_store` to isolate mission orchestration from backend model latency and persistent environment state.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Filesystem path collisions or failure to enforce project-level resource isolation during a multi-cluster deployment.
 * - **Telemetry Link**: Search `[workspace_store.test]` in tracing logs.
 */


/**
 * @file workspace_store.test.ts
 * @description Suite for the Swarm Hierarchy and Cluster Management.
 * @module Stores/WorkspaceStore
 * @testedBehavior
 * - Cluster Lifecycle: CRUD for tactical mission clusters and budget governance.
 * - Agent Assignment: Orchestration of agent-to-cluster mapping and Alpha node elevation.
 * - Intelligence Proposals: Generation and application of AI-driven architectural refinements.
 * - Handoffs: Inter-cluster task delegation and status synchronization.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks proposal_service for deterministic AI-driven mission refinement.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { use_workspace_store } from './workspace_store';
import * as settings_store from './settings_store';
import { proposal_service } from '../services/proposal_service';
import { log_error } from '../services/system_utils';

// Mock Dependencies
vi.mock('./settings_store', () => ({
    get_settings: vi.fn()
}));
vi.mock('../services/proposal_service', () => ({
    proposal_service: {
        generate_proposal: vi.fn()
    }
}));

vi.mock('../services/system_utils', () => ({
    log_error: vi.fn(),
}));

describe('use_workspace_store', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.useFakeTimers();
        use_workspace_store.setState({
            clusters: [{
                id: 'cl-command',
                name: 'Strategic Command',
                department: 'Executive',
                path: '/workspaces/strategic-command',
                collaborators: ['1', '2'],
                alpha_id: '1',
                objective: 'Global swarm oversight.',
                theme: 'blue',
                pending_tasks: [],
                is_active: true
            }],
            active_proposals: {}
        });
        
        // Mock default settings for workspace test
        vi.mocked(settings_store.get_settings).mockReturnValue({ max_clusters: 5 } as any);
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe('Cluster CRUD', () => {
        it('creates a cluster within limits', () => {
            const store = use_workspace_store.getState();
            store.create_cluster({ name: 'New Test Cluster' });

            const state = use_workspace_store.getState();
            expect(state.clusters).toHaveLength(2);
            expect(state.clusters[1].name).toBe('New Test Cluster');
            expect(state.clusters[1].department).toBe('Engineering'); // Default fallback
            expect(state.clusters[1].theme).toBe('blue');            // Default fallback
        });

        it('prevents creating a cluster if limit reached', () => {
            vi.mocked(settings_store.get_settings).mockReturnValue({ max_clusters: 1 } as any);

            const store = use_workspace_store.getState();
            store.create_cluster({ name: 'Blocked Cluster' });

            const state = use_workspace_store.getState();
            expect(state.clusters).toHaveLength(1); // Still 1 from beforeEach
            expect(log_error).toHaveBeenCalledWith(
                'WorkspaceStore', 
                expect.stringContaining('Cluster limit reached'), 
                null, 
                'warning'
            );
        });

        it('deletes a cluster', () => {
            const store = use_workspace_store.getState();
            store.delete_cluster('cl-command');

            const state = use_workspace_store.getState();
            expect(state.clusters).toHaveLength(0);
        });

        it('toggles cluster active state', () => {
            const store = use_workspace_store.getState();
            store.toggle_cluster_active('cl-command'); // toggle off default true
            
            const state = use_workspace_store.getState();
            expect(state.clusters[0].is_active).toBe(false);

            store.toggle_cluster_active('cl-command'); // toggle on
            expect(use_workspace_store.getState().clusters[0].is_active).toBe(true);
        });

        it('toggles mission analysis state', () => {
             const store = use_workspace_store.getState();
             store.toggle_mission_analysis('cl-command');
  
             const state = use_workspace_store.getState();
             expect(state.clusters[0].analysis_enabled).toBe(true);
        });
    });

    describe('Agent Assignments', () => {
        it('assigns agent to cluster uniquely', () => {
            const store = use_workspace_store.getState();
            store.assign_agent_to_cluster('3', 'cl-command');
            store.assign_agent_to_cluster('3', 'cl-command'); // Duplicate

            const state = use_workspace_store.getState();
            expect(state.clusters[0].collaborators).toEqual(['1', '2', '3']);
        });

        it('unassigns an agent from cluster and clears alpha_id if matching', () => {
            const store = use_workspace_store.getState();
            store.unassign_agent_from_cluster('1', 'cl-command');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].collaborators).toEqual(['2']);
            expect(state.clusters[0].alpha_id).toBeUndefined(); // '1' was the alpha_id
        });

        it('sets alpha node explicitly', () => {
            const store = use_workspace_store.getState();
            store.set_alpha_node('cl-command', '2');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].alpha_id).toBe('2');
        });
    });

    describe('Cluster Properties', () => {
        it('updates department', () => {
            const store = use_workspace_store.getState();
            store.update_cluster_department('cl-command', 'Sales');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].department).toBe('Sales');
        });

        it('updates budget', () => {
            const store = use_workspace_store.getState();
            store.update_cluster_budget('cl-command', 500);

            const state = use_workspace_store.getState();
            expect(state.clusters[0].budget_usd).toBe(500);
        });
    });

    describe('Proposals', () => {
        it('debounces proposal generation on objective update', () => {
            const store = use_workspace_store.getState();
            store.update_cluster_objective('cl-command', 'New objective here');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].objective).toBe('New objective here');

            expect(proposal_service.generate_proposal).not.toHaveBeenCalled();

            vi.advanceTimersByTime(1100);

            expect(proposal_service.generate_proposal).toHaveBeenCalled();
        });

        it('generates proposal directly and merges into active', () => {
            vi.mocked(proposal_service.generate_proposal).mockReturnValue({
                cluster_id: 'cl-command',
                reasoning: 'AI generated reason',
                changes: [],
                timestamp: 1000
            });

            const store = use_workspace_store.getState();
            store.generate_proposal('cl-command');

            const state = use_workspace_store.getState();
            expect(state.active_proposals['cl-command'].reasoning).toBe('AI generated reason');
        });

        it('applies proposal completely and clears it', () => {
            use_workspace_store.setState({ active_proposals: { 'cl-command': { cluster_id: 'cl-command', reasoning: 'test', changes: [], timestamp: 1 } } });
            
            const store = use_workspace_store.getState();
            store.apply_proposal('cl-command');

            const state = use_workspace_store.getState();
            expect(state.active_proposals['cl-command']).toBeUndefined();
        });

        it('dismisses proposal and clears it', () => {
            use_workspace_store.setState({ active_proposals: { 'cl-command': { cluster_id: 'cl-command', reasoning: 'test', changes: [], timestamp: 1 } } });
            
            const store = use_workspace_store.getState();
            store.dismiss_proposal('cl-command');

            const state = use_workspace_store.getState();
            expect(state.active_proposals['cl-command']).toBeUndefined();
        });
        
        it('ignores generating proposal for missing cluster', () => {
            const store = use_workspace_store.getState();
            store.generate_proposal('fake-id');
            expect(proposal_service.generate_proposal).not.toHaveBeenCalled();
        });

        it('ignores applying missing proposals', () => {
            const store = use_workspace_store.getState();
            store.apply_proposal('cl-command');
            expect(Object.keys(use_workspace_store.getState().active_proposals).length).toBe(0);
        });
    });

    describe('Branching & Handoffs', () => {
        it('adds a branch', () => {
            const store = use_workspace_store.getState();
            store.add_branch('cl-command', { agent_id: '1', description: 'test', target_path: '/dev' });

            const state = use_workspace_store.getState();
            expect(state.clusters[0].pending_tasks).toHaveLength(1);
            expect(state.clusters[0].pending_tasks[0].description).toBe('test');
            expect(state.clusters[0].pending_tasks[0].status).toBe('pending');
        });

        it('approves a branch', () => {
            const branch_id = 'br-123';
            use_workspace_store.setState({
                clusters: [{
                    id: 'cl-command',
                    pending_tasks: [{ id: branch_id, agent_id: '1', description: 'test', target_path: '/dev', status: 'pending', timestamp: 1 }]
                } as any]
            });

            const store = use_workspace_store.getState();
            store.approve_branch('cl-command', branch_id);

            const state = use_workspace_store.getState();
            expect(state.clusters[0].pending_tasks[0].status).toBe('completed');
        });

        it('rejects a branch', () => {
             const branch_id = 'br-456';
             use_workspace_store.setState({
                clusters: [{
                    id: 'cl-command',
                    pending_tasks: [{ id: branch_id, agent_id: '1', description: 'test', target_path: '/dev', status: 'pending', timestamp: 1 }]
                } as any]
            });

            const store = use_workspace_store.getState();
            store.reject_branch('cl-command', branch_id);

            const state = use_workspace_store.getState();
            expect(state.clusters[0].pending_tasks[0].status).toBe('rejected');
        });

        it('receives a handoff', () => {
             const store = use_workspace_store.getState();
             store.receive_handoff('cl-source', 'cl-command', 'Passing the torch');
  
             const state = use_workspace_store.getState();
             expect(state.clusters[0].pending_tasks[0].description).toContain('[HANDOFF FROM cl-source]');
        });
    });

    describe('Path Calculations', () => {
        it('resolves correct cluster path for members', () => {
            const store = use_workspace_store.getState();
            const path = store.get_agent_path('2');
            expect(path).toBe('/workspaces/strategic-command'); // defined in mock
        });

        it('resolves fallback silo path for wandering agents', () => {
            const store = use_workspace_store.getState();
            const path = store.get_agent_path('999');
            expect(path).toBe('/workspaces/agent-silo-999');
        });
    });
});


// Metadata: [workspace_store_test]

// Metadata: [workspace_store_test]
