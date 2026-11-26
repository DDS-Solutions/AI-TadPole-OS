/**
 * @docs ARCHITECTURE:Stores
 *
 * ### AI Assist Note
 * - Review changes preserve origin security and propagate confirmed operation outcomes.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / agent_store
 * - **Primary Entrypoints**: `use_agent_store`, `Agent_State`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: Routed through the shared log_error helper.
 */

import { create } from 'zustand';
import type { Agent } from '../types';
import { log_error } from '../services/system_utils';
import { get_tadpole_os_socket } from '../services/socket';
import { agent_api_service } from '../services/agent';

/**
 * Agent Store State Diagram
 * ```mermaid
 * stateDiagram-v2
 *   [*] --> Idle
 *   Idle --> Loading : fetch
 *   Loading --> Idle : success
 *   Loading --> Error : failure
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
    update_agent: (id: string, updates: Partial<Agent>) => Promise<boolean>;
    add_agent: (agent: Agent) => Promise<boolean>;
    delete_agent: (id: string) => Promise<void>;
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
        try {
            const { agent_service } = await import('../services/agent_service');
            return await agent_service.update_agent(id, updates);
        } catch (err) {
            log_error('AgentStore', 'Persistence Failed', err, 'warning');
            set({ error: 'Failed to save agent.' });
            return false;
        }
    },

    add_agent: async (agent) => {
        // Optimistic add
        set((state) => ({
            agents: [...(state.agents || []), agent]
        }));
        try {
            await agent_api_service.create_agent(agent);
            set({ error: null });
            return true;
        } catch (err) {
            log_error('AgentStore', 'Agent Registration Blocked', err);
            // Revert optimistic add
            const error_msg = err instanceof Error ? err.message : String(err);
            set((state) => ({ error: error_msg, agents: state.agents.filter(a => a.id !== agent.id) }));
            return false;
        }
    },

    delete_agent: async (id: string) => {
        const { agent_service } = await import('../services/agent_service');
        await agent_service.delete_agent(id);
    },


}));

// Synchronize agent name mappings to the socket cache (defensively check for mocks)
use_agent_store.subscribe((state) => {
    const socket = get_tadpole_os_socket();
    if (socket && typeof socket.set_agent_name_cache === 'function') {
        socket.set_agent_name_cache(state.agents || []);
    }
});

import { register_store_for_test_reset } from '../utils/store_registry';

register_store_for_test_reset(() => {
    use_agent_store.setState(use_agent_store.getInitialState(), true);
});
