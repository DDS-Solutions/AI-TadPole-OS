/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 *
 * ### AI Assist Note
 * **Agent Memory**: Vector store CRUD and semantic search for agent memory entries.
 * Normalizes raw entries from the wire format into the domain model.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memory fragmentation during semantic search or 404 on unknown agent.
 * - **Telemetry Link**: Search `[AgentAPI]` in backend tracing.
 */

import type { Agent_Memory_Entry, Raw_Agent_Memory_Entry } from '../../contracts/agent';
import { api_request, map_api_error } from '../base_api_service';
import { normalize_agent_memory_entry } from '../../domain/agents/normalizers';

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

// Metadata: [AgentMemoryService]
