/**
 * @file workspaceStore.test.ts
 * @description Suite for Org-level workspace clustering and AI-driven proposals.
 * @module Stores/WorkspaceStore
 * @testedBehavior
 * - Governance: Enforcing cluster limits based on global user settings.
 * - AI Intelligence: Debounced generation of 'Next Step' proposals via proposal_service.
 * - Collaboration: Handoff logic and alpha-agent assignment within clusters.
 * - Routing: Resolution of agent-silo paths vs cluster-shared paths.
 * @aiContext
 * - Uses vitest fake timers to validate debounced proposal generation.
 * - Mocks proposal_service and settingsStore.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { use_workspace_store } from './workspace_store';
import * as settingsStore from './settings_store';
import { proposal_service } from '../services/proposal_service';

// Mock Dependencies
vi.mock('./settings_store', () => ({
    getSettings: vi.fn()
}));
vi.mock('../services/proposal_service', () => ({
    proposal_service: {
        generateProposal: vi.fn()
    }
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
                alphaId: '1',
                objective: 'Global swarm oversight.',
                theme: 'blue',
                pendingTasks: [],
                isActive: true
            }],
            activeProposals: {}
        });
        
        // Mock default settings for workspace test
        vi.mocked(settingsStore.getSettings).mockReturnValue({ maxClusters: 5 } as any);
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe('Cluster CRUD', () => {
        it('creates a cluster within limits', () => {
            const store = use_workspace_store.getState();
            store.createCluster({ name: 'New Test Cluster' });

            const state = use_workspace_store.getState();
            expect(state.clusters).toHaveLength(2);
            expect(state.clusters[1].name).toBe('New Test Cluster');
            expect(state.clusters[1].department).toBe('Engineering'); // Default fallback
            expect(state.clusters[1].theme).toBe('blue');            // Default fallback
        });

        it('prevents creating a cluster if limit reached', () => {
            vi.mocked(settingsStore.getSettings).mockReturnValue({ maxClusters: 1 } as any);
            const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

            const store = use_workspace_store.getState();
            store.createCluster({ name: 'Blocked Cluster' });

            const state = use_workspace_store.getState();
            expect(state.clusters).toHaveLength(1); // Still 1 from beforeEach
            expect(consoleSpy).toHaveBeenCalledWith(expect.stringContaining('Cluster limit reached'));

            consoleSpy.mockRestore();
        });

        it('deletes a cluster', () => {
            const store = use_workspace_store.getState();
            store.deleteCluster('cl-command');

            const state = use_workspace_store.getState();
            expect(state.clusters).toHaveLength(0);
        });

        it('toggles cluster active state', () => {
            const store = use_workspace_store.getState();
            store.toggleClusterActive('cl-command'); // toggle off default true

            const state = use_workspace_store.getState();
            expect(state.clusters[0].isActive).toBe(false);

            store.toggleClusterActive('cl-command'); // toggle on
            expect(use_workspace_store.getState().clusters[0].isActive).toBe(true);
        });

        it('toggles mission analysis state', () => {
             const store = use_workspace_store.getState();
             store.toggleMissionAnalysis('cl-command');
 
             const state = use_workspace_store.getState();
             expect(state.clusters[0].analysisEnabled).toBe(true);
        });
    });

    describe('Agent Assignments', () => {
        it('assigns agent to cluster uniquely', () => {
            const store = use_workspace_store.getState();
            store.assignAgentToCluster('3', 'cl-command');
            store.assignAgentToCluster('3', 'cl-command'); // Duplicate

            const state = use_workspace_store.getState();
            expect(state.clusters[0].collaborators).toEqual(['1', '2', '3']);
        });

        it('unassigns an agent from cluster and clears alphaId if matching', () => {
            const store = use_workspace_store.getState();
            store.unassignAgentFromCluster('1', 'cl-command');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].collaborators).toEqual(['2']);
            expect(state.clusters[0].alphaId).toBeUndefined(); // '1' was the alphaId
        });

        it('sets alpha node explicitly', () => {
            const store = use_workspace_store.getState();
            store.setAlphaNode('cl-command', '2');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].alphaId).toBe('2');
        });
    });

    describe('Cluster Properties', () => {
        it('updates department', () => {
            const store = use_workspace_store.getState();
            store.updateClusterDepartment('cl-command', 'Sales');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].department).toBe('Sales');
        });

        it('updates budget', () => {
            const store = use_workspace_store.getState();
            store.updateClusterBudget('cl-command', 500);

            const state = use_workspace_store.getState();
            expect(state.clusters[0].budget_usd).toBe(500);
        });
    });

    describe('Proposals', () => {
        it('debounces proposal generation on objective update', () => {
            const store = use_workspace_store.getState();
            store.updateClusterObjective('cl-command', 'New objective here');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].objective).toBe('New objective here');

            expect(proposal_service.generateProposal).not.toHaveBeenCalled();

            vi.advanceTimersByTime(1100);

            expect(proposal_service.generateProposal).toHaveBeenCalled();
        });

        it('generates proposal directly and merges into active', () => {
            vi.mocked(proposal_service.generateProposal).mockReturnValue({
                clusterId: 'cl-command',
                reasoning: 'AI generated reason',
                changes: [],
                timestamp: 1000
            });

            const store = use_workspace_store.getState();
            store.generateProposal('cl-command');

            const state = use_workspace_store.getState();
            expect(state.activeProposals['cl-command'].reasoning).toBe('AI generated reason');
        });

        it('applies proposal completely and clears it', () => {
            use_workspace_store.setState({ activeProposals: { 'cl-command': { clusterId: 'cl-command', reasoning: 'test', changes: [], timestamp: 1 } } });
            
            const store = use_workspace_store.getState();
            store.applyProposal('cl-command');

            const state = use_workspace_store.getState();
            expect(state.activeProposals['cl-command']).toBeUndefined();
        });

        it('dismisses proposal and clears it', () => {
            use_workspace_store.setState({ activeProposals: { 'cl-command': { clusterId: 'cl-command', reasoning: 'test', changes: [], timestamp: 1 } } });
            
            const store = use_workspace_store.getState();
            store.dismissProposal('cl-command');

            const state = use_workspace_store.getState();
            expect(state.activeProposals['cl-command']).toBeUndefined();
        });
        
        it('ignores generating proposal for missing cluster', () => {
            const store = use_workspace_store.getState();
            store.generateProposal('fake-id');
            expect(proposal_service.generateProposal).not.toHaveBeenCalled();
        });

        it('ignores applying missing proposals', () => {
            const store = use_workspace_store.getState();
            store.applyProposal('cl-command');
            expect(Object.keys(use_workspace_store.getState().activeProposals).length).toBe(0);
        });
    });

    describe('Branching & Handoffs', () => {
        it('adds a branch', () => {
            const store = use_workspace_store.getState();
            store.addBranch('cl-command', { agent_id: '1', description: 'test', targetPath: '/dev' });

            const state = use_workspace_store.getState();
            expect(state.clusters[0].pendingTasks).toHaveLength(1);
            expect(state.clusters[0].pendingTasks[0].description).toBe('test');
            expect(state.clusters[0].pendingTasks[0].status).toBe('pending');
        });

        it('approves a branch', () => {
            use_workspace_store.setState({
                clusters: [{
                    id: 'cl-command',
                    pendingTasks: [{ id: 'b1', agent_id: '1', description: 'test', targetPath: '/dev', status: 'pending', timestamp: 1 }]
                } as any]
            });

            const store = use_workspace_store.getState();
            store.approveBranch('cl-command', 'b1');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].pendingTasks[0].status).toBe('completed');
        });

        it('rejects a branch', () => {
             use_workspace_store.setState({
                clusters: [{
                    id: 'cl-command',
                    pendingTasks: [{ id: 'b2', agent_id: '1', description: 'test', targetPath: '/dev', status: 'pending', timestamp: 1 }]
                } as any]
            });

            const store = use_workspace_store.getState();
            store.rejectBranch('cl-command', 'b2');

            const state = use_workspace_store.getState();
            expect(state.clusters[0].pendingTasks[0].status).toBe('rejected');
        });

        it('receives a handoff', () => {
             const store = use_workspace_store.getState();
             store.receiveHandoff('cl-source', 'cl-command', 'Passing the torch');
 
             const state = use_workspace_store.getState();
             expect(state.clusters[0].pendingTasks[0].description).toContain('[HANDOFF FROM cl-source]');
        });
    });

    describe('Path Calculations', () => {
        it('resolves correct cluster path for members', () => {
            const store = use_workspace_store.getState();
            const path = store.getAgentPath('2');
            expect(path).toBe('/workspaces/strategic-command'); // defined in mock
        });

        it('resolves fallback silo path for wandering agents', () => {
            const store = use_workspace_store.getState();
            const path = store.getAgentPath('999');
            expect(path).toBe('/workspaces/agent-silo-999');
        });
    });
});

