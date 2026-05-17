/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Pending_Queue]` in observability traces.
 */

import React from 'react';
import { AlertTriangle, Terminal as TerminalIcon } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { OversightEntry } from '../../data/mock_oversight';

interface PendingQueueProps {
    pending: OversightEntry[];
    resolve_agent_name: (id: string) => string;
    handle_decide: (id: string, decision: 'approved' | 'rejected') => void;
}

export const Pending_Queue: React.FC<PendingQueueProps> = ({ pending, resolve_agent_name, handle_decide }) => {
    if (pending.length === 0) return null;

    return (
        <div className="bg-[color:var(--color-surface)] border border-yellow-500/30 rounded-lg overflow-hidden">
            <div className="bg-yellow-500/10 p-3 border-b border-yellow-500/20 flex items-center gap-2">
                <AlertTriangle className="w-4 h-4 text-yellow-500" />
                <h2 className="font-semibold text-yellow-100">{i18n.t('oversight.awaiting_approval_title', { count: pending.length })}</h2>
            </div>
            <div className="divide-y divide-zinc-800">
                {pending.map(entry => (
                    <div key={entry.id} className="p-4 flex flex-col md:flex-row gap-4 items-start md:items-center justify-between animate-in fade-in slide-in-from-top-2">
                        <div className="space-y-1 flex-1">
                            <div className="flex items-center gap-2">
                                <span className="text-[10px] font-bold bg-zinc-800 text-zinc-300 px-2 py-0.5 rounded border border-zinc-700">
                                    {resolve_agent_name(entry.tool_call?.agent_id || entry.agent_id || '')}
                                </span>
                                <span className="text-sm font-bold text-green-400 flex items-center gap-1">
                                    <TerminalIcon className="w-3 h-3" />
                                    {entry.tool_call?.skill || entry.skill || i18n.t('oversight.capability_proposal')}
                                </span>
                                <span className="text-[10px] text-zinc-500 font-mono">
                                    {new Date(entry.created_at).toLocaleTimeString()}
                                </span>
                            </div>
                            <p className="text-sm text-zinc-300">{entry.tool_call?.description || entry.description || i18n.t('oversight.awaiting_authorization')}</p>
                            <div className="bg-black/50 p-2 rounded border border-zinc-800/50 flex flex-col gap-1">
                                <div className="text-[8px] text-zinc-600 font-bold uppercase tracking-widest">{i18n.t('oversight.payload_parameters')}</div>
                                <pre className="text-xs text-zinc-500 font-mono overflow-auto max-w-2xl">
                                    {JSON.stringify(entry.tool_call?.params || entry.params || {}, null, 2)}
                                </pre>
                            </div>
                        </div>
                        <div className="flex gap-2 w-full md:w-auto">
                            <Tooltip content={i18n.t('oversight.approve_action_tooltip')} position="top">
                                <button
                                    onClick={() => handle_decide(entry.id, 'approved')}
                                    className="flex-1 md:flex-none bg-green-500/20 hover:bg-green-500/30 text-green-400 px-4 py-2 rounded border border-green-500/30 font-medium transition-colors"
                                >
                                    {i18n.t('oversight.approve_button')}
                                </button>
                            </Tooltip>
                            <Tooltip content={i18n.t('oversight.reject_action_tooltip')} position="top">
                                <button
                                    onClick={() => handle_decide(entry.id, 'rejected')}
                                    className="flex-1 md:flex-none bg-red-500/20 hover:bg-red-500/30 text-red-400 px-4 py-2 rounded border border-red-500/30 font-medium transition-colors"
                                >
                                    {i18n.t('oversight.reject_button')}
                                </button>
                            </Tooltip>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};

// Metadata: [Pending_Queue]
