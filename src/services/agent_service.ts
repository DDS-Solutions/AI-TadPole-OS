/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Assist Note
 * - Review changes preserve origin security and propagate confirmed operation outcomes.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / agent_service
 * - **Primary Entrypoints**: `agent_service`, `load_agents`, `persist_agent_update`, `AgentPatchPayload`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[AgentService]`
 */

import { agent_api_service } from './agent';
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
import { reduce_agent_state } from '../domain/agents/state_reducer';

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
    _server_timestamp?: number;
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

export type { AgentDto as Raw_Agent } from '../contracts/agent';

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;
const TAB_ID = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `tab-${Date.now()}`;

function is_valid_agent(a: unknown): a is Agent {
    if (!a || typeof a !== 'object') return false;
    const candidate = a as Agent;
    return typeof candidate.id === 'string' && typeof candidate.name === 'string';
}

/**
 * Rolls back individual fields to previous snapshot if they still hold unconfirmed local write.
 */
function rollback_agent_fields(
    current: Agent,
    previous: Agent,
    updates: Partial<Agent>,
    timestamp: number
): Agent {
    if (current._local_timestamp !== timestamp) return current;

    const restored: Agent = { ...current, _local_timestamp: previous._local_timestamp };
    for (const key of Object.keys(updates) as (keyof Agent)[]) {
        if (Object.is(current[key], updates[key])) {
            Object.assign(restored, { [key]: previous[key] });
        }
    }
    return restored;
}

class Agent_Service {
    private unsubscribe_socket: (() => void) | null = null;
    private unsubscribe_tab: (() => void) | null = null;
    private is_applying_remote_sync = false;
    private path_cache = new Map<string, string>();
    public agent_store: typeof use_agent_store = use_agent_store;

    public clear_path_cache(agent_id?: string): void {
        if (agent_id) {
            this.path_cache.delete(agent_id);
        } else {
            this.path_cache.clear();
        }
    }

    private get_agent_key(data: { agent_id?: string; agentId?: string; id?: string }): string | undefined {
        return data.agent_id || data.agentId || data.id;
    }

    /**
     * Retrieves the initialized agent store with a warning if unavailable.
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

    private _dispatch_state_update(
        source: 'API' | 'SOCKET' | 'SYNC',
        agent_id: string,
        patch: AgentPatchPayload,
        options?: { workspace_path?: string; timestamp?: number }
    ): void {
        const store = this._get_store_or_warn('_dispatch_state_update');
        if (!store) return;
        const action = {
            type: 'patch' as const,
            id: agent_id,
            patch: patch as Partial<Agent>,
            source,
            workspace: options?.workspace_path,
            timestamp: options?.timestamp
        };
        store.setState(state => ({
            agents: reduce_agent_state(state.agents || [], action)
        }));
    }

    private handle_tab_sync_message(message: unknown, store: typeof use_agent_store): void {
        if (!message || typeof message !== 'object') return;
        const msg = message as { type?: string; source_id?: string; payload?: unknown };
        if (msg.source_id === TAB_ID) return;

        this.is_applying_remote_sync = true;
        try {
            const { type, payload } = msg;
            const payload_obj = payload && typeof payload === 'object' && !Array.isArray(payload) ? (payload as Record<string, unknown>) : undefined;
            const has_valid_id = typeof payload_obj?.id === 'string';

            if (type === 'agent:update' && has_valid_id && payload_obj) {
                this._dispatch_state_update('SYNC', payload_obj.id as string, payload_obj.updates as Partial<Agent>);
                return;
            }
            if (type === 'agent:add' && has_valid_id && payload_obj) {
                this._dispatch_state_update('SYNC', payload_obj.id as string, payload_obj as unknown as Partial<Agent>);
                return;
            }
            if (type === 'agents:replace' && Array.isArray(payload)) {
                store.setState({ agents: payload.filter(is_valid_agent) });
            }
        } finally {
            this.is_applying_remote_sync = false;
        }
    }

    public init_tab_sync(): () => void {
        const store = this._get_store_or_warn('init_tab_sync');
        if (!store || !sync_channel) return () => {};

        const on_message = (event: MessageEvent) => {
            this.handle_tab_sync_message(event.data, store);
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

    private get_or_resolve_workspace_path(id_str: string): string {
        const cached = this.path_cache.get(id_str);
        if (cached) return cached;

        const workspace_store = use_workspace_store.getState();
        const cluster = (workspace_store.clusters || []).find(c => (c.collaborators || []).includes(id_str));
        const workspace_path = cluster ? cluster.path : `/workspaces/agent-silo-${id_str}`;
        this.path_cache.set(id_str, workspace_path);
        return workspace_path;
    }

    private handle_agent_update_event(event: Agent_Update_Event, id_str: string): void {
        const workspace_path = this.get_or_resolve_workspace_path(id_str);
        const event_time = (event as Agent_Update_Event & { timestamp?: number }).timestamp || Date.now();
        this._dispatch_state_update('SOCKET', id_str, event.data as Partial<Agent>, {
            workspace_path,
            timestamp: event_time
        });
    }

    public init_telemetry(): () => void {
        const store = this._get_store_or_warn('init_telemetry');
        if (!store) return () => {};

        return get_tadpole_os_socket().subscribe('agentUpdates', (event) => {
            if (!event || event.source_id === TAB_ID) return;
            if (event.type === 'engine:ui_invalidate' && event.resource === 'agents') {
                agent_api_service.invalidate_cache();
                this.clear_path_cache();
                void this.load_agents_into_store();
                return;
            }

            const id_str = this.get_agent_key(event);
            if (!id_str || !event.data) return;

            if (event.type === 'agent:update' || event.type === 'agent:create') {
                this.handle_agent_update_event(event, id_str);
            }
        });
    }

    public async fetch_agents_from_api(options: RequestInit = {}): Promise<AgentDto[]> {
        return await agent_api_service.get_agents(options);
    }

    public async load_agents_into_store(options: RequestInit = {}): Promise<void> {
        const store = this._get_store_or_warn('load_agents_into_store');
        if (!store) return;
        const storeState = store.getState();
        storeState.set_loading(true);
        try {
            const raw_agents = await this.fetch_agents_from_api(options);

            if (options.signal?.aborted) return;

            const nextAgents = reduce_agent_state(store.getState().agents || [], {
                type: 'snapshot',
                agents: raw_agents,
                workspace: use_workspace_store.getState().get_agent_path
            });
            store.setState({ agents: nextAgents, is_loading: false, error: null });
        } catch (err) {
            if (options.signal?.aborted) return;
            log_error('AgentService', 'Registry Load Failure', err);
            const err_msg = 'Failed to load agent registry. The last known agents are still shown.';
            if (typeof storeState?.set_error === 'function') {
                storeState.set_error(err_msg);
            } else if (typeof store?.setState === 'function') {
                store.setState({ error: err_msg });
            }
        } finally {
            if (typeof storeState?.set_loading === 'function') {
                storeState.set_loading(false);
            } else if (typeof store?.setState === 'function') {
                store.setState({ is_loading: false });
            }
        }
    }

    public async update_agent(id: string, updates: Partial<Agent>, local_only: boolean = false): Promise<boolean> {
        const store = this._get_store_or_warn('update_agent');
        const previous = store?.getState().agents.find(a => a.id === id);
        const timestamp = Math.max(Date.now(), (previous?._local_timestamp ?? 0) + 1);
        this._dispatch_state_update('API', id, { ...updates, _local_timestamp: timestamp });

        if (local_only) return true;

        try {
            await agent_api_service.update_agent(id, updates as AgentPatch);
            this.broadcast_update(id, updates);
            if (store) {
                store.setState(state => ({
                    agents: state.agents.map(a => a.id === id && a._local_timestamp === timestamp ? { ...a, _local_timestamp: undefined } : a)
                }));
            }
            if (typeof store?.getState()?.set_error === 'function') {
                store.getState().set_error(null);
            } else if (typeof store?.setState === 'function') {
                store.setState({ error: null });
            }
            return true;
        } catch (err) {
            log_error('AgentService', 'Persistence Failed - Mandated Recovery Protocol initiated', err, 'warning');
            // Roll back only fields still holding this write; preserve newer socket or user changes.
            if (previous && store) {
                store.setState(state => ({
                    agents: state.agents.map(a => a.id === id ? rollback_agent_fields(a, previous, updates, timestamp) : a)
                }));
            }
            const err_msg = 'Failed to save agent. Your changes were not confirmed.';
            if (typeof store?.getState()?.set_error === 'function') {
                store.getState().set_error(err_msg);
            } else if (typeof store?.setState === 'function') {
                store.setState({ error: err_msg });
            }
            agent_api_service.invalidate_cache();
            return false;
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
        const store = this._get_store_or_warn('delete_agent');
        try {
            const success = await agent_api_service.delete_agent(id);
            if (!success) {
                store?.setState({ error: 'Failed to delete agent.' });
                return false;
            }

            store?.setState(state => ({
                agents: state.agents.filter(a => a.id !== id),
                error: null
            }));
            use_workspace_store.setState(state => ({
                clusters: (state.clusters || []).map(c => ({
                    ...c,
                    collaborators: (c.collaborators || []).filter(cid => cid !== id),
                    alpha_id: c.alpha_id === id ? undefined : c.alpha_id
                }))
            }));
            this.clear_path_cache(id);
            return true;
        } catch (err) {
            log_error('AgentService', 'Failed to delete agent', err, 'warning');
            store?.setState({ error: 'Failed to delete agent. Check system logs for details.' });
            return false;
        }
    }

    public broadcast_update(id: string, updates: Partial<Agent>): void {
        sync_channel?.postMessage({ type: 'agent:update', payload: { id, updates }, source_id: TAB_ID });
    }

    /**
     * Compiles a Socratic Context Envelope for task preview and pre-cleared gate dispatch.
     */
    public build_socratic_envelope(
        agent: Agent,
        mission_vector: string,
        target_paths?: string[]
    ): {
        status: string;
        scope_contract: Record<string, unknown>;
        performance_threshold: Record<string, unknown>;
        architecture_mode: Record<string, unknown>;
        pre_cleared_failure_modes: string[];
    } {
        const slot = Number(agent.active_model_slot || 2);
        const latency_target = slot === 2 ? 1500 : 10000;
        const paths = target_paths && target_paths.length > 0 ? target_paths : ['src/', 'server-rs/src/'];

        const roleLower = (agent.role || '').toLowerCase();
        const vectorLower = (mission_vector || '').toLowerCase();
        const isSecurityOrAudit = vectorLower.includes('audit') || vectorLower.includes('security') ||
            roleLower.includes('security') || roleLower.includes('auditor') || roleLower.includes('qa');

        const mode_name = isSecurityOrAudit
            ? 'Nexus Engineer Mode (Zero Trust Auditor + Principal QA)'
            : 'Sovereign 3-Layer Architecture (Directives -> Orchestration -> Execution)';

        return {
            status: 'PRE_CLEARED_GATE_PASS',
            scope_contract: {
                target_paths: paths,
                blast_radius_level: 2,
                mutation_allowed: true,
                database_mutation_allowed: false,
                mission_vector
            },
            performance_threshold: {
                budget_usd_cap: agent.budget_usd || 1.0,
                max_swarm_depth: 5,
                target_turn_latency_ms: latency_target,
                active_model_slot: slot,
                timeout_seconds: 300
            },
            architecture_mode: {
                mode_name,
                privacy_shield_enforced: true,
                standards_compliance: ['Zero Trust', 'L1/L2/L3 agentskills.io', 'DESIGN.md'],
                assigned_persona: agent.role
            },
            pre_cleared_failure_modes: [
                'Circuit Breaker: Halt after 3 non-convergent iterations as Logic-Blocker (Directive #3).',
                'Air-Gap Shield: Local Ollama fallback strictly enforced if cloud providers blocked (PRIVACY_MODE).',
                'Tool Execution Rule: Check execution/ for existing tools before writing ad-hoc scripts (Layer 1/2/3).',
                'Verification Gate: Require automated validation pass (parity_guard / vitest) before state commit.'
            ]
        };
    }

    /**
     * Retrieves the complete swarm knowledge graph for visualization.
     */
    public async get_swarm_graph(options?: RequestInit) {
        return agent_api_service.get_swarm_graph(options);
    }
}

export const agent_service = new Agent_Service();

// Re-export legacy functions if needed for minimal diffs, but preferably migrate all calls
export const load_agents = (opt?: RequestInit) => agent_service.load_agents_into_store(opt);
export const persist_agent_update = (id: string, up: AgentPatch) => agent_service.update_agent(id, up);
export const get_swarm_graph = (opt?: RequestInit) => agent_service.get_swarm_graph(opt);
export { normalize_agent_dto as normalize_agent };
