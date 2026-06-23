/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 *
 * ### AI Assist Note
 * **AgentApiService Facade**: Composes the five agent sub-services into a single stable
 * interface. All callers import from `agent_api_service` — this module is the
 * implementation behind that stable path.
 *
 * Sub-services:
 * - `AgentRegistryService`   — CRUD lifecycle (list, create, update, pause, resume, reset)
 * - `AgentTaskDispatchService` — Command dispatch with vault/key pre-flight checks
 * - `AgentMemoryService`     — Vector store CRUD and semantic search
 * - `CapabilityRegistryService` — Skill/workflow/hook blueprint import & register
 * - `GovernanceService`      — Role blueprint persistence
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: 429 Rate Limit, Vault lock-out, or memory fragmentation.
 * - **Telemetry Link**: Search `[AgentAPI]` in backend tracing.
 *
 * @aiContext
 * - **Dependencies**: `base_api_service`, `vault_store`, `model_store`, `provider_store`.
 * - **Side Effects**: Modifies global agent registry in Rust sidecar.
 * - **Mocking**: Mock `api_request` from `base_api_service` for unit tests.
 */

import { api_request } from '../base_api_service';
import { use_vault_store } from '../../stores/vault_store';
import { use_model_store } from '../../stores/model_store';
import { use_provider_store } from '../../stores/provider_store';
import { event_bus } from '../event_bus';

import { AgentRegistryService } from './registry_service';
import { AgentTaskDispatchService, type DispatchCommandInput } from './dispatch_service';
import { AgentMemoryService } from './memory_service';
import { CapabilityRegistryService } from './capability_service';
import { GovernanceService } from './governance_service';

import type { Agent, AgentPatch, AgentDto, Agent_Memory_Entry } from '../../contracts/agent';
import type { Skill_Definition, Workflow_Definition, Hook_Definition } from '../../stores/skill_store';
import type { Role } from '../../contracts/role/domain';

export type { DispatchCommandInput };
export { AgentRegistryService, AgentTaskDispatchService, AgentMemoryService, CapabilityRegistryService, GovernanceService };

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
