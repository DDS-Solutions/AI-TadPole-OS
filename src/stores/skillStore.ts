import { TadpoleOSService } from '../services/tadpoleosService';
import type { SkillManifest } from '../services/MissionApiService';
import { create } from 'zustand';

export interface SkillDefinition {
    name: string;
    description: string;
    execution_command: string;
    schema: Record<string, unknown>;
    category: 'user' | 'ai';
}

export interface WorkflowDefinition {
    name: string;
    content: string;
    category: 'user' | 'ai';
}

export interface HookDefinition {
    name: string;
    description: string;
    hook_type: string;
    content: string;
    active: boolean;
    category: 'user' | 'ai';
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
    category: 'user' | 'ai';
    isPulsing?: boolean;
}

interface SkillState {
    manifests: SkillManifest[];
    scripts: SkillDefinition[];
    workflows: WorkflowDefinition[];
    hooks: HookDefinition[];
    mcpTools: McpToolHubDefinition[];
    isLoading: boolean;
    error: string | null;
}

interface SkillActions {
    fetchSkills: () => Promise<void>;
    fetchMcpTools: () => Promise<void>;
    saveSkillScript: (skill: SkillDefinition) => Promise<void>;
    deleteSkillScript: (name: string) => Promise<void>;
    saveWorkflow: (workflow: WorkflowDefinition) => Promise<void>;
    deleteWorkflow: (name: string) => Promise<void>;
    saveHook: (hook: HookDefinition) => Promise<void>;
    deleteHook: (name: string) => Promise<void>;
    handlePulse: (toolName: string, status: 'success' | 'error', latency: number) => void;
}

export const useSkillStore = create<SkillState & SkillActions>()((set, get) => ({
    manifests: [],
    scripts: [],
    workflows: [],
    hooks: [],
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

    fetchSkills: async () => {
        set({ isLoading: true, error: null });
        try {
            const data = await TadpoleOSService.getUnifiedSkills();

            const pData = data as { manifests?: SkillManifest[]; scripts?: SkillDefinition[]; workflows?: WorkflowDefinition[]; hooks?: HookDefinition[] };

            set({
                manifests: (pData.manifests || []).sort((a, b) => a.name.localeCompare(b.name)),
                scripts: (pData.scripts || []).sort((a, b) => a.name.localeCompare(b.name)),
                workflows: (pData.workflows || []).sort((a, b) => a.name.localeCompare(b.name)),
                hooks: (pData.hooks || []).sort((a, b) => a.name.localeCompare(b.name)),
                isLoading: false
            });
        } catch (_error: unknown) {
            set({ error: _error instanceof Error ? _error.message : String(_error), isLoading: false });
        }
    },

    saveSkillScript: async (skill) => {
        await TadpoleOSService.saveSkillScript(skill.name, skill);
        await get().fetchSkills();
    },

    deleteSkillScript: async (name) => {
        await TadpoleOSService.deleteSkillScript(name);
        await get().fetchSkills();
    },

    saveWorkflow: async (workflow) => {
        await TadpoleOSService.saveWorkflow(workflow.name, workflow);
        await get().fetchSkills();
    },

    deleteWorkflow: async (name) => {
        await TadpoleOSService.deleteWorkflow(name);
        await get().fetchSkills();
    },

    saveHook: async (hook) => {
        await TadpoleOSService.saveHook(hook.name, hook);
        await get().fetchSkills();
    },

    deleteHook: async (name) => {
        await TadpoleOSService.deleteHook(name);
        await get().fetchSkills();
    },

    handlePulse: (toolName, status, latency) => {
        const { mcpTools } = get();
        const updatedTools = mcpTools.map(t => {
            if (t.name === toolName) {
                const newStats = { ...t.stats };
                newStats.invocations += 1;
                if (status === 'success') newStats.success_count += 1;
                else newStats.failure_count += 1;

                // Simple moving average match with backend
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
