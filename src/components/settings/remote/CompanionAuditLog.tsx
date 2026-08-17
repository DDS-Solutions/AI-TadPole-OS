/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Companion Security Audit Log Component**
 * Displays immutable audit trail of companion device pairings, modifications, and revocations.
 * Adheres to design.md high-contrast typography and semantic status coloring.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing audit trail entry or timestamp formatting error.
 * - **Telemetry Link**: Search `[CompanionAuditLog]` in UI logs.
 */

import React, { memo } from 'react';
import { History, Trash2 } from 'lucide-react';
import { Tooltip } from '../../ui';

export interface CompanionAuditEntry {
    id: string;
    action: 'DEVICE_PAIRED' | 'DEVICE_EDITED' | 'DEVICE_REVOKED';
    deviceName: string;
    userName: string;
    key?: string;
    details?: string;
    timestamp: string;
}

export interface CompanionAuditLogProps {
    logs: CompanionAuditEntry[];
    onClearLogs: () => void;
}

export const CompanionAuditLog: React.FC<CompanionAuditLogProps> = memo(({
    logs,
    onClearLogs,
}) => {
    return (
        <div data-testid="audit-log-container" className="bg-zinc-900/60 backdrop-blur-xl border border-white/5 rounded-2xl p-6 shadow-2xl space-y-4 relative overflow-hidden">
            <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/30 to-transparent" />
            
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="p-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">
                        <History className="w-5 h-5" />
                    </div>
                    <div>
                        <h2 className="text-base font-semibold text-zinc-100 tracking-tight flex items-center gap-2">
                            Companion Bridge Security Audit Log
                            <span className="px-2 py-0.5 rounded-full bg-zinc-800 text-[10px] font-mono text-zinc-400">
                                {logs.length}
                            </span>
                        </h2>
                        <p className="text-xs text-zinc-400 mt-0.5">
                            Append-only ledger of all companion authorization and pairing events.
                        </p>
                    </div>
                </div>

                {logs.length > 0 && (
                    <Tooltip content="Clear local companion audit entries" position="top">
                        <button
                            type="button"
                            onClick={onClearLogs}
                            className="flex items-center gap-1.5 px-3 py-1.5 text-xs text-zinc-400 hover:text-zinc-200 bg-zinc-800/60 hover:bg-zinc-800 border border-zinc-700/60 rounded-xl transition-all cursor-pointer"
                        >
                            <Trash2 className="w-3.5 h-3.5" />
                            <span>Clear Audit Log</span>
                        </button>
                    </Tooltip>
                )}
            </div>

            {logs.length === 0 ? (
                <div className="p-6 text-center bg-zinc-950/40 border border-zinc-800/60 rounded-xl">
                    <p className="text-xs text-zinc-500 italic">No security audit records logged.</p>
                </div>
            ) : (
                <div className="max-h-60 overflow-y-auto space-y-2 pr-1 custom-scrollbar">
                    {logs.map(log => {
                        const isPaired = log.action === 'DEVICE_PAIRED';
                        const isEdited = log.action === 'DEVICE_EDITED';

                        const badgeClass = isPaired
                            ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30'
                            : isEdited
                            ? 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30'
                            : 'bg-red-500/20 text-red-400 border-red-500/30';

                        const actionLabel = log.action.replace('DEVICE_', '');

                        return (
                            <div
                                key={log.id}
                                className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 p-3 bg-zinc-950/40 border border-zinc-800/60 rounded-xl text-xs hover:border-zinc-700/80 transition-colors font-sans"
                            >
                                <div className="flex items-center gap-2 min-w-0 pr-2">
                                    <span className={`px-2 py-0.5 rounded text-[9px] font-mono border uppercase font-bold shrink-0 ${badgeClass}`}>
                                        {actionLabel}
                                    </span>
                                    <span className="font-bold text-zinc-200 truncate">
                                        {log.deviceName}
                                    </span>
                                    <span className="text-[11px] text-zinc-400 font-mono">
                                        ({log.userName})
                                    </span>
                                </div>
                                <div className="flex items-center gap-3 text-[11px] font-mono text-zinc-500 shrink-0">
                                    {log.key && <span className="text-zinc-400">{log.key}</span>}
                                    <span>{log.timestamp}</span>
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
});

CompanionAuditLog.displayName = 'CompanionAuditLog';

// Metadata: [CompanionAuditLog]
