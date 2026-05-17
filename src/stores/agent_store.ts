/**
 * @docs ARCHITECTURE:Stores
 * 
 * ### AI Assist Note
 * **Zustand State**: Standardized reactive store for the entire agent swarm.
 * Refactored to be a pure state container. Side-effects and telemetry are handled by `agent_service`.
 */

import { create } from 'zustand';
import type { Agent } from '../types';

export interface Agent_State {
    agents: Agent[];
    is_loading: boolean;
    error: string | null;

    // Pure State Mutations
    set_loading: (loading: boolean) => void;
    set_error: (error: string | null) => void;
    get_agent: (id: string) => Agent | undefined;
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
    get_agent: (id) => (get().agents || []).find(a => a.id === id)
}));

// Metadata: [agent_store]
