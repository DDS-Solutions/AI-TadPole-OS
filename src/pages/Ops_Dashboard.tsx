/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Ops_Dashboard
 * - **Primary Entrypoints**: `Ops_Dashboard`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[OpsDashboard]`
 * - **Witness Tests**: none declared
 */

import { useState, useCallback, useMemo } from 'react';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { resolve_provider, resolve_agent_model_config, parse_active_model_slot } from '../utils/model_utils';
import { event_bus } from '../services/event_bus';
import { useDashboardData } from '../hooks/use_dashboard_data';
import { use_dropdown_store, type Dropdown_State } from '../stores/dropdown_store';
import { use_role_store } from '../stores/role_store';
import { use_tab_store } from '../stores/tab_store';
import { use_agent_store } from '../stores/agent_store';
import { i18n } from '../i18n';

import TerminalComponent from '../components/Terminal';
import AgentConfigPanel from '../components/AgentConfigPanel';
import Error_Boundary from '../components/Error_Boundary';

import { Stat_Metrics } from '../components/dashboard/Stat_Metrics';
import { Agent_Status_Grid } from '../components/dashboard/Agent_Status_Grid';
import { Portal_Window } from '../components/ui';
import { ExternalLink } from 'lucide-react';
import type { Agent } from '../types';

const DEFAULT_MODEL = 'gemini-1.5-flash';
const brand_color = '#10b981';

const SLOT_MAP = {
    1: { model: 'model' as const, config: 'model_config' as const },
    2: { model: 'model_2' as const, config: 'model_config2' as const },
    3: { model: 'model_3' as const, config: 'model_config3' as const },
} as const;

/**
 * Ops_Dashboard
 * 
 * The central command-and-control center for the Tadpole OS agent swarm.
 */
export default function Ops_Dashboard() {
    const {
        agents_list, active_agents, online_count, total_cost, total_tokens, 
        total_input_tokens, total_output_tokens, total_budget, budget_util,
        assigned_agent_ids, available_roles,
        clusters, toggle_cluster_active, update_agent, add_agent, delete_agent, recruit_velocity
    } = useDashboardData();

    const { is_agent_grid_detached, toggle_agent_grid_detachment } = use_tab_store();

    const [config_agent_id, set_config_agent_id] = useState<string | null>(null);

    const close_dropdowns = use_dropdown_store((s: Dropdown_State) => s.close_dropdown);

    // Memoized agent lookup for P3 performance O(1) rendering support
    const agentMap = useMemo(() => new Map(agents_list.map(a => [a.id, a])), [agents_list]);
    const effective_config_agent_id = useMemo(() => {
        if (!config_agent_id || config_agent_id === 'new') return config_agent_id;
        return agentMap.has(config_agent_id) ? config_agent_id : null;
    }, [agentMap, config_agent_id]);

    // ── Memoized Callbacks to prevent sub-component re-renders ──────────────────────

    const handle_agent_update = useCallback((id: string, updates: Partial<Agent>) => {
        update_agent(id, updates);
    }, [update_agent]);

    const handle_create_agent = useCallback(async (params: Partial<Agent>) => {
        try {
            // Generate standard cryptographic / randomized safe ID
            const new_id = `agent-${crypto.randomUUID ? crypto.randomUUID().substring(0, 8) : Date.now().toString(36)}`;
            const new_agent: Agent = {
                id: new_id,
                name: params.name || i18n.t('ops.placeholder_name'),
                role: params.role || 'assistant',
                department: params.department || 'Operations',
                status: 'idle',
                tokens_used: 0,
                model: params.model || DEFAULT_MODEL,
                skills: params.skills || [],
                workflows: params.workflows || [],
                cost_usd: 0,
                budget_usd: params.budget_usd || 0,
                theme_color: params.theme_color || brand_color,
                last_pulse: new Date().toISOString(),
                category: 'user',
                ...params
            };

            const success = await add_agent(new_agent);
            if (success) {
                event_bus.emit_log({ text: i18n.t('ops.event_agent_init', { name: new_agent.name }), severity: 'success', source: 'System' });
            }
        } catch (error) {
            console.error('Failed to create agent:', error);
            event_bus.emit_log({ text: i18n.t('ops.event_agent_fail'), severity: 'error', source: 'System' });
        }
    }, [add_agent]);

    const handle_role_change = useCallback((agent_id: string, new_role: string) => {
        const roles = use_role_store.getState().roles;
        const new_actions = roles[new_role] || { skills: [], workflows: [] };
        handle_agent_update(agent_id, {
            role: new_role,
            skills: new_actions.skills,
            workflows: new_actions.workflows
        });
    }, [handle_agent_update]);

    const handle_skill_trigger = useCallback(async (agent_id: string, skill: string, slot?: 1 | 2 | 3) => {
        const initialAgent = agentMap.get(agent_id);
        if (!initialAgent) return;

        const target_slot = slot !== undefined ? slot : parse_active_model_slot(initialAgent.active_model_slot);

        // Optimistic UI state transition
        update_agent(agent_id, {
            status: 'active' as const,
            current_task: i18n.t('ops.event_executing', { skill }),
            active_model_slot: target_slot
        });

        try {
            // Hot lookup directly from store state to avoid closures stale-state bugs
            const freshAgent = use_agent_store.getState().agents.find(a => a.id === agent_id) || initialAgent;
            const targetAgent = { ...freshAgent, active_model_slot: target_slot };
            
            const { model_id, provider } = resolve_agent_model_config(targetAgent);

            const agent_cluster = clusters.find(c => c.collaborators.includes(agent_id));
            const success = await tadpole_os_service.send_command(
                agent_id, 
                skill, 
                model_id, 
                provider, 
                agent_cluster?.id, 
                freshAgent.department, 
                agent_cluster?.budget_usd
            );

            if (!success) {
                update_agent(agent_id, { status: 'idle' });
            }
        } catch (e) {
            console.error("❌ [OpsDashboard] Failed to trigger skill:", e);
            update_agent(agent_id, { status: 'idle' });
            event_bus.emit_log({
                text: i18n.t('ops.event_trigger_fail', { skill, name: initialAgent.name, error: String(e) }),
                severity: 'error',
                source: 'System'
            });
        }
    }, [agentMap, clusters, update_agent]);

    // Parameterized Model Change Handler to completely deduplicate code blocks
    const handle_model_change_by_slot = useCallback((agent_id: string, new_model: string, slot: 1 | 2 | 3 = 1) => {
        const agent = agentMap.get(agent_id);
        if (!agent) return;
        const provider = resolve_provider(new_model);

        const { model: modelKey, config: configKey } = SLOT_MAP[slot];

        const currentConfig = agent[configKey];
        const model_config = currentConfig
            ? { ...currentConfig, modelId: new_model, provider }
            : { modelId: new_model, provider };

        handle_agent_update(agent_id, {
            [modelKey]: new_model,
            [configKey]: model_config
        });
    }, [agentMap, handle_agent_update]);

    const handle_model_change = useCallback((agent_id: string, new_model: string) => {
        handle_model_change_by_slot(agent_id, new_model, 1);
    }, [handle_model_change_by_slot]);

    const handle_model_2_change = useCallback((agent_id: string, new_model: string) => {
        handle_model_change_by_slot(agent_id, new_model, 2);
    }, [handle_model_change_by_slot]);

    const handle_model_3_change = useCallback((agent_id: string, new_model: string) => {
        handle_model_change_by_slot(agent_id, new_model, 3);
    }, [handle_model_change_by_slot]);

    const handle_configure_click = useCallback((id: string) => {
        set_config_agent_id(id);
    }, []);

    const handle_delete_agent = useCallback(async (id: string) => {
        try {
            await delete_agent(id);
            set_config_agent_id(null);
        } catch (error) {
            console.error('Failed to delete agent:', error);
        }
    }, [delete_agent]);

    const config_agent = useMemo(() => {
        if (!effective_config_agent_id || effective_config_agent_id === 'new') return undefined;
        return agentMap.get(effective_config_agent_id);
    }, [effective_config_agent_id, agentMap]);

    // SEO / GEO Static optimization script
    const seo_script = useMemo(() => JSON.stringify({
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "Tadpole OS Operations Dashboard",
        "description": "Central command-and-control center for real-time telemetry, swarm visualization, and multi-agent task dispatching. Integrated operational oversight.",
        "author": { "@type": "Organization", "name": "Sovereign Engineering" },
        "applicationCategory": "Control Center",
        "operatingSystem": "Tadpole OS"
    }), []);

    return (
        <Error_Boundary>
            <div className="flex flex-col h-full gap-6">
                <script 
                    type="application/ld+json"
                    dangerouslySetInnerHTML={{ __html: seo_script }} // security-scan:ignore
                />
                <h1 className="sr-only">Tadpole OS Operations Command Center</h1>
                <h2 className="sr-only">Swarm Telemetry Visualization</h2>
                <h2 className="sr-only">Multi-Agent Task Dispatching</h2>
                
                {/* Semantic background clicks without keyboard interference / role contradictions */}
                <div 
                    className="flex-1 min-h-0"
                    onClick={() => close_dropdowns()}
                >
                    <div className="flex flex-col gap-6 min-h-0 h-full">
                        <Stat_Metrics
                            active_agents={active_agents}
                            online_count={online_count}
                            total_cost={total_cost}
                            total_tokens={total_tokens}
                            total_input_tokens={total_input_tokens}
                            total_output_tokens={total_output_tokens}
                            total_budget={total_budget}
                            budget_util={budget_util}
                            recruit_velocity={recruit_velocity}
                        />

                        {is_agent_grid_detached ? (
                            <div className="flex-1 bg-[color:var(--color-background)]/20 backdrop-blur-sm border border-[color:var(--color-border)] rounded-xl overflow-hidden group flex items-center justify-center relative min-h-[400px]">
                                <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,rgba(59,130,246,0.05),transparent)]" />
                                <div className="text-center space-y-4 relative z-10">
                                    <div className="relative inline-block">
                                        <ExternalLink size={40} className="text-zinc-800 animate-pulse" />
                                        <div className="absolute inset-0 bg-green-500/10 blur-xl rounded-full" />
                                    </div>
                                    <div className="space-y-1">
                                        <h3 className="text-sm font-bold tracking-tight text-zinc-300 uppercase tracking-[0.2em]">{i18n.t('layout.sector_detached')}</h3>
                                        <p className="text-[10px] text-zinc-500 font-mono uppercase tracking-widest">{i18n.t('layout.link_established')} :: AGENT_GRID_DETACHED</p>
                                    </div>
                                    <button 
                                        onClick={toggle_agent_grid_detachment}
                                        className="px-6 py-2.5 bg-zinc-100 text-zinc-950 text-[10px] font-black uppercase tracking-[0.2em] rounded-xl hover:bg-white transition-all shadow-xl active:scale-95"
                                    >
                                        {i18n.t('layout.recall_sector')}
                                    </button>
                                </div>
                            </div>
                        ) : (
                            <Agent_Status_Grid
                                agents={agents_list}
                                assigned_agent_ids={assigned_agent_ids}
                                available_roles={available_roles}
                                clusters={clusters}
                                on_skill_trigger={handle_skill_trigger}
                                on_model_change={handle_model_change}
                                on_model_2_change={handle_model_2_change}
                                on_model_3_change={handle_model_3_change}
                                on_role_change={handle_role_change}
                                on_configure_click={handle_configure_click}
                                handle_agent_update={handle_agent_update}
                                on_toggle_cluster={toggle_cluster_active}
                                on_detach={toggle_agent_grid_detachment}
                            />
                        )}

                        {is_agent_grid_detached && (
                            <Portal_Window
                                id="agent-status-grid-detached"
                                title={i18n.t('dashboard.live_status_agent_grid')}
                                url="/detached-view?type=agent-status"
                                on_close={toggle_agent_grid_detachment}
                            >
                                <div className="h-screen bg-[color:var(--color-background)] p-6 flex flex-col overflow-hidden">
                                    <Agent_Status_Grid
                                        agents={agents_list}
                                        assigned_agent_ids={assigned_agent_ids}
                                        available_roles={available_roles}
                                        clusters={clusters}
                                        on_skill_trigger={handle_skill_trigger}
                                        on_model_change={handle_model_change}
                                        on_model_2_change={handle_model_2_change}
                                        on_model_3_change={handle_model_3_change}
                                        on_role_change={handle_role_change}
                                        on_configure_click={handle_configure_click}
                                        handle_agent_update={handle_agent_update}
                                        on_toggle_cluster={toggle_cluster_active}
                                        on_detach={toggle_agent_grid_detachment}
                                    />
                                </div>
                            </Portal_Window>
                        )}
                    </div>
                </div>

                {effective_config_agent_id && (effective_config_agent_id === 'new' || config_agent) && (
                    <AgentConfigPanel
                        agent={config_agent} 
                        onClose={() => set_config_agent_id(null)}
                        onUpdate={(id: string, updates: Partial<Agent>) => {
                            if (id === 'new') {
                                handle_create_agent(updates);
                            } else {
                                handle_agent_update(id, updates);
                            }
                        }}
                        onDelete={handle_delete_agent}
                        isNew={effective_config_agent_id === 'new'}
                    />
                )}

                <TerminalComponent agents={agents_list} />
            </div>
        </Error_Boundary>
    );
}
