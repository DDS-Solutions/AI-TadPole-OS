/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-density grid for real-time agent status monitoring. 
 * Orchestrates status-aware node rendering (Active, Thinking, Idle) and facilitates deep-link navigation to the `Agent_Config_Panel`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Grid re-order flicker during high-frequency status transitions, or node occlusion on smaller layouts.
 * - **Telemetry Link**: Search for `[Agent_Status_Grid]` or `status_change` in UI tracing.
 */

import React, { useState, useCallback } from 'react';
import { AnimatePresence } from 'framer-motion';
import type { Agent_Status } from '../../types';
import { Hierarchy_Node } from '../Hierarchy_Node';
import type { Agent, Mission_Cluster } from '../../types';
import { Operation_Tab_Header } from './Operation_Tab_Header';
import { Portal_Window } from '../ui/Portal_Window';
import { ExternalLink } from 'lucide-react';
import { i18n } from '../../i18n';

interface Agent_Status_Grid_Props {
    agents: Agent[];
    assigned_agent_ids: Set<string>;
    available_roles: string[];
    clusters: Mission_Cluster[];
    on_skill_trigger: (agent_id: string, skill: string, slot?: 1 | 2 | 3) => void;
    on_configure_click: (id: string) => void;
    on_model_change: (agent_id: string, new_model: string) => void;
    on_model_2_change: (agent_id: string, new_model: string) => void;
    on_model_3_change: (agent_id: string, new_model: string) => void;
    on_role_change: (agent_id: string, new_role: string) => void;
    handle_agent_update: (agent_id: string, updates: Partial<Agent>) => void;
    on_toggle_cluster: (cluster_id: string) => void;
    on_detach?: () => void;
    initial_tab_id?: string;
}

