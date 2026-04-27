/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Operational command center for active agent missions. 
 * Orchestrates mission initialization, real-time status tracking, and swarm collaboration logs.
 * 
 * ### 🔍 Debugging & Observability
 * - **Telemetry Link**: Uses `crypto.randomUUID()` for `trace_id` generation. Search `[MissionsBoard]` or specific `mission_id` in logs.
 */
/* eslint-disable react-hooks/preserve-manual-memoization */

import { Repeat, Check, X, Cpu, Target } from 'lucide-react';
import { use_workspace_store, type Task_Branch } from '../stores/workspace_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import { tadpole_os_socket, type Handoff_Event } from '../services/socket';
import { useState, useEffect, useCallback, useMemo } from 'react';
import { get_theme_colors } from '../utils/agent_uiutils';
import { Tw_Empty_State } from '../components/ui';
import { use_agent_store } from '../stores/agent_store';
import { resolve_agent_model_config } from '../utils/model_utils';
import { use_trace_store } from '../stores/trace_store';
import { i18n } from '../i18n';
import type { Agent } from '../types';

// Modular Components
import { Cluster_Sidebar } from '../components/missions/Cluster_Sidebar';
import { Mission_Header } from '../components/missions/Mission_Header';
import { Neural_Map } from '../components/missions/Neural_Map';
import { Agent_Team_View } from '../components/missions/Agent_Team_View';
import { Buffered_Transcript_View } from '../components/transcript/Buffered_Transcript_View';

