/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / workspace_service
 * - **Primary Entrypoints**: `set_workspace_store`, `workspace_service`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { system_api_service } from './system_api_service';
import { proposal_service, type Swarm_Proposal } from './proposal_service';
import { log_error } from './system_utils';
import { get_settings } from '../stores/settings_store';
import type { Mission_Cluster } from '../stores/workspace_store';

let workspace_store: typeof import('../stores/workspace_store').use_workspace_store | null = null;
export const set_workspace_store = (store: typeof import('../stores/workspace_store').use_workspace_store) => {
    workspace_store = store;
};

class Workspace_Service {
    private proposal_timeout: ReturnType<typeof setTimeout> | undefined;

    /**
     * Helper to safely access the workspace store and prevent null pointer exceptions.
     */
    private get_store() {
        if (!workspace_store) {
            throw new Error('[Workspace_Service] Store not initialized. Call set_workspace_store() first.');
        }
        return workspace_store;
    }

    /**
     * Synchronizes mission quotas and cluster privacy policies from the backend to the store.
     */
    public async sync_quotas(): Promise<void> {
        try {
            const [quotasRes, govRes] = await Promise.allSettled([
                system_api_service.oversight.get_mission_quotas(),
                system_api_service.oversight.get_governance_settings()
            ]);

            const quotas = quotasRes.status === 'fulfilled' ? (quotasRes.value as { quotas?: Array<{ entity_id: string; budget_usd: number; used_usd: number }> })?.quotas || [] : [];
            const clusterPolicies = govRes.status === 'fulfilled' ? (govRes.value as { cluster_privacy_policies?: Record<string, boolean> })?.cluster_privacy_policies || {} : {};

            const store = this.get_store();
            const state = store.getState();
            
            // Optimization: O(1) lookup map
            const quotaMap = new Map((quotas || []).map(q => [q.entity_id, q]));

            const updated_clusters = (state.clusters || []).map(cluster => {
                const q = quotaMap.get(cluster.id);
                const backendPrivacy = clusterPolicies[cluster.id];
                return {
                    ...cluster,
                    budget_usd: q ? q.budget_usd : cluster.budget_usd,
                    cost_usd: q ? q.used_usd : cluster.cost_usd,
                    privacy_mode: backendPrivacy !== undefined ? backendPrivacy : (cluster.privacy_mode ?? false)
                };
            });

            store.setState({ clusters: updated_clusters });
        } catch (error) {
            log_error('WorkspaceService', 'Quota & Governance Retrieval Failed', error);
        }
    }

    /**
     * Initializes a new Mission Cluster with proper ID and Quota.
     */
    public async create_mission_cluster(mission: Partial<Mission_Cluster>): Promise<void> {
        const settings = get_settings();
        const store = this.get_store();
        const { clusters } = store.getState();

        if (clusters.length >= settings.max_clusters) {
            log_error('WorkspaceService', `Cluster limit reached (${settings.max_clusters}).`, null, 'warning');
            return;
        }

        // Full UUID to avoid collision risk
        const new_cluster_id = (typeof crypto !== 'undefined' && crypto.randomUUID) 
            ? `cl-${crypto.randomUUID()}` 
            : `cl-${Date.now().toString(36)}-${(typeof performance !== 'undefined' ? Math.floor(performance.now() * 1000) : 0).toString(36)}`;

        const new_cluster: Mission_Cluster = {
            id: new_cluster_id,
            name: mission.name || 'New Cluster',
            department: mission.department || 'Engineering',
            path: mission.path || `/workspaces/${Date.now()}`,
            collaborators: mission.collaborators || [],
            theme: mission.theme || 'blue',
            pending_tasks: [],
            ...mission
        };

        store.setState(state => ({
            clusters: [...state.clusters, new_cluster]
        }));

        try {
            await system_api_service.oversight.update_mission_quota(new_cluster_id, new_cluster.budget_usd || 0);
        } catch (error) {
            log_error('WorkspaceService', 'Cluster Quota Sync Failed', error);
        }
    }

    /**
     * Updates cluster budget and synchronizes with backend, rolling back local state on error.
     */
    public async update_budget(cluster_id: string, budget: number): Promise<void> {
        const store = this.get_store();
        const previous_clusters = store.getState().clusters;

        store.setState(state => ({
            clusters: state.clusters.map(c => c.id === cluster_id ? { ...c, budget_usd: budget } : c)
        }));

        try {
            await system_api_service.oversight.update_mission_quota(cluster_id, budget);
        } catch (error) {
            log_error('WorkspaceService', 'Quota Update Failed — Reverting optimistic UI state', error);
            store.setState({ clusters: previous_clusters });
        }
    }

    /**
     * Toggles cluster privacy mode and synchronizes with governance backend.
     */
    public async update_cluster_privacy(cluster_id: string, privacy_mode: boolean): Promise<void> {
        const store = this.get_store();
        const previous_clusters = store.getState().clusters;

        store.setState(state => ({
            clusters: state.clusters.map(c => c.id === cluster_id ? { ...c, privacy_mode } : c)
        }));

        try {
            await system_api_service.oversight.update_governance_settings({
                cluster_privacy_policies: { [cluster_id]: privacy_mode }
            });
        } catch (error) {
            log_error('WorkspaceService', 'Failed to sync cluster privacy settings — Reverting state', error);
            store.setState({ clusters: previous_clusters });
        }
    }

    /**
     * Refreshes all workspace sync telemetry.
     */
    public async refresh_telemetry(): Promise<void> {
        try {
            const status_data = await system_api_service.workspace.get_workspaces_status();
            const store = this.get_store();
            const next_status: Record<string, {
                status: string;
                last_sync_at: string | null;
                file_count: number;
                total_bytes: number;
                detected_environments?: string[];
                mounted_okf_nodes?: Array<{ id: string; title: string; concept_type: string; security_tier?: string; parent_id?: string }>;
                okf_validation?: {
                    status: 'nominal' | 'warning' | 'critical';
                    message?: string;
                };
            }> = {};
            
            status_data.forEach(item => {
                next_status[item.source_uri] = {
                    status: item.status,
                    last_sync_at: item.last_sync_at,
                    file_count: item.file_count,
                    total_bytes: item.total_bytes,
                    detected_environments: item.detected_environments,
                    mounted_okf_nodes: item.mounted_okf_nodes,
                    okf_validation: item.okf_validation
                };
            });

            store.setState({ sync_status: next_status });
        } catch (error) {
            log_error('WorkspaceService', 'Sync Status Refresh Failed', error);
        }
    }



    /**
     * Triggers debounced or immediate swarm proposal generation.
     */
    public request_proposal(cluster_id: string, immediate = false): Promise<Swarm_Proposal | null> {
        if (this.proposal_timeout) {
            clearTimeout(this.proposal_timeout);
            this.proposal_timeout = undefined;
        }

        const execute = (): Swarm_Proposal | null => {
            const store = this.get_store();
            const state = store.getState();
            const cluster = state.clusters.find(c => c.id === cluster_id);
            if (!cluster) return null;

            const proposal = proposal_service.generate_proposal(cluster);
            if (!proposal) return null;

            store.setState(s => ({
                active_proposals: {
                    ...s.active_proposals,
                    [cluster_id]: proposal
                }
            }));
            return proposal;
        };

        if (immediate) {
            return Promise.resolve(execute());
        }

        return new Promise((resolve) => {
            this.proposal_timeout = setTimeout(() => {
                resolve(execute());
            }, 1000);
        });
    }
}

export const workspace_service = new Workspace_Service();
