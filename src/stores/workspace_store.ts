/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / workspace_store
 * - **Primary Entrypoints**: `use_workspace_store`, `Mission_Cluster`, `Team_Cluster_Preset`, `Swarm_Proposal`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { workspace_service } from '../services/workspace_service';

/**
 * Workspace Store State Diagram
 * ```mermaid
 * stateDiagram-v2
 *   [*] --> Idle
 *   Idle --> Active : select_workspace
 *   Active --> Mutating : update_cluster
 *   Mutating --> Active : sync_complete
 * ```
 */
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
    privacy_mode?: boolean;
    phase?: 'waiting_for_model' | 'streaming_reasoning' | 'tool_execution' | 'completed' | 'failed';
    token_burn_rate?: number;
    is_team?: boolean;
    team_badge?: string;
}

export interface Team_Cluster_Preset {
    id: string;
    name: string;
    description: string;
    department: Mission_Cluster['department'];
    theme: Mission_Cluster['theme'];
    default_budget_usd: number;
    badge_label: string;
    collaborators: string[]; // Agent IDs
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
    phase?: 'waiting_for_model' | 'streaming_reasoning' | 'tool_execution' | 'completed' | 'failed';
    permission_choice?: 'approve' | 'deny' | 'approve_once' | 'always_allow';
    timestamp: number;
}

const MAX_TASK_BRANCHES = 50;

export interface Workspace_State {
    clusters: Mission_Cluster[];
    team_presets: Team_Cluster_Preset[];
    active_proposals: Record<string, Swarm_Proposal>; // cluster_id -> proposal

    // Actions
    load_quotas: () => Promise<void>;
    create_cluster: (mission: Partial<Mission_Cluster>) => Promise<void>;
    assign_agent_to_cluster: (agent_id: string, cluster_id: string) => void;
    unassign_agent_from_cluster: (agent_id: string, cluster_id: string) => void;
    update_cluster_objective: (cluster_id: string, objective: string) => void;
    update_cluster_department: (cluster_id: string, department: Mission_Cluster['department']) => void;
    update_cluster_budget: (cluster_id: string, budget: number) => Promise<void>;
    add_team_preset: (preset: Team_Cluster_Preset) => void;
    update_team_preset: (id: string, updates: Partial<Team_Cluster_Preset>) => void;
    delete_team_preset: (id: string) => void;
    generate_proposal: (cluster_id: string, immediate?: boolean) => Promise<Swarm_Proposal | null>;
    apply_proposal: (cluster_id: string) => void;
    dismiss_proposal: (cluster_id: string) => void;
    set_alpha_node: (cluster_id: string, agent_id: string) => void;
    delete_cluster: (cluster_id: string) => void;
    toggle_cluster_active: (cluster_id: string) => void;
    toggle_mission_analysis: (cluster_id: string) => void;
    toggle_cluster_privacy: (cluster_id: string) => Promise<void>;
    add_branch: (cluster_id: string, branch: Omit<Task_Branch, 'id' | 'status' | 'timestamp'>) => void;
    approve_branch: (cluster_id: string, branch_id: string) => void;
    reject_branch: (cluster_id: string, branch_id: string) => void;
    receive_handoff: (source_cluster_id: string, target_cluster_id: string, description: string) => void;

    // Internal path calculation
    get_agent_path: (agent_id: string) => string;

    // Sync Telemetry
    sync_status: Record<string, {
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
    }>;
    refresh_sync_status: () => Promise<void>;
}

