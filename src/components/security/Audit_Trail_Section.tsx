/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Audit_Trail_Section]` in observability traces.
 */

import { History, CheckCircle2, AlertCircle } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Audit_Entry, Agent_Health } from '../../services/tadpoleos_service';

interface AuditTrailSectionProps {
    audit_trail: Audit_Entry[];
    agent_health: Agent_Health[];
    audit_page: number;
    audit_total: number;
    set_audit_page: (page: number) => void;
}

export function Audit_Trail_Section({ audit_trail, agent_health, audit_page, audit_total, set_audit_page }: AuditTrailSectionProps) {
    return (
        <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-2xl flex flex-col h-[400px]">
            <div className="p-4 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface)]/50 flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <History className="w-4 h-4 text-emerald-400" />
                    <h2 className="font-semibold text-zinc-100">{i18n.t('security.audit_trail_title')}</h2>
                </div>
                <Tooltip content={i18n.t('security.tooltip_merkle_chain')}>
                    <span className="text-[10px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/20 font-mono uppercase tracking-widest cursor-help">
                        {i18n.t('security.merkle_chain_active')}
                    </span>
                </Tooltip>
            </div>
            <div className="overflow-auto flex-1">
                <table className="w-full text-left text-xs">
                    <thead className="bg-black/20 text-zinc-500 sticky top-0 z-10">
                        <tr>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('security.th_decided')}</th>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('security.th_agent')}</th>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('security.th_skill')}</th>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('common.label_hash')}</th>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('security.th_status')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-zinc-800/50">
                        {audit_trail.length === 0 ? (
                            <tr>
                                <td colSpan={5} className="p-8 text-center text-zinc-600 italic">
                                    {i18n.t('security.status_recorded_none')}
                                </td>
                            </tr>
                        ) : audit_trail.map(entry => {
                            const agent_name = agent_health.find(h => h.agent_id === entry.agent_id)?.name || entry.agent_id;
                            return (
                                <tr key={entry.id} className="hover:bg-zinc-800/20 transition-colors group">
                                    <td className="p-3 text-zinc-500 font-mono">
                                        {entry.decided_at ? (() => {
                                            const d = new Date(entry.decided_at);
                                            return isNaN(d.getTime()) ? i18n.t('security.status_pending') : d.toLocaleTimeString();
                                        })() : i18n.t('security.status_pending')}
                                    </td>
                                    <td className="p-3 text-zinc-300 font-medium">
                                        <div className="flex flex-col">
                                            <span>{agent_name}</span>
                                            <span className="text-[9px] text-zinc-600 font-mono uppercase">{entry.agent_id}</span>
                                        </div>
                                    </td>
                                    <td className="p-3 text-green-400 font-mono">
                                        {entry.skill || '—'}
                                    </td>
                                    <td className="p-3">
                                        <Tooltip content={entry.id}>
                                            <span className="text-[9px] font-mono text-zinc-600 bg-zinc-900/50 px-1.5 py-0.5 rounded border border-zinc-800 cursor-help">
                                                {entry.id.substring(0, 8)}...
                                            </span>
                                        </Tooltip>
                                    </td>
                                    <td className="p-3">
                                        <div className="flex items-center gap-2">
                                            {entry.is_verified ? (
                                                <CheckCircle2 size={12} className="text-emerald-500" />
                                            ) : (
                                                <AlertCircle size={12} className="text-red-500" />
                                            )}
                                            <span className={`uppercase font-bold tracking-tighter ${entry.is_verified ? 'text-emerald-500' : 'text-red-500'}`}>
                                                {entry.is_verified ? (entry.decision || i18n.t('security.status_recorded')) : i18n.t('security.status_recorded')}
                                            </span>
                                        </div>
                                    </td>
                                </tr>
                            )
                        })}
                    </tbody>
                </table>
            </div>
            <div className="p-3 border-t border-[color:var(--color-border)] bg-black/10 flex items-center justify-between text-[10px] font-bold uppercase tracking-widest text-zinc-500">
                <div className="flex items-center gap-4">
                    <span>{i18n.t('common.label_page')} {audit_page} / {Math.max(1, Math.ceil(audit_total / 10))}</span>
                    <span>{i18n.t('common.label_total_records')}: {audit_total}</span>
                </div>
                <div className="flex items-center gap-2">
                    <button
                        disabled={audit_page <= 1}
                        onClick={() => set_audit_page(audit_page - 1)}
                        className="px-3 py-1 rounded bg-zinc-800 border border-zinc-700 hover:bg-zinc-700 disabled:opacity-30 transition-colors"
                    >
                        {i18n.t('common.label_previous')}
                    </button>
                    <button
                        disabled={audit_page >= Math.ceil(audit_total / 10)}
                        onClick={() => set_audit_page(audit_page + 1)}
                        className="px-3 py-1 rounded bg-zinc-800 border border-zinc-700 hover:bg-zinc-700 disabled:opacity-30 transition-colors"
                    >
                        {i18n.t('common.label_next')}
                    </button>
                </div>
            </div>
        </div>
    );
}

// Metadata: [Audit_Trail_Section]
