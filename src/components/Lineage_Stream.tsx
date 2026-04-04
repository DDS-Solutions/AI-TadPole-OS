import React, { useState, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Activity, Share2, Clock, GitCommit, ExternalLink, Minimize2 } from 'lucide-react';
import { use_agent_store } from '../stores/agent_store';
import { use_trace_store, type Trace_Node } from '../stores/trace_store';
import { use_tab_store } from '../stores/tab_store';
import { i18n } from '../i18n';
import clsx from 'clsx';
import { Tooltip } from './ui';

// Recursive component to render the OTel trace tree
const Trace_Tree_Node: React.FC<{ node: Trace_Node; depth: number }> = ({ node, depth }): React.ReactElement => {
    const { get_agent } = use_agent_store();
    const agent_name = get_agent(node.agent_id)?.name || node.agent_id;

    // Status colors
    const status_color = node.status === 'running'
        ? 'text-cyan-400 bg-cyan-400/10 border-cyan-400/30'
        : node.status === 'error'
            ? 'text-red-400 bg-red-400/10 border-red-400/30'
            : 'text-emerald-400 bg-emerald-400/10 border-emerald-400/30';

    return (
        <div className="w-full flex flex-col pt-2">
            <div
                className="flex items-start gap-3 relative"
                style={{ marginLeft: `${depth * 16}px` }}
            >
                {/* Visual line connecting parent to child */}
                {depth > 0 && (
                    <div className="absolute -left-4 top-4 w-4 h-px bg-zinc-700" />
                )}
                {depth > 0 && (
                    <div className="absolute -left-4 -top-full bottom-auto h-[calc(100%+16px)] w-px bg-zinc-700" />
                )}

                <div className="flex-1 p-3 bg-zinc-900/60 border border-zinc-800 rounded-xl hover:bg-zinc-800/80 transition-all">
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                            <div className={`w-2 h-2 rounded-full shadow-[0_0_8px_currentColor] ${status_color}`} />
                            <span className="text-[10px] font-mono font-bold text-zinc-300 uppercase tracking-widest">{agent_name}</span>
                            <span className="text-[9px] text-zinc-500 font-mono px-1.5 py-0.5 rounded bg-zinc-950/50">
                                {node.name}
                            </span>
                        </div>
                        <span className="text-[9px] text-zinc-600 font-mono flex items-center gap-1">
                            <Clock size={10} />
                            {node.end_time ? `${node.end_time - node.start_time}ms` : i18n.t('trace.running')}
                        </span>
                    </div>

                    <div className="flex items-center gap-4 mt-2 pt-2 border-t border-zinc-800/50">
                        <span className="text-[8px] font-mono text-zinc-500 flex items-center gap-1">
                            <GitCommit size={10} /> {i18n.t('trace.span')}: {node.id.toUpperCase()}
                        </span>
                        {node.attributes && Object.keys(node.attributes).length > 0 && (
                            <span className="text-[8px] font-mono text-zinc-500 truncate max-w-[150px]">
                                {JSON.stringify(node.attributes)}
                            </span>
                        )}
                    </div>
                </div>
            </div>

            {/* Recursively render children */}
            {node.children && node.children.length > 0 && (
                <div className="flex flex-col relative w-full">
                    {node.children.map((child): React.ReactElement => (
                        <Trace_Tree_Node key={child.id} node={child} depth={depth + 1} />
                    ))}
                </div>
            )}
        </div>
    );
};

/**
 * Lineage_Stream
 * A real-time hierarchical telemetry feed visualizing the swarm's activity and parent-child relationships.
 * Refactored for strict snake_case compliance and consistent prop propagation.
 */
