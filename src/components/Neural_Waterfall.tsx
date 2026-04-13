/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-fidelity timeline visualization (Gantt style) of swarm operations. 
 * Renders trace spans as interactive bars with real-time "Now" tickers for running tasks.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Timeline drift if the system clock (min/max time) is malformed, bar width underflow (< 0.1%), or performance lag during high-frequency (500ms) re-renders on large traces.
 * - **Telemetry Link**: Search for `[Neural_Waterfall]` or `timeline_spans` in UI tracing.
 */

import React, { useMemo, useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Network, ExternalLink, Minimize2 } from 'lucide-react';
import { use_trace_store, type Trace_Node } from '../stores/trace_store';
import { use_agent_store } from '../stores/agent_store';
import { useSafeLifecycle } from '../hooks/use_safe_lifecycle';
import { use_tab_store } from '../stores/tab_store';
import { i18n } from '../i18n';
import clsx from 'clsx';
import { Tooltip } from './ui';

/**
 * Neural_Waterfall
 * A high-fidelity Gantt chart visualization of system-wide neural traces.
 * Refactored for strict snake_case compliance and consistent prop propagation.
 */
export const Neural_Waterfall: React.FC<{ is_detached_view?: boolean }> = ({ is_detached_view = false }) => {
    const { isMounted } = useSafeLifecycle();
    const { active_trace_id, get_trace_tree } = use_trace_store();
    const { get_agent } = use_agent_store();
    const { is_trace_stream_detached, toggle_trace_stream_detachment } = use_tab_store();
    const [now, set_now] = useState(() => Date.now());

    // PERF: Reduced frequency of "now" ticker for Gantt chart. 
    // 500ms is sufficient for visual feedback of running tasks.
    useEffect(() => {
        const interval = setInterval(() => {
            if (isMounted()) set_now(Date.now());
        }, 500);
        return () => clearInterval(interval);
    }, [isMounted]);

    // Flatten tree and calculate timeline metrics
    const { timeline_spans, min_time, total_duration } = useMemo(() => {
        if (!active_trace_id) return { timeline_spans: [], min_time: 0, total_duration: 0 };

        const raw_tree = get_trace_tree(active_trace_id);
        const flat: (Trace_Node & { depth: number })[] = [];

        // PERF: iterative or simple recursive flattening with depth
        const flatten = (nodes: Trace_Node[], depth: number) => {
            for (const node of nodes) {
                flat.push({ ...node, depth });
                if (node.children?.length) flatten(node.children, depth + 1);
            }
        };
        flatten(raw_tree, 0);

        if (flat.length === 0) return { timeline_spans: [], min_time: 0, total_duration: 0 };

        const min = Math.min(...(flat || []).map(s => s.start_time));
        const max = Math.max(...(flat || []).map(s => s.end_time || now));
        const duration = Math.max(1, max - min);

        return { timeline_spans: flat, min_time: min, total_duration: duration };
    }, [active_trace_id, get_trace_tree, now]);

    return (
        <div className={clsx(
            "flex-1 flex flex-col overflow-hidden relative group",
            !is_detached_view && "sovereign-card overflow-hidden h-64 border-t border-zinc-900 shrink-0",
            is_detached_view && "h-full"
        )}>
            {!is_detached_view && <div className="neural-grid opacity-[0.05]" />}
            
            <Tooltip content={i18n.t('trace_stream.tooltip')} position="left">
                <div className="relative z-10 p-3 border-b border-zinc-800 bg-zinc-950 flex items-center justify-between transition-colors cursor-help">
                    <h3 className="sovereign-header-text flex items-center gap-2">
                        <Network size={12} className="text-cyan-500" />
                        {i18n.t('trace_stream.title')}
                        {total_duration > 0 && (
                            <span className="text-[9px] font-mono text-zinc-600 ml-2 normal-case tracking-normal">
                                {total_duration}ms Total
                            </span>
                        )}
                    </h3>
                    
                    <div className="flex items-center gap-3">
                        <div className="flex gap-1.5 mr-2">
                            <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                            <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                        </div>
                        <button
                            onClick={() => toggle_trace_stream_detachment()}
                            className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                            title={is_trace_stream_detached ? i18n.t('trace_stream.recall_tooltip') : i18n.t('trace_stream.detach_tooltip')}
                        >
                            {is_trace_stream_detached ? <Minimize2 size={14} /> : <ExternalLink size={14} />}
                        </button>
                    </div>
                </div>
            </Tooltip>

            <div className="flex-1 overflow-y-auto p-4 custom-scrollbar relative z-10">
                {!active_trace_id || timeline_spans.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-full opacity-30 text-center px-6">
                        <Network size={24} className="mb-3 text-cyan-500/50" />
                        <p className="sovereign-header-text !text-zinc-500">
                             LINK READY :: AWAITING TELEMETRY
                        </p>
                    </div>
                ) : (
                    <div className="relative w-full">
                        {(timeline_spans || []).map((span) => {
                            const is_running = !span.end_time;
                            const duration = is_running ? (now - span.start_time) : (span.end_time! - span.start_time);

                            const left_percent = ((span.start_time - min_time) / total_duration) * 100;
                            const width_percent = Math.max(0.1, (duration / total_duration) * 100);

                            const agent_name = get_agent(span.agent_id)?.name || span.agent_id;

                            const bar_color = is_running
                                ? 'bg-cyan-500/80 shadow-[0_0_10px_rgba(6,182,212,0.3)]'
                                : span.status === 'error'
                                    ? 'bg-red-500/80'
                                    : 'bg-emerald-500/80';

                            return (
                                <div key={span.id} className="flex items-center mb-1.5 group">
                                    <div className="w-32 flex-shrink-0 text-right pr-4 truncate pt-1">
                                        <span className="text-[9px] font-mono text-zinc-500 block uppercase tracking-wider">{agent_name}</span>
                                        <span className="text-[8px] font-mono text-zinc-600 block truncate">{span.name}</span>
                                    </div>

                                    <div className="flex-1 relative h-6 bg-zinc-900/50 rounded overflow-hidden">
                                        {/* Grid lines background */}
                                        <div className="absolute inset-0 bg-[linear-gradient(to_right,#ffffff05_1px,transparent_1px)] bg-[size:10%] pointer-events-none" />

                                        <motion.div
                                            initial={{ opacity: 0, width: 0 }}
                                            animate={{
                                                opacity: 1,
                                                left: `${left_percent}%`,
                                                width: `${width_percent}%`
                                            }}
                                            className={`absolute top-1 bottom-1 rounded-sm ${bar_color} flex items-center px-1 overflow-hidden`}
                                        >
                                            {duration > 50 && (
                                                <span className="text-[8px] font-mono text-white/90 truncate drop-shadow-md">
                                                    {duration}ms
                                                </span>
                                            )}
                                        </motion.div>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>
        </div>
    );
};

