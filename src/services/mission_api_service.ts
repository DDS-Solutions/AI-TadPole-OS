/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 * 
 * ### AI Assist Note
 * **Capability Domain Service**: Interface for mission definitions, skill manifests, and MCP tool execution. 
 * Orchestrates the synchronization of YAML/JSON capability definitions with the global Rust forge.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: YAML parsing error (during skill import), MCP handshake timeout, or workflow dependency cycle.
 * - **Telemetry Link**: Search for `[MissionAPI]` or `/v1/skills` in backend audit trails.
 * 
 * @aiContext
 * - **Dependencies**: `base_api_service`.
 * - **Side Effects**: Modifies global mission, skill, and workflow registry in Rust sidecar.
 * - **Mocking**: Mock `api_request` from `base_api_service` for unit tests.
 */


import type { Mission } from '../types/index';
import { api_request } from './base_api_service';
import { track_operation } from '../utils/telemetry';

export interface Skill_Manifest {
    schema_version: string;
    name: string;
    display_name?: string;
    description: string;
    version: string;
    author?: string;
    permissions: string[];
    toolset_group?: string;
    danger_level: 'low' | 'medium' | 'high' | 'critical';
    requires_oversight: boolean;
    category: 'user' | 'ai';
}

export const mission_api_service = {
    /**
     * Synchronizes a mission to an agent's local workspace.
     */
    sync_mission: async (agent_id: string, mission: Mission): Promise<boolean> => {
        return track_operation('MissionAPI', `Synchronizing mission to agent: ${agent_id.toUpperCase()}`, async () => {
            await api_request(`/v1/agents/${agent_id}/mission`, {
                method: 'POST',
                body: JSON.stringify(mission)
            });
            return true;
        }, { agent_id, mission_id: mission.id });
    },

    /**
     * Fetches just the skill manifests (llm-tool schemas).
     */
    get_skill_manifests: async (): Promise<Skill_Manifest[]> => {
        return api_request<Skill_Manifest[]>('/v1/skills/manifests', { method: 'GET' });
    },

    /**
     * Fetches consolidated skills (scripts + manifests + workflows).
     */
    get_unified_skills: async (): Promise<{ manifests: Skill_Manifest[]; scripts: unknown[]; workflows: unknown[] }> => {
        return api_request<{ manifests: Skill_Manifest[]; scripts: unknown[]; workflows: unknown[] }>('/v1/skills', { method: 'GET' });
    },

    /**
     * Saves or updates a dynamic skill script.
     */
    save_skill_script: async (name: string, definition: unknown): Promise<void> => {
        await api_request(`/v1/skills/scripts/${name}`, {
            method: 'PUT',
            body: JSON.stringify(definition)
        });
    },

    /**
     * Deletes a dynamic skill script.
     */
    delete_skill_script: async (name: string): Promise<void> => {
        await api_request(`/v1/skills/scripts/${name}`, { method: 'DELETE' });
    },

    /**
     * Saves or updates a workflow.
     */
    save_workflow: async (name: string, content: unknown): Promise<void> => {
        await api_request(`/v1/skills/workflows/${name}`, {
            method: 'PUT',
            body: JSON.stringify(content)
        });
    },

    /**
     * Deletes a workflow.
     */
    delete_workflow: async (name: string): Promise<void> => {
        await api_request(`/v1/skills/workflows/${name}`, { method: 'DELETE' });
    },

    /**
     * Lists all available MCP tools with telemetry.
     */
    get_mcp_tools: async (): Promise<unknown[]> => {
        return api_request<unknown[]>('/v1/skills/mcp-tools', { method: 'GET' });
    },

    /**
     * Executes a specific MCP tool with given arguments.
     */
    execute_mcp_tool: async (tool_name: string, args: Record<string, unknown>): Promise<unknown> => {
        return track_operation('MissionAPI', `Executing MCP tool: ${tool_name.toUpperCase()}`, async () => {
            return api_request(`/v1/skills/mcp-tools/${tool_name}/execute`, {
                method: 'POST',
                body: JSON.stringify(args)
            });
        }, { metadata: { tool_name } });
    },

    /**
     * Saves or updates a lifecycle hook.
     */
    save_hook: async (name: string, hook: unknown): Promise<void> => {
        await api_request(`/v1/skills/hooks/${encodeURIComponent(name)}`, {
            method: 'PUT',
            body: JSON.stringify(hook)
        });
    },

    /**
     * Deletes a lifecycle hook.
     */
    delete_hook: async (name: string): Promise<void> => {
        await api_request(`/v1/skills/hooks/${encodeURIComponent(name)}`, { method: 'DELETE' });
    }
};


// Metadata: [mission_api_service]

// Metadata: [mission_api_service]
