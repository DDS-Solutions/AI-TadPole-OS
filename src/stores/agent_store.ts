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
import { use_workspace_store } from './workspace_store';
import { log_error } from '../services/system_utils';
import { get_tadpole_os_socket } from '../services/socket';
import { agent_api_service } from '../services/agent';
import { normalize_agent_dto } from '../domain/agents/normalizers';

/**
 * Agent Store State Diagram
 * ```mermaid
 * stateDiagram-v2
 *   [*] --> Idle
 *   Idle --> Loading : fetch
 *   Loading --> Idle : success
 *   Loading --> Error : failure
 *   Error --> Idle : reset
 *   Error --> Idle : reset
 * ```
 */
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
    delete_agent: (id: string) => Promise<void>;
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
            const { agent_service } = await import('../services/agent_service');
            await agent_service.load_agents_into_store(options);
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
            const { agent_service } = await import('../services/agent_service');
            await agent_service.update_agent(id, updates);
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
            const error_msg = err instanceof Error ? err.message : String(err);
            set({ error: error_msg });
            try {
                const { agent_service } = await import('../services/agent_service');
                await agent_service.load_agents_into_store();
            } catch {
                set((state) => ({
                    agents: state.agents.filter((a) => a.id !== agent.id)
                }));
            }
            return false;
        }
    },

    delete_agent: async (id: string) => {
        set((state) => ({
            agents: (state.agents || []).filter((a) => a.id !== id)
        }));
        use_workspace_store.setState((state) => ({
            clusters: (state.clusters || []).map((c) => ({
                ...c,
                collaborators: (c.collaborators || []).filter((cid) => cid !== id),
                alpha_id: c.alpha_id === id ? undefined : c.alpha_id
            }))
        }));
    },

    init_telemetry: () => {
        const unsubscribe = get_tadpole_os_socket().subscribe('agentUpdates', (event) => {
            if (!event || (!event.agent_id && !event.agentId) || !event.data) return;

            const id_str = (event.agent_id || event.agentId) as string;
            const workspace_store = use_workspace_store.getState();
            const cluster = (workspace_store.clusters || []).find(c => (c.collaborators || []).includes(id_str));
            const path = cluster ? cluster.path : `/workspaces/agent-silo-${id_str}`;

            set((state) => {
                const existing = state.agents.find(a => a.id === id_str);
                const normalized = normalize_agent_dto(
                    { id: id_str, ...event.data } as unknown as AgentDto,
                    path,
                    existing
                );
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

// Synchronize agent name mappings to the socket cache (defensively check for mocks)
use_agent_store.subscribe((state) => {
    const socket = get_tadpole_os_socket();
    if (socket && typeof socket.set_agent_name_cache === 'function') {
        socket.set_agent_name_cache(state.agents || []);
    }
});

// Metadata: [agent_store]
