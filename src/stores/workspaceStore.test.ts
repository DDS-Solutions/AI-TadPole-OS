/**
 * @file workspaceStore.test.ts
 * @description Suite for Org-level workspace clustering and AI-driven proposals.
 * @module Stores/WorkspaceStore
 * @testedBehavior
 * - Governance: Enforcing cluster limits based on global user settings.
 * - AI Intelligence: Debounced generation of 'Next Step' proposals via ProposalService.
 * - Collaboration: Handoff logic and alpha-agent assignment within clusters.
 * - Routing: Resolution of agent-silo paths vs cluster-shared paths.
 * @aiContext
 * - Uses vitest fake timers to validate debounced proposal generation.
 * - Mocks ProposalService and settingsStore.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useWorkspaceStore } from './workspaceStore';
import * as settingsStore from './settingsStore';
import { ProposalService } from '../services/ProposalService';

// Mock Dependencies
vi.mock('./settingsStore', () => ({
    getSettings: vi.fn()
}));
vi.mock('../services/ProposalService', () => ({
    ProposalService: {
        generateProposal: vi.fn()
    }
}));

describe('useWorkspaceStore', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.useFakeTimers();
        useWorkspaceStore.setState({
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
            const store = useWorkspaceStore.getState();
            store.createCluster({ name: 'New Test Cluster' });

            const state = useWorkspaceStore.getState();
            expect(state.clusters).toHaveLength(2);
            expect(state.clusters[1].name).toBe('New Test Cluster');
            expect(state.clusters[1].department).toBe('Engineering'); // Default fallback
            expect(state.clusters[1].theme).toBe('blue');            // Default fallback
        });

        it('prevents creating a cluster if limit reached', () => {
            vi.mocked(settingsStore.getSettings).mockReturnValue({ maxClusters: 1 } as any);
            const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

            const store = useWorkspaceStore.getState();
            store.createCluster({ name: 'Blocked Cluster' });

            const state = useWorkspaceStore.getState();
            expect(state.clusters).toHaveLength(1); // Still 1 from beforeEach
            expect(consoleSpy).toHaveBeenCalledWith(expect.stringContaining('Cluster limit reached'));

            consoleSpy.mockRestore();
        });

        it('deletes a cluster', () => {
            const store = useWorkspaceStore.getState();
            store.deleteCluster('cl-command');

            const state = useWorkspaceStore.getState();
            expect(state.clusters).toHaveLength(0);
        });

        it('toggles cluster active state', () => {
            const store = useWorkspaceStore.getState();
            store.toggleClusterActive('cl-command'); // toggle off default true

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].isActive).toBe(false);

            store.toggleClusterActive('cl-command'); // toggle on
            expect(useWorkspaceStore.getState().clusters[0].isActive).toBe(true);
        });

        it('toggles mission analysis state', () => {
             const store = useWorkspaceStore.getState();
             store.toggleMissionAnalysis('cl-command');
 
             const state = useWorkspaceStore.getState();
             expect(state.clusters[0].analysisEnabled).toBe(true);
        });
    });

    describe('Agent Assignments', () => {
        it('assigns agent to cluster uniquely', () => {
            const store = useWorkspaceStore.getState();
            store.assignAgentToCluster('3', 'cl-command');
            store.assignAgentToCluster('3', 'cl-command'); // Duplicate

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].collaborators).toEqual(['1', '2', '3']);
        });

        it('unassigns an agent from cluster and clears alphaId if matching', () => {
            const store = useWorkspaceStore.getState();
            store.unassignAgentFromCluster('1', 'cl-command');

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].collaborators).toEqual(['2']);
            expect(state.clusters[0].alphaId).toBeUndefined(); // '1' was the alphaId
        });

        it('sets alpha node explicitly', () => {
            const store = useWorkspaceStore.getState();
            store.setAlphaNode('cl-command', '2');

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].alphaId).toBe('2');
        });
    });

    describe('Cluster Properties', () => {
        it('updates department', () => {
            const store = useWorkspaceStore.getState();
            store.updateClusterDepartment('cl-command', 'Sales');

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].department).toBe('Sales');
        });

        it('updates budget', () => {
            const store = useWorkspaceStore.getState();
            store.updateClusterBudget('cl-command', 500);

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].budgetUsd).toBe(500);
        });
    });

    describe('Proposals', () => {
        it('debounces proposal generation on objective update', () => {
            const store = useWorkspaceStore.getState();
            store.updateClusterObjective('cl-command', 'New objective here');

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].objective).toBe('New objective here');

            expect(ProposalService.generateProposal).not.toHaveBeenCalled();

            vi.advanceTimersByTime(1100);

            expect(ProposalService.generateProposal).toHaveBeenCalled();
        });

        it('generates proposal directly and merges into active', () => {
            vi.mocked(ProposalService.generateProposal).mockReturnValue({
                clusterId: 'cl-command',
                reasoning: 'AI generated reason',
                changes: [],
                timestamp: 1000
            });

            const store = useWorkspaceStore.getState();
            store.generateProposal('cl-command');

            const state = useWorkspaceStore.getState();
            expect(state.activeProposals['cl-command'].reasoning).toBe('AI generated reason');
        });

        it('applies proposal completely and clears it', () => {
            useWorkspaceStore.setState({ activeProposals: { 'cl-command': { clusterId: 'cl-command', reasoning: 'test', changes: [], timestamp: 1 } } });
            
            const store = useWorkspaceStore.getState();
            store.applyProposal('cl-command');

            const state = useWorkspaceStore.getState();
            expect(state.activeProposals['cl-command']).toBeUndefined();
        });

        it('dismisses proposal and clears it', () => {
            useWorkspaceStore.setState({ activeProposals: { 'cl-command': { clusterId: 'cl-command', reasoning: 'test', changes: [], timestamp: 1 } } });
            
            const store = useWorkspaceStore.getState();
            store.dismissProposal('cl-command');

            const state = useWorkspaceStore.getState();
            expect(state.activeProposals['cl-command']).toBeUndefined();
        });
        
        it('ignores generating proposal for missing cluster', () => {
            const store = useWorkspaceStore.getState();
            store.generateProposal('fake-id');
            expect(ProposalService.generateProposal).not.toHaveBeenCalled();
        });

        it('ignores applying missing proposals', () => {
            const store = useWorkspaceStore.getState();
            store.applyProposal('cl-command');
            expect(Object.keys(useWorkspaceStore.getState().activeProposals).length).toBe(0);
        });
    });

    describe('Branching & Handoffs', () => {
        it('adds a branch', () => {
            const store = useWorkspaceStore.getState();
            store.addBranch('cl-command', { agentId: '1', description: 'test', targetPath: '/dev' });

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].pendingTasks).toHaveLength(1);
            expect(state.clusters[0].pendingTasks[0].description).toBe('test');
            expect(state.clusters[0].pendingTasks[0].status).toBe('pending');
        });

        it('approves a branch', () => {
            useWorkspaceStore.setState({
                clusters: [{
                    id: 'cl-command',
                    pendingTasks: [{ id: 'b1', agentId: '1', description: 'test', targetPath: '/dev', status: 'pending', timestamp: 1 }]
                } as any]
            });

            const store = useWorkspaceStore.getState();
            store.approveBranch('cl-command', 'b1');

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].pendingTasks[0].status).toBe('completed');
        });

        it('rejects a branch', () => {
             useWorkspaceStore.setState({
                clusters: [{
                    id: 'cl-command',
                    pendingTasks: [{ id: 'b2', agentId: '1', description: 'test', targetPath: '/dev', status: 'pending', timestamp: 1 }]
                } as any]
            });

            const store = useWorkspaceStore.getState();
            store.rejectBranch('cl-command', 'b2');

            const state = useWorkspaceStore.getState();
            expect(state.clusters[0].pendingTasks[0].status).toBe('rejected');
        });

        it('receives a handoff', () => {
             const store = useWorkspaceStore.getState();
             store.receiveHandoff('cl-source', 'cl-command', 'Passing the torch');
 
             const state = useWorkspaceStore.getState();
             expect(state.clusters[0].pendingTasks[0].description).toContain('[HANDOFF FROM cl-source]');
        });
    });

    describe('Path Calculations', () => {
        it('resolves correct cluster path for members', () => {
            const store = useWorkspaceStore.getState();
            const path = store.getAgentPath('2');
            expect(path).toBe('/workspaces/strategic-command'); // defined in mock
        });

        it('resolves fallback silo path for wandering agents', () => {
            const store = useWorkspaceStore.getState();
            const path = store.getAgentPath('999');
            expect(path).toBe('/workspaces/agent-silo-999');
        });
    });
});
