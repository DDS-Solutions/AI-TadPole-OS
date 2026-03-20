import { create } from 'zustand';
import type { Agent } from '../types';
import { loadAgents, persistAgentUpdate, normalizeAgent, type RawAgent } from '../services/agentService';
import { TadpoleOSSocket } from '../services/socket';
import { EventBus } from '../services/eventBus';
import { useWorkspaceStore } from './workspaceStore';

interface AgentState {
    agents: Agent[];
    isLoading: boolean;
    error: string | null;

    // Actions
    fetchAgents: () => Promise<void>;
    updateAgent: (id: string, updates: Partial<Agent>) => Promise<void>;
    addAgent: (agent: Agent) => Promise<void>;
    getAgent: (id: string) => Agent | undefined;

    /** Initializes real-time telemetry listeners. */
    initTelemetry: () => () => void;
}

/**
 * Standardized reactive store for the entire agent swarm.
 * 
 * Powered by Zustand, this store multiplexes HTTP persistence and 
 * real-time WebSocket status updates, ensuring UI-wide consistency.
 */
export const useAgentStore = create<AgentState>((set, get) => ({
    agents: [],
    isLoading: false,
    error: null,

    /**
     * Synchronizes the local store with the global Rust agent registry.
     */
    fetchAgents: async () => {
        set({ isLoading: true, error: null });
        try {
            const agents = await loadAgents();
            set({ agents, isLoading: false });
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            set({ error: message, isLoading: false });
            EventBus.emit({
                source: 'System',
                text: `Agent Registry Failure: ${message}`,
                severity: 'error'
            });
        }
    },

    /**
     * Updates an agent's configuration with optimistic UI support.
     * Reverts to backend state on persistence failure.
     */
    updateAgent: async (id, updates) => {
        // 1. Optimistic Update
        set(state => ({
            agents: state.agents.map(a => a.id === id ? { ...a, ...updates } : a)
        }));

        // 2. Persist to Backend & LocalStorage
        try {
            await persistAgentUpdate(id, updates);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            console.error('[AgentStore] Sync failure:', message);

            // Revert on critical failure (simple refetch for consistency)
            const agents = await loadAgents();
            set({ agents, error: message });

            EventBus.emit({
                source: 'System',
                agentId: id,
                text: `Agent Sync Failure: ${message}. Reverting to last known state.`,
                severity: 'warning'
            });
        }
    },

    /**
     * Registers a new agent node in the swarm.
     */
    addAgent: async (agent) => {
        set(state => ({
            agents: [...state.agents, agent]
        }));
        try {
            await persistAgentUpdate(agent.id, agent);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            console.error('[AgentStore] Creation failure:', message);
            const agents = await loadAgents();
            set({ agents, error: message });

            EventBus.emit({
                source: 'System',
                text: `Agent Registration Blocked: ${message}`,
                severity: 'error'
            });
        }
    },

    getAgent: (id) => get().agents.find(a => a.id === id),

    initTelemetry: () => {
        const unsubscribe = TadpoleOSSocket.subscribeAgentUpdates((event) => {
            if (event.type === 'agent:update' && event.agentId && event.data) {
                const workspaceStore = useWorkspaceStore.getState();
                const getAgentPath = (agentId: string) => {
                    const idStr = String(agentId);
                    const cluster = workspaceStore.clusters.find(c => c.collaborators.map(String).includes(idStr));
                    return cluster ? cluster.path : `/workspaces/agent-silo-${idStr}`;
                };
                const workspacePath = getAgentPath(event.agentId);
                set(state => ({
                    agents: state.agents.map(a => {
                        if (a.id === event.agentId) {
                            // Merge incoming partial data with existing agent to prevent property loss
                            const mergedRaw = { ...a, ...event.data } as unknown as RawAgent;
                            return normalizeAgent(mergedRaw, workspacePath);
                        }
                        return a;
                    })
                }));
            }
        });

        return unsubscribe;
    }
}));

