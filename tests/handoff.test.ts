/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[handoff_test]` in observability traces.
 */

import { describe, it, expect } from 'vitest';
import { use_workspace_store } from '../src/stores/workspace_store';

describe('Mission Handoffs', () => {
    it('successfully receives a handoff from another cluster', () => {
        const store = use_workspace_store.getState();

        // Setup mock cluster
        const cluster_id = 'cl-test';
        store.clusters = [{
            id: cluster_id,
            name: 'Test Cluster',
            objective: 'Testing',
            path: './test',
            collaborators: [],
            pending_tasks: [],
            is_active: false,
            department: 'Engineering',
            theme: 'cyan'
        }];

        // Execute handoff
        const source_id = 'cl-alpha';
        const description = 'Analyze logs for anomalies';
        store.receive_handoff(source_id, cluster_id, description);

        // Verify result
        const updatedStore = use_workspace_store.getState();
        const cluster = updatedStore.clusters.find(c => c.id === cluster_id);

        expect(cluster?.pending_tasks.length).toBe(1);
        expect(cluster?.pending_tasks[0].agent_id).toBe('System (Handoff)');
        expect(cluster?.pending_tasks[0].description).toContain(source_id);
        expect(cluster?.pending_tasks[0].description).toContain(description);
        expect(cluster?.pending_tasks[0].id).toMatch(/^ho-/);
    });
});

// Metadata: [handoff_test]
