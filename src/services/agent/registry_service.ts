/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 *
 * ### AI Assist Note
 * **Agent Registry**: Handles agent lifecycle CRUD (list, create, update, pause, resume, reset)
 * and a deduplicated request cache to prevent redundant fetches on concurrent mounts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: 429 Rate Limit or cache invalidation race on rapid updates.
 * - **Telemetry Link**: Search `[AgentAPI]` in backend tracing.
 */

import type { Agent, AgentPatch, AgentDto } from '../../contracts/agent';
import { api_request, map_api_error } from '../base_api_service';
import { track_operation } from '../../utils/telemetry';
import { serialize_agent_update } from '../../domain/agents/serializers';

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

// Metadata: [AgentRegistryService]
