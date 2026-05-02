/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Sandboxed execution and project structure manager. 
 * Orchestrates the hierarchy of files, artifacts, and execution contexts for distributed agent swarms.
 * 
 * ### 🧬 Logic Flow (Mermaid)
 * ```mermaid
 * stateDiagram-v2
 *     [*] --> Cluster:Inactive: initialize
 *     Cluster:Inactive --> Cluster:Active: toggle_cluster_active()
 *     Cluster:Active --> Cluster:Inactive: toggle_cluster_active()
 *     
 *     state "Mission Cluster" as MC {
 *         [*] --> Branch:Pending: add_branch()
 *         Branch:Pending --> Branch:Completed: approve_branch()
 *         Branch:Pending --> Branch:Rejected: reject_branch()
 *         
 *         [*] --> Proposal:Idle
 *         Proposal:Idle --> Proposal:Pending: update_cluster_objective() [Debounced]
 *         Proposal:Pending --> Proposal:Idle: apply_proposal() / dismiss_proposal()
 *     }
 * ```
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Filesystem mount collision, recursive project structure overflow, or artifact persistence lag.
 * - **Telemetry Link**: Search for `[WorkspaceStore]` or `PROJECT_SYNC` in service logs.
 */


/**
 * Workspace_Store
 * Managed state for collaborative Mission Clusters and Task Branching.
 * Refactored for strict snake_case compliance for backend parity.
 * 
 * DESIGN PATTERN: Mission-Based Clusters
 * - Replaces individual agent silos with shared paths based on active missions.
 * - Implements a "Task Branch" workflow where changes can be reviewed/approved by Alpha nodes.
 */
import { create } from 'zustand';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { persist } from 'zustand/middleware';
import { get_settings } from './settings_store';
import { proposal_service } from '../services/proposal_service';
import { log_error } from '../services/system_utils';

export interface Mission_Cluster {
    /** Unique identifier for the mission cluster (e.g. 'cl-command'). */
    id: string;
    /** Human-readable name of the workspace sector. */
    name: string;
    /** The organizational department governing this cluster's AI nodes. */
    department: 'Executive' | 'Engineering' | 'Product' | 'Sales' | 'Operations' | 'Quality Assurance' | 'Design' | 'Research' | 'Support' | 'Marketing' | 'Intelligence' | 'Finance' | 'Growth' | 'Success';
    path: string;
    collaborators: string[]; // Agent IDs
    alpha_id?: string; // The leader of the cluster
    objective?: string; // High-level mission objective
    theme: 'cyan' | 'zinc' | 'amber' | 'blue';
    pending_tasks: Task_Branch[];
    is_active?: boolean;
    budget_usd?: number;
    cost_usd?: number;
    analysis_enabled?: boolean;
}

export interface Swarm_Proposal {
    cluster_id: string;
    reasoning: string;
    changes: {
        agent_id: string;
        proposed_role?: string;
        proposed_model?: string;
        added_skills?: string[];
        added_workflows?: string[];
    }[];
    timestamp: number;
}

export interface Task_Branch {
    id: string;
    agent_id: string;
    description: string;
    target_path: string;
    status: 'pending' | 'merging' | 'completed' | 'rejected';
    timestamp: number;
}

interface Workspace_State {
    clusters: Mission_Cluster[];
    active_proposals: Record<string, Swarm_Proposal>; // cluster_id -> proposal

    // Actions
    load_quotas: () => Promise<void>;
    create_cluster: (mission: Partial<Mission_Cluster>) => void;
    assign_agent_to_cluster: (agent_id: string, cluster_id: string) => void;
    unassign_agent_from_cluster: (agent_id: string, cluster_id: string) => void;
    update_cluster_objective: (cluster_id: string, objective: string) => void;
    update_cluster_department: (cluster_id: string, department: Mission_Cluster['department']) => void;
    update_cluster_budget: (cluster_id: string, budget: number) => void;
    generate_proposal: (cluster_id: string) => void;
    apply_proposal: (cluster_id: string) => void;
    dismiss_proposal: (cluster_id: string) => void;
    set_alpha_node: (cluster_id: string, agent_id: string) => void;
    delete_cluster: (cluster_id: string) => void;
    toggle_cluster_active: (cluster_id: string) => void;
    toggle_mission_analysis: (cluster_id: string) => void;
    add_branch: (cluster_id: string, branch: Omit<Task_Branch, 'id' | 'status' | 'timestamp'>) => void;
    approve_branch: (cluster_id: string, branch_id: string) => void;
    reject_branch: (cluster_id: string, branch_id: string) => void;
    receive_handoff: (source_cluster_id: string, target_cluster_id: string, description: string) => void;

    // Internal path calculation
    get_agent_path: (agent_id: string) => string;
}