const DEFAULT_TEAM_PRESETS: Team_Cluster_Preset[] = [
    {
        id: 'preset-sec-audit',
        name: 'Security & Audit Cluster',
        description: 'Multi-role unit specialized in vulnerability scans, code reviews, and threat analysis.',
        department: 'Quality Assurance',
        theme: 'amber',
        default_budget_usd: 1500,
        badge_label: 'SEC-AUDIT',
        collaborators: ['12', '26']
    },
    {
        id: 'preset-eng-swarm',
        name: 'Full-Stack Engineering Swarm',
        description: 'High-velocity development cluster with architectural oversight and automated refactoring.',
        department: 'Engineering',
        theme: 'blue',
        default_budget_usd: 2500,
        badge_label: 'ENG-SWARM',
        collaborators: ['3', '8', '13']
    },
    {
        id: 'preset-research-pub',
        name: 'Research & Intelligence Cluster',
        description: 'Data ingestion and documentation group for knowledge synthesis and repository analytics.',
        department: 'Research',
        theme: 'cyan',
        default_budget_usd: 1000,
        badge_label: 'RESEARCH-PUB',
        collaborators: ['16', '17']
    }
];

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
        is_active: true,
        is_team: true,
        team_badge: 'STRAT-CMD'
    },
    {
        id: 'cl-chain-a',
        name: 'Strategic Ops (Chain A)',
        department: 'Operations',
        path: '/workspaces/strategic-ops',
        collaborators: ['3', '4', '5', 'alpha'],
        alpha_id: '3',
        objective: 'Optimize swarm coordination and strategic resource allocation.',
        theme: 'cyan',
        pending_tasks: [],
        is_active: false,
        is_team: true,
        team_badge: 'SWARM-OPS'
    },
    {
        id: 'cl-chain-b',
        name: 'Core Intelligence (Chain B)',
        department: 'Engineering',
        path: '/workspaces/core-intelligence',
        collaborators: ['7', 'Tadpole_OS_Specialist', 'ResearcherAgent', 'GitHubResearcher'],
        alpha_id: '7',
        objective: 'Enhance neural processing efficiency and knowledge synthesis.',
        theme: 'zinc',
        pending_tasks: [],
        is_team: true,
        team_badge: 'CORE-INT'
    },
    {
        id: 'cl-chain-c',
        name: 'Applied Growth (Chain C)',
        department: 'Product',
        path: '/workspaces/applied-growth',
        collaborators: ['99', '23', '26', 'Risk_Analyzer'],
        alpha_id: '99',
        objective: 'Iterate on user-facing features and scale operational impact.',
        theme: 'amber',
        pending_tasks: [],
        is_team: true,
        team_badge: 'APPLIED-DEV'
    }
];

