/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Resource_Quotas_Section]` in observability traces.
 */

import { DollarSign, ArrowUpDown, Minus, Plus } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Agent_Health, Quota_Details } from '../../services/tadpoleos_service';
import { Node_Task_Box } from '../hierarchy/Node_Task_Box';
import { use_agent_store } from '../../stores/agent_store';

interface ResourceQuotasSectionProps {
    sorted_quotas: Quota_Details[];
    agent_health: Agent_Health[];
    sort_config: { key: 'name' | 'status' | 'quota', direction: 'asc' | 'desc' };
    set_sort_config: (config: { key: 'name' | 'status' | 'quota', direction: 'asc' | 'desc' }) => void;
    is_updating_quota: string | null;
    handle_update_quota: (entity_id: string, current_budget: number, increment: number) => void;
}

export function Resource_Quotas_Section({ 
    sorted_quotas, 
    agent_health, 
    sort_config, 
    set_sort_config, 
    is_updating_quota, 
    handle_update_quota 
}: ResourceQuotasSectionProps) {
    const live_agents = use_agent_store(s => s.agents);

    const toggle_sort = (key: 'name' | 'status' | 'quota') => {
        set_sort_config({
            key,
            direction: sort_config.key === key && sort_config.direction === 'asc' ? 'desc' : 'asc'
        });
    };

    return (
        <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-2xl flex flex-col overflow-hidden">
            <div className="p-4 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface)]/50 flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <DollarSign className="w-4 h-4 text-green-400" />
                    <h2 className="font-semibold text-zinc-100">{i18n.t('security.resource_quotas')}</h2>
                </div>
                <div className="flex items-center gap-2">
                    {(['name', 'status', 'quota'] as const).map(key => (
                        <button
                            key={key}
                            onClick={() => toggle_sort(key)}
                            className={`px-3 py-1 rounded text-[10px] font-bold uppercase transition-colors flex items-center gap-1 ${sort_config.key === key ? 'bg-green-500/20 text-green-400' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}`}
                        >
                            <ArrowUpDown size={10} /> {i18n.t(`security.sort_${key}`)}
                        </button>
                    ))}
                </div>
            </div>
            <div className="p-4 grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                {sorted_quotas.map(q => {
                    const health = agent_health.find(h => h.agent_id === q.entity_id);
                    const live_agent = live_agents.find(ag => ag.id === q.entity_id);
                    const is_exceeded = q.used_usd >= q.budget_usd;
                    return (
                        <div key={q.entity_id} className={`p-4 rounded-xl border flex flex-col gap-4 ${is_exceeded ? 'bg-red-500/5 border-red-500/30' : 'bg-[color:color-mix(in_srgb,var(--color-background)_50%,transparent)] border-[color:var(--color-border)]'}`}>
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-3">
                                    <div className={`w-8 h-8 rounded-lg flex items-center justify-center font-bold text-xs transition-all duration-300 ${health?.is_healthy ? 'bg-emerald-500/10 text-emerald-500 border border-emerald-500/20' : 'bg-zinc-800 text-zinc-500 border border-zinc-700'}`}>
                                        {(health?.name || q.entity_id).charAt(0).toUpperCase()}
                                    </div>
                                    <div>
                                        <h3 className="text-sm font-bold text-zinc-200">{health?.name || q.entity_id}</h3>
                                        <div className="flex items-center gap-2">
                                            <span className={`w-1.5 h-1.5 rounded-full ${['active', 'thinking', 'speaking', 'coding'].includes(health?.status || '') ? 'bg-emerald-500 animate-pulse' : 'bg-zinc-600'}`} />
                                            <p className="text-[10px] text-zinc-500 font-mono uppercase">
                                                {health?.status === 'idle' ? i18n.t('agent_card.label_idle_task').replace('System Idle • ', '').replace('...', '') : (health?.status || 'offline')}
                                            </p>
                                        </div>
                                    </div>
                                </div>
                                <div className="text-right">
                                    <p className={`text-xs font-bold font-mono ${is_exceeded ? 'text-red-500' : 'text-zinc-300'}`}>
                                        ${(q.used_usd || 0).toFixed(2)} / ${(q.budget_usd || 0).toFixed(2)}
                                    </p>
                                    <div className="flex flex-col items-end">
                                        <span className="text-[9px] text-zinc-500 uppercase font-bold tracking-tighter">
                                            {i18n.t('security.reset_label', { period: q.reset_period })}
                                        </span>
                                        {q.next_reset_at && (
                                            <span className="text-[8px] text-zinc-600 font-mono">
                                                Reset: {new Date(q.next_reset_at).toLocaleDateString()}
                                            </span>
                                        )}
                                    </div>
                                </div>
                            </div>

                            {live_agent && <Node_Task_Box agent={live_agent} />}

                            <div className="space-y-2">
                                <div className="h-1.5 bg-[color:var(--color-surface)] rounded-full overflow-hidden border border-[color:var(--color-border)]">
                                    <div
                                        className={`h-full transition-all duration-500 ${is_exceeded ? 'bg-red-500' : (q.used_usd / Math.max(q.budget_usd, 0.001) > 0.8 ? 'bg-amber-500' : 'bg-green-500')}`}
                                        style={{ width: `${Math.min((q.used_usd / Math.max(q.budget_usd, 0.001)) * 100, 100)}%` }}
                                    />
                                </div>
                                <div className="flex justify-between items-center">
                                    <div className="flex flex-col">
                                        <span className="text-[10px] text-zinc-500 font-mono">
                                            {i18n.t('security.usage_label', { percentage: (((q.used_usd || 0) / Math.max(q.budget_usd, 0.001)) * 100).toFixed(1) })}
                                        </span>
                                        {!is_exceeded && (
                                            <span className="text-[8px] text-zinc-600 font-mono">
                                                Remaining: ${(q.budget_usd - q.used_usd).toFixed(2)}
                                            </span>
                                        )}
                                    </div>
                                    <div className="flex items-center gap-2">
                                        <Tooltip content={i18n.t('security.tooltip_quota_decrease')}>
                                            <button
                                                disabled={is_updating_quota === q.entity_id || q.budget_usd <= 0.1}
                                                onClick={() => handle_update_quota(q.entity_id, q.budget_usd, -0.5)}
                                                className="p-1.5 rounded-lg bg-[color:var(--color-surface)] border border-[color:var(--color-border)] text-zinc-400 hover:text-red-400 hover:border-red-500/30 disabled:opacity-50 transition-all shadow-sm"
                                            >
                                                <Minus size={12} />
                                            </button>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('security.tooltip_quota_increase')}>
                                            <button
                                                disabled={is_updating_quota === q.entity_id}
                                                onClick={() => handle_update_quota(q.entity_id, q.budget_usd, 0.5)}
                                                className="p-1.5 rounded-lg bg-[color:var(--color-surface)] border border-[color:var(--color-border)] text-zinc-400 hover:text-emerald-400 hover:border-emerald-500/30 disabled:opacity-50 transition-all shadow-sm flex items-center gap-1 pr-2"
                                            >
                                                <Plus size={12} />
                                                <span className="text-[10px] font-bold">+$0.50</span>
                                            </button>
                                        </Tooltip>
                                    </div>
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

// Metadata: [Resource_Quotas_Section]