const DEFAULT_CLUSTERS: Mission_Cluster[] = [
    {
        id: 'cl-command',
        name: 'Strategic Command',
        department: 'Executive',
        path: '/workspaces/strategic-command',
        collaborators: ['1', '2'],
        alpha_id: '1',
        objective: 'Global swarm oversight and strategic mission planning.',
        theme: 'blue',
        pending_tasks: [],
        is_active: true
    },
    {
        id: 'cl-chain-a',
        name: 'Strategic Ops (Chain A)',
        department: 'Operations',
        path: '/workspaces/strategic-ops',
        collaborators: ['3', '4', '5', '6'],
        alpha_id: '3',
        objective: 'Optimize swarm coordination and strategic resource allocation.',
        theme: 'cyan',
        pending_tasks: [],
        is_active: false
    },
    {
        id: 'cl-chain-b',
        name: 'Core Intelligence (Chain B)',
        department: 'Engineering',
        path: '/workspaces/core-intelligence',
        collaborators: ['7', '8', '9', '10'],
        alpha_id: '7',
        objective: 'Enhance neural processing efficiency and knowledge synthesis.',
        theme: 'zinc',
        pending_tasks: []
    },
    {
        id: 'cl-chain-c',
        name: 'Applied Growth (Chain C)',
        department: 'Product',
        path: '/workspaces/applied-growth',
        collaborators: ['11', '12', '13', '14'],
        alpha_id: '11',
        objective: 'Iterate on user-facing features and scale operational impact.',
        theme: 'amber',
        pending_tasks: []
    }
];

let proposal_timeout: ReturnType<typeof setTimeout> | undefined;

