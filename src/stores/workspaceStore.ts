/**
 * @module workspaceStore
 * Managed state for collaborative Mission Clusters and Task Branching.
 * 
 * DESIGN PATTERN: Mission-Based Clusters
 * - Replaces individual agent silos with shared paths based on active missions.
 * - Implements a "Task Branch" workflow where changes can be reviewed/approved by Alpha nodes.
 */
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { getSettings } from './settingsStore';
import { ProposalService } from '../services/ProposalService';

export interface MissionCluster {
    id: string;
    name: string;
    department: 'Executive' | 'Engineering' | 'Product' | 'Sales' | 'Operations' | 'Quality Assurance' | 'Design' | 'Research' | 'Support' | 'Marketing';
    path: string;
    collaborators: string[]; // Agent IDs
    alphaId?: string; // The leader of the cluster
    objective?: string; // High-level mission objective
    theme: 'cyan' | 'zinc' | 'amber' | 'blue';
    pendingTasks: TaskBranch[];
    isActive?: boolean;
    budgetUsd?: number;
    analysisEnabled?: boolean;
}

export interface SwarmProposal {
    clusterId: string;
    reasoning: string;
    changes: {
        agentId: string;
        proposedRole?: string;
        proposedModel?: string;
        addedSkills?: string[];
        addedWorkflows?: string[];
    }[];
    timestamp: number;
}

export interface TaskBranch {
    id: string;
    agentId: string;
    description: string;
    targetPath: string;
    status: 'pending' | 'merging' | 'completed' | 'rejected';
    timestamp: number;
}

interface WorkspaceState {
    clusters: MissionCluster[];
    activeProposals: Record<string, SwarmProposal>; // clusterId -> proposal

    // Actions
    createCluster: (mission: Partial<MissionCluster>) => void;
    assignAgentToCluster: (agentId: string, clusterId: string) => void;
    unassignAgentFromCluster: (agentId: string, clusterId: string) => void;
    updateClusterObjective: (clusterId: string, objective: string) => void;
    updateClusterDepartment: (clusterId: string, department: MissionCluster['department']) => void;
    updateClusterBudget: (clusterId: string, budget: number) => void;
    generateProposal: (clusterId: string) => void;
    applyProposal: (clusterId: string) => void;
    dismissProposal: (clusterId: string) => void;
    setAlphaNode: (clusterId: string, agentId: string) => void;
    deleteCluster: (clusterId: string) => void;
    toggleClusterActive: (clusterId: string) => void;
    toggleMissionAnalysis: (clusterId: string) => void;
    addBranch: (clusterId: string, branch: Omit<TaskBranch, 'id' | 'status' | 'timestamp'>) => void;
    approveBranch: (clusterId: string, branchId: string) => void;
    rejectBranch: (clusterId: string, branchId: string) => void;
    receiveHandoff: (sourceClusterId: string, targetClusterId: string, description: string) => void;

    // Internal path calculation
    getAgentPath: (agentId: string) => string;
}

const DEFAULT_CLUSTERS: MissionCluster[] = [
    {
        id: 'cl-command',
        name: 'Strategic Command',
        department: 'Executive',
        path: '/workspaces/strategic-command',
        collaborators: ['1', '2'],
        alphaId: '1',
        objective: 'Global swarm oversight and strategic mission planning.',
        theme: 'blue',
        pendingTasks: [],
        isActive: true
    },
    {
        id: 'cl-chain-a',
        name: 'Strategic Ops (Chain A)',
        department: 'Operations',
        path: '/workspaces/strategic-ops',
        collaborators: ['3', '4', '5', '6'],
        alphaId: '3',
        objective: 'Optimize swarm coordination and strategic resource allocation.',
        theme: 'cyan',
        pendingTasks: [],
        isActive: false
    },
    {
        id: 'cl-chain-b',
        name: 'Core Intelligence (Chain B)',
        department: 'Engineering',
        path: '/workspaces/core-intelligence',
        collaborators: ['7', '8', '9', '10'],
        alphaId: '7',
        objective: 'Enhance neural processing efficiency and knowledge synthesis.',
        theme: 'zinc',
        pendingTasks: []
    },
    {
        id: 'cl-chain-c',
        name: 'Applied Growth (Chain C)',
        department: 'Product',
        path: '/workspaces/applied-growth',
        collaborators: ['11', '12', '13', '14'],
        alphaId: '11',
        objective: 'Iterate on user-facing features and scale operational impact.',
        theme: 'amber',
        pendingTasks: []
    }
];

let proposalTimeout: ReturnType<typeof setTimeout> | undefined;

