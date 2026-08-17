/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Services:Agent**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[agent_service]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Services:Agent
 * 
 * ### AI Assist Note
 * **Agent Swarm Orchestrator**: Hardens the agent lifecycle by extracting side-effects and telemetry.
 * Manages the connection between reactive stores, Socket.io streams, and backend REST persistence.
 */

import { agents as mock_agents } from '../data/mock_agents';
import { agent_api_service } from './agent';
import { system_api_service } from './system_api_service';
import { get_tadpole_os_socket, type Agent_Update_Event } from './socket';
import { log_error } from './system_utils';
import type { 
    Agent, 
    AgentPatch, 
    AgentDto, 
    Department, 
    Agent_Status, 
    Agent_Voice_Engine, 
    Agent_Stt_Engine, 
    Agent_Connector_Config, 
    Agent_Metadata, 
    ModelConfigDto 
} from '../contracts/agent';
import { use_workspace_store } from '../stores/workspace_store';
import { use_agent_store, type Agent_State } from '../stores/agent_store';
import { normalize_agent_dto } from '../domain/agents/normalizers';

export interface AgentPatchPayload {
    name?: string;
    role?: string;
    department?: Department;
    description?: string;
    status?: Agent_Status;
    tokens_used?: number;
    current_task?: string;
    model?: string;
    model_2?: string;
    model_3?: string;
    model_config?: ModelConfigDto;
    model_config2?: ModelConfigDto;
    model_config3?: ModelConfigDto;
    active_model_slot?: 1 | 2 | 3;
    skills?: string[];
    workflows?: string[];
    mcp_tools?: string[];
    theme_color?: string;
    budget_usd?: number;
    cost_usd?: number;
    requires_oversight?: boolean;
    voice_id?: string;
    voice_engine?: Agent_Voice_Engine;
    stt_engine?: Agent_Stt_Engine;
    last_pulse?: string | null;
    created_at?: string;
    input_tokens?: number;
    output_tokens?: number;
    failure_count?: number;
    last_failure_at?: string;
    category?: string;
    connector_configs?: Agent_Connector_Config[];
    metadata?: Agent_Metadata;
    current_reasoning_turn?: number;
    reasoning_depth?: number;
    workspace_path?: string;
    _local_timestamp?: number;
    active_mission?: { 
        id: string;
        objective?: string;
        constraints?: string[];
        priority?: string;
        is_degraded?: boolean;
    };
    valence?: number;
    reports_to?: string;
    [key: string]: unknown;
}

// Removed set_agent_store runtime setter

export type { AgentDto as Raw_Agent } from '../contracts/agent';

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;
const TAB_ID = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `tab-${Date.now()}`;

class Agent_Service {
    private is_applying_remote_sync = false;
    private path_cache = new Map<string, string>();
    private unsubscribe_socket: (() => void) | null = null;
    private unsubscribe_tab: (() => void) | null = null;
    public agent_store: typeof use_agent_store = use_agent_store;

    /**
     * Retrieves the initialized agent store.
     */
    private _get_store_or_warn(action_name: string): typeof import('../stores/agent_store').use_agent_store | null {
        if (!this.agent_store) {
            console.warn(`[AgentService] Recoverable Setup Warning: agent_store is not set during ${action_name}`);
            return null;
        }
        return this.agent_store;
    }

    /**
     * Initializes the service, setting up telemetry and cross-tab synchronization.
     */
    public init(): () => void {
        this._get_store_or_warn('init');
        this.dispose(); // Clean up any existing subscriptions first to avoid duplicates
        this.unsubscribe_socket = this.init_telemetry();
        this.unsubscribe_tab = this.init_tab_sync();

        return () => this.dispose();
    }

    public dispose(): void {
        if (this.unsubscribe_socket) {
            this.unsubscribe_socket();
            this.unsubscribe_socket = null;
        }
        if (this.unsubscribe_tab) {
            this.unsubscribe_tab();
            this.unsubscribe_tab = null;
        }
    }

    private _get_agent_key(data: { agent_id?: string; agentId?: string; id?: string }): string | undefined {
        return data.agent_id || data.agentId || data.id;
    }