export const use_workspace_store = create<Workspace_State>()(
    persist(
        (set, get) => ({
            clusters: DEFAULT_CLUSTERS,
            active_proposals: {},

            // Actions
            load_quotas: async (): Promise<void> => {
                try {
                    const { quotas } = await tadpole_os_service.get_mission_quotas();
                    set((state) => ({
                        clusters: (state.clusters || []).map(cluster => {
                            const q = (quotas || []).find(q => q.entity_id === cluster.id);
                            return q ? {
                                ...cluster,
                                budget_usd: q.budget_usd,
                                cost_usd: q.used_usd
                            } : cluster;
                        })
                    }));
                } catch (error) {
                    log_error('WorkspaceStore', 'Quota Retrieval Failed', error);
                }
            },

            /**
             * create_cluster
             * Creates a new Mission Cluster and initializes its budget quota.
             */
            create_cluster: async (mission: Partial<Mission_Cluster>): Promise<void> => {
                const settings = get_settings();
                const clusters = get().clusters;

                if (clusters.length >= settings.max_clusters) {
                    log_error('WorkspaceStore', `Cluster limit reached (${settings.max_clusters}). Cannot create new workspace.`, null, 'warning');
                    return;
                }

                const new_cluster_id = (typeof crypto !== 'undefined' && crypto.randomUUID) 
                    ? `cl-${crypto.randomUUID().split('-')[0]}` 
                    : `cl-${Math.random().toString(36).substring(2, 9)}`;
                const new_cluster: Mission_Cluster = {
                    ...mission,
                    id: new_cluster_id,
                    name: mission.name || 'New Cluster',
                    department: mission.department || 'Engineering',
                    path: mission.path || `/workspaces/${Date.now()}`,
                    collaborators: mission.collaborators || [],
                    theme: mission.theme || 'blue',
                    pending_tasks: []
                };

                set(state => ({
                    clusters: [...state.clusters, new_cluster]
                }));

                try {
                    await tadpole_os_service.update_mission_quota(new_cluster_id, new_cluster.budget_usd || 0);
                } catch (error) {
                    log_error('WorkspaceStore', 'Cluster Quota Sync Failed', error);
                }
            },

            assign_agent_to_cluster: (agent_id: string, cluster_id: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? { ...c, collaborators: [...new Set([...(c.collaborators || []), agent_id])] } : c
                    )
                }));
            },

            unassign_agent_from_cluster: (agent_id: string, cluster_id: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? {
                            ...c,
                            collaborators: (c.collaborators || []).filter(id => id !== agent_id),
                            alpha_id: c.alpha_id === agent_id ? undefined : c.alpha_id
                        } : c
                    )
                }));
            },

            update_cluster_objective: (cluster_id: string, objective: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? { ...c, objective } : c
                    )
                }));

                if (proposal_timeout) clearTimeout(proposal_timeout);
                proposal_timeout = setTimeout(() => {
                    get().generate_proposal(cluster_id);
                }, 1000);
            },

            update_cluster_department: (cluster_id: string, department: Mission_Cluster['department']): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? { ...c, department } : c
                    )
                }));
            },

            update_cluster_budget: async (cluster_id: string, budget: number): Promise<void> => {
                set((state) => ({
                    clusters: (state.clusters || []).map((c) =>
                        c.id === cluster_id ? { ...c, budget_usd: budget } : c
                    ),
                }));

                try {
                    await tadpole_os_service.update_mission_quota(cluster_id, budget);
                } catch (error) {
                    log_error('WorkspaceStore', 'Quota Update Failed', error);
                }
            },

            generate_proposal: (cluster_id: string): void => {
                const cluster = get().clusters.find(c => c.id === cluster_id);
                if (!cluster) return;
    
                const proposal = proposal_service.generate_proposal(cluster);
                if (!proposal) return;
    
                set(state => ({
                    active_proposals: {
                        ...state.active_proposals,
                        [cluster_id]: proposal
                    }
                }));
            },

            apply_proposal: (cluster_id: string): void => {
                const proposal = get().active_proposals[cluster_id];
                if (!proposal) return;

                set(state => {
                    const next_proposals = { ...state.active_proposals };
                    delete next_proposals[cluster_id];
                    return { active_proposals: next_proposals };
                });
            },

            dismiss_proposal: (cluster_id: string): void => {
                set(state => {
                    const next_proposals = { ...state.active_proposals };
                    delete next_proposals[cluster_id];
                    return { active_proposals: next_proposals };
                });
            },

            set_alpha_node: (cluster_id: string, agent_id: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? { ...c, alpha_id: agent_id } : c
                    )
                }));
            },

            delete_cluster: (cluster_id: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).filter(c => c.id !== cluster_id)
                }));
            },

            toggle_cluster_active: (cluster_id: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c => ({
                        ...c,
                        is_active: c.id === cluster_id ? !c.is_active : false
                    }))
                }));
            },

            toggle_mission_analysis: (cluster_id: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? { ...c, analysis_enabled: !c.analysis_enabled } : c
                    )
                }));
            },

            add_branch: (cluster_id: string, branch: Omit<Task_Branch, 'id' | 'status' | 'timestamp'>): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? {
                            ...c,
                            pending_tasks: [...(c.pending_tasks || []), {
                                ...branch,
                                id: (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `br-${Date.now()}`,
                                status: 'pending',
                                timestamp: Date.now()
                            }]
                        } : c
                    )
                }));
            },

            approve_branch: (cluster_id: string, branch_id: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? {
                            ...c,
                            pending_tasks: (c.pending_tasks || []).map(t => t.id === branch_id ? { ...t, status: 'completed' } : t)
                        } : c
                    )
                }));
            },

            reject_branch: (cluster_id: string, branch_id: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === cluster_id ? {
                            ...c,
                            pending_tasks: (c.pending_tasks || []).map(t => t.id === branch_id ? { ...t, status: 'rejected' } : t)
                        } : c
                    )
                }));
            },

            receive_handoff: (source_cluster_id: string, target_cluster_id: string, description: string): void => {
                set(state => ({
                    clusters: (state.clusters || []).map(c =>
                        c.id === target_cluster_id ? {
                            ...c,
                            pending_tasks: [...(c.pending_tasks || []), {
                                id: `ho-${Date.now()}`,
                                agent_id: 'System (Handoff)',
                                description: `[HANDOFF FROM ${source_cluster_id}] ${description}`,
                                target_path: c.path,
                                status: 'pending',
                                timestamp: Date.now()
                            }]
                        } : c
                    )
                }));
            },

            /**
             * get_agent_path
             * Resolves the filesystem-like path for an agent based on its cluster membership.
             */
            get_agent_path: (agent_id: string): string => {
                const clusters = get().clusters || [];
                const cluster = clusters.find(c => (c.collaborators || []).includes(agent_id));
                return cluster ? cluster.path : `/workspaces/agent-silo-${agent_id}`;
            }
        }),
        {
            name: 'tadpole-workspaces-v4',
            version: 1,
            migrate: (persisted_state: unknown, version: number) => {
                const state = persisted_state as Record<string, unknown>;
                if (version === 0 || version === 1) {
                    const clusters = (state.clusters as Mission_Cluster[]) || [];
                    
                    // Repair any clusters missing collaborators (Fix for Core Fault)
                    const repaired_clusters = (clusters || []).map(c => ({
                        ...c,
                        collaborators: c.collaborators || [],
                        pending_tasks: c.pending_tasks || []
                    }));

                    const has_command = repaired_clusters.some(c => c.id === 'cl-command');
                    if (!has_command) {
                        return {
                            ...state,
                            clusters: [
                                {
                                    id: 'cl-command',
                                    name: 'Strategic Command',
                                    department: 'Executive',
                                    path: '/workspaces/strategic-command',
                                    collaborators: ['1', '2'],
                                    alpha_id: '1',
                                    objective: 'Global swarm oversight and strategic mission planning.',
                                    theme: 'blue',
                                    pending_tasks: [],
                                    is_active: true
                                },
                                ...repaired_clusters
                            ]
                        };
                    }
                    return { ...state, clusters: repaired_clusters };
                }
                return persisted_state;
            }
        }
    )
);


// Metadata: [workspace_store]

// Metadata: [workspace_store]