export const Agent_Status_Grid: React.FC<Agent_Status_Grid_Props> = ({
    agents,
    assigned_agent_ids,
    available_roles,
    clusters,
    on_skill_trigger,
    on_configure_click,
    on_model_change,
    on_model_2_change,
    on_model_3_change,
    on_role_change,
    handle_agent_update,
    on_toggle_cluster,
    on_detach,
    initial_tab_id
}): React.ReactElement => {
    const [active_tab_id, set_active_tab_id] = useState(initial_tab_id || 'global');
    const [detached_tab_ids, set_detached_tab_ids] = useState<string[]>([]);

    // ── Detachment Logic ────────────────────────────────────────

    const handle_detach_tab = useCallback((id: string) => {
        set_detached_tab_ids(prev => prev.includes(id) ? prev : [...prev, id]);
    }, []);

    const handle_reattach_tab = useCallback((id: string) => {
        set_detached_tab_ids(prev => prev.filter(tid => tid !== id));
    }, []);

    // ── Filtering Logic ────────────────────────────────────────

    const get_filtered_agents = useCallback((tab_id: string) => {
        if (tab_id === 'global') {
            return (agents || []).filter(agent => 
                assigned_agent_ids.has(String(agent.id)) || 
                (['active', 'thinking', 'speaking', 'coding'] as Agent_Status[]).includes(agent.status)
            );
        }

        // Must be a cluster ID
        const active_cluster = clusters.find(c => c.id === tab_id);
        if (!active_cluster) return [];

        return (agents || []).filter(agent => 
            (active_cluster.collaborators || []).map(String).includes(String(agent.id))
        );
    }, [agents, assigned_agent_ids, clusters]);

    const render_grid_content = (tab_id: string) => {
        const filtered = get_filtered_agents(tab_id);
        
        return (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6 pb-4 relative z-10">
                {(filtered || []).map((agent): React.ReactElement => {
                    const agent_cluster = (clusters || []).find(c => (c.collaborators || []).map(String).includes(String(agent.id)));
                    const is_alpha = agent_cluster?.alpha_id === agent.id;
                    const is_active = agent_cluster?.is_active;
                    const mission_objective = agent_cluster?.objective || agent.active_mission?.objective;
                    const agent_color = agent.theme_color || '#10b981';

                    return (
                        <div key={agent.id} className="agent-grid-item group/item relative">
                            <div
                                className="absolute -inset-4 blur-3xl rounded-full opacity-0 group-hover/item:opacity-20 transition-opacity duration-700 pointer-events-none"
                                style={{ backgroundColor: agent_color }}
                            />
                            <Hierarchy_Node
                                agent={agent}
                                available_roles={available_roles}
                                on_skill_trigger={on_skill_trigger}
                                on_configure_click={on_configure_click}
                                is_alpha={is_alpha}
                                is_active={is_active}
                                mission_objective={mission_objective}
                                on_model_change={on_model_change}
                                on_model_2_change={on_model_2_change}
                                on_model_3_change={on_model_3_change}
                                on_role_change={on_role_change}
                                on_update={handle_agent_update}
                            />
                        </div>
                    );
                })}
                
                {filtered.length === 0 && (
                    <div className="col-span-full py-20 flex flex-col items-center justify-center text-zinc-500 gap-2 opacity-50">
                        <div className="w-10 h-10 border border-dashed border-zinc-700 rounded-lg flex items-center justify-center">
                            <span className="text-xl">!</span>
                        </div>
                        <p className="text-[10px] uppercase tracking-widest font-bold">{i18n.t('dashboard.no_nodes_detected')}</p>
                    </div>
                )}
            </div>
        );
    };

    const is_current_tab_detached = detached_tab_ids.includes(active_tab_id);

    return (
        <>
            <div className="flex-1 overflow-y-auto min-h-0 bg-zinc-950 border border-zinc-800 rounded-xl flex flex-col custom-scrollbar relative">
                <div className="neural-grid opacity-[0.1]" />
                
                <Operation_Tab_Header 
                    active_tab_id={active_tab_id}
                    clusters={clusters}
                    on_tab_change={set_active_tab_id}
                    on_toggle_cluster={on_toggle_cluster}
                    on_detach_tab={handle_detach_tab}
                    on_detach_grid={on_detach}
                />

                <div className="flex-1 overflow-y-auto px-6 pt-6 custom-scrollbar pb-6 relative">
                    {is_current_tab_detached ? (
                        <div className="absolute inset-0 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm z-20">
                            <div className="text-center space-y-4">
                                <div className="relative inline-block">
                                    <ExternalLink size={48} className="text-zinc-800 animate-pulse" />
                                    <div className="absolute inset-0 bg-blue-500/10 blur-xl rounded-full" />
                                </div>
                                <div className="space-y-1">
                                    <h3 className="text-lg font-bold tracking-tight text-zinc-200">{i18n.t('layout.sector_detached')}</h3>
                                    <p className="text-sm text-zinc-500 font-mono">
                                        {i18n.t('layout.link_established')} :: {active_tab_id.toUpperCase()}
                                    </p>
                                </div>
                                <button 
                                    onClick={() => handle_reattach_tab(active_tab_id)}
                                    className="px-4 py-2 bg-zinc-100 text-black text-xs font-bold uppercase tracking-widest rounded-lg hover:bg-white transition-all shadow-lg active:scale-95"
                                >
                                    {i18n.t('layout.recall_sector')}
                                </button>
                            </div>
                        </div>
                    ) : (
                        render_grid_content(active_tab_id)
                    )}
                </div>
            </div>

            <AnimatePresence>
                {(detached_tab_ids || []).map(tab_id => {
                    const cluster = clusters.find(c => c.id === tab_id);
                    const title = tab_id === 'global' ? i18n.t('dashboard.global_view') : (cluster?.name || tab_id.toUpperCase());
                    
                    return (
                        <Portal_Window 
                            key={tab_id}
                            on_close={() => handle_reattach_tab(tab_id)} 
                            title={`LIVE STATUS: ${title}`}
                            id={`detached-op-${tab_id}`}
                            url={`/detached-view?type=agent-status&tabId=${tab_id}`}
                        >
                            <div className="bg-zinc-950 p-6 h-full overflow-y-auto custom-scrollbar">
                                {render_grid_content(tab_id)}
                            </div>
                        </Portal_Window>
                    );
                })}
            </AnimatePresence>
        </>
    );
};

