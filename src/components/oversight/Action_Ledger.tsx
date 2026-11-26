/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Oversight / Action_Ledger
 * - **Primary Entrypoints**: `Action_Ledger`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useMemo } from 'react';
import { Activity, Target, Search } from 'lucide-react';
import clsx from 'clsx';
import { Tooltip, Tw_Empty_State } from '../ui';
import { i18n } from '../../i18n';
import type { LedgerEntry } from '../../data/mock_oversight';
import type { Mission_Cluster } from '../../stores/workspace_store';
import { Params_Cell } from './Params_Cell';

const is_auto_approved = (entry: LedgerEntry): boolean => {
    return (
        entry.auto_approved === true ||
        entry.approval_type === 'auto' ||
        entry.requires_oversight === false ||
        entry.decision === 'auto_approved' ||
        entry.decided_by === 'auto_policy' ||
        entry.decided_by === 'system'
    );
};

interface ActionLedgerRowProps {
    entry: LedgerEntry;
    resolve_agent_name: (id: string) => string;
}

export const Action_Ledger_Row: React.FC<ActionLedgerRowProps> = ({ entry, resolve_agent_name }) => {
    const is_auto = is_auto_approved(entry);
    const agent_id = entry.tool_call?.agent_id || entry.agent_id || '';
    const agent_name = resolve_agent_name(agent_id);
    const time_str = new Date(entry.timestamp || entry.created_at || entry.decided_at || '').toLocaleTimeString();
    const skill_name = entry.tool_call?.skill || entry.skill || i18n.t('oversight.proposal_label');

    return (
        <tr className="hover:bg-zinc-800/20 transition-colors">
            <td className="p-3 text-zinc-500 whitespace-nowrap font-mono text-[10px]">
                {time_str}
            </td>
            <td className="p-3">
                <div className="flex items-center gap-2">
                    <div className="w-6 h-6 rounded-md bg-zinc-800 flex items-center justify-center text-[10px] font-mono font-bold text-zinc-400 border border-zinc-700">
                        {agent_name.charAt(0)}
                    </div>
                    <span className="text-zinc-300 text-xs font-bold">
                        {agent_name}
                    </span>
                </div>
            </td>
            <td className="p-3">
                <div className="flex items-center gap-2">
                    {is_auto ? (
                        <span className="px-1.5 py-0.5 rounded text-[8px] font-mono font-bold uppercase tracking-wider bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                            AUTO
                        </span>
                    ) : (
                        <span
                            className={clsx(
                                "px-1.5 py-0.5 rounded text-[8px] font-mono font-bold uppercase tracking-wider",
                                entry.decision === 'approved'
                                    ? "bg-green-500/10 text-green-400 border border-green-500/20"
                                    : "bg-red-500/10 text-red-400 border border-red-500/20"
                            )}
                        >
                            {entry.decision}
                        </span>
                    )}
                    <span className="font-mono text-[10px] text-green-400/80">
                        {skill_name}
                    </span>
                </div>
            </td>
            <td className="p-3">
                <Params_Cell params={entry.tool_call?.params || entry.params} />
            </td>
            <td className="p-3">
                {entry.decision === 'rejected' ? (
                    <span className="text-red-400 font-mono text-xs uppercase font-bold tracking-wider">
                        {i18n.t('oversight.blocked_label')}
                    </span>
                ) : entry.result == null ? (
                    <span className="text-amber-400 font-mono text-xs uppercase font-bold tracking-wider">
                        {entry.decision === 'approved' ? (i18n.t('oversight.approved_label') || 'APPROVED') : '—'}
                    </span>
                ) : (
                    <span
                        className={clsx(
                            "text-xs font-mono font-bold uppercase tracking-wider",
                            entry.result?.success ? "text-green-400" : "text-red-400"
                        )}
                    >
                        {entry.result?.success ? (i18n.t('oversight.success_label') || 'SUCCESS') : (i18n.t('oversight.failed_label') || 'FAILED')}
                        {entry.result?.duration_ms != null && (
                            <span className="text-zinc-600 ml-1 normal-case font-normal">
                                ({entry.result.duration_ms}ms)
                            </span>
                        )}
                    </span>
                )}
            </td>
        </tr>
    );
};

export interface ActionLedgerProps {
    ledger: LedgerEntry[];
    filter: string;
    set_filter: (filter: string) => void;
    selected_cluster_id: string;
    set_selected_cluster_id: (id: string) => void;
    clusters: Mission_Cluster[];
    resolve_agent_name: (id: string) => string;
}

