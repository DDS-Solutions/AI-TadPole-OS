import type { Mission } from '../types/index';
import { apiRequest } from './BaseApiService';

export interface SkillManifest {
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

export const MissionApiService = {
    /**
     * Synchronizes a mission to an agent's local workspace.
     */
    syncMission: async (agentId: string, mission: Mission): Promise<boolean> => {
        await apiRequest(`/v1/agents/${agentId}/mission`, {
            method: 'POST',
            body: JSON.stringify(mission)
        });
        return true;
    },

    /**
     * Fetches just the skill manifests (llm-tool schemas).
     */
    getSkillManifests: async (): Promise<SkillManifest[]> => {
        return apiRequest<SkillManifest[]>('/v1/skills/manifests', { method: 'GET' });
    },

    /**
     * Fetches consolidated skills (scripts + manifests + workflows).
     */
    getUnifiedSkills: async (): Promise<{ manifests: SkillManifest[]; scripts: unknown[]; workflows: unknown[] }> => {
        return apiRequest<{ manifests: SkillManifest[]; scripts: unknown[]; workflows: unknown[] }>('/v1/skills', { method: 'GET' });
    },

    /**
     * Saves or updates a dynamic skill script.
     */
    saveSkillScript: async (name: string, definition: unknown): Promise<void> => {
        await apiRequest(`/v1/skills/scripts/${name}`, {
            method: 'PUT',
            body: JSON.stringify(definition)
        });
    },

    /**
     * Deletes a dynamic skill script.
     */
    deleteSkillScript: async (name: string): Promise<void> => {
        await apiRequest(`/v1/skills/scripts/${name}`, { method: 'DELETE' });
    },

    /**
     * Saves or updates a workflow.
     */
    saveWorkflow: async (name: string, content: unknown): Promise<void> => {
        await apiRequest(`/v1/skills/workflows/${name}`, {
            method: 'PUT',
            body: JSON.stringify(content)
        });
    },

    /**
     * Deletes a workflow.
     */
    deleteWorkflow: async (name: string): Promise<void> => {
        await apiRequest(`/v1/skills/workflows/${name}`, { method: 'DELETE' });
    },

    /**
     * Lists all available MCP tools with telemetry.
     */
    getMcpTools: async (): Promise<unknown[]> => {
        return apiRequest<unknown[]>('/v1/skills/mcp-tools', { method: 'GET' });
    },

    /**
     * Executes a specific MCP tool with given arguments.
     */
    executeMcpTool: async (toolName: string, args: Record<string, unknown>): Promise<unknown> => {
        return apiRequest(`/v1/skills/mcp-tools/${toolName}/execute`, {
            method: 'POST',
            body: JSON.stringify(args)
        });
    },

    /**
     * Saves or updates a lifecycle hook.
     */
    saveHook: async (name: string, hook: unknown): Promise<void> => {
        await apiRequest(`/v1/skills/hooks/${encodeURIComponent(name)}`, {
            method: 'PUT',
            body: JSON.stringify(hook)
        });
    },

    /**
     * Deletes a lifecycle hook.
     */
    deleteHook: async (name: string): Promise<void> => {
        await apiRequest(`/v1/skills/hooks/${encodeURIComponent(name)}`, { method: 'DELETE' });
    }
};
