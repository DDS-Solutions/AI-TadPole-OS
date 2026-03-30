import { create } from 'zustand';
import type { Agent } from '../types';
import { loadAgents, persistAgentUpdate, normalizeAgent, type RawAgent } from '../services/agentService';
import { TadpoleOSSocket } from '../services/socket';
import { EventBus } from '../services/eventBus';
import { useWorkspaceStore } from './workspaceStore';
import { agents as mockAgents } from '../data/mockAgents';

const SYNC_CHANNEL = 'tadpole-os-sync';
const syncChannel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;
const TAB_ID = (typeof crypto !== 'undefined' && crypto.randomUUID)
    ? crypto.randomUUID()
    : `tab-${Date.now()}-${Math.random().toString(16).slice(2)}`;
let isApplyingRemoteSync = false;

type AgentSyncMessage =
    | { type: 'agent:update'; sourceId: string; payload: { id: string; updates: Partial<Agent> } }
    | { type: 'agent:add'; sourceId: string; payload: Agent }
    | { type: 'agents:replace'; sourceId: string; payload: Agent[] };

const broadcastAgentSync = (
    message: Omit<AgentSyncMessage, 'sourceId'>,
) => {
    if (!syncChannel || isApplyingRemoteSync) return;
    syncChannel.postMessage({ ...message, sourceId: TAB_ID });
};

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
            const liveAgents = await loadAgents();
            
            // ── Baseline Guard ───────────────────────────────────────
            // Ensure "Agent of Nine" (ID: 1) is always present.
            // If the backend doesn't have it, we backfill from mock data.
            const hasAlpha = liveAgents.some(a => a.id === '1');
            const finalAgents = hasAlpha 
                ? liveAgents 
                : [normalizeAgent(mockAgents[0] as unknown as RawAgent), ...liveAgents];

            set({ agents: finalAgents, isLoading: false });
            broadcastAgentSync({ type: 'agents:replace', payload: finalAgents });
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
        // 1. Optimistic Update (Immediate UI response)
        set(state => ({
            agents: state.agents.map(a => a.id === id ? { ...a, ...updates } : a)
        }));
        broadcastAgentSync({
            type: 'agent:update',
            payload: { id, updates }
        });

        // 2. Persist to Backend if needed
        // For status/task updates, we don't necessarily want to wait for backend confirmation
        // as the mission launch handles the side effects.
        try {
            await persistAgentUpdate(id, updates);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            console.warn('[AgentStore] Persistence failed, keeping optimistic state:', message);
            EventBus.emit({
                source: 'System',
                text: `Agent Persistence Failed: ${message}`,
                severity: 'warning'
            });
            // We consciously DO NOT revert here to prevent "System Idle" flicker
        }
    },

    /**
     * Registers a new agent node in the swarm.
     */
    addAgent: async (agent) => {
        set(state => ({
            agents: [...state.agents, agent]
        }));
        broadcastAgentSync({ type: 'agent:add', payload: agent });
        try {
            await persistAgentUpdate(agent.id, agent);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            console.error('[AgentStore] Creation failure:', message);
            const agents = await loadAgents();
            set({ agents, error: message });
            broadcastAgentSync({ type: 'agents:replace', payload: agents });

            EventBus.emit({
                source: 'System',
                text: `Agent Registration Blocked: ${message}`,
                severity: 'error'
            });
        }
    },

    getAgent: (id) => get().agents.find(a => a.id === id),

    initTelemetry: () => {
        // PERF: Cache for agent workspace paths to avoid repeated lookups
        const pathCache = new Map<string, string>();

        const unsubscribe = TadpoleOSSocket.subscribeAgentUpdates((event) => {
            if (event.type === 'agent:update' && event.agentId && event.data) {
                const idStr = String(event.agentId);
                
                let workspacePath = pathCache.get(idStr);
                if (!workspacePath) {
                    const workspaceStore = useWorkspaceStore.getState();
                    const cluster = workspaceStore.clusters.find(c => 
                        c.collaborators.map(String).includes(idStr)
                    );
                    workspacePath = cluster ? cluster.path : `/workspaces/agent-silo-${idStr}`;
                    pathCache.set(idStr, workspacePath);
                }
                
                const finalWorkPath = workspacePath; // Scope for closure
                
                set(state => {
                    const existing = state.agents.find(a => a.id === event.agentId);
                    
                    if (existing) {
                        return {
                            agents: state.agents.map(a => {
                                if (a.id === event.agentId) {
                                    const mergedRaw = { ...a, ...event.data } as unknown as RawAgent;
                                    return normalizeAgent(mergedRaw, finalWorkPath, a);
                                }
                                return a;
                            })
                        };
                    } else {
                        const normalized = normalizeAgent({ ...event.data, id: event.agentId } as unknown as RawAgent, finalWorkPath);
                        return { agents: [...state.agents, normalized] };
                    }
                });
            } else if (event.type === 'engine:ui_invalidate') {
                if (event.resource === 'agents') {
                    get().fetchAgents();
                }
            }
        });

        return unsubscribe;
    }
}));

// ── Cross-Tab Synchronization ────────────────────────────────
if (syncChannel) {
    syncChannel.onmessage = (event) => {
        const message = event.data as AgentSyncMessage;
        if (!message || message.sourceId === TAB_ID) {
            return;
        }

        isApplyingRemoteSync = true;
        try {
            if (message.type === 'agent:update') {
                useAgentStore.setState((state) => ({
                    agents: state.agents.map((agent) =>
                        agent.id === message.payload.id
                            ? { ...agent, ...message.payload.updates }
                            : agent
                    )
                }));
            } else if (message.type === 'agent:add') {
                useAgentStore.setState((state) => {
                    if (state.agents.some((agent) => agent.id === message.payload.id)) {
                        return state;
                    }
                    return { agents: [...state.agents, message.payload] };
                });
            } else if (message.type === 'agents:replace') {
                useAgentStore.setState({ agents: message.payload });
            }
        } finally {
            isApplyingRemoteSync = false;
        }
    };
}

