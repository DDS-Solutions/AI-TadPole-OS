import { create } from 'zustand';
import type { Agent } from '../types';
import { load_agents, persist_agent_update, normalize_agent, type Raw_Agent } from '../services/agent_service';
import { agent_api_service } from '../services/agent_api_service';
import { tadpole_os_socket } from '../services/socket';
import { event_bus } from '../services/event_bus';
import { use_workspace_store } from './workspace_store';
import { agents as mock_agents } from '../data/mock_agents';

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;
const TAB_ID = (typeof crypto !== 'undefined' && crypto.randomUUID)
    ? crypto.randomUUID()
    : `tab-${Date.now()}-${Math.random().toString(16).slice(2)}`;
let is_applying_remote_sync = false;

type Agent_Sync_Message =
    | { type: 'agent:update'; source_id: string; payload: { id: string; updates: Partial<Agent> } }
    | { type: 'agent:add'; source_id: string; payload: Agent }
    | { type: 'agents:replace'; source_id: string; payload: Agent[] };

/**
 * broadcast_agent_sync
 * Propagates agent state changes across all browser tabs for UI parity.
 */
const broadcast_agent_sync = (
    message: Omit<Agent_Sync_Message, 'source_id'>,
) => {
    if (!sync_channel || is_applying_remote_sync) return;
    sync_channel.postMessage({ ...message, source_id: TAB_ID });
};

export interface Agent_State {
    agents: Agent[];
    is_loading: boolean;
    error: string | null;

    // Actions
    fetch_agents: () => Promise<void>;
    update_agent: (id: string, updates: Partial<Agent>) => Promise<void>;
    add_agent: (agent: Agent) => Promise<boolean>;
    get_agent: (id: string) => Agent | undefined;

    /** Initializes real-time telemetry listeners. */
    init_telemetry: () => () => void;
}

/**
 * use_agent_store
 * Standardized reactive store for the entire agent swarm.
 */
export const use_agent_store = create<Agent_State>((set, get) => ({
    agents: [],
    is_loading: false,
    error: null,

    fetch_agents: async () => {
        set({ is_loading: true, error: null });
        try {
            const live_agents = await load_agents();
            const has_alpha = live_agents.some(a => a.id === '1');
            const final_agents = has_alpha 
                ? live_agents 
                : [normalize_agent(mock_agents[0] as unknown as Raw_Agent), ...live_agents];

            set({ agents: final_agents, is_loading: false });
            broadcast_agent_sync({ type: 'agents:replace', payload: final_agents });
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            set({ error: message, is_loading: false });
            event_bus.emit_log({
                source: 'System',
                text: `Agent Registry Failure: ${message}`,
                severity: 'error'
            });
        }
    },

    update_agent: async (id: string, updates: Partial<Agent>) => {
        set((state: Agent_State) => ({
            agents: state.agents.map(a => a.id === id ? { ...a, ...updates } : a)
        }));
        broadcast_agent_sync({
            type: 'agent:update',
            payload: { id, updates }
        });

        try {
            await persist_agent_update(id, updates);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            event_bus.emit_log({
                source: 'System',
                text: `Agent Persistence Failed: ${message}`,
                severity: 'warning'
            });
        }
    },

    add_agent: async (agent: Agent) => {
        set((state: Agent_State) => ({
            agents: [...state.agents, agent]
        }));
        broadcast_agent_sync({ type: 'agent:add', payload: agent });
        try {
            await agent_api_service.create_agent(agent);
            return true;
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            const agents = await load_agents();
            set({ agents, error: message });
            broadcast_agent_sync({ type: 'agents:replace', payload: agents });

            event_bus.emit_log({
                source: 'System',
                text: `Agent Registration Blocked: ${message}`,
                severity: 'error'
            });
            return false;
        }
    },

    get_agent: (id: string) => get().agents.find(a => a.id === id),

    init_telemetry: () => {
        const path_cache = new Map<string, string>();

        const unsubscribe = tadpole_os_socket.subscribe_agent_updates((event) => {
            if ((event.type === 'agent:update' || event.type === 'agent:create') && event.agent_id && event.data) {
                const id_str = String(event.agent_id);
                
                let workspace_path = path_cache.get(id_str);
                if (!workspace_path) {
                    const workspace_store = use_workspace_store.getState();
                    const cluster = workspace_store.clusters.find(c => 
                        c.collaborators.map(String).includes(id_str)
                    );
                    workspace_path = cluster ? cluster.path : `/workspaces/agent-silo-${id_str}`;
                    path_cache.set(id_str, workspace_path);
                }
                
                const final_work_path = workspace_path;
                
                set((state: Agent_State) => {
                    const existing = state.agents.find(a => a.id === (event.agent_id as string));
                    
                    if (existing) {
                        return {
                            agents: state.agents.map(a => {
                                if (a.id === event.agent_id) {
                                    const mergedRaw = { ...a, ...event.data } as unknown as Raw_Agent;
                                    if (!Object.prototype.hasOwnProperty.call(event.data as Record<string, unknown>, 'current_task')) {
                                        delete mergedRaw.current_task;
                                    }
                                    return normalize_agent(mergedRaw, final_work_path, a);
                                }
                                return a;
                            } )
                        };
                    } else {
                        const normalized = normalize_agent({ ...event.data, id: event.agent_id } as unknown as Raw_Agent, final_work_path);
                        return { agents: [...state.agents, normalized] };
                    }
                });
            } else if (event.type === 'engine:ui_invalidate') {
                if (event.resource === 'agents') {
                    get().fetch_agents();
                }
            }
        });

        return unsubscribe;
    }
}));

if (sync_channel) {
    sync_channel.onmessage = (event) => {
        const message = event.data as Agent_Sync_Message;
        if (!message || message.source_id === TAB_ID) {
            return;
        }

        is_applying_remote_sync = true;
        try {
            if (message.type === 'agent:update') {
                use_agent_store.setState((state: Agent_State) => ({
                    agents: state.agents.map((agent) =>
                        agent.id === message.payload.id
                            ? { ...agent, ...message.payload.updates }
                            : agent
                    )
                }));
            } else if (message.type === 'agent:add') {
                use_agent_store.setState((state: Agent_State) => {
                    if (state.agents.some((agent) => agent.id === message.payload.id)) {
                        return state;
                    }
                    return { agents: [...state.agents, message.payload] };
                });
            } else if (message.type === 'agents:replace') {
                use_agent_store.setState({ agents: message.payload });
            }
        } finally {
            is_applying_remote_sync = false;
        }
    };
}
