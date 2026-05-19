/**
 * @docs ARCHITECTURE:UI-Stores
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Stores**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[agent_store]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Stores
 * 
 * ### AI Assist Note
 * **Zustand State**: Standardized reactive store for the entire agent swarm.
 * Refactored to support pure state mutations, but provides backwards-compatible
 * wrappers for side-effects and telemetry delegating to the appropriate services.
 */

import { create } from 'zustand';
import type { Agent } from '../types';
import type { AgentDto } from '../contracts/agent';
import { tadpole_os_socket } from '../services/socket';
import { use_workspace_store } from './workspace_store';
import { load_agents, persist_agent_update, normalize_agent } from '../services/agent_service';
import { agent_api_service } from '../services/agent_api_service';
import { log_error } from '../services/system_utils';

export interface Agent_State {
    agents: Agent[];
    is_loading: boolean;
    error: string | null;

    // Pure State Mutations
    set_loading: (loading: boolean) => void;
    set_error: (error: string | null) => void;
    get_agent: (id: string) => Agent | undefined;

    // Side-effects & Telemetry (Delegated to Services)
    fetch_agents: (options?: RequestInit) => Promise<void>;
    update_agent: (id: string, updates: Partial<Agent>) => Promise<void>;
    add_agent: (agent: Agent) => Promise<boolean>;
    init_telemetry: () => () => void;
}

/**
 * use_agent_store
 * Reactive store for the agent swarm registry.
 */
export const use_agent_store = create<Agent_State>((set, get) => ({
    agents: [],
    is_loading: false,
    error: null,

    set_loading: (loading) => set({ is_loading: loading }),
    set_error: (error) => set({ error }),
    get_agent: (id) => (get().agents || []).find(a => a.id === id),

    fetch_agents: async (options) => {
        set({ is_loading: true, error: null });
        try {
            const agents = await load_agents(options);
            if (Array.isArray(agents)) {
                set({ agents, is_loading: false });
            } else {
                set({ is_loading: false });
            }
        } catch (err) {
            log_error('AgentStore', 'Agent Registry Failure', err);
            set({ is_loading: false, error: 'Failed to load agent registry. Check system logs for details.' });
        }
    },

    update_agent: async (id, updates) => {
        // Optimistic update
        set((state) => ({
            agents: (state.agents || []).map((a) =>
                a.id === id ? { ...a, ...updates } : a
            )
        }));
        try {
            await persist_agent_update(id, updates);
        } catch (err) {
            log_error('AgentStore', 'Persistence Failed', err, 'warning');
        }
    },

    add_agent: async (agent) => {
        // Optimistic add
        set((state) => ({
            agents: [...(state.agents || []), agent]
        }));
        try {
            await agent_api_service.create_agent(agent);
            return true;
        } catch (err) {
            log_error('AgentStore', 'Agent Registration Blocked', err);
            // Revert optimistic add
            try {
                const agents = await load_agents();
                if (Array.isArray(agents)) {
                    set({ agents, error: err instanceof Error ? err.message : String(err) });
                } else {
                    set((state) => ({
                        agents: state.agents.filter((a) => a.id !== agent.id),
                        error: err instanceof Error ? err.message : String(err)
                    }));
                }
            } catch {
                set((state) => ({
                    agents: state.agents.filter((a) => a.id !== agent.id),
                    error: err instanceof Error ? err.message : String(err)
                }));
            }
            return false;
        }
    },

    init_telemetry: () => {
        const unsubscribe = tadpole_os_socket.subscribe_agent_updates((event) => {
            if (!event || (!event.agent_id && !event.agentId) || !event.data) return;

            const id_str = (event.agent_id || event.agentId) as string;
            const workspace_store = use_workspace_store.getState();
            const cluster = (workspace_store.clusters || []).find(c => (c.collaborators || []).includes(id_str));
            const path = cluster ? cluster.path : `/workspaces/agent-silo-${id_str}`;

            set((state) => {
                const existing = state.agents.find(a => a.id === id_str);
                const normalized = normalize_agent({ id: id_str, ...event.data } as unknown as AgentDto, path, existing);
                if (existing) {
                    return {
                        agents: state.agents.map(a => a.id === id_str ? normalized : a)
                    };
                } else {
                    return {
                        agents: [...state.agents, normalized]
                    };
                }
            });
        });

        return unsubscribe;
    }
}));

// Metadata: [agent_store]

// Metadata: [agent_store]
