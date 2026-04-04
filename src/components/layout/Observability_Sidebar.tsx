import React from 'react';
import { clsx } from 'clsx';
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

    // If all sectors are detached globally, we don't need to render the sidebar container
    const is_all_detached = is_system_log_detached && is_trace_stream_detached && is_lineage_stream_detached;
    if (is_all_detached && !is_detached_context) return null;

    return (
        <div className={clsx(
            "flex flex-col gap-4 relative pb-10",
            is_detached_context ? "w-[400px] border-l border-zinc-900 bg-black/20" : "w-[400px]",
            className
        )}>
            {/* Sector Stack: Lineage -> Waterfall -> Log */}
            {!is_lineage_stream_detached && <Lineage_Stream />}
            {!is_trace_stream_detached && <Neural_Waterfall />}
            {!is_system_log_detached && <System_Log />}

            {/* Empty State / Legend if nothing is here but context is forced */}
            {!(!is_lineage_stream_detached || !is_trace_stream_detached || !is_system_log_detached) && (
                <div className="flex-1 flex flex-col items-center justify-center text-zinc-800 p-8 text-center">
                    <div className="w-12 h-12 rounded-full border border-zinc-900 flex items-center justify-center mb-4">
                        <div className="w-2 h-2 rounded-full bg-zinc-900 animate-pulse" />
                    </div>
                    <p className="text-[10px] uppercase tracking-[0.2em] font-bold">Monitor Channels Detached</p>
                </div>
            )}
        </div>
    );
};