export const Action_Ledger: React.FC<ActionLedgerProps> = ({
    ledger,
    filter,
    set_filter,
    selected_cluster_id,
    set_selected_cluster_id,
    clusters,
    resolve_agent_name
}) => {
    const [view_mode, set_view_mode] = useState<'hitl' | 'auto'>('hitl');

    // Filter ledger entries by search filter, cluster filter, and view_mode toggle
    const filtered_ledger = useMemo(() => {
        return ledger.filter(entry => {
            // Cluster Filter
            if (selected_cluster_id && selected_cluster_id !== 'all') {
                const entry_cluster = entry.cluster_id || entry.mission_id || entry.tool_call?.cluster_id || entry.tool_call?.mission_id;
                if (entry_cluster !== selected_cluster_id) return false;
            }

            // Approval type filter
            const is_auto = is_auto_approved(entry);
            if (view_mode === 'hitl' && is_auto) return false;
            if (view_mode === 'auto' && !is_auto) return false;

            // Search filter
            if (filter) {
                const search_lower = filter.toLowerCase();
                const skill_name = (entry.tool_call?.skill || entry.skill || '').toLowerCase();
                const agent_name = resolve_agent_name(entry.tool_call?.agent_id || entry.agent_id || '').toLowerCase();
                const desc = (entry.tool_call?.description || '').toLowerCase();
                if (!skill_name.includes(search_lower) && !agent_name.includes(search_lower) && !desc.includes(search_lower)) {
                    return false;
                }
            }

            return true;
        });
    }, [ledger, view_mode, filter, selected_cluster_id, resolve_agent_name]);

    return (
        <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-xl overflow-hidden flex flex-col h-[600px] sovereign-transition">
            {/* Header Bar */}
            <div className="p-4 border-b border-[color:var(--color-border)] flex flex-wrap items-center justify-between gap-3 bg-[color:var(--color-surface)]/50 backdrop-blur-md">
                <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2">
                        <Tooltip content={i18n.t('oversight.ledger_tooltip') || "Real-time record of all governance decisions"} position="right">
                            <Activity className="w-4 h-4 text-green-400 cursor-help" />
                        </Tooltip>
                        <h2 className="font-semibold text-zinc-100">{i18n.t('oversight.ledger_title') || "Action Ledger"}</h2>
                    </div>

                    {/* Segmented Toggle Control (HITL Approvals vs Auto-Approved) */}
                    <div className="flex items-center bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-0.5 font-mono text-[10px] select-none">
                        <button
                            type="button"
                            onClick={() => set_view_mode('hitl')}
                            className={clsx(
                                "px-2.5 py-1 rounded-md font-bold uppercase transition-all duration-200 focus-sovereign cursor-pointer",
                                view_mode === 'hitl'
                                    ? "bg-green-500/15 text-green-300 border border-green-500/30 shadow-sm"
                                    : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40"
                            )}
                            title="Show Human-in-the-Loop Approvals and Rejections"
                        >
                            HITL Approvals
                        </button>
                        <button
                            type="button"
                            onClick={() => set_view_mode('auto')}
                            className={clsx(
                                "px-2.5 py-1 rounded-md font-bold uppercase transition-all duration-200 focus-sovereign cursor-pointer",
                                view_mode === 'auto'
                                    ? "bg-cyan-500/15 text-cyan-300 border border-cyan-500/30 shadow-sm"
                                    : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40"
                            )}
                            title="Show System & Agent Auto-Approved Actions"
                        >
                            Auto-Approved
                        </button>
                    </div>
                </div>

                <div className="flex items-center gap-3">
                    <div className="relative">
                        <Tooltip content={i18n.t('oversight.filter_cluster_tooltip')} position="top">
                            <Target className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 cursor-help" />
                        </Tooltip>
                        <select
                            value={selected_cluster_id}
                            onChange={(e) => set_selected_cluster_id(e.target.value)}
                            className="bg-[color:var(--color-background)] border border-zinc-700/80 rounded-full pl-9 pr-8 py-1.5 text-xs text-zinc-200 focus-sovereign focus:border-green-500 appearance-none cursor-pointer"
                        >
                            <option value="all">{i18n.t('oversight.all_missions')}</option>
                            {(clusters || []).map(c => (
                                <option key={c.id} value={c.id}>{c.name}</option>
                            ))}
                        </select>
                    </div>
                    <div className="relative">
                        <Tooltip content={i18n.t('oversight.search_ledger_tooltip')} position="top">
                            <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 cursor-help" />
                        </Tooltip>
                        <input
                            type="text"
                            placeholder={i18n.t('oversight.filter_actions_placeholder')}
                            value={filter}
                            onChange={(e) => set_filter(e.target.value)}
                            className="bg-[color:var(--color-background)] border border-zinc-700/80 rounded-full pl-9 pr-4 py-1.5 text-xs text-zinc-200 focus-sovereign focus:border-green-500 w-48 font-mono"
                        />
                    </div>
                </div>
            </div>

            {/* Table Area */}
            <div className="overflow-auto flex-1 p-0 custom-scrollbar">
                <table className="w-full text-left text-sm">
                    <thead className="bg-[color:var(--color-background)] text-zinc-400 sticky top-0 z-10 border-b border-[color:var(--color-border)]">
                        <tr>
                            <th className="p-3 font-mono text-[10px] font-bold uppercase tracking-wider text-zinc-400">{i18n.t('oversight.table_time')}</th>
                            <th className="p-3 font-mono text-[10px] font-bold uppercase tracking-wider text-zinc-400">{i18n.t('oversight.table_agent')}</th>
                            <th className="p-3 font-mono text-[10px] font-bold uppercase tracking-wider text-zinc-400">{i18n.t('oversight.table_action')}</th>
                            <th className="p-3 font-mono text-[10px] font-bold uppercase tracking-wider text-zinc-400">{i18n.t('oversight.table_params')}</th>
                            <th className="p-3 font-mono text-[10px] font-bold uppercase tracking-wider text-zinc-400">{i18n.t('oversight.table_result')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-zinc-800/50">
                        {filtered_ledger.map(entry => (
                            <Action_Ledger_Row
                                key={entry.id}
                                entry={entry}
                                resolve_agent_name={resolve_agent_name}
                            />
                        ))}
                        {filtered_ledger.length === 0 && (
                            <tr>
                                <td colSpan={5} className="p-8 text-center">
                                    <Tw_Empty_State 
                                        title={view_mode === 'hitl' ? (i18n.t('oversight.no_actions_title') || 'No HITL Approvals') : 'No Auto-Approved Actions'} 
                                        description={view_mode === 'hitl' ? (i18n.t('oversight.no_actions_description') || 'No human-in-the-loop governance actions logged yet.') : 'No autonomous system or agent auto-approved actions logged in this timeframe.'} 
                                    />
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
};
