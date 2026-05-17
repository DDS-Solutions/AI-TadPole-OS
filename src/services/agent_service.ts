/**
 * @docs ARCHITECTURE:Services:Agent
 * 
 * ### AI Assist Note
 * **Agent Swarm Orchestrator**: Hardens the agent lifecycle by extracting side-effects and telemetry.
 * Manages the connection between reactive stores, Socket.io streams, and backend REST persistence.
 */

import { agents as mock_agents } from '../data/mock_agents';
import { agent_api_service } from './agent_api_service';
import { system_api_service } from './system_api_service';
import { tadpole_os_socket } from './socket';
import { log_error } from './system_utils';
import type { Agent, AgentPatch, AgentDto } from '../contracts/agent';
import { use_workspace_store } from '../stores/workspace_store';
import { use_agent_store } from '../stores/agent_store';
import { normalize_agent_dto } from '../domain/agents/normalizers';

export type { AgentDto as Raw_Agent } from '../contracts/agent';

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;
const TAB_ID = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `tab-${Date.now()}`;

class Agent_Service {
    private is_applying_remote_sync = false;
    private path_cache = new Map<string, string>();

    /**
     * Initializes the service, setting up telemetry and cross-tab synchronization.
     */
    public init(): () => void {
        const unsubscribe_socket = this.init_telemetry();
        const unsubscribe_tab = this.init_tab_sync();

        return () => {
            unsubscribe_socket();
            unsubscribe_tab();
        };
    }

    private init_tab_sync(): () => void {
        if (!sync_channel) return () => {};

        const on_message = (event: MessageEvent) => {
            const message = event.data;
            if (!message || message.source_id === TAB_ID) return;

            this.is_applying_remote_sync = true;
            try {
                if (message.type === 'agent:update') {
                    use_agent_store.setState((state) => ({
                        agents: (state.agents || []).map((agent) =>
                            agent.id === message.payload.id
                                ? { ...agent, ...message.payload.updates }
                                : agent
                        )
                    }));
                } else if (message.type === 'agent:add') {
                    use_agent_store.setState((state) => {
                        const agents = state.agents || [];
                        if (agents.some((agent) => agent.id === message.payload.id)) return state;
                        return { agents: [...agents, message.payload] };
                    });
                } else if (message.type === 'agents:replace') {
                    use_agent_store.setState({ agents: message.payload });
                }
            } finally {
                this.is_applying_remote_sync = false;
            }
        };

        sync_channel.addEventListener('message', on_message);

        const unsubscribe_store = use_agent_store.subscribe((state, prev) => {
            if (this.is_applying_remote_sync) return;

            // Simplified diffing: if lengths differ or if a full replace happened
            if (state.agents.length !== prev.agents.length) {
                sync_channel?.postMessage({ type: 'agents:replace', payload: state.agents, source_id: TAB_ID });
            }
        });

        return () => {
            sync_channel.removeEventListener('message', on_message);
            unsubscribe_store();
        };
    }

    private init_telemetry(): () => void {
        return tadpole_os_socket.subscribe_agent_updates((event) => {
            if (!event || !event.agent_id || !event.data || event.source_id === TAB_ID) return;

            if (event.type === 'agent:update' || event.type === 'agent:create') {
                const id_str = event.agent_id;
                let workspace_path = this.path_cache.get(id_str);
                
                if (!workspace_path) {
                    const workspace_store = use_workspace_store.getState();
                    const cluster = (workspace_store.clusters || []).find(c => (c.collaborators || []).includes(id_str));
                    workspace_path = cluster ? cluster.path : `/workspaces/agent-silo-${id_str}`;
                    this.path_cache.set(id_str, workspace_path);
                }

                use_agent_store.setState((state) => {
                    const existing = state.agents.find(a => a.id === id_str);
                    if (existing) {
                        const event_time = (event as Agent_Update_Event & { timestamp?: number }).timestamp || Date.now();
                        if (event_time < (existing._local_timestamp || 0)) return state;

                        const updates = event.data as Partial<Agent>;
                        return {
                            agents: state.agents.map(a => a.id === id_str ? normalize_agent_dto({ ...a, ...updates } as unknown as AgentDto, workspace_path!) : a)
                        };
                    } else {
                        const normalized = normalize_agent_dto({ ...event.data, id: id_str } as unknown as AgentDto, workspace_path!);
                        return { agents: [...state.agents, normalized] };
                    }
                });
            } else if (event.type === 'engine:ui_invalidate' && event.resource === 'agents') {
                void this.load_agents_into_store();
            }
        });
    }

    public async load_agents_into_store(options: RequestInit = {}): Promise<void> {
        const store = use_agent_store.getState();
        store.set_loading(true);
        try {
            const is_connected = await system_api_service.check_health();
            let raw_agents: AgentDto[] = [];

            if (is_connected) {
                raw_agents = await agent_api_service.get_agents(options);
            }

            const workspace_path_fn = use_workspace_store.getState().get_agent_path;
            const final_agents = raw_agents.map(raw => normalize_agent_dto(raw, workspace_path_fn(raw.id)));

            // Mock fallback if registry is empty
            if (final_agents.length === 0 && mock_agents.length > 0) {
                final_agents.push(normalize_agent_dto(mock_agents[0] as unknown as AgentDto, '/workspaces/mock'));
            }

            use_agent_store.setState({ agents: final_agents, is_loading: false });
        } catch (err) {
            log_error('AgentService', 'Registry Load Failure', err);
            store.set_loading(false);
        }
    }

    public async update_agent(id: string, updates: Partial<Agent>, local_only: boolean = false): Promise<void> {
        const timestamp = Date.now();
        use_agent_store.setState(state => ({
            agents: state.agents.map(a => a.id === id ? { ...a, ...updates, _local_timestamp: timestamp } : a)
        }));

        if (!local_only) {
            try {
                await agent_api_service.update_agent(id, updates as AgentPatch);
                this.broadcast_update(id, updates);
            } catch (err) {
                log_error('AgentService', 'Persistence Failed', err, 'warning');
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

    public broadcast_update(id: string, updates: Partial<Agent>) {
        sync_channel?.postMessage({ type: 'agent:update', payload: { id, updates }, source_id: TAB_ID });
    }
}

export const agent_service = new Agent_Service();

// Re-export legacy functions if needed for minimal diffs, but preferably migrate all calls
export const load_agents = (opt?: RequestInit) => agent_service.load_agents_into_store(opt);
export const persist_agent_update = (id: string, up: AgentPatch) => agent_service.update_agent(id, up);
export { normalize_agent_dto as normalize_agent };