export default function Missions() {
    const {
        clusters,
        assign_agent_to_cluster,
        unassign_agent_from_cluster,
        update_cluster_objective,
        set_alpha_node,
        delete_cluster,
        toggle_cluster_active,
        approve_branch,
        reject_branch,
        update_cluster_department,
        update_cluster_budget,
        toggle_mission_analysis
    } = use_workspace_store();
    const [selected_cluster_id, set_selected_cluster_id] = useState<string | null>(clusters[0]?.id || null);
    const { agents, fetch_agents, update_agent: store_update_agent, is_loading: agents_loading } = use_agent_store();

    useEffect(() => {
        console.debug("[MissionsBoard] Component mounted");
        const controller = new AbortController();
        if (agents.length === 0) {
            fetch_agents({ signal: controller.signal });
        }

        const unsubscribeHandoff = tadpole_os_socket.subscribe_handoff((event: Handoff_Event) => {
            const tgt = event.to_cluster || 'unknown';
            const desc = (event.payload?.description as string) || `Cross-cluster task handoff triggered for agent ${event.agent_id}.`;

            use_workspace_store.getState().receive_handoff(event.from_cluster || 'unknown', tgt, desc);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_handoff', { tgt }),
                severity: 'info',
                mission_id: tgt
            });
        });

        return () => {
            controller.abort();
            unsubscribeHandoff();
        };
    }, [fetch_agents, agents.length]);

    const active_cluster = useMemo(() => 
        (clusters || []).find(c => c.id === selected_cluster_id),
    [clusters, selected_cluster_id]);

    const assigned_agent_ids = useMemo(() => 
        new Set((clusters || []).flatMap(c => (c.collaborators || []))),
    [clusters]);

    const available_agents = useMemo(() => 
        agents.filter(a => !assigned_agent_ids.has(a.id)),
    [agents, assigned_agent_ids]);

    const handle_run_mission = useCallback(async () => {
        if (!active_cluster) return;

        if (!active_cluster.alpha_id) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_fail_alpha', { name: active_cluster.name }),
                severity: 'error',
                mission_id: active_cluster.id
            });
            return;
        }

        if (!active_cluster.objective) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_fail_objective', { name: active_cluster.name }),
                severity: 'error',
                mission_id: active_cluster.id
            });
            return;
        }

        const alpha_agent = agents.find(a => a.id === active_cluster.alpha_id);
        if (!alpha_agent) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_error_alpha_not_found'),
                severity: 'error',
                mission_id: active_cluster.id
            });
            return;
        }

        event_bus.emit_log({
            source: 'System',
            text: i18n.t('missions.event_launching', { objective: active_cluster.objective }),
            severity: 'warning',
            mission_id: active_cluster.id
        });

        // --- TRACING ACTIVATION ---
        // Generate a trace_id for this mission run to enable the Lineage Stream
        const request_id = (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `tr-${Date.now()}`;
        const trace_id = request_id.replace(/-/g, '').padEnd(32, '0').slice(0, 32);
        use_trace_store.getState().set_active_trace(trace_id);
        // --- END TRACING ---

        try {
            const { get_settings } = await import('../stores/settings_store');
            const { model_id, provider } = resolve_agent_model_config(alpha_agent as Agent, get_settings().default_model);
            const success = await tadpole_os_service.send_command(
                alpha_agent.id,
                active_cluster.objective,
                model_id,
                provider,
                active_cluster.id,
                active_cluster.department as string,
                active_cluster.budget_usd,
                undefined,
                undefined,
                active_cluster.analysis_enabled,
                request_id
            );
            if (success) {
                store_update_agent(alpha_agent.id, { 
                    status: 'active', 
                    current_task: active_cluster.objective 
                });
                event_bus.emit_log({ 
                    source: 'System', 
                    text: i18n.t('missions.event_dispatched', { name: alpha_agent.name }), 
                    severity: 'success',
                    mission_id: active_cluster.id
                });
            } else {
                throw new Error("Engine rejected the command.");
            }
        } catch (err: unknown) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_launch_fail', { error: err instanceof Error ? err.message : String(err) }),
                severity: 'error',
                mission_id: active_cluster.id
            });
        }
    }, [
        active_cluster,
        agents,
        store_update_agent
    ]);

    const active_proposals = use_workspace_store(state => state.active_proposals);
    const dismiss_proposal = use_workspace_store(state => state.dismiss_proposal);
    const apply_proposal = use_workspace_store(state => state.apply_proposal);

    return (
        <div className="h-full flex flex-col animate-in fade-in duration-500" aria-label="Missions Board">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS Swarm Missions Board",
              "description": "Real-time orchestration and status tracking for distributed agent missions. Features multi-agent collaboration logs and mission lifecycle management.",
              "author": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "Mission Control",
              "operatingSystem": "Tadpole OS"
            })}
            </script>
            <h1 className="sr-only">Tadpole OS Swarm Missions Board & Autonomous Analytics</h1>
            <h2 className="sr-only">Mission Lifecycle Orchestration</h2>
            <h2 className="sr-only">Swarm Collaboration Intelligence</h2>



            <div className="grid grid-cols-1 md:grid-cols-3 gap-6 flex-1 min-h-0">
                <Cluster_Sidebar
                    clusters={clusters}
                    selected_cluster_id={selected_cluster_id}
                    agents={agents}
                    on_select_cluster={set_selected_cluster_id}
                    on_create_cluster={use_workspace_store.getState().create_cluster}
                    on_delete_cluster={delete_cluster}
                    on_toggle_active={toggle_cluster_active}
                    on_update_department={update_cluster_department}
                    on_update_budget={update_cluster_budget}
                />

                <div className="md:col-span-2 bg-zinc-950 border border-zinc-800 rounded-2xl flex flex-col overflow-hidden">
                    {active_cluster ? (
                        <>
                            <Mission_Header
                                active_cluster={active_cluster}
                                agents_loading={agents_loading}
                                has_agents={agents.length > 0}
                                on_run_mission={handle_run_mission}
                                on_toggle_analysis={toggle_mission_analysis}
                            />

                            <div className="p-6 flex flex-col gap-8 flex-1 overflow-y-auto custom-scrollbar">
                                <Neural_Map
                                    cluster={active_cluster}
                                    agents={agents}
                                    theme_color={get_theme_colors(active_cluster.theme).hex}
                                />

                                {/* Objective Input */}
                                <div className="p-4 bg-zinc-900 border border-zinc-800 rounded-xl">
                                    <div className="flex items-center gap-2 text-zinc-500 mb-2">
                                        <Target size={14} className={get_theme_colors(active_cluster.theme).text} />
                                        <span className="text-[10px] font-bold uppercase tracking-widest">{i18n.t('missions.header_objective')}</span>
                                    </div>
                                    <input 
                                        type="text"
                                        placeholder={i18n.t('missions.objective_placeholder')}
                                        aria-label={i18n.t('missions.header_objective')}
                                        className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-zinc-100 focus:outline-none focus:border-green-500 transition-colors"
                                        value={active_cluster.objective}
                                        onChange={(e) => update_cluster_objective(active_cluster.id, e.target.value)}
                                    />
                                </div>

                                {/* Proposals */}
                                {active_proposals[active_cluster.id] && (
                                    <div className={`p-4 rounded-xl border ${get_theme_colors(active_cluster.theme).bg} ${get_theme_colors(active_cluster.theme).border} animate-in slide-in-from-top-4`}>
                                        <div className="flex justify-between items-start gap-4">
                                            <div className="flex gap-4">
                                                <div className={`p-2 rounded-lg bg-zinc-900 border ${get_theme_colors(active_cluster.theme).border}`}>
                                                    <Cpu size={20} className={get_theme_colors(active_cluster.theme).text} />
                                                </div>
                                                <div className="space-y-1">
                                                    <h4 className="text-xs font-bold text-zinc-100 uppercase tracking-tight">{i18n.t('missions.proposal_title')}</h4>
                                                    <p className="text-xs text-zinc-400 font-mono leading-relaxed max-w-xl">
                                                        {active_proposals[active_cluster.id].reasoning}
                                                    </p>
                                                </div>
                                            </div>
                                            <div className="flex gap-2">
                                                    <button
                                                        onClick={() => dismiss_proposal(active_cluster.id)}
                                                        className="px-3 py-1.5 rounded-lg border border-zinc-700 bg-zinc-900 text-zinc-500 text-xs font-bold uppercase transition-colors"
                                                    >
                                                        {i18n.t('missions.btn_dismiss')}
                                                    </button>
                                                <button
                                                    onClick={() => apply_proposal(active_cluster.id)}
                                                    className={`px-3 py-1.5 rounded-lg border ${get_theme_colors(active_cluster.theme).text} ${get_theme_colors(active_cluster.theme).border} bg-zinc-900 text-xs font-bold uppercase shadow-lg ${get_theme_colors(active_cluster.theme).glow}`}
                                                >
                                                    {i18n.t('missions.btn_authorize')}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                )}

                                {/* Handoffs */}
                                {((active_cluster.pending_tasks || []).filter((t: Task_Branch) => (t.id || '').startsWith('ho-') && t.status === 'pending')).length > 0 && (
                                    <div className="p-4 rounded-xl border border-amber-500/30 bg-amber-500/5">
                                        <h4 className="text-xs font-bold text-amber-400 uppercase tracking-widest mb-3 flex items-center gap-2">
                                            <Repeat size={12} /> {i18n.t('missions.header_handoffs')}
                                        </h4>
                                        <div className="space-y-3">
                                            {(active_cluster.pending_tasks || []).filter((t: Task_Branch) => (t.id || '').startsWith('ho-') && t.status === 'pending').map((task: Task_Branch) => (
                                                <div key={task.id} className="p-3 bg-black/40 border border-amber-500/20 rounded-lg flex items-center justify-between gap-4">
                                                    <div className="flex flex-col gap-1">
                                                        <p className="text-xs text-zinc-100">{task.description}</p>
                                                        <span className="text-[8px] text-amber-500/60 uppercase font-mono">{i18n.t('missions.label_delegation_request')}</span>
                                                    </div>
                                                    <div className="flex items-center gap-2">
                                                        <button
                                                            onClick={() => approve_branch(active_cluster.id, task.id)}
                                                            className="p-1.5 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                                                        >
                                                            <Check size={14} />
                                                        </button>
                                                        <button
                                                            onClick={() => reject_branch(active_cluster.id, task.id)}
                                                            className="p-1.5 rounded-lg bg-red-500/10 text-red-400 border border-red-500/30"
                                                        >
                                                            <X size={14} />
                                                        </button>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                <Agent_Team_View 
                                    active_cluster={active_cluster}
                                    agents={agents}
                                    available_agents={available_agents}
                                on_assign={(id) => assign_agent_to_cluster(id, active_cluster.id)}
                                on_unassign={(id) => unassign_agent_from_cluster(id, active_cluster.id)}
                                on_set_alpha={(id) => set_alpha_node(active_cluster.id, id)}
                                />

                                <Buffered_Transcript_View 
                                    mission_id={active_cluster.id}
                                />
                            </div>
                        </>
                    ) : (
                        <div className="flex-1 flex flex-col items-center justify-center p-12 text-center">
                            <Tw_Empty_State 
                                title={i18n.t('missions.empty_title')}
                                description={i18n.t('missions.empty_description')}
                            />
                            <div className="mt-8 p-4 bg-green-500/5 border border-green-500/10 rounded-xl max-w-sm">
                                <p className="text-[10px] text-green-400/60 uppercase font-bold tracking-widest mb-2">{i18n.t('missions.pro_tip')}</p>
                               <p className="text-xs text-zinc-500 leading-relaxed">
                                    {i18n.t('missions.tip_details')}
                                </p>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

// Metadata: [Missions]

// Metadata: [Missions]
