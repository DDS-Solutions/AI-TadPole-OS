/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Agent_Health_Section]` in observability traces.
 */

import { Activity, Search, ArrowUpDown } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Agent_Health } from '../../services/tadpoleos_service';
import { Node_Task_Box } from '../hierarchy/Node_Task_Box';
import { use_agent_store } from '../../stores/agent_store';

interface AgentHealthSectionProps {
    sorted_health: Agent_Health[];
    health_search: string;
    set_health_search: (search: string) => void;
    health_sort: 'name' | 'failures';
    set_health_sort: (sort: 'name' | 'failures') => void;
}

export function Agent_Health_Section({ sorted_health, health_search, set_health_search, health_sort, set_health_sort }: AgentHealthSectionProps) {
    const live_agents = use_agent_store(s => s.agents);

    return (
        <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-2xl flex flex-col h-[400px]">
            <div className="p-4 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface)]/50 flex flex-col md:flex-row md:items-center justify-between gap-3">
                <div className="flex items-center gap-2">
                    <Activity className="w-4 h-4 text-amber-400" />
                    <h2 className="font-semibold text-zinc-100">{i18n.t('security.swarm_health_monitor')}</h2>
                </div>
                <div className="flex items-center gap-2 flex-1 md:max-w-xs">
                    <div className="relative flex-1">
                        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3 h-3 text-zinc-500" />
                        <input
                            type="text"
                            placeholder={i18n.t('agent_manager.search_placeholder')}
                            value={health_search}
                            onChange={(e) => set_health_search(e.target.value)}
                            className="w-full bg-black/30 border border-zinc-800 rounded-lg py-1 pl-8 pr-3 text-[10px] text-zinc-300 focus:outline-none focus:border-amber-500/50 transition-colors"
                        />
                    </div>
                    <button
                        onClick={() => set_health_sort(health_sort === 'failures' ? 'name' : 'failures')}
                        className={`px-3 py-1 rounded text-[10px] font-bold uppercase transition-colors flex items-center gap-1 whitespace-nowrap ${health_sort === 'failures' ? 'bg-amber-500/20 text-amber-400' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}`}
                    >
                        <ArrowUpDown size={10} /> {health_sort === 'failures' ? i18n.t('security.sort_status') : i18n.t('security.sort_name')}
                    </button>
                </div>
            </div>
            <div className="p-4 grid grid-cols-1 lg:grid-cols-2 gap-3 overflow-auto">
                {sorted_health.length === 0 ? (
                    <div className="col-span-full py-12 text-center text-zinc-600 italic text-xs">
                        No agents matching criteria.
                    </div>
                ) : sorted_health.map(a => {
                    const live_agent = live_agents.find(ag => ag.id === a.agent_id);
                    return (
                        <div key={a.agent_id} className={`p-4 rounded-xl border flex flex-col gap-3 transition-all duration-300 group ${a.is_healthy ? 'bg-[color:color-mix(in_srgb,var(--color-background)_50%,transparent)] border-[color:var(--color-border)] hover:border-zinc-700' : 'bg-red-500/5 border-red-500/20 ring-1 ring-red-500/10'}`}>
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-3">
                                    <div className={`w-10 h-10 rounded-xl border flex items-center justify-center font-bold transition-transform group-hover:scale-105 ${a.is_healthy ? 'bg-[color:var(--color-surface)] border-[color:var(--color-border)] text-zinc-300' : 'bg-red-500/10 border-red-500/20 text-red-400'}`}>
                                        {a.name.charAt(0)}
                                    </div>
                                    <div>
                                        <h3 className="text-sm font-bold text-zinc-200">{a.name}</h3>
                                        <div className="flex items-center gap-2">
                                            <p className="text-[10px] text-zinc-500 font-mono">{a.agent_id}</p>
                                            {a.last_failure_at && (
                                                <span className="text-[9px] text-zinc-600">
                                                    • Fail: {(() => {
                                                        const d = new Date(a.last_failure_at);
                                                        return isNaN(d.getTime()) ? '—' : d.toLocaleTimeString();
                                                    })()}
                                                </span>
                                            )}
                                        </div>
                                    </div>
                                </div>
                                <div className="flex flex-col items-end gap-1">
                                    <div className="flex items-center gap-1.5">
                                        <span className={`w-2 h-2 rounded-full ${a.is_healthy ? 'bg-emerald-500' : 'bg-red-500'}`} />
                                        <span className={`text-[10px] font-bold uppercase ${a.is_healthy ? 'text-zinc-500' : 'text-red-400'}`}>
                                            {i18n.t('security.failures_label', { count: a.failure_count })}
                                        </span>
                                    </div>
                                    {a.is_throttled && (
                                        <span className="text-[9px] bg-red-500/20 text-red-500 px-2 py-0.5 rounded-full font-bold uppercase tracking-widest animate-pulse">
                                            {i18n.t('security.throttled')}
                                        </span>
                                    )}
                                </div>
                            </div>
                            {live_agent && <Node_Task_Box agent={live_agent} />}
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

// Metadata: [Agent_Health_Section]
