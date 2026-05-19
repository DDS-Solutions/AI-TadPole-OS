/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Services:Workspace**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[workspace_service]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Services:Workspace
 * 
 * ### AI Assist Note
 * **Workspace Orchestrator**: Hardens the state management by extracting side-effects and API interactions.
 * Manages the lifecycle of Mission Clusters, Quotas, and Sync Telemetry.
 */

import { tadpole_os_service } from './tadpoleos_service';
import { system_api_service } from './system_api_service';
import { proposal_service } from './proposal_service';
import { log_error } from './system_utils';
import { use_workspace_store } from '../stores/workspace_store';
import type { Mission_Cluster } from '../stores/workspace_store';
import { get_settings } from '../stores/settings_store';

class Workspace_Service {
    private proposal_timeout: ReturnType<typeof setTimeout> | undefined;

    /**
     * Synchronizes mission quotas from the backend to the store.
     */
    public async sync_quotas(): Promise<void> {
        try {
            const { quotas } = await tadpole_os_service.get_mission_quotas();
            const store = use_workspace_store.getState();
            
            const updated_clusters = (store.clusters || []).map(cluster => {
                const q = (quotas || []).find(q => q.entity_id === cluster.id);
                return q ? {
                    ...cluster,
                    budget_usd: q.budget_usd,
                    cost_usd: q.used_usd
                } : cluster;
            });

            use_workspace_store.setState({ clusters: updated_clusters });
        } catch (error) {
            log_error('WorkspaceService', 'Quota Retrieval Failed', error);
        }
    }

    /**
     * Initializes a new Mission Cluster with proper ID and Quota.
     */
    public async create_mission_cluster(mission: Partial<Mission_Cluster>): Promise<void> {
        const settings = get_settings();
        const { clusters } = use_workspace_store.getState();

        if (clusters.length >= settings.max_clusters) {
            log_error('WorkspaceService', `Cluster limit reached (${settings.max_clusters}).`, null, 'warning');
            return;
        }

        const new_cluster_id = (typeof crypto !== 'undefined' && crypto.randomUUID) 
            ? `cl-${crypto.randomUUID().split('-')[0]}` 
            : `cl-${Math.random().toString(36).substring(2, 9)}`;

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

        use_workspace_store.setState(state => ({
            clusters: [...state.clusters, new_cluster]
        }));

        try {
            await tadpole_os_service.update_mission_quota(new_cluster_id, new_cluster.budget_usd || 0);
        } catch (error) {
            log_error('WorkspaceService', 'Cluster Quota Sync Failed', error);
        }
    }

    /**
     * Updates cluster budget and synchronizes with backend.
     */
    public async update_budget(cluster_id: string, budget: number): Promise<void> {
        use_workspace_store.setState(state => ({
            clusters: state.clusters.map(c => c.id === cluster_id ? { ...c, budget_usd: budget } : c)
        }));

        try {
            await tadpole_os_service.update_mission_quota(cluster_id, budget);
        } catch (error) {
            log_error('WorkspaceService', 'Quota Update Failed', error);
        }
    }

    /**
     * Refreshes all workspace sync telemetry.
     */
    public async refresh_telemetry(): Promise<void> {
        try {
            const status_data = await system_api_service.get_workspaces_status();
            const next_status: Record<string, { status: string; last_sync_at: string | null; file_count: number; total_bytes: number }> = {};
            
            status_data.forEach(item => {
                next_status[item.source_uri] = {
                    status: item.status,
                    last_sync_at: item.last_sync_at,
                    file_count: item.file_count,
                    total_bytes: item.total_bytes
                };
            });

            use_workspace_store.setState({ sync_status: next_status });
        } catch (error) {
            log_error('WorkspaceService', 'Sync Status Refresh Failed', error);
        }
    }

    /**
     * Triggers a debounced swarm proposal generation.
     */
    public request_proposal(cluster_id: string): void {
        if (this.proposal_timeout) clearTimeout(this.proposal_timeout);
        
        this.proposal_timeout = setTimeout(() => {
            const store = use_workspace_store.getState();
            const cluster = store.clusters.find(c => c.id === cluster_id);
            if (!cluster) return;

            const proposal = proposal_service.generate_proposal(cluster);
            if (!proposal) return;

            use_workspace_store.setState(state => ({
                active_proposals: {
                    ...state.active_proposals,
                    [cluster_id]: proposal
                }
            }));
        }, 1000);
    }
}

export const workspace_service = new Workspace_Service();

// Metadata: [workspace_service]
