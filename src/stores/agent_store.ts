/**
 * @docs ARCHITECTURE:Stores
 * 
 * ### AI Assist Note
 * **Core functional element for the Tadpole OS engine.**
 * Standardized reactive store for the entire agent swarm. Orchestrates agent lifecycle, persistence, and cross-tab synchronization.
 * 
 * ### 🧬 Logic Flow (Mermaid)
 * ```mermaid
 * stateDiagram-v2
 *     [*] --> Loading: fetch_agents()
 *     Loading --> Ready: Success [Normalize & Sync]
 *     Loading --> Error: Failure [Emit Error]
 *     
 *     state "Agent Synchronization" as Sync {
 *         [*] --> LocalUpdate: update_agent()
 *         LocalUpdate --> BackendPersistence: persist_agent_update()
 *         LocalUpdate --> TabBroadcast: BroadcastChannel (agent:update)
 *         
 *         [*] --> RemoteUpdate: Socket (agent:update)
 *         RemoteUpdate --> LocalState: normalize_agent() [State Merge]
 *     }
 * ```
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path: Runtime logic error or state corruption.**
 * - **Telemetry Link**: Search `[agent_store.ts]` in tracing logs.
 */

import { create } from 'zustand';
import type { Agent } from '../types';
import { load_agents, persist_agent_update, normalize_agent, type Raw_Agent } from '../services/agent_service';
import { agent_api_service } from '../services/agent_api_service';
import { tadpole_os_socket } from '../services/socket';
import { log_error } from '../services/system_utils';
import { use_workspace_store } from './workspace_store';
import { agents as mock_agents } from '../data/mock_agents';

// --- Type Definitions & Constants ---

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;

/** Unique identifier for this browser tab instance. */
const TAB_ID = (typeof crypto !== 'undefined' && crypto.randomUUID)
    ? crypto.randomUUID()
    : `tab-${Date.now()}-${Math.random().toString(16).slice(2)}`;

/** Global guard to prevent recursive state synchronization across tabs. 
 * This prevents "Infinite Broadcast Storms" where Tab A updates Tab B, 
 * which then tries to update Tab A again (SYNC-02).
 */
let is_applying_remote_sync = false;

type Agent_Sync_Message =
    | { type: 'agent:update'; source_id: string; payload: { id: string; updates: Partial<Agent> } }
    | { type: 'agent:add'; source_id: string; payload: Agent }
    | { type: 'agents:replace'; source_id: string; payload: Agent[] };

// --- Utility Functions ---


/**
 * Propagates agent state changes across all browser tabs for UI parity.
 */
const broadcast_agent_sync = (
    message: Omit<Agent_Sync_Message, 'source_id'>,
) => {
    if (!sync_channel || is_applying_remote_sync) return;
    sync_channel.postMessage({ ...message, source_id: TAB_ID });
};

/**
 * Merges an incoming partial update into an existing agent object.
 * @param existing_agent - The current state of the agent.
 * @param updates - The partial update payload from the event.
 * @param work_path - The resolved workspace path for normalization.
 * @returns The newly merged and normalized agent state.
 */
const merge_agent_state = (existing_agent: Agent, updates: Partial<Agent>, work_path: string): Agent => {
    // Ensure the ID is always preserved from the existing domain model 
    // to prevent record loss during partial telemetry updates.
    return normalize_agent({ ...updates, id: existing_agent.id } as Raw_Agent, work_path, existing_agent);
};

// --- Store Implementation ---

export interface Agent_State {
    agents: Agent[];
    is_loading: boolean;
    error: string | null;

    // Actions
    fetch_agents: (options?: RequestInit) => Promise<void>;
    update_agent: (id: string, updates: Partial<Agent>) => Promise<void>;
    add_agent: (agent: Agent) => Promise<boolean>;
    get_agent: (id: string) => Agent | undefined;

    /** Initializes real-time telemetry listeners. Returns an unsubscribe function. */
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

    /// ### 📡 Remote Orchestration: Fetch Discovery
    /// Retrieves the current agent registry from the backend physical persistence.
    /// Implements a **Mock Fallback** mechanism to ensure the UI remains 
    /// interactive even during early-stage infrastructure boot.
    fetch_agents: async (options: RequestInit = {}): Promise<void> => {
        set({ is_loading: true, error: null });
        try {
            const live_agents = (await load_agents(options)) || [];
            
            // Handle Mock Fallback: Ensures 'Alpha' agent is always present 
            // for development parity if the server is in a clean-state.
            const has_alpha = (live_agents || []).some(a => a.id === '1');
            let final_agents: Agent[];

            if (!has_alpha && mock_agents.length > 0) {
                const mock_agent: Agent = normalize_agent(mock_agents[0] as unknown as Raw_Agent);
                final_agents = [mock_agent, ...live_agents];
            } else {
                final_agents = live_agents;
            }

            set({ agents: final_agents, is_loading: false });
            // Propagate the new registry to all other open dashboard portals.
            broadcast_agent_sync({ type: 'agents:replace', payload: final_agents });
        } catch (err: unknown) {
            log_error('AgentStore', 'Agent Registry Failure', err);
            set({ error: 'Failed to load agent registry. Check system logs for details.', is_loading: false });
        }
    },

