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
 * 
 * ```mermaid
 * stateDiagram-v2
 *     [*] --> Idle
 *     Idle --> Loading: fetch_skills() / fetch_mcp_tools()
 *     Loading --> Error: Fetch Failed
 *     Loading --> Unified_Sync: fetch_unified_skills()
 *     Unified_Sync --> Sorting: sort by name
 *     Sorting --> Ready: set({manifests, scripts, workflows, hooks})
 *     Ready --> Saving: save_skill_script() / save_workflow()
 *     Saving --> Loading: re-fetch_skills()
 *     Ready --> Pulsing: handle_pulse(tool_name)
 *     Pulsing --> State_Update: update stats (invocations, latency)
 *     State_Update --> Ready: timer reset (1s)
 * ```
 */


import { tadpole_os_service } from '../services/tadpoleos_service';
import type { Skill_Manifest } from '../services/mission_api_service';
import { log_error } from '../services/system_utils';
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

/**
 * Common shape for anything that is an "Ability" in the swarm.
 */
export interface Unified_Ability {
    name: string;
    display_name: string;
    description: string;
    type: 'native' | 'script' | 'workflow' | 'mcp';
    category: 'user' | 'ai';
    icon_hint?: string;
    raw: Skill_Manifest | Skill_Definition | Workflow_Definition | Mcp_Tool_Hub_Definition;
}

export interface Skill_State {
    manifests: Skill_Manifest[];
    scripts: Skill_Definition[];
    workflows: Workflow_Definition[];
    hooks: Hook_Definition[];
    mcp_tools: Mcp_Tool_Hub_Definition[];
    is_loading: boolean;
    initialized_skills: boolean;
    initialized_mcp: boolean;
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
    initialized_skills: false,
    initialized_mcp: false,
    error: null,

    fetch_mcp_tools: async () => {
        set({ is_loading: true, error: null });
        try {
            const data = await tadpole_os_service.get_mcp_tools();
            set({ mcp_tools: data as Mcp_Tool_Hub_Definition[], is_loading: false, initialized_mcp: true });
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
                is_loading: false,
                initialized_skills: true
            });
        } catch (_error: unknown) {
            set({ error: _error instanceof Error ? _error.message : String(_error), is_loading: false });
        }
    },

    save_skill_script: async (skill) => {
        try {
            await tadpole_os_service.save_skill_script(skill.name, skill);
            await get().fetch_skills();
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            set({ error: message });
            log_error('SkillStore', 'Skill Save Failed', error);
        }
    },

    delete_skill_script: async (name) => {
        try {
            await tadpole_os_service.delete_skill_script(name);
            await get().fetch_skills();
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            set({ error: message });
            log_error('SkillStore', 'Skill Deletion Failed', error);
        }
    },

    save_workflow: async (workflow) => {
        try {
            await tadpole_os_service.save_workflow(workflow.name, workflow);
            await get().fetch_skills();
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            set({ error: message });
            log_error('SkillStore', 'Workflow Save Failed', error);
        }
    },

    delete_workflow: async (name) => {
        try {
            await tadpole_os_service.delete_workflow(name);
            await get().fetch_skills();
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            set({ error: message });
            log_error('SkillStore', 'Workflow Deletion Failed', error);
        }
    },

    save_hook: async (hook) => {
        try {
            await tadpole_os_service.save_hook(hook.name, hook);
            await get().fetch_skills();
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            set({ error: message });
            log_error('SkillStore', 'Hook Save Failed', error);
        }
    },

    delete_hook: async (name) => {
        try {
            await tadpole_os_service.delete_hook(name);
            await get().fetch_skills();
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            set({ error: message });
            log_error('SkillStore', 'Hook Deletion Failed', error);
        }
    },

    handle_pulse: (tool_name, status, latency) => {
        const { mcp_tools } = get();
        const updated_tools = (mcp_tools || []).map(t => {
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
                mcp_tools: (current_tools || []).map(t =>
                    t.name === tool_name ? { ...t, is_pulsing: false } : t
                )
            });
        }, 1000);
    }
}));


// Metadata: [skill_store]

// Metadata: [skill_store]
