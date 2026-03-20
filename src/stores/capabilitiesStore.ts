import { TadpoleOSService } from '../services/tadpoleosService';

import { create } from 'zustand';

export interface SkillDefinition {
    name: string;
    description: string;
    execution_command: string;
    schema: Record<string, unknown>;
}

export interface WorkflowDefinition {
    name: string;
    content: string;
}

export interface McpToolStats {
    invocations: number;
    success_count: number;
    failure_count: number;
    avg_latency_ms: number;
}

export interface McpToolHubDefinition {
    name: string;
    description: string;
    input_schema: Record<string, unknown>;
    source: string;
    stats: McpToolStats;
    isPulsing?: boolean;
}

interface CapabilitiesState {
    skills: SkillDefinition[];
    workflows: WorkflowDefinition[];
    mcpTools: McpToolHubDefinition[];
    isLoading: boolean;
    error: string | null;
}

interface CapabilitiesActions {
    fetchCapabilities: () => Promise<void>;
    fetchMcpTools: () => Promise<void>;
    saveSkill: (skill: SkillDefinition) => Promise<void>;
    deleteSkill: (name: string) => Promise<void>;
    saveWorkflow: (workflow: WorkflowDefinition) => Promise<void>;
    deleteWorkflow: (name: string) => Promise<void>;
    handlePulse: (toolName: string, status: 'success' | 'error', latency: number) => void;
}

export const useCapabilitiesStore = create<CapabilitiesState & CapabilitiesActions>()((set, get) => ({
    skills: [],
    workflows: [],
    mcpTools: [],
    isLoading: false,
    error: null,

    fetchMcpTools: async () => {
        set({ isLoading: true, error: null });
        try {
            const data = await TadpoleOSService.getMcpTools();
            set({ mcpTools: data as McpToolHubDefinition[], isLoading: false });
        } catch (_error: unknown) {
            set({ error: _error instanceof Error ? _error.message : String(_error), isLoading: false });
        }
    },

    fetchCapabilities: async () => {
        set({ isLoading: true, error: null });
        try {
            const data = await TadpoleOSService.getCapabilities();

            set({
                skills: ((data.skills || []) as SkillDefinition[]).sort((a: SkillDefinition, b: SkillDefinition) => a.name.localeCompare(b.name)),
                workflows: ((data.workflows || []) as WorkflowDefinition[]).sort((a: WorkflowDefinition, b: WorkflowDefinition) => a.name.localeCompare(b.name)),
                isLoading: false
            });
        } catch (_error: unknown) {
            set({ error: _error instanceof Error ? _error.message : String(_error), isLoading: false });
        }
    },

    saveSkill: async (skill) => {
        await TadpoleOSService.saveSkill(skill.name, skill);
        await get().fetchCapabilities();
    },

    deleteSkill: async (name) => {
        await TadpoleOSService.deleteSkill(name);
        await get().fetchCapabilities();
    },

    saveWorkflow: async (workflow) => {
        await TadpoleOSService.saveWorkflow(workflow.name, workflow);
        await get().fetchCapabilities();
    },

    deleteWorkflow: async (name) => {
        await TadpoleOSService.deleteWorkflow(name);
        await get().fetchCapabilities();
    },

    handlePulse: (toolName, status, latency) => {
        const { mcpTools } = get();
        const updatedTools = mcpTools.map(t => {
            if (t.name === toolName) {
                const newStats = { ...t.stats };
                newStats.invocations += 1;
                if (status === 'success') newStats.success_count += 1;
                else newStats.failure_count += 1;

                // Simplified moving average match with backend
                newStats.avg_latency_ms = newStats.avg_latency_ms === 0 ? latency : Math.round((newStats.avg_latency_ms + latency) / 2);

                return { ...t, stats: newStats, isPulsing: true };
            }
            return t;
        });

        set({ mcpTools: updatedTools });

        // Reset pulsing after 1 second
        setTimeout(() => {
            const currentTools = get().mcpTools;
            set({
                mcpTools: currentTools.map(t =>
                    t.name === toolName ? { ...t, isPulsing: false } : t
                )
            });
        }, 1000);
    }
}));