    update_agent: async (id: string, updates: Partial<Agent>): Promise<void> => {
        const timestamp = Date.now();

        // 1. Optimistic UI Update
        // Transition the UI immediately to ensure "Premium" responsiveness.
        // Persistence happens asynchronously to prevent blocking the UI thread.
        // We attach a local timestamp to prevent stale server events from reverting local changes.
        set((state: Agent_State) => ({
            agents: (state.agents || []).map(a => 
                a.id === id ? { ...a, ...updates, _local_timestamp: timestamp } : a
            )
        }));
        broadcast_agent_sync({
            type: 'agent:update',
            payload: { id, updates }
        });

        // 2. Persistence Call
        try {
            await persist_agent_update(id, updates);
        } catch (err: unknown) {
            log_error('AgentStore', `Persistence Failed for ID ${id}`, err, 'warning');
            // We keep it optimistic to prevent UI flickering, backend retry logic handles real reconciliation
        }
    },

    add_agent: async (agent: Agent): Promise<boolean> => {
        // 1. Optimistic UI Update
        set((state: Agent_State) => ({
            agents: [...(state.agents || []), agent]
        }));
        broadcast_agent_sync({ type: 'agent:add', payload: agent });

        try {
            await agent_api_service.create_agent(agent);
            return true;
        } catch (err: unknown) {
            log_error('AgentStore', 'Agent Registration Blocked', err);

            // Revert on failure
            const agents = await load_agents();
            set({ agents: agents || [], error: (err as Error).message || 'Registration failed' });
            broadcast_agent_sync({ type: 'agents:replace', payload: agents || [] });
            return false;
        }
    },

    get_agent: (id: string): Agent | undefined => (get().agents || []).find(a => a.id === id),

    /// ### 📡 Telemetry Engine: Binary Update Stream
    /// Connects the local reactive state to the engine's real-time telemetry socket.
    /// Ensures that agent capability changes, status shifts, and mission 
    /// progress are reflected with zero manual refresh.
    init_telemetry: (): (() => void) => {
        const path_cache = new Map<string, string>();

        const unsubscribe = tadpole_os_socket.subscribe_agent_updates((event) => {
            // Guard: Ignore self-updates or malformed events (SYNC-02)
            if (!event || !event.agent_id || !event.data || (event as { source_id?: string }).source_id === TAB_ID) {
                return;
            }

            if (event.type === 'agent:update' || event.type === 'agent:create') {
                const id_str = event.agent_id;
                
                // Resolve Workspace Context for normalization
                let workspace_path = path_cache.get(id_str);
                if (!workspace_path) {
                    const workspace_store = use_workspace_store.getState();
                    const cluster = (workspace_store.clusters || []).find(c => {
                        const members = c.collaborators || [];
                        // event.agent_id is already checked to be truthy at line 200
                        return members.includes(id_str) || members.includes(event.agent_id!);
                    });
                    workspace_path = cluster ? cluster.path : `/workspaces/agent-silo-${id_str}`;
                    path_cache.set(id_str, workspace_path);
                }
                
                const final_work_path = workspace_path;
                
                set((state: Agent_State) => {
                    const agents = state.agents || [];
                    const existing = agents.find(a => a.id === id_str);
                    
                    if (existing) {
                        const event_any = (event as unknown) as Record<string, unknown>;
                        const event_time = typeof event_any.timestamp === 'number' ? event_any.timestamp : Date.now();
                        const local_time = existing._local_timestamp || 0;
                        
                        // Staleness Guard: Don't let old server events overwrite fresh optimistic local state (SYNC-03)
                        if (event_time < local_time) {
                            return state;
                        }

                        const updates = (event.data || {}) as Partial<Agent>;
                        const new_agent = merge_agent_state(existing, updates, final_work_path);
                        return {
                            agents: agents.map(a => a.id === id_str ? new_agent : a)
                        };
                    } else {
                        // Discovery: Register a new agent detected in the swarm
                        const normalized = normalize_agent({ ...event.data, id: id_str } as unknown as Raw_Agent, final_work_path);
                        return { agents: [...agents, normalized] };
                    }
                });
            } else if (event.type === 'engine:ui_invalidate') {
                // Cold-path: Trigger a full refresh if the backend signals 
                // a structural registry change.
                if (event.resource === 'agents') {
                    get().fetch_agents();
                }
            }
        });

        return unsubscribe;
    }
}));

// --- Global Sync Handler ---

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
                    agents: (state.agents || []).map((agent) =>
                        agent.id === message.payload.id
                            ? { ...agent, ...message.payload.updates }
                            : agent
                    )
                }));
            } else if (message.type === 'agent:add') {
                use_agent_store.setState((state: Agent_State) => {
                    const agents = state.agents || [];
                    if (agents.some((agent) => agent.id === message.payload.id)) {
                        return state;
                    }
                    return { agents: [...agents, message.payload] };
                });
            } else if (message.type === 'agents:replace') {
                use_agent_store.setState({ agents: message.payload });
            }
        } finally {
            is_applying_remote_sync = false;
        }
    };
}

// Metadata: [agent_store]

// Metadata: [agent_store]
