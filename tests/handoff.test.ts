import { describe, it, expect } from 'vitest';
import { useWorkspaceStore } from '../src/stores/workspaceStore';

describe('Mission Handoffs', () => {
    it('successfully receives a handoff from another cluster', () => {
        const store = useWorkspaceStore.getState();

        // Setup mock cluster
        const clusterId = 'cl-test';
        store.clusters = [{
            id: clusterId,
            name: 'Test Cluster',
            objective: 'Testing',
            path: './test',
            collaborators: [],
            pendingTasks: [],
            isActive: false,
            department: 'Engineering',
            theme: 'cyan'
        }];

        // Execute handoff
        const sourceId = 'cl-alpha';
        const description = 'Analyze logs for anomalies';
        store.receiveHandoff(sourceId, clusterId, description);

        // Verify result
        const updatedStore = useWorkspaceStore.getState();
        const cluster = updatedStore.clusters.find(c => c.id === clusterId);

        expect(cluster?.pendingTasks.length).toBe(1);
        expect(cluster?.pendingTasks[0].agentId).toBe('System (Handoff)');
        expect(cluster?.pendingTasks[0].description).toContain(sourceId);
        expect(cluster?.pendingTasks[0].description).toContain(description);
        expect(cluster?.pendingTasks[0].id).toMatch(/^ho-/);
    });
});