export const useWorkspaceStore = create<WorkspaceState>()(
    persist(
        (set, get) => ({
            clusters: DEFAULT_CLUSTERS,
        activeProposals: {},

            // Actions
            createCluster: (mission) => {
                const settings = getSettings();
                const clusters = get().clusters;

                if (clusters.length >= settings.maxClusters) {
                    // We could trigger a notification here, but for now we just prevent creation
                    console.warn(`⚠️ Cluster limit reached (${settings.maxClusters}). Cannot create new workspace.`);
                    return;
                }

                set(state => ({
                    clusters: [...state.clusters, {
                        ...mission,
                        id: `cl-${(typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID().split('-')[0] : Date.now().toString(36)}`,
                        name: mission.name || 'New Cluster',
                        department: mission.department || 'Engineering',
                        path: mission.path || `/workspaces/${Date.now()}`,
                        collaborators: mission.collaborators || [],
                        theme: mission.theme || 'blue',
                        pendingTasks: []
                    }]
                }));
            },

            assignAgentToCluster: (agentId, clusterId) => set(state => ({
                clusters: state.clusters.map(c =>
                    c.id === clusterId ? { ...c, collaborators: [...new Set([...c.collaborators, String(agentId)])] } : c
                )
            })),

            unassignAgentFromCluster: (agentId, clusterId) => set(state => ({
                clusters: state.clusters.map(c =>
                    c.id === clusterId ? {
                        ...c,
                        collaborators: c.collaborators.filter(id => String(id) !== String(agentId)),
                        alphaId: c.alphaId === String(agentId) ? undefined : c.alphaId
                    } : c
                )
            })),

            updateClusterObjective: (clusterId, objective) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === clusterId ? { ...c, objective } : c
                    )
                }));

                // Debounce proposal generation
                if (proposalTimeout) clearTimeout(proposalTimeout);
                proposalTimeout = setTimeout(() => {
                    get().generateProposal(clusterId);
                }, 1000);
            },

            updateClusterDepartment: (clusterId, department) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === clusterId ? { ...c, department } : c
                    )
                }));
            },

            updateClusterBudget: (clusterId, budget) => {
                set(state => ({
                    clusters: state.clusters.map(c =>
                        c.id === clusterId ? { ...c, budgetUsd: budget } : c
                    )
                }));
            },

            generateProposal: (clusterId) => {
                const cluster = get().clusters.find(c => c.id === clusterId);
                if (!cluster) return;

                const proposal = ProposalService.generateProposal(cluster);
                if (!proposal) return;

                set(state => ({
                    activeProposals: {
                        ...state.activeProposals,
                        [clusterId]: proposal
                    }
                }));
            },

            applyProposal: (clusterId) => {
                const proposal = get().activeProposals[clusterId];
                if (!proposal) return;

                // In a real app, this would update the global agent list in agentStore.
                // For the prototype, we assume these are mission-specific overrides
                // handled by the UI when the mission is active.

                // We clear the proposal once applied
                set(state => {
                    const nextProposals = { ...state.activeProposals };
                    delete nextProposals[clusterId];
                    return { activeProposals: nextProposals };
                });
            },

            dismissProposal: (clusterId) => set(state => {
                const nextProposals = { ...state.activeProposals };
                delete nextProposals[clusterId];
                return { activeProposals: nextProposals };
            }),

            setAlphaNode: (clusterId, agentId) => set(state => ({
                clusters: state.clusters.map(c =>
                    c.id === clusterId ? { ...c, alphaId: agentId } : c
                )
            })),

            deleteCluster: (clusterId: string) => set(state => ({
                clusters: state.clusters.filter(c => c.id !== clusterId)
            })),

            toggleClusterActive: (clusterId: string) => set(state => ({
                clusters: state.clusters.map(c => ({
                    ...c,
                    isActive: c.id === clusterId ? !c.isActive : false
                }))
            })),

            toggleMissionAnalysis: (clusterId: string) => set(state => ({
                clusters: state.clusters.map(c =>
                    c.id === clusterId ? { ...c, analysisEnabled: !c.analysisEnabled } : c
                )
            })),

            addBranch: (clusterId, branch) => set(state => ({
                clusters: state.clusters.map(c =>
                    c.id === clusterId ? {
                        ...c,
                        pendingTasks: [...c.pendingTasks, {
                            ...branch,
                            id: `br-${Date.now()}`,
                            status: 'pending',
                            timestamp: Date.now()
                        }]
                    } : c
                )
            })),

            approveBranch: (clusterId, branchId) => set(state => ({
                clusters: state.clusters.map(c =>
                    c.id === clusterId ? {
                        ...c,
                        pendingTasks: c.pendingTasks.map(t => t.id === branchId ? { ...t, status: 'completed' } : t)
                    } : c
                )
            })),

            rejectBranch: (clusterId, branchId) => set(state => ({
                clusters: state.clusters.map(c =>
                    c.id === clusterId ? {
                        ...c,
                        pendingTasks: c.pendingTasks.map(t => t.id === branchId ? { ...t, status: 'rejected' } : t)
                    } : c
                )
            })),

            receiveHandoff: (sourceClusterId, targetClusterId, description) => set(state => ({
                clusters: state.clusters.map(c =>
                    c.id === targetClusterId ? {
                        ...c,
                        pendingTasks: [...c.pendingTasks, {
                            id: `ho-${Date.now()}`,
                            agentId: 'System (Handoff)',
                            description: `[HANDOFF FROM ${sourceClusterId}] ${description}`,
                            targetPath: c.path,
                            status: 'pending',
                            timestamp: Date.now()
                        }]
                    } : c
                )
            })),

            getAgentPath: (agentId) => {
                const idStr = String(agentId);
                const cluster = get().clusters.find(c => c.collaborators.map(String).includes(idStr));
                return cluster ? cluster.path : `/workspaces/agent-silo-${idStr}`;
            }
        }),
        {
            name: 'tadpole-workspaces-v3',
            version: 1,
            migrate: (persistedState: unknown, version: number) => {
                const state = persistedState as Record<string, unknown>;
                if (version === 0) {
                    const clusters = (state.clusters as MissionCluster[]) || [];
                    const hasCommand = clusters.some((c: MissionCluster) => c.id === 'cl-command');
                    if (!hasCommand) {
                        return {
                            ...state,
                            clusters: [
                                {
                                    id: 'cl-command',
                                    name: 'Strategic Command',
                                    department: 'Executive',
                                    path: '/workspaces/strategic-command',
                                    collaborators: ['1', '2'],
                                    alphaId: '1',
                                    objective: 'Global swarm oversight and strategic mission planning.',
                                    theme: 'blue',
                                    pendingTasks: [],
                                    isActive: true
                                },
                                ...clusters
                            ]
                        };
                    }
                }
                return persistedState;
            }
        }
    )
);