export const Lineage_Stream: React.FC<{ is_detached_view?: boolean }> = ({ is_detached_view = false }): React.ReactElement => {
    const { active_trace_id, get_trace_tree } = use_trace_store();
    const { is_lineage_stream_detached, toggle_lineage_stream_detachment } = use_tab_store();
    const [sidebar_width, set_sidebar_width] = useState(380);
    const stream_ref = useRef<HTMLDivElement>(null);

    const active_tree = active_trace_id ? get_trace_tree(active_trace_id) : [];

    const handle_sidebar_resize_start = (e: React.MouseEvent): void => {
        if (is_detached_view) return;
        e.preventDefault();
        const start_x = e.clientX;
        const start_width = sidebar_width;

        const on_mouse_move = (move_event: MouseEvent): void => {
            const current_x = move_event.clientX;
            const delta_x = start_x - current_x;
            set_sidebar_width(Math.min(800, Math.max(300, start_width + delta_x)));
        };

        const on_mouse_up = (): void => {
            document.removeEventListener('mousemove', on_mouse_move);
            document.removeEventListener('mouseup', on_mouse_up);
        };

        document.addEventListener('mousemove', on_mouse_move);
        document.addEventListener('mouseup', on_mouse_up);
    };

    return (
        <div
            className={clsx(
                "flex flex-col bg-zinc-950/20 border-l border-zinc-900 overflow-hidden relative group/sidebar",
                !is_detached_view && "flex-1 sovereign-card mb-4",
                is_detached_view && "h-full"
            )}
            style={{ width: is_detached_view ? '100%' : sidebar_width }}
            ref={stream_ref}
        >
            {!is_detached_view && <div className="neural-grid opacity-[0.05]" />}
            {!is_detached_view && (
                <div
                    onMouseDown={handle_sidebar_resize_start}
                    className="absolute inset-y-0 left-0 w-1 cursor-col-resize hover:bg-emerald-500/20 active:bg-emerald-500/40 transition-colors z-20"
                />
            )}

            <Tooltip content={i18n.t('trace.tooltip')} position="left">
                <div className="relative z-10 p-3 border-b border-zinc-800 bg-zinc-950 flex items-center justify-between transition-colors cursor-help">
                    <h3 className="sovereign-header-text flex items-center gap-2">
                        <Activity size={12} className="text-emerald-500" />
                        {i18n.t('trace.stream_title')}
                        {active_trace_id && (
                            <span className="text-[9px] font-mono px-2 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-500 truncate max-w-[120px] ml-2 normal-case tracking-normal">
                                {active_trace_id}
                            </span>
                        )}
                    </h3>
                    
                    <div className="flex items-center gap-3">
                        <div className="flex gap-1.5 mr-2">
                            <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                            <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                        </div>
                        <button
                            onClick={() => toggle_lineage_stream_detachment()}
                            className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                            title={is_lineage_stream_detached ? i18n.t('trace_stream.recall_tooltip') : i18n.t('trace_stream.detach_tooltip')}
                        >
                            {is_lineage_stream_detached ? <Minimize2 size={14} /> : <ExternalLink size={14} />}
                        </button>
                    </div>
                </div>
            </Tooltip>

            <div className="flex-1 overflow-y-auto p-4 space-y-2 custom-scrollbar relative">
                <AnimatePresence>
                    {active_tree.map((root_node): React.ReactElement => (
                        <motion.div
                            key={root_node.id}
                            initial={{ opacity: 0, scale: 0.95 }}
                            animate={{ opacity: 1, scale: 1 }}
                            exit={{ opacity: 0 }}
                        >
                            <Trace_Tree_Node node={root_node} depth={0} />
                        </motion.div>
                    ))}
                </AnimatePresence>

                {active_tree.length === 0 && (
                    <div className="flex flex-col items-center justify-center h-48 opacity-20">
                        <Share2 size={32} className="text-zinc-500 mb-2" />
                        <span className="sovereign-header-text !text-zinc-600 text-center px-4">
                            {i18n.t('trace.waiting')}<br />{i18n.t('trace.waiting_hint')}
                        </span>
                    </div>
                )}
            </div>
        </div>
    );
};