    private _normalize_patch(
        raw: AgentPatchPayload,
        source: 'SOCKET' | 'API' | 'SYNC',
        existing?: Agent,
        options?: { workspace_path?: string }
    ): AgentPatchPayload {
        if (source === 'SOCKET') {
            const workspace_path = options?.workspace_path;
            if (workspace_path) {
                return normalize_agent_dto(
                    { ...(existing || {}), ...raw } as unknown as AgentDto,
                    workspace_path
                ) as unknown as AgentPatchPayload;
            }
        }
        return raw;
    }

    private _dispatch_state_update(
        source: 'API' | 'SOCKET' | 'SYNC',
        agent_id: string,
        patch: AgentPatchPayload,
        options?: { workspace_path?: string; timestamp?: number }
    ): void {
        const store = this._get_store_or_warn('_dispatch_state_update');
        if (!store) return;

        store.setState((state: Agent_State) => {
            const agents = state.agents || [];
            const existingIndex = agents.findIndex((a: Agent) => a.id === agent_id);

            if (existingIndex !== -1) {
                const existing = agents[existingIndex];
                
                // For SOCKET events, ignore updates older than local optimistic timestamp
                if (source === 'SOCKET') {
                    const event_time = options?.timestamp || Date.now();
                    if (event_time < (existing._local_timestamp || 0)) return state;
                }

                const updatedAgents = [...agents];
                const normalizedPatch = this._normalize_patch(patch, source, existing, { workspace_path: options?.workspace_path });

                updatedAgents[existingIndex] = {
                    ...existing,
                    ...normalizedPatch
                } as Agent;

                return { agents: updatedAgents };
            } else {
                // If it doesn't exist, we only add it if we can normalize it or if it is a full Agent object
                const normalized = this._normalize_patch(patch, source, undefined, { workspace_path: options?.workspace_path });
                if (source === 'SOCKET' && options?.workspace_path) {
                    return { agents: [...agents, { ...normalized, id: agent_id } as unknown as Agent] };
                } else if (source === 'SYNC') {
                    // TAB SYNC 'agent:add' payload
                    return { agents: [...agents, normalized as unknown as Agent] };
                }
                
                return state;
            }
        });
    }

    private init_tab_sync(): () => void {
        const store = this._get_store_or_warn('init_tab_sync');
        if (!store) return () => {};
        if (!sync_channel) return () => {};

        const on_message = (event: MessageEvent) => {
            const message = event.data;
            if (!message || message.source_id === TAB_ID) return;

            this.is_applying_remote_sync = true;
            try {
                if (message.type === 'agent:update') {
                    this._dispatch_state_update('SYNC', message.payload.id, message.payload.updates);
                } else if (message.type === 'agent:add') {
                    this._dispatch_state_update('SYNC', message.payload.id, message.payload);
                } else if (message.type === 'agents:replace') {
                    store.setState({ agents: message.payload });
                }
            } finally {
                this.is_applying_remote_sync = false;
            }
        };

        if (typeof sync_channel.addEventListener === 'function') {
            sync_channel.addEventListener('message', on_message);
        } else {
            sync_channel.onmessage = on_message;
        }

        const unsubscribe_store = store.subscribe((state: Agent_State, prev: Agent_State) => {
            if (this.is_applying_remote_sync) return;

            // Simplified diffing: if lengths differ or if a full replace happened
            if (state.agents.length !== prev.agents.length) {
                sync_channel?.postMessage({ type: 'agents:replace', payload: state.agents, source_id: TAB_ID });
            }
        });

        return () => {
            if (typeof sync_channel.removeEventListener === 'function') {
                sync_channel.removeEventListener('message', on_message);
            } else {
                sync_channel.onmessage = null;
            }
            unsubscribe_store();
        };
    }

