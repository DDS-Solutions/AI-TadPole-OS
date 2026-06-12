/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 * 
 * ### AI Assist Note
 * **Agent Domain Service**: Dedicated interface for agent lifecycle management, task dispatching, and vector memory operations. 
 * Implements Maturity Level 3 HATEOAS envelopes for paginated agent lists and secure API key injection via `NeuralVault`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: 429 Rate Limit (governed by `model_store` RPM/TPM), Vault lock-out (prevents task dispatch), or memory fragmentation during semantic search.
 * - **Telemetry Link**: Look for `X-Request-Id` in backend logs or search `[AgentAPI]` in backend tracing.
 * 
 * @aiContext
 * - **Dependencies**: `base_api_service`, `vault_store`, `model_store`, `provider_store`.
 * - **Side Effects**: Modifies global agent registry in Rust sidecar.
 * - **Mocking**: Mock `api_request` from `base_api_service` for unit tests.
 */

import type { 
    Agent, 
    AgentPatch, 
    AgentDto, 
    Task_Payload,
    Agent_Memory_Entry,
    Raw_Agent_Memory_Entry
} from '../contracts/agent';
import { api_request, map_api_error, ValidationError } from './base_api_service';
import type { Skill_Definition, Workflow_Definition, Hook_Definition } from '../stores/skill_store';
import { PROVIDERS } from '../constants';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store, type Model_Entry } from '../stores/model_store';
import { event_bus } from './event_bus';
import { track_operation } from '../utils/telemetry';
import { serialize_agent_update } from '../domain/agents/serializers';
import { serialize_role } from '../domain/roles/normalizer';
import type { Role } from '../contracts/role/domain';
import { normalize_agent_memory_entry } from '../domain/agents/normalizers';

export interface DispatchCommandInput {
    agent_id: string;
    message: string;
    model_id: string;
    provider: string;
    cluster_id?: string;
    department?: string;
    budget_usd?: number;
    external_id?: string;
    safe_mode?: boolean;
    analysis?: boolean;
    request_id?: string;
}

export class AgentRegistryService {
    private readonly api_request_fn: typeof api_request;
    private agents_cache_promise: Promise<AgentDto[]> | null = null;

    constructor(api_request_fn: typeof api_request) {
        this.api_request_fn = api_request_fn;
    }

    public invalidate_agents_cache(): void {
        this.agents_cache_promise = null;
    }

    public get_agents(options: RequestInit = {}): Promise<AgentDto[]> {
        const signal = options.signal || undefined;
        const promise = this.get_agents_internal(signal);

        if (!signal) {
            return promise;
        }

        if (signal.aborted) {
            return Promise.reject(new DOMException('The user aborted a request.', 'AbortError'));
        }

        return new Promise<AgentDto[]>((resolve, reject) => {
            const onAbort = () => {
                reject(new DOMException('The user aborted a request.', 'AbortError'));
            };

            signal.addEventListener('abort', onAbort);

            promise.then(
                (res) => {
                    signal.removeEventListener('abort', onAbort);
                    resolve(res);
                },
                (err) => {
                    signal.removeEventListener('abort', onAbort);
                    reject(err);
                }
            );
        });
    }

    private get_agents_internal(signal?: AbortSignal): Promise<AgentDto[]> {
        if (this.agents_cache_promise) {
            return this.agents_cache_promise;
        }

        this.agents_cache_promise = track_operation('AgentAPI', 'Fetching agent registry...', async () => {
            try {
                type Agent_List_Envelope = { data?: AgentDto[] } | AgentDto[];
                const result = await this.api_request_fn<Agent_List_Envelope>('/v1/agents?per_page=500', { 
                    method: 'GET',
                    signal
                });

                if (result && typeof result === 'object' && !Array.isArray(result) && 'data' in result) {
                    return result.data ?? [];
                }

                return Array.isArray(result) ? result : [];
            } catch (err) {
                this.agents_cache_promise = null;
                throw map_api_error(err);
            }
        });

        return this.agents_cache_promise;
    }

    public async update_agent(agent_id: string, patch: AgentPatch): Promise<boolean> {
        return track_operation('AgentAPI', `Updating configuration for agent: ${agent_id.toUpperCase()}`, async () => {
            try {
                const body = serialize_agent_update(patch);
                await this.api_request_fn(`/v1/agents/${agent_id}`, {
                    method: 'PUT',
                    body: JSON.stringify(body)
                });
                this.invalidate_agents_cache();
                return true;
            } catch (err) {
                throw map_api_error(err);
            }
        });
    }

