/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Capability Forge backend and MCP tool registry. 
 * Orchestrates the management of Skills, Workflows, Hooks, and MCP server handshakes for the autonomous swarm.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Hook trigger failure (regex mismatch), MCP server handshake timeout, or workflow sequence ordering corruption.
 * - **Telemetry Link**: Search for `[SkillStore]` or `FORGE_SYNC` in UI/Service logs.
 */

import { tadpole_os_service } from '../services/tadpoleos_service';
import type { Skill_Manifest } from '../services/mission_api_service';
import { create } from 'zustand';

export interface Skill_Definition {
    name: string;
    description: string;
    execution_command: string;
    schema: Record<string, unknown>;
    category: 'user' | 'ai';
}

export interface Workflow_Definition {
    name: string;
    content: string;
    category: 'user' | 'ai';
}

export interface Hook_Definition {
    name: string;
    description: string;
    hook_type: string;
    content: string;
    active: boolean;
    category: 'user' | 'ai';
}

export interface Mcp_Tool_Stats {
    invocations: number;
    success_count: number;
    failure_count: number;
    avg_latency_ms: number;
}

export interface Mcp_Tool_Hub_Definition {
    name: string;
    description: string;
    input_schema: Record<string, unknown>;
    source: string;
    stats: Mcp_Tool_Stats;
    category: 'user' | 'ai';
    is_pulsing?: boolean;
}

export interface Skill_State {
    manifests: Skill_Manifest[];
    scripts: Skill_Definition[];
    workflows: Workflow_Definition[];
    hooks: Hook_Definition[];
    mcp_tools: Mcp_Tool_Hub_Definition[];
    is_loading: boolean;
    error: string | null;
}

export interface Skill_Actions {
    fetch_skills: () => Promise<void>;
    fetch_mcp_tools: () => Promise<void>;
    save_skill_script: (skill: Skill_Definition) => Promise<void>;
    delete_skill_script: (name: string) => Promise<void>;
    save_workflow: (workflow: Workflow_Definition) => Promise<void>;
    delete_workflow: (name: string) => Promise<void>;
    save_hook: (hook: Hook_Definition) => Promise<void>;
    delete_hook: (name: string) => Promise<void>;
    handle_pulse: (tool_name: string, status: 'success' | 'error', latency: number) => void;
}

export const use_skill_store = create<Skill_State & Skill_Actions>()((set, get) => ({
    manifests: [],
    scripts: [],
    workflows: [],
    hooks: [],
    mcp_tools: [],
    is_loading: false,
    error: null,

    fetch_mcp_tools: async () => {
        set({ is_loading: true, error: null });
        try {
            const data = await tadpole_os_service.get_mcp_tools();
            set({ mcp_tools: data as Mcp_Tool_Hub_Definition[], is_loading: false });
        } catch (_error: unknown) {
            set({ error: _error instanceof Error ? _error.message : String(_error), is_loading: false });
        }
    },

    fetch_skills: async () => {
        set({ is_loading: true, error: null });
        try {
            const data = await tadpole_os_service.get_unified_skills();

            const p_data = data as { manifests?: Skill_Manifest[]; scripts?: Skill_Definition[]; workflows?: Workflow_Definition[]; hooks?: Hook_Definition[] };

            set({
                manifests: (p_data.manifests || []).sort((a, b) => a.name.localeCompare(b.name)),
                scripts: (p_data.scripts || []).sort((a, b) => a.name.localeCompare(b.name)),
                workflows: (p_data.workflows || []).sort((a, b) => a.name.localeCompare(b.name)),
                hooks: (p_data.hooks || []).sort((a, b) => a.name.localeCompare(b.name)),
                is_loading: false
            });
        } catch (_error: unknown) {
            set({ error: _error instanceof Error ? _error.message : String(_error), is_loading: false });
        }
    },

    save_skill_script: async (skill) => {
        await tadpole_os_service.save_skill_script(skill.name, skill);
        await get().fetch_skills();
    },

    delete_skill_script: async (name) => {
        await tadpole_os_service.delete_skill_script(name);
        await get().fetch_skills();
    },

    save_workflow: async (workflow) => {
        await tadpole_os_service.save_workflow(workflow.name, workflow);
        await get().fetch_skills();
    },

    delete_workflow: async (name) => {
        await tadpole_os_service.delete_workflow(name);
        await get().fetch_skills();
    },

    save_hook: async (hook) => {
        await tadpole_os_service.save_hook(hook.name, hook);
        await get().fetch_skills();
    },

    delete_hook: async (name) => {
        await tadpole_os_service.delete_hook(name);
        await get().fetch_skills();
    },

    handle_pulse: (tool_name, status, latency) => {
        const { mcp_tools } = get();
        const updated_tools = mcp_tools.map(t => {
            if (t.name === tool_name) {
                const new_stats = { ...t.stats };
                new_stats.invocations += 1;
                if (status === 'success') new_stats.success_count += 1;
                else new_stats.failure_count += 1;

                // Simple moving average match with backend
                new_stats.avg_latency_ms = new_stats.avg_latency_ms === 0 ? latency : Math.round((new_stats.avg_latency_ms + latency) / 2);

                return { ...t, stats: new_stats, is_pulsing: true };
            }
            return t;
        });

        set({ mcp_tools: updated_tools });

        // Reset pulsing after 1 second
        setTimeout(() => {
            const current_tools = get().mcp_tools;
            set({
                mcp_tools: current_tools.map(t =>
                    t.name === tool_name ? { ...t, is_pulsing: false } : t
                )
            });
        }, 1000);
    }
}));

