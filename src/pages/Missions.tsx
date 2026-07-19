/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Operational command center for active agent missions. 
 * Orchestrates mission initialization, real-time status tracking, and swarm collaboration logs via `useMissionsManager`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Telemetry Link**: Search `[Missions]` in system logs. (Uses crypto.randomUUID() for trace tracking).
 */

import { useEffect, useState } from 'react';

import { Target } from 'lucide-react';
import { useMissionsManager } from '../hooks/useMissionsManager';
import { get_theme_colors } from '../utils/agent_uiutils';
import { i18n } from '../i18n';
import { Tw_Empty_State } from '../components/ui';

// Modular Components
import { Cluster_Sidebar } from '../components/missions/Cluster_Sidebar';
import { Mission_Header } from '../components/missions/Mission_Header';
import { Neural_Map } from '../components/missions/Neural_Map';
import { Agent_Team_View } from '../components/missions/Agent_Team_View';
import { Buffered_Transcript_View } from '../components/transcript/Buffered_Transcript_View';
import { Mission_Proposals } from '../components/missions/Mission_Proposals';

export default function Missions() {
    useEffect(() => {
        console.debug("🚀 [Missions] Mounting Missions Board...");
    }, []);

    const {
        clusters, agents, agents_loading, active_cluster, available_agents,
        selected_cluster_id, active_proposals,
        set_selected_cluster_id,
        assign_agent_to_cluster, unassign_agent_from_cluster,
        update_cluster_objective, set_alpha_node,
        delete_cluster, toggle_cluster_active,
        update_cluster_department, update_cluster_budget,
        toggle_mission_analysis,
        dismiss_proposal, apply_proposal, create_cluster,
        handle_run_mission, is_launching, send_socratic_response
    } = useMissionsManager();

    const [response_text, set_response_text] = useState('');

    const alpha_agent = active_cluster && agents.find(a => a.id === active_cluster.alpha_id);
    const show_response_bar = active_cluster && alpha_agent && alpha_agent.status !== 'thinking' && alpha_agent.status !== 'coding';

    const handle_send_response = () => {
        if (!response_text.trim()) return;
        send_socratic_response(response_text);
        set_response_text('');
    };

    const handle_proceed_directly = () => {
        send_socratic_response("Proceed with the current objective.");
    };

    return (
        <div className="h-full flex flex-col animate-in fade-in duration-500 bg-[color:var(--color-background)] overflow-hidden" aria-label="Missions Board">
            {/* Semantic Header */}
            <h1 className="sr-only">Tadpole OS Swarm Missions Board</h1>
            
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6 p-6 h-full overflow-hidden">
                <Cluster_Sidebar
                    clusters={clusters}
                    selected_cluster_id={selected_cluster_id}
                    agents={agents}
                    on_select_cluster={set_selected_cluster_id}
                    on_create_cluster={create_cluster}
                    on_delete_cluster={delete_cluster}
                    on_toggle_active={toggle_cluster_active}
                    on_update_department={update_cluster_department}
                    on_update_budget={update_cluster_budget}
                />

                <div className="md:col-span-3 flex flex-col gap-6 overflow-hidden">
                    {active_cluster ? (
                        <div className="flex flex-col h-full gap-6 overflow-hidden bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-2xl">
                            <Mission_Header
                                active_cluster={active_cluster}
                                agents_loading={agents_loading}
                                has_agents={agents.length > 0}
                                on_run_mission={handle_run_mission}
                                on_toggle_analysis={toggle_mission_analysis}
                                is_launching={is_launching}
                            />

                            <div className="p-6 flex flex-col gap-8 flex-1 overflow-y-auto custom-scrollbar">
                                <Neural_Map
                                    cluster={active_cluster}
                                    agents={agents}
                                    theme_color={get_theme_colors(active_cluster.theme).hex}
                                />

                                {/* Objective Input */}
                                <div className="p-4 bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-xl">
                                    <div className="flex items-center gap-2 text-zinc-500 mb-2">
                                        <Target size={14} className={get_theme_colors(active_cluster.theme).text} />
                                        <span className="text-[10px] font-bold uppercase tracking-widest">{i18n.t('missions.header_objective')}</span>
                                    </div>
                                    <input 
                                        type="text"
                                        placeholder={i18n.t('missions.objective_placeholder')}
                                        aria-label={i18n.t('missions.header_objective')}
                                        className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-2 text-sm text-zinc-100 focus:outline-none focus:border-green-500 transition-colors"
                                        value={active_cluster.objective || ''}
                                        onChange={(e) => update_cluster_objective(active_cluster.id, e.target.value)}
                                    />
                                </div>

                                {/* Proposals */}
                                <Mission_Proposals 
                                    proposal={active_proposals[active_cluster.id]}
                                    theme={active_cluster.theme}
                                    on_dismiss={() => dismiss_proposal(active_cluster.id)}
                                    on_authorize={() => apply_proposal(active_cluster.id)}
                                />

                                {/* Handoffs & Branches */}
                                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                    <Agent_Team_View 
                                        active_cluster={active_cluster}
                                        agents={agents}
                                        available_agents={available_agents}
                                        on_assign={(id) => assign_agent_to_cluster(id, active_cluster.id)}
                                        on_unassign={(id) => unassign_agent_from_cluster(id, active_cluster.id)}
                                        on_set_alpha={(id) => set_alpha_node(active_cluster.id, id)}
                                    />
                                    
                                    <div className="flex flex-col h-full min-h-[400px] gap-4">
                                        <Buffered_Transcript_View 
                                            mission_id={active_cluster.id}
                                        />
                                        {show_response_bar && alpha_agent && (
                                            <div className={`p-4 bg-[color:var(--color-surface)] border ${get_theme_colors(active_cluster.theme).border} rounded-xl flex flex-col gap-3 animate-in slide-in-from-bottom-4`}>
                                                <div className="flex items-center justify-between">
                                                    <div className="flex items-center gap-2">
                                                        <div className="w-2 h-2 rounded-full bg-amber-500 animate-pulse" />
                                                        <span className="text-[10px] font-bold text-zinc-300 uppercase tracking-widest">
                                                            Swarm Socratic Gate / Interaction
                                                        </span>
                                                    </div>
                                                    <span className="text-[9px] text-zinc-500 font-mono">
                                                        Leader: {alpha_agent.name}
                                                    </span>
                                                </div>
                                                <div className="flex gap-2">
                                                    <input 
                                                        type="text"
                                                        placeholder="Provide clarifying answers or instructions..."
                                                        className="flex-1 bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-2 text-xs text-zinc-100 focus:outline-none focus:border-amber-500 transition-colors"
                                                        value={response_text}
                                                        onChange={(e) => set_response_text(e.target.value)}
                                                        onKeyDown={(e) => {
                                                            if (e.key === 'Enter') {
                                                                handle_send_response();
                                                            }
                                                        }}
                                                    />
                                                    <button
                                                        disabled={is_launching || !response_text.trim()}
                                                        onClick={handle_send_response}
                                                        className="px-4 py-2 bg-amber-600 hover:bg-amber-500 disabled:opacity-40 disabled:hover:bg-amber-600 transition-colors rounded-lg text-xs font-bold text-zinc-900 uppercase"
                                                    >
                                                        Send Answer
                                                    </button>
                                                    <button
                                                        disabled={is_launching}
                                                        onClick={handle_proceed_directly}
                                                        className="px-4 py-2 border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors rounded-lg text-xs font-bold uppercase"
                                                    >
                                                        Proceed
                                                    </button>
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                </div>
                            </div>
                        </div>
                    ) : (
                        <Tw_Empty_State
                            title={i18n.t('missions.empty_title')}
                            description={i18n.t('missions.empty_desc')}
                        />
                    )}
                </div>
            </div>
        </div>
    );
}

// Metadata: [Missions]