    public async create_agent(agent: Agent): Promise<boolean> {
        try {
            const body = {
                ...serialize_agent_update(agent),
                id: agent.id,
                description: agent.description || "New Agent Node",
                status: agent.status || "idle",
                created_at: agent.created_at || new Date().toISOString(),
            };

            await this.api_request_fn('/v1/agents', {
                method: 'POST',
                body: JSON.stringify(body)
            });
            this.invalidate_agents_cache();
            return true;
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async pause_agent(agent_id: string): Promise<boolean> {
        try {
            await this.api_request_fn(`/v1/agents/${agent_id}/pause`, { method: 'POST' });
            this.invalidate_agents_cache();
            return true;
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async resume_agent(agent_id: string): Promise<boolean> {
        try {
            await this.api_request_fn(`/v1/agents/${agent_id}/resume`, { method: 'POST' });
            this.invalidate_agents_cache();
            return true;
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async reset_agent(agent_id: string): Promise<{ status: string; message: string }> {
        try {
            const result = await this.api_request_fn<{ status: string; message: string }>(`/v1/agents/${agent_id}/reset`, {
                method: 'POST'
            });
            this.invalidate_agents_cache();
            return result;
        } catch (err) {
            throw map_api_error(err);
        }
    }
}

export class AgentTaskDispatchService {
    private readonly api_request_fn: typeof api_request;
    private readonly vault_store: typeof use_vault_store;
    private readonly model_store: typeof use_model_store;
    private readonly provider_store: typeof use_provider_store;
    private readonly event_bus_inst: typeof event_bus;

    constructor(
        api_request_fn: typeof api_request,
        vault_store: typeof use_vault_store,
        model_store: typeof use_model_store,
        provider_store: typeof use_provider_store,
        event_bus_inst: typeof event_bus
    ) {
        this.api_request_fn = api_request_fn;
        this.vault_store = vault_store;
        this.model_store = model_store;
        this.provider_store = provider_store;
        this.event_bus_inst = event_bus_inst;
    }

    public async checkPrerequisites(provider: string, agent_id: string): Promise<{ provider_api_key: string | null; is_actually_locked: boolean; warning?: string }> {
        const vault_store_state = this.vault_store.getState();
        const provider_api_key = await vault_store_state.get_api_key(provider);
        const is_actually_locked = !vault_store_state.is_unlocked();
        const is_local = provider === PROVIDERS.OLLAMA || provider === PROVIDERS.LOCAL;

        let warning: string | undefined;
        if (!provider_api_key && !is_local) {
            const reason = is_actually_locked ? 'Vault is Locked' : `No Key for ${provider.toUpperCase()}`;
            warning = `🔒 Neural Security: ${reason} for ${agent_id.toUpperCase()}.`;
        }

        return { provider_api_key, is_actually_locked, warning };
    }

    public buildCommandPayload(
        message: string,
        model_id: string,
        provider: string,
        provider_api_key: string | null,
        cluster_id?: string,
        department?: string,
        budget_usd?: number,
        external_id?: string,
        safe_mode?: boolean,
        analysis?: boolean
    ): Task_Payload {
        const body: Task_Payload = { message, cluster_id, department, provider, model_id, budget_usd, external_id, safe_mode, analysis };

        if (provider_api_key) {
            body.api_key = provider_api_key;
            const model_store_state = this.model_store.getState();
            const inventory_model = model_store_state.models.find((m: Model_Entry) => m.name === model_id);
            if (inventory_model) {
                if (inventory_model.rpm) body.rpm = inventory_model.rpm;
                if (inventory_model.tpm) body.tpm = inventory_model.tpm;
                if (inventory_model.rpd) body.rpd = inventory_model.rpd;
                if (inventory_model.tpd) body.tpd = inventory_model.tpd;
            }
        }

        const base_url = this.provider_store.getState().base_urls[provider];
        if (base_url) {
            body.base_url = base_url;
        }

        return body;
    }

    public async send_command(input: DispatchCommandInput): Promise<boolean> {
        return track_operation('AgentAPI', `Dispatching command to agent: ${input.agent_id.toUpperCase()}`, async () => {
            try {
                const { provider_api_key, warning } = await this.checkPrerequisites(input.provider, input.agent_id);
                if (warning) {
                    this.event_bus_inst.emit_log({
                        source: 'System',
                        text: warning,
                        severity: 'warning'
                    });
                }
                const body = this.buildCommandPayload(
                    input.message,
                    input.model_id,
                    input.provider,
                    provider_api_key,
                    input.cluster_id,
                    input.department,
                    input.budget_usd,
                    input.external_id,
                    input.safe_mode,
                    input.analysis
                );

                await this.api_request_fn(`/v1/agents/${input.agent_id}/tasks`, {
                    method: 'POST',
                    body: JSON.stringify(body),
                    headers: input.request_id ? { 'X-Request-Id': input.request_id } : undefined
                });

                return true;
            } catch (err) {
                throw map_api_error(err);
            }
        }, { agent_id: input.agent_id, mission_id: input.cluster_id });
    }
}

export class AgentMemoryService {
    private readonly api_request_fn: typeof api_request;

    constructor(api_request_fn: typeof api_request) {
        this.api_request_fn = api_request_fn;
    }

    public async get_agent_memory(agent_id: string): Promise<{ status: string; entries: Agent_Memory_Entry[] }> {
        try {
            const result = await this.api_request_fn<{ status: string; entries: Raw_Agent_Memory_Entry[] }>(`/v1/agents/${agent_id}/memories`, { method: 'GET' });
            return {
                ...result,
                entries: (result.entries ?? []).map(normalize_agent_memory_entry),
            };
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async delete_agent_memory(agent_id: string, row_id: string): Promise<{ status: string }> {
        try {
            return await this.api_request_fn<{ status: string }>(`/v1/agents/${agent_id}/memories/${row_id}`, { method: 'DELETE' });
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async save_agent_memory(agent_id: string, text: string): Promise<{ status: string; id: string }> {
        try {
            return await this.api_request_fn<{ status: string; id: string }>(`/v1/agents/${agent_id}/memories`, {
                method: 'POST',
                body: JSON.stringify({ text })
            });
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async search_memory(query: string, agent_id?: string): Promise<{ status: string; entries: Agent_Memory_Entry[] }> {
        try {
            let path = `/v1/search/memory?query=${encodeURIComponent(query)}`;
            if (agent_id) {
                path += `&agent_id=${encodeURIComponent(agent_id)}`;
            }
            const result = await this.api_request_fn<{ status: string; entries: Raw_Agent_Memory_Entry[] }>(path, {
                method: 'GET'
            });
            return {
                ...result,
                entries: (result.entries ?? []).map(normalize_agent_memory_entry),
            };
        } catch (err) {
            throw map_api_error(err);
        }
    }
}

export class CapabilityRegistryService {
    private readonly api_request_fn: typeof api_request;

    constructor(api_request_fn: typeof api_request) {
        this.api_request_fn = api_request_fn;
    }

    public async import_capability(file: File): Promise<{ type: string; data: Skill_Definition | Workflow_Definition | Hook_Definition; preview: string }> {
        if (file.size > 5 * 1024 * 1024) {
            throw new ValidationError(
                'File size exceeds maximum allowed limit of 5MB.',
                'about:blank',
                400
            );
        }
        const name = file.name.toLowerCase();
        if (!name.endsWith('.json') && !name.endsWith('.yaml') && !name.endsWith('.yml')) {
            throw new ValidationError(
                'Invalid file type. Only .json, .yaml, and .yml capability blueprints are allowed.',
                'about:blank',
                400
            );
        }

        try {
            const form_data = new FormData();
            form_data.append('file', file);
            return await this.api_request_fn('/v1/skills/import', {
                method: 'POST',
                body: form_data,
            });
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async register_capability(type: string, data: Skill_Definition | Workflow_Definition | Hook_Definition, category: string): Promise<{ status: string; name: string }> {
        try {
            return await this.api_request_fn('/v1/skills/register', {
                method: 'POST',
                body: JSON.stringify({ type, data, category })
            });
        } catch (err) {
            throw map_api_error(err);
        }
    }
}

export class GovernanceService {
    private readonly api_request_fn: typeof api_request;

    constructor(api_request_fn: typeof api_request) {
        this.api_request_fn = api_request_fn;
    }

    public async save_role_blueprint(blueprint: Role): Promise<boolean> {
        try {
            await this.api_request_fn('/v1/governance/blueprints', {
                method: 'POST',
                body: JSON.stringify(serialize_role(blueprint))
            });
            return true;
        } catch (err) {
            throw map_api_error(err);
        }
    }
}

export class AgentApiService {
    private readonly registry_service: AgentRegistryService;
    private readonly task_dispatch_service: AgentTaskDispatchService;
    private readonly memory_service: AgentMemoryService;
    private readonly capability_service: CapabilityRegistryService;
    private readonly governance_service: GovernanceService;

    constructor(
        api_request_fn: typeof api_request,
        vault_store: typeof use_vault_store,
        model_store: typeof use_model_store,
        provider_store: typeof use_provider_store,
        event_bus_inst: typeof event_bus
    ) {
        this.registry_service = new AgentRegistryService(api_request_fn);
        this.task_dispatch_service = new AgentTaskDispatchService(api_request_fn, vault_store, model_store, provider_store, event_bus_inst);
        this.memory_service = new AgentMemoryService(api_request_fn);
        this.capability_service = new CapabilityRegistryService(api_request_fn);
        this.governance_service = new GovernanceService(api_request_fn);

        // Bind public methods to prevent context loss (e.g. when called via tadpole_os_service proxy getters)
        this.invalidate_cache = this.invalidate_cache.bind(this);
        this.get_agents = this.get_agents.bind(this);
        this.update_agent = this.update_agent.bind(this);
        this.create_agent = this.create_agent.bind(this);
        this.pause_agent = this.pause_agent.bind(this);
        this.resume_agent = this.resume_agent.bind(this);
        this.send_command = this.send_command.bind(this);
        this.get_agent_memory = this.get_agent_memory.bind(this);
        this.delete_agent_memory = this.delete_agent_memory.bind(this);
        this.save_agent_memory = this.save_agent_memory.bind(this);
        this.save_role_blueprint = this.save_role_blueprint.bind(this);
        this.reset_agent = this.reset_agent.bind(this);
        this.import_capability = this.import_capability.bind(this);
        this.register_capability = this.register_capability.bind(this);
        this.search_memory = this.search_memory.bind(this);
    }

    public invalidate_cache(): void {
        this.registry_service.invalidate_agents_cache();
    }

    public get_agents(options: RequestInit = {}): Promise<AgentDto[]> {
        return this.registry_service.get_agents(options);
    }

    public update_agent(agent_id: string, patch: AgentPatch): Promise<boolean> {
        return this.registry_service.update_agent(agent_id, patch);
    }

    public create_agent(agent: Agent): Promise<boolean> {
        return this.registry_service.create_agent(agent);
    }

    public pause_agent(agent_id: string): Promise<boolean> {
        return this.registry_service.pause_agent(agent_id);
    }

    public resume_agent(agent_id: string): Promise<boolean> {
        return this.registry_service.resume_agent(agent_id);
    }

    public reset_agent(agent_id: string): Promise<{ status: string; message: string }> {
        return this.registry_service.reset_agent(agent_id);
    }

    public get_agent_memory(agent_id: string): Promise<{ status: string; entries: Agent_Memory_Entry[] }> {
        return this.memory_service.get_agent_memory(agent_id);
    }

    public delete_agent_memory(agent_id: string, row_id: string): Promise<{ status: string }> {
        return this.memory_service.delete_agent_memory(agent_id, row_id);
    }

    public save_agent_memory(agent_id: string, text: string): Promise<{ status: string; id: string }> {
        return this.memory_service.save_agent_memory(agent_id, text);
    }

    public search_memory(query: string, agent_id?: string): Promise<{ status: string; entries: Agent_Memory_Entry[] }> {
        return this.memory_service.search_memory(query, agent_id);
    }

    public import_capability(file: File): Promise<{ type: string; data: Skill_Definition | Workflow_Definition | Hook_Definition; preview: string }> {
        return this.capability_service.import_capability(file);
    }

    public register_capability(type: string, data: Skill_Definition | Workflow_Definition | Hook_Definition, category: string): Promise<{ status: string; name: string }> {
        return this.capability_service.register_capability(type, data, category);
    }

    public save_role_blueprint(blueprint: Role): Promise<boolean> {
        return this.governance_service.save_role_blueprint(blueprint);
    }

    public send_command(input: DispatchCommandInput): Promise<boolean>;
    public send_command(
        agent_id: string,
        message: string,
        model_id: string,
        provider: string,
        cluster_id?: string,
        department?: string,
        budget_usd?: number,
        external_id?: string,
        safe_mode?: boolean,
        analysis?: boolean,
        request_id?: string
    ): Promise<boolean>;
    public send_command(
        first: string | DispatchCommandInput,
        message?: string,
        model_id?: string,
        provider?: string,
        cluster_id?: string,
        department?: string,
        budget_usd?: number,
        external_id?: string,
        safe_mode?: boolean,
        analysis?: boolean,
        request_id?: string
    ): Promise<boolean> {
        if (typeof first === 'object' && first !== null) {
            return this.task_dispatch_service.send_command(first);
        }
        return this.task_dispatch_service.send_command({
            agent_id: first,
            message: message!,
            model_id: model_id!,
            provider: provider!,
            cluster_id,
            department,
            budget_usd,
            external_id,
            safe_mode,
            analysis,
            request_id
        });
    }
}

export const agent_api_service = new AgentApiService(
    api_request,
    use_vault_store,
    use_model_store,
    use_provider_store,
    event_bus
);

// Metadata: [agent_api_service]

// Metadata: [agent_api_service]
