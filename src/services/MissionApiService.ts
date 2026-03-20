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
}

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
     * Fetches all registered skills with their rich manifests.
     */
    getSkills: async (): Promise<SkillManifest[]> => {
        return apiRequest<SkillManifest[]>('/v1/skills', { method: 'GET' });
    },

    /**
     * Fetches consolidated capabilities (skills + workflows).
     */
    getCapabilities: async (): Promise<{ skills: unknown[]; workflows: unknown[] }> => {
        return apiRequest<{ skills: unknown[]; workflows: unknown[] }>('/v1/capabilities', { method: 'GET' });
    },

    /**
     * Saves or updates a skill manifest.
     */
    saveSkill: async (name: string, manifest: unknown): Promise<void> => {
        await apiRequest(`/v1/capabilities/skills/${name}`, {
            method: 'PUT',
            body: JSON.stringify(manifest)
        });
    },

    /**
     * Deletes a skill manifest.
     */
    deleteSkill: async (name: string): Promise<void> => {
        await apiRequest(`/v1/capabilities/skills/${name}`, { method: 'DELETE' });
    },

    /**
     * Saves or updates a workflow.
     */
    saveWorkflow: async (name: string, manifest: unknown): Promise<void> => {
        await apiRequest(`/v1/capabilities/workflows/${name}`, {
            method: 'PUT',
            body: JSON.stringify(manifest)
        });
    },

    /**
     * Deletes a workflow.
     */
    deleteWorkflow: async (name: string): Promise<void> => {
        await apiRequest(`/v1/capabilities/workflows/${name}`, { method: 'DELETE' });
    },

    /**
     * Lists all available MCP tools with telemetry.
     */
    getMcpTools: async (): Promise<unknown[]> => {
        return apiRequest<unknown[]>('/v1/capabilities/mcp-tools', { method: 'GET' });
    },

    /**
     * Executes a specific MCP tool with given arguments.
     */
    executeMcpTool: async (toolName: string, args: Record<string, unknown>): Promise<unknown> => {
        return apiRequest(`/v1/capabilities/mcp-tools/${toolName}/execute`, {
            method: 'POST',
            body: JSON.stringify(args)
        });
    }
};