export const use_workspace_store = create<Workspace_State>()(
    persist(
        (set, get) => ({
            clusters: DEFAULT_CLUSTERS,
            team_presets: DEFAULT_TEAM_PRESETS,
            active_proposals: {},
            sync_status: {},

            // Actions - Delegated to Workspace_Service
            load_quotas: async () => workspace_service.sync_quotas(),
            create_cluster: async (mission) => workspace_service.create_mission_cluster(mission),
            refresh_sync_status: async () => workspace_service.refresh_telemetry(),
            update_cluster_budget: async (id, budget) => workspace_service.update_budget(id, budget),

            add_team_preset: (preset) => {
                set(state => ({
                    team_presets: [...state.team_presets, preset]
                }));
            },

            update_team_preset: (id, updates) => {
                set(state => ({
                    team_presets: state.team_presets.map(p => p.id === id ? { ...p, ...updates } : p)
                }));
            },

            delete_team_preset: (id) => {
                set(state => ({
                    team_presets: state.team_presets.filter(p => p.id !== id)
                }));
            },

            // Pure State Actions
            assign_agent_to_cluster: (agent_id, cluster_id) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === cluster_id ? { ...c, collaborators: [...new Set([...c.collaborators, agent_id])] } : c
                    )
                }));
            },

            unassign_agent_from_cluster: (agent_id, cluster_id) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === cluster_id ? {
                            ...c,
                            collaborators: c.collaborators.filter(id => id !== agent_id),
                            alpha_id: c.alpha_id === agent_id ? undefined : c.alpha_id
                        } : c
                    )
                }));
            },

            update_cluster_objective: (cluster_id, objective) => {
                set(state => ({
                    clusters: state.clusters.map(c => c.id === cluster_id ? { ...c, objective } : c)
                }));
                workspace_service.request_proposal(cluster_id);
            },

            update_cluster_department: (cluster_id, department) => {
                set(state => ({
                    clusters: state.clusters.map(c => c.id === cluster_id ? { ...c, department } : c)
                }));
            },

            generate_proposal: (cluster_id, immediate = false) => workspace_service.request_proposal(cluster_id, immediate),

            apply_proposal: (cluster_id) => {
                set(state => {
                    const next_proposals = { ...state.active_proposals };
                    delete next_proposals[cluster_id];
                    return { active_proposals: next_proposals };
                });
            },

            dismiss_proposal: (cluster_id) => {
                set(state => {
                    const next_proposals = { ...state.active_proposals };
                    delete next_proposals[cluster_id];
                    return { active_proposals: next_proposals };
                });
            },

            set_alpha_node: (cluster_id, agent_id) => {
                set(state => ({
                    clusters: state.clusters.map(c => c.id === cluster_id ? { ...c, alpha_id: agent_id } : c)
                }));
            },

            delete_cluster: (cluster_id) => {
                set(state => ({
                    clusters: state.clusters.filter(c => c.id !== cluster_id)
                }));
            },

            toggle_cluster_active: (cluster_id) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === cluster_id ? { ...c, is_active: !c.is_active } : c
                    )
                }));
            },

            toggle_mission_analysis: (cluster_id) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === cluster_id ? { ...c, analysis_enabled: !c.analysis_enabled } : c
                    )
                }));
            },

            toggle_cluster_privacy: async (cluster_id) => {
                const cluster = get().clusters.find(c => c.id === cluster_id);
                if (!cluster) return;
                const target_mode = !cluster.privacy_mode;
                await workspace_service.update_cluster_privacy(cluster_id, target_mode);
            },

            add_branch: (cluster_id, branch) => {
                set(state => ({
                    clusters: state.clusters.map(c => {
                        if (c.id !== cluster_id) return c;
                        const next_tasks = [...c.pending_tasks, {
                            ...branch,
                            id: (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `br-${Date.now()}`,
                            status: 'pending' as const,
                            timestamp: Date.now()
                        }];
                        return {
                            ...c,
                            pending_tasks: next_tasks.length > MAX_TASK_BRANCHES ? next_tasks.slice(-MAX_TASK_BRANCHES) : next_tasks
                        };
                    })
                }));
            },

            approve_branch: (cluster_id, branch_id) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === cluster_id ? {
                            ...c,
                            pending_tasks: c.pending_tasks.map(t => t.id === branch_id ? { ...t, status: 'completed' } : t)
                        } : c
                    )
                }));
            },

            reject_branch: (cluster_id, branch_id) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === cluster_id ? {
                            ...c,
                            pending_tasks: c.pending_tasks.map(t => t.id === branch_id ? { ...t, status: 'rejected' } : t)
                        } : c
                    )
                }));
            },

            receive_handoff: (source_cluster_id, target_cluster_id, description) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === target_cluster_id ? {
                            ...c,
                            pending_tasks: [...c.pending_tasks, {
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

            get_agent_path: (agent_id) => {
                const clusters = get().clusters || [];
                const cluster = clusters.find(c => (c.collaborators || []).includes(agent_id));
                return cluster ? cluster.path : `/workspaces/agent-silo-${agent_id}`;
            },
        }),
        {
            name: 'tadpole-workspaces-v4',
            version: 2,
            migrate: (persisted_state: unknown, version: number) => {
                if (version < 2) {
                    // Reset to default clusters to load new valid collaborator IDs
                    return {
                        clusters: DEFAULT_CLUSTERS,
                        active_proposals: {},
                        sync_status: {}
                    };
                }
                return persisted_state as Record<string, unknown>;
            }
        }
    )
);

import('../services/workspace_service').then((m) => {
    m.set_workspace_store?.(use_workspace_store);
}).catch(() => {});
