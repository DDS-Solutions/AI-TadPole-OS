/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Real-time event log for the OS Dashboard. 
 * Streams system events (Auth, Engine, Error) from the event_bus into a scrollable, high-fidelity monitoring pane.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Buffer starvation during high-frequency telemetry storms, or scroll-to-bottom anchor failure.
 * - **Telemetry Link**: Search for `[System_Log]` or emit_log in service tracing.
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
    
    const [is_expanded, set_is_expanded] = React.useState(false);
    const [original_rect, set_original_rect] = React.useState<{ left: number; top: number; width: number } | null>(null);
    const wrapper_ref = React.useRef<HTMLDivElement>(null);
    const click_timer_ref = React.useRef<NodeJS.Timeout | null>(null);

    React.useEffect(() => {
        return () => {
            if (click_timer_ref.current) {
                clearTimeout(click_timer_ref.current);
            }
        };
    }, []);

    const calculate_and_set_rect = React.useCallback(() => {
        if (wrapper_ref.current) {
            const rect = wrapper_ref.current.getBoundingClientRect();
            const parent = wrapper_ref.current.parentElement;
            const parent_top = parent ? parent.getBoundingClientRect().top : rect.top;
            set_original_rect({ left: rect.left, top: parent_top, width: rect.width });
        }
    }, []);

    React.useEffect(() => {
        if (!is_expanded || is_detached_view || !wrapper_ref.current) return;
        calculate_and_set_rect();
        const observer = new ResizeObserver(calculate_and_set_rect);
        observer.observe(wrapper_ref.current);
        if (wrapper_ref.current.parentElement) {
            observer.observe(wrapper_ref.current.parentElement);
        }
        return () => observer.disconnect();
    }, [is_expanded, is_detached_view, calculate_and_set_rect]);

    const handle_header_click = (e: React.MouseEvent) => {
        if (is_detached_view) return;

        const target = e.target as HTMLElement;
        if (target.closest('button') || target.closest('input')) {
            return;
        }

        if (e.detail === 1) {
            click_timer_ref.current = setTimeout(() => {
                if (!is_expanded && wrapper_ref.current) {
                    calculate_and_set_rect();
                    set_is_expanded(true);
                }
            }, 200);
        } else if (e.detail === 2) {
            if (click_timer_ref.current) {
                clearTimeout(click_timer_ref.current);
                click_timer_ref.current = null;
            }
            if (is_expanded) {
                set_is_expanded(false);
            }
        }
    };

    const is_client = React.useSyncExternalStore(
        () => () => {},
        () => true,
        () => false
    );
    
    return (
        <div
            ref={wrapper_ref}
            className={clsx(
                !is_detached_view && "relative flex flex-col min-h-0",
                is_detached_view && "h-full w-full"
            )}
        >
            <div
                className={clsx(
                    "flex flex-col overflow-hidden relative group transition-all duration-300 w-full h-full",
                    !is_detached_view && !is_expanded && "sovereign-card",
                    is_detached_view && "h-full w-full",
                    is_expanded && "shadow-2xl z-50 backdrop-blur-md rounded-xl"
                )}
                style={is_expanded && original_rect ? {
                    position: 'fixed',
                    left: `${original_rect.left}px`,
                    top: `${original_rect.top}px`,
                    width: `${original_rect.width}px`,
                    height: `${window.innerHeight - original_rect.top - 16}px`,
                    zIndex: 50,
                    backgroundColor: 'rgba(9, 9, 11, 0.95)',
                    border: '1px solid rgba(63, 63, 70, 0.5)',
                    transform: 'none',
                } : undefined}
            >
                {!is_detached_view && !is_expanded && <div className="neural-grid opacity-[0.05]" />}
                <Tooltip content={i18n.t('system_log.tooltip')} position="left">
                    <div 
                        onClick={handle_header_click}
                        className="relative z-10 p-3 border-b border-[color:var(--color-border)] bg-[color:color-mix(in_srgb,var(--color-background)_80%,transparent)] backdrop-blur-md flex items-center justify-between cursor-help select-none"
                    >
                        <h3 className="sovereign-header-text flex items-center gap-2">
                            <TerminalIcon size={12} /> {i18n.t('system_log.title')}
                        </h3>
                        <div className="flex items-center gap-3">
                            <div className="flex gap-1.5 mr-2">
                                <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                                <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                            </div>
                            <Tooltip content={is_system_log_detached ? i18n.t('layout.recall_sector') : i18n.t('layout.sector_detached')} position="bottom">
                                <button
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        toggle_system_log_detachment();
                                    }}
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
                        <div className="text-zinc-600 italic text-center mt-10">{i18n.t('system_log.empty')}</div>
                    )}
                    {(logs || []).map((log) => (
                        <div key={log.id} className="space-y-1 group animate-in fade-in slide-in-from-left-4 duration-500 ease-out">
                            <div className="flex items-center gap-2">
                                <span className="text-zinc-500 text-[10px]">
                                    {is_client ? log.timestamp.toLocaleTimeString([], { hour12: false }) : '--:--:--'}
                                </span>
                                {log.agent_id ? (
                                    <span className="text-green-400 font-bold">
                                        [{log.source}:{log.agent_id}{log.agent_name ? ` (${log.agent_name})` : ''}]
                                    </span>
                                ) : (
                                    <span className="text-zinc-500 font-bold">[{log.source}]</span>
                                )}
                            </div>
                            <div className={`pl-2 border-l-2 ${log.severity === 'error' ? 'border-red-500 text-red-400' :
                                log.severity === 'success' ? 'border-emerald-500 text-emerald-400' :
                                    log.severity === 'warning' ? 'border-amber-500 text-amber-400' :
                                        'border-[color:var(--color-border)] text-zinc-300'
                                }`}>
                                {log.text}
                            </div>
                        </div>
                    ))}
                    <div ref={logs_end_ref} />
                </div>
            </div>
        </div>
    );
};


// Metadata: [System_Log]