    private init_telemetry(): () => void {
        const store = this._get_store_or_warn('init_telemetry');
        if (!store) return () => {};
        return get_tadpole_os_socket().subscribe('agentUpdates', (event) => {
            const id_str = this._get_agent_key(event);
            if (!event || !id_str || !event.data || event.source_id === TAB_ID) return;

            if (event.type === 'agent:update' || event.type === 'agent:create') {
                let workspace_path = this.path_cache.get(id_str);
                
                if (!workspace_path) {
                    const workspace_store = use_workspace_store.getState();
                    const cluster = (workspace_store.clusters || []).find(c => (c.collaborators || []).includes(id_str));
                    workspace_path = cluster ? cluster.path : `/workspaces/agent-silo-${id_str}`;
                    this.path_cache.set(id_str, workspace_path);
                }

                const event_time = (event as Agent_Update_Event & { timestamp?: number }).timestamp || Date.now();
                this._dispatch_state_update('SOCKET', id_str, event.data as Partial<Agent>, {
                    workspace_path,
                    timestamp: event_time
                });
            } else if (event.type === 'engine:ui_invalidate' && event.resource === 'agents') {
                void this.load_agents_into_store();
            }
        });
    }

    public async fetch_agents_from_api(options: RequestInit = {}): Promise<AgentDto[]> {
        const is_connected = await system_api_service.engine.check_health();
        if (is_connected) {
            return await agent_api_service.get_agents(options);
        }
        return [];
    }

    public async load_agents_into_store(options: RequestInit = {}): Promise<void> {
        const store = this._get_store_or_warn('load_agents_into_store');
        if (!store) return;
        const storeState = store.getState();
        storeState.set_loading(true);
        try {
            const raw_agents = await this.fetch_agents_from_api(options);

            const existing_agents_map = new Map((storeState.agents || []).map(a => [a.id, a]));
            const workspace_path_fn = use_workspace_store.getState().get_agent_path;
            const final_agents = raw_agents.map(raw => {
                const existing = existing_agents_map.get(raw.id);
                return normalize_agent_dto(raw, workspace_path_fn(raw.id), existing);
            });

            // Mock fallback if registry is empty
            if (final_agents.length === 0 && mock_agents.length > 0) {
                final_agents.push(normalize_agent_dto(mock_agents[0] as unknown as AgentDto, '/workspaces/mock'));
            }

            store.setState({ agents: final_agents, is_loading: false });
        } catch (err) {
            log_error('AgentService', 'Registry Load Failure', err);
            storeState.set_loading(false);
        }
    }

    public async update_agent(id: string, updates: Partial<Agent>, local_only: boolean = false): Promise<void> {
        const timestamp = Date.now();
        this._dispatch_state_update('API', id, { ...updates, _local_timestamp: timestamp });

        if (!local_only) {
            try {
                await agent_api_service.update_agent(id, updates as AgentPatch);
                this.broadcast_update(id, updates);
            } catch (err) {
                log_error('AgentService', 'Persistence Failed - Mandated Recovery Protocol initiated', err, 'warning');
                // Trigger self-healing background synchronization reload to correct version mismatch
                void this.load_agents_into_store();
            }
        }
    }

    public async pause_agent(id: string): Promise<boolean> {
        const success = await agent_api_service.pause_agent(id);
        if (success) {
            await this.update_agent(id, { status: 'idle' }, true);
        }
        return success;
    }

    public async resume_agent(id: string): Promise<boolean> {
        const success = await agent_api_service.resume_agent(id);
        if (success) {
            await this.update_agent(id, { status: 'active' }, true);
        }
        return success;
    }

    public async delete_agent(id: string): Promise<boolean> {
        try {
            const success = await agent_api_service.delete_agent(id);
            if (success) {
                use_agent_store.setState(state => ({
                    agents: state.agents.filter(a => a.id !== id)
                }));
            }
            return success;
        } catch (err) {
            log_error('AgentService', 'Failed to delete agent', err, 'warning');
            return false;
        }
    }

    public broadcast_update(id: string, updates: Partial<Agent>) {
        sync_channel?.postMessage({ type: 'agent:update', payload: { id, updates }, source_id: TAB_ID });
    }
}

export const agent_service = new Agent_Service();

// Re-export legacy functions if needed for minimal diffs, but preferably migrate all calls
export const load_agents = (opt?: RequestInit) => agent_service.load_agents_into_store(opt);
export const persist_agent_update = (id: string, up: AgentPatch) => agent_service.update_agent(id, up);
export { normalize_agent_dto as normalize_agent };

// Metadata: [agent_service]
