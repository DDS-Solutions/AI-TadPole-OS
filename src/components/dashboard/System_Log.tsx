/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Real-time event log for the OS Dashboard. 
 * Streams system events (Auth, Engine, Error) from the `event_bus` into a scrollable, high-fidelity monitoring pane.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Buffer starvation during high-frequency telemetry storms, or scroll-to-bottom anchor failure.
 * - **Telemetry Link**: Search for `[System_Log]` or `emit_log` in service tracing.
 */

import React from 'react';
import { Terminal as TerminalIcon, ExternalLink, Minimize2 } from 'lucide-react';
import clsx from 'clsx';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { useLogs } from '../../hooks/use_logs';
import { use_tab_store } from '../../stores/tab_store';

interface System_Log_Props {
    is_detached_view?: boolean;
}

/**
 * System_Log
 * A high-fidelity terminal component for monitoring the unified log stream.
 * Refactored for strict snake_case compliance and consistent prop propagation.
 */
export const System_Log: React.FC<System_Log_Props> = ({ is_detached_view = false }) => {
    const { logs, logs_end_ref } = useLogs();
    const { is_system_log_detached, toggle_system_log_detachment } = use_tab_store();
    
    return (
        <div className={clsx(
            "xl:col-span-1 flex flex-col overflow-hidden relative group",
            !is_detached_view && "sovereign-card"
        )}>
            {!is_detached_view && <div className="neural-grid opacity-[0.05]" />}
            <Tooltip content={i18n.t('dashboard.log_tooltip')} position="left">
                <div className="relative z-10 p-3 border-b border-zinc-800 bg-zinc-950/80 backdrop-blur-md flex items-center justify-between cursor-help">
                    <h3 className="sovereign-header-text flex items-center gap-2">
                        <TerminalIcon size={12} /> {i18n.t('dashboard.log_title')}
                    </h3>
                    <div className="flex items-center gap-3">
                        <div className="flex gap-1.5 mr-2">
                            <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                            <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                        </div>
                        <Tooltip content={is_system_log_detached ? i18n.t('layout.recall_sector') : i18n.t('layout.sector_detached')} position="bottom">
                            <button
                                onClick={() => toggle_system_log_detachment()}
                                className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                                aria-label={is_system_log_detached ? i18n.t('layout.recall_sector') : i18n.t('layout.sector_detached')}
                            >
                                {is_system_log_detached ? <Minimize2 size={14} /> : <ExternalLink size={14} />}
                            </button>
                        </Tooltip>
                    </div>
                </div>
            </Tooltip>
            <div className="flex-1 overflow-y-auto p-4 space-y-3 font-mono text-[11px] custom-scrollbar">
                {logs.length === 0 && (
                    <div className="text-zinc-600 italic text-center mt-10">{i18n.t('dashboard.log_empty')}</div>
                )}
                {logs.map((log) => (
                    <div key={log.id} className="space-y-1 group animate-in fade-in slide-in-from-left-4 duration-500 ease-out">
                        <div className="flex items-center gap-2">
                            <span className="text-zinc-500 text-[10px]">{log.timestamp.toLocaleTimeString([], { hour12: false })}</span>
                            {log.agent_id ? (
                                <span className="text-blue-400 font-bold">
                                    [{log.source}:{log.agent_id}{log.agent_name ? ` (${log.agent_name})` : ''}]
                                </span>
                            ) : (
                                <span className="text-zinc-500 font-bold">[{log.source}]</span>
                            )}
                        </div>
                        <div className={`pl-2 border-l-2 ${log.severity === 'error' ? 'border-red-500 text-red-400' :
                            log.severity === 'success' ? 'border-emerald-500 text-emerald-400' :
                                log.severity === 'warning' ? 'border-amber-500 text-amber-400' :
                                    'border-zinc-800 text-zinc-300'
                            }`}>
                            {log.text}
                        </div>
                    </div>
                ))}
                <div ref={logs_end_ref} />
            </div>
        </div>
    );
};

