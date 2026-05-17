/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Action_Ledger]` in observability traces.
 */

import React from 'react';
import { Activity, Target, Search } from 'lucide-react';
import { Tooltip, Tw_Empty_State } from '../ui';
import { i18n } from '../../i18n';
import type { LedgerEntry } from '../../data/mock_oversight';
import type { Mission_Cluster } from '../../stores/workspace_store';

interface ActionLedgerProps {
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
    return (
        <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-lg overflow-hidden flex flex-col h-[600px]">
            <div className="p-4 border-b border-[color:var(--color-border)] flex items-center justify-between bg-[color:var(--color-surface)]/50">
                <div className="flex items-center gap-2">
                    <Tooltip content={i18n.t('oversight.ledger_tooltip')} position="right">
                        <Activity className="w-4 h-4 text-green-400 cursor-help" />
                    </Tooltip>
                    <h2 className="font-semibold text-zinc-100">{i18n.t('oversight.ledger_title')}</h2>
                </div>
                <div className="flex items-center gap-4">
                    <div className="relative">
                        <Tooltip content="Filter logs by mission cluster" position="top">
                            <Target className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 cursor-help" />
                        </Tooltip>
                        <select
                            value={selected_cluster_id}
                            onChange={(e) => set_selected_cluster_id(e.target.value)}
                            className="bg-[color:var(--color-background)] border border-zinc-700 rounded-full pl-9 pr-8 py-1.5 text-sm text-zinc-200 focus:outline-none focus:border-green-500 appearance-none cursor-pointer"
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
                            className="bg-[color:var(--color-background)] border border-zinc-700 rounded-full pl-9 pr-4 py-1.5 text-sm text-zinc-200 focus:outline-none focus:border-green-500 w-48"
                        />
                    </div>
                </div>
            </div>

            <div className="overflow-auto flex-1 p-0 custom-scrollbar">
                <table className="w-full text-left text-sm">
                    <thead className="bg-[color:var(--color-background)] text-zinc-400 sticky top-0 z-10">
                        <tr>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('oversight.table_time')}</th>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('oversight.table_agent')}</th>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('oversight.table_action')}</th>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('oversight.table_params')}</th>
                            <th className="p-3 font-medium border-b border-[color:var(--color-border)]">{i18n.t('oversight.table_result')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-zinc-800/50">
                        {ledger.map(entry => (
                            <tr key={entry.id} className="hover:bg-zinc-800/20 transition-colors">
                                <td className="p-3 text-zinc-500 whitespace-nowrap font-mono text-[10px]">
                                    {new Date(entry.timestamp).toLocaleTimeString()}
                                </td>
                                <td className="p-3">
                                    <div className="flex items-center gap-2">
                                        <div className="w-6 h-6 rounded-md bg-zinc-800 flex items-center justify-center text-[10px] font-bold text-zinc-500 border border-zinc-700">
                                            {resolve_agent_name(entry.tool_call?.agent_id || entry.agent_id || '').charAt(0)}
                                        </div>
                                        <span className="text-zinc-300 text-xs font-bold">
                                            {resolve_agent_name(entry.tool_call?.agent_id || entry.agent_id || '')}
                                        </span>
                                    </div>
                                </td>
                                <td className="p-3">
                                    <div className="flex items-center gap-2">
                                        <span className={`px-1.5 py-0.5 rounded text-[8px] font-bold uppercase ${entry.decision === 'approved' ? 'bg-green-500/10 text-green-500 border border-green-500/20' : 'bg-red-500/10 text-red-500 border border-red-500/20'}`}>
                                            {entry.decision}
                                        </span>
                                        <span className="font-mono text-[10px] text-green-400/80">{entry.tool_call?.skill || entry.skill || i18n.t('oversight.proposal_label')}</span>
                                    </div>
                                </td>
                                <td className="p-3">
                                    <Tooltip content={JSON.stringify(entry.tool_call?.params || entry.params || {}, null, 2)}>
                                        <div className="max-w-[120px] truncate text-[10px] text-zinc-500 font-mono bg-black/20 px-1.5 py-0.5 rounded border border-zinc-800/30 cursor-help">
                                            {JSON.stringify(entry.tool_call?.params || entry.params || {})}
                                        </div>
                                    </Tooltip>
                                </td>
                                <td className="p-3">
                                    {entry.decision === 'rejected' ? (
                                        <span className="text-red-400 text-xs uppercase font-bold tracking-wider">{i18n.t('oversight.blocked_label')}</span>
                                    ) : (
                                        <span className={`text-xs ${entry.result?.success ? 'text-green-400' : 'text-red-400'}`}>
                                            {entry.result?.success ? i18n.t('oversight.success_label') : i18n.t('oversight.failed_label')}
                                            <span className="text-zinc-600 ml-1">({entry.result?.duration_ms}ms)</span>
                                        </span>
                                    )}
                                </td>
                            </tr>
                        ))}
                        {ledger.length === 0 && (
                            <tr>
                                <td colSpan={5}>
                                    <Tw_Empty_State title={i18n.t('oversight.no_actions_title')} description={i18n.t('oversight.no_actions_description')} />
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
};

// Metadata: [Action_Ledger]
