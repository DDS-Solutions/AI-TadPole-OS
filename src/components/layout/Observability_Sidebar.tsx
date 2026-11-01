/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Layout / Observability_Sidebar
 * - **Primary Entrypoints**: `Observability_Sidebar`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { clsx } from 'clsx';
import { i18n } from '../../i18n';
import { use_tab_store } from '../../stores/tab_store';
import { Lineage_Stream } from '../Lineage_Stream';
import { Neural_Waterfall } from '../Neural_Waterfall';
import { System_Log } from '../dashboard/System_Log';

interface Observability_Sidebar_Props {
    is_detached_context?: boolean;
    className?: string;
}

/**
 * Observability_Sidebar
 * 
 * A unified sidebar containing the core observability sectors of Tadpole OS.
 * It intelligently hides sectors that are globally detached into their own windows.
 * Refactored for strict snake_case compliance and consistent prop propagation.
 */
export const Observability_Sidebar: React.FC<Observability_Sidebar_Props> = ({ 
    is_detached_context = false,
    className 
}) => {
    const { 
        is_system_log_detached, 
        is_trace_stream_detached, 
        is_lineage_stream_detached 
    } = use_tab_store();

    const is_all_detached = is_system_log_detached && is_trace_stream_detached && is_lineage_stream_detached;

    return (
        <div className={clsx(
            "flex flex-col gap-4 relative h-full overflow-y-auto custom-scrollbar p-3 border-l border-[color:var(--color-surface)] bg-zinc-950/20",
            is_detached_context ? "w-[400px]" : "w-[400px] shrink-0",
            className
        )}>
            {/* Sector Stack: Lineage -> Waterfall -> Log */}
            {!is_lineage_stream_detached && <Lineage_Stream />}
            {!is_trace_stream_detached && <Neural_Waterfall />}
            {!is_system_log_detached && <System_Log />}

            {/* Empty State / Recall affordance if all sectors are detached */}
            {is_all_detached && (
                <div className="flex-1 flex flex-col items-center justify-center text-zinc-500 p-8 text-center space-y-4 my-auto">
                    <div className="w-12 h-12 rounded-full border border-[color:var(--color-surface)] flex items-center justify-center">
                        <div className="w-2 h-2 rounded-full bg-cyan-500 animate-pulse" />
                    </div>
                    <div className="space-y-1">
                        <p className="text-[10px] uppercase tracking-[0.2em] font-bold text-zinc-400">
                            {i18n.t('layout.monitor_detached') || 'Monitors Detached'}
                        </p>
                        <p className="text-[9px] text-zinc-600 font-mono">
                            Event Log, Waterfall & OTel detached to external windows
                        </p>
                    </div>
                    <button
                        onClick={() => use_tab_store.getState().reattach_observability_sectors()}
                        className="px-4 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-[10px] font-mono uppercase tracking-widest rounded border border-zinc-700 transition-colors shadow-sm"
                    >
                        Recall All Monitors
                    </button>
                </div>
            )}
        </div>
    );
};
