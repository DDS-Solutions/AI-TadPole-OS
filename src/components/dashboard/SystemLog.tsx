import { Terminal as TerminalIcon, ExternalLink, Minimize2 } from 'lucide-react';
import clsx from 'clsx';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { useLogs } from '../../hooks/useLogs';
import { useTabStore } from '../../stores/tabStore';

interface SystemLogProps {
    isDetachedView?: boolean;
}

export const SystemLog: React.FC<SystemLogProps> = ({ isDetachedView = false }) => {
    const { logs, logsEndRef } = useLogs();
    const { isSystemLogDetached, toggleSystemLogDetachment } = useTabStore();
    return (
        <div className={clsx(
            "xl:col-span-1 flex flex-col overflow-hidden relative group",
            !isDetachedView && "sovereign-card"
        )}>
            {!isDetachedView && <div className="neural-grid opacity-[0.05]" />}
            <Tooltip content={i18n.t('dashboard.log_tooltip')} position="left">
                <div className="relative z-10 p-3 border-b border-zinc-800 bg-zinc-950 flex items-center justify-between cursor-help">
                    <h3 className="sovereign-header-text flex items-center gap-2">
                        <TerminalIcon size={12} /> {i18n.t('dashboard.log_title')}
                    </h3>
                    <div className="flex items-center gap-3">
                        <div className="flex gap-1.5 mr-2">
                            <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                            <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                        </div>
                        <Tooltip content={isSystemLogDetached ? i18n.t('layout.recall_sector') : i18n.t('layout.sector_detached')} position="bottom">
                            <button
                                onClick={() => toggleSystemLogDetachment()}
                                className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                            >
                                {isSystemLogDetached ? <Minimize2 size={14} /> : <ExternalLink size={14} />}
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
                    <div key={log.id} className="space-y-1 group animate-in fade-in slide-in-from-left-2 duration-300">
                        <div className="flex items-center gap-2">
                            <span className="text-zinc-500 text-[10px]">{log.timestamp.toLocaleTimeString([], { hour12: false })}</span>
                            {log.agentId ? (
                                <span className="text-blue-400 font-bold">[{log.source}:{log.agentId}]</span>
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
                <div ref={logsEndRef} />
            </div>
        </div>
    );
};
