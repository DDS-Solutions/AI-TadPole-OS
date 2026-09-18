/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / workspace_service.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { workspace_service, set_workspace_store } from './workspace_service';
import { use_workspace_store } from '../stores/workspace_store';
import { system_api_service } from './system_api_service';
import { proposal_service } from './proposal_service';

vi.mock('./system_api_service', () => ({
    system_api_service: {
        oversight: {
            get_mission_quotas: vi.fn(),
            get_governance_settings: vi.fn(),
            update_mission_quota: vi.fn(),
            update_governance_settings: vi.fn()
        },
        workspace: {
            get_workspaces_status: vi.fn()
        }
    }
}));

vi.mock('./proposal_service', () => ({
    proposal_service: {
        generate_proposal: vi.fn()
    }
}));

describe('workspace_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        set_workspace_store(use_workspace_store);
        use_workspace_store.setState({
            clusters: [
                {
                    id: 'cl-1',
                    name: 'Test Cluster',
                    department: 'Engineering',
                    path: '/workspaces/cl-1',
                    collaborators: ['agent-1'],
                    budget_usd: 100,
                    cost_usd: 20,
                    privacy_mode: false
                } as any
            ],
            sync_status: {},
            active_proposals: {}
        });
    });

    describe('sync_quotas', () => {
        it('syncs mission quotas and governance privacy settings into clusters', async () => {
            vi.mocked(system_api_service.oversight.get_mission_quotas).mockResolvedValueOnce({
                quotas: [{ entity_id: 'cl-1', budget_usd: 500, used_usd: 150 }]
            } as any);
            vi.mocked(system_api_service.oversight.get_governance_settings).mockResolvedValueOnce({
                cluster_privacy_policies: { 'cl-1': true }
            } as any);

            await workspace_service.sync_quotas();

            const cluster = use_workspace_store.getState().clusters.find(c => c.id === 'cl-1');
            expect(cluster?.budget_usd).toBe(500);
            expect(cluster?.cost_usd).toBe(150);
            expect(cluster?.privacy_mode).toBe(true);
        });
    });

    describe('create_mission_cluster', () => {
        it('creates a new cluster and synchronizes quota', async () => {
            vi.mocked(system_api_service.oversight.update_mission_quota).mockResolvedValueOnce(undefined as any);

            await workspace_service.create_mission_cluster({
                name: 'Alpha Cluster',
                department: 'Research',
                budget_usd: 250
            });

            const clusters = use_workspace_store.getState().clusters;
            expect(clusters).toHaveLength(2);
            const created = clusters.find(c => c.name === 'Alpha Cluster');
            expect(created).toBeDefined();
            expect(system_api_service.oversight.update_mission_quota).toHaveBeenCalledWith(created?.id, 250);
        });

        it('reverts cluster addition if quota sync fails', async () => {
            vi.mocked(system_api_service.oversight.update_mission_quota).mockRejectedValueOnce(new Error('Quota error'));

            await workspace_service.create_mission_cluster({
                name: 'Failing Cluster'
            });

            const clusters = use_workspace_store.getState().clusters;
            expect(clusters).toHaveLength(1);
            expect(clusters.find(c => c.name === 'Failing Cluster')).toBeUndefined();
        });
    });

    describe('update_budget', () => {
        it('updates cluster budget optimistically and syncs with backend', async () => {
            vi.mocked(system_api_service.oversight.update_mission_quota).mockResolvedValueOnce(undefined as any);

            await workspace_service.update_budget('cl-1', 1000);

            const cluster = use_workspace_store.getState().clusters.find(c => c.id === 'cl-1');
            expect(cluster?.budget_usd).toBe(1000);
            expect(system_api_service.oversight.update_mission_quota).toHaveBeenCalledWith('cl-1', 1000);
        });

        it('reverts budget update when backend fails', async () => {
            vi.mocked(system_api_service.oversight.update_mission_quota).mockRejectedValueOnce(new Error('Quota fail'));

            await workspace_service.update_budget('cl-1', 9999);

            const cluster = use_workspace_store.getState().clusters.find(c => c.id === 'cl-1');
            expect(cluster?.budget_usd).toBe(100);
        });
    });

    describe('update_cluster_privacy', () => {
        it('updates privacy mode and syncs with governance backend', async () => {
            vi.mocked(system_api_service.oversight.update_governance_settings).mockResolvedValueOnce(undefined as any);

            await workspace_service.update_cluster_privacy('cl-1', true);

            const cluster = use_workspace_store.getState().clusters.find(c => c.id === 'cl-1');
            expect(cluster?.privacy_mode).toBe(true);
            expect(system_api_service.oversight.update_governance_settings).toHaveBeenCalledWith({
                cluster_privacy_policies: { 'cl-1': true }
            });
        });

        it('reverts privacy update on backend failure', async () => {
            vi.mocked(system_api_service.oversight.update_governance_settings).mockRejectedValueOnce(new Error('Gov error'));

            await workspace_service.update_cluster_privacy('cl-1', true);

            const cluster = use_workspace_store.getState().clusters.find(c => c.id === 'cl-1');
            expect(cluster?.privacy_mode).toBe(false);
        });
    });

    describe('refresh_telemetry', () => {
        it('updates sync_status map in store', async () => {
            vi.mocked(system_api_service.workspace.get_workspaces_status).mockResolvedValueOnce([
                {
                    source_uri: 'file:///repo',
                    status: 'synced',
                    last_sync_at: '2026-09-09T00:00:00Z',
                    file_count: 50,
                    total_bytes: 102400
                } as any
            ]);

            await workspace_service.refresh_telemetry();

            const status = use_workspace_store.getState().sync_status;
            expect(status['file:///repo']).toBeDefined();
            expect(status['file:///repo'].file_count).toBe(50);
        });
    });

    describe('request_proposal', () => {
        it('executes immediately when immediate=true', async () => {
            const mock_proposal = { cluster_id: 'cl-1', reasoning: 'test', changes: [], timestamp: 123 };
            vi.mocked(proposal_service.generate_proposal).mockReturnValueOnce(mock_proposal as any);

            const result = await workspace_service.request_proposal('cl-1', true);

            expect(result).toEqual(mock_proposal);
            expect(use_workspace_store.getState().active_proposals['cl-1']).toEqual(mock_proposal);
        });

        it('resolves all queued promises when debounced requests are made', async () => {
            const mock_proposal = { cluster_id: 'cl-1', reasoning: 'debounced', changes: [], timestamp: 456 };
            vi.mocked(proposal_service.generate_proposal).mockReturnValue(mock_proposal as any);

            // Trigger two calls without immediate flag
            const p1 = workspace_service.request_proposal('cl-1', false);
            const p2 = workspace_service.request_proposal('cl-1', false);

            // Trigger immediate to flush pending debounces
            const pImmediate = workspace_service.request_proposal('cl-1', true);

            const [r1, r2, rImm] = await Promise.all([p1, p2, pImmediate]);
            expect(r1).toEqual(mock_proposal);
            expect(r2).toEqual(mock_proposal);
            expect(rImm).toEqual(mock_proposal);
        });
    });
});
