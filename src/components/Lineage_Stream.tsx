/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Lineage_Stream
 * - **Primary Entrypoints**: `Lineage_Stream`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useRef, useEffect, useCallback, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Activity, Share2, Clock, GitCommit, ExternalLink, Minimize2, AlertTriangle } from 'lucide-react';
import { use_agent_store } from '../stores/agent_store';
import { use_trace_store } from '../stores/trace_store';
import { type Trace_Node, get_span_error_details } from '../types';
import { use_tab_store } from '../stores/tab_store';
import { i18n } from '../i18n';
import clsx from 'clsx';
import { Tooltip } from './ui';
import { decodeAAAK, isAAAK } from '../utils/aaak_decoder';
import { Trace_Detail_Panel, LocalErrorBoundary } from './intelligence/waterfall/TraceDetailPanel';

interface TraceTreeNodeProps {
    node: Trace_Node;
    depth: number;
    on_select: (node: Trace_Node) => void;
    selected_id: string | null;
}

// Recursive component to render the OTel trace tree
// PERF: Memoize to prevent full tree re-renders on minor status updates
const Trace_Tree_Node = React.memo(({ 
    node, 
    depth,
    on_select,
    selected_id
}: TraceTreeNodeProps): React.ReactElement | null => {

    const store_res = use_agent_store(s => s?.get_agent);

    const get_agent = typeof store_res === 'function' ? store_res : ((store_res as unknown as { get_agent?: (id: string) => { name?: string } | undefined })?.get_agent || (() => undefined));
    const agent_name = get_agent(node.agent_id)?.name || node.agent_id;
    const is_selected = selected_id === node.id;

    const error_info = useMemo(() => get_span_error_details(node), [node]);
    if (depth > 50) return null;
    const is_node_error = node.status === 'error' || Boolean(error_info.error || (error_info.http_status && error_info.http_status >= 400));

    // Status colors
    const status_color = node.status === 'running'
        ? 'text-cyan-400 bg-cyan-400/10 border-cyan-400/30'
        : is_node_error
            ? 'text-red-400 bg-red-400/10 border-red-400/30'
            : 'text-emerald-400 bg-emerald-400/10 border-emerald-400/30';

    const handle_key_down = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            on_select(node);
        }
    };

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

                <div 
                    onClick={() => on_select(node)}
                    onKeyDown={handle_key_down}
                    role="button"
                    tabIndex={0}
                    aria-label={`Span ${node.name} (${node.status})`}
                    className={clsx(
                        "flex-1 p-3 bg-[color:var(--color-surface)]/60 border rounded-xl hover:bg-zinc-800/80 transition-all cursor-pointer focus:outline-none focus:ring-1 focus:ring-cyan-500/50",
                        is_selected 
                            ? "border-cyan-500/50 ring-1 ring-cyan-500/40 bg-zinc-800/90" 
                            : is_node_error 
                                ? "border-red-500/30 hover:border-red-500/50 bg-red-950/10" 
                                : "border-[color:var(--color-border)]"
                    )}
                >
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2 flex-wrap">
                            <div className={`w-2 h-2 rounded-full shadow-[0_0_8px_currentColor] ${status_color}`} />
                            <span className="text-[10px] font-mono font-bold text-zinc-300 uppercase tracking-widest">{agent_name}</span>
                            <span className="text-[9px] text-zinc-500 font-mono px-1.5 py-0.5 rounded bg-[color:color-mix(in_srgb,var(--color-background)_50%,transparent)]">
                                {node.name}
                            </span>
                            {is_node_error && (
                                <span 
                                    className="px-1.5 py-0.5 rounded text-[8px] font-mono font-bold bg-red-950/80 text-red-400 border border-red-800/50 flex items-center gap-1"
                                    title={error_info.reason || error_info.error || 'Execution Fault'}
                                >
                                    <AlertTriangle size={9} className="shrink-0" />
                                    <span className="truncate max-w-[120px]">{error_info.error || (error_info.http_status ? `HTTP ${error_info.http_status}` : 'ERROR')}</span>
                                </span>
                            )}
                        </div>
                        <span className="text-[9px] text-zinc-600 font-mono flex items-center gap-1 shrink-0">
                            <Clock size={10} />
                            {node.end_time ? `${node.end_time - node.start_time}ms` : i18n.t('trace.running')}
                        </span>
                    </div>

                    <div className="flex items-center gap-4 mt-2 pt-2 border-t border-[color:var(--color-border)]/50">
                        <span className="text-[8px] font-mono text-zinc-500 flex items-center gap-1">
                            <GitCommit size={10} /> {i18n.t('trace.span')}: {node.id.toUpperCase()}
                        </span>
                        {node.attributes && Object.keys(node.attributes).length > 0 && (
                            <span className="text-[8px] font-mono text-zinc-500 truncate max-w-[150px]">
                                {Object.entries(node.attributes).map(([k, v]) => {
                                    const val = String(v);
                                    return (
                                        <div key={k} className="flex gap-1">
                                            <span>{k}:</span>
                                            {isAAAK(val) ? (
                                                <Tooltip content={decodeAAAK(val)} position="top">
                                                    <span className="text-zinc-400 cursor-help border-b border-[color:var(--color-border)]">{val}</span>
                                                </Tooltip>
                                            ) : (
                                                <span>{val}</span>
                                            )}
                                        </div>
                                    );
                                })}
                            </span>
                        )}
                    </div>
                </div>
            </div>

            {/* Recursively render children */}
            {node.children && node.children.length > 0 && (
                <div className="flex flex-col relative w-full">
                    {(node.children || []).map((child: Trace_Node): React.ReactElement => (
                        <Trace_Tree_Node 
                            key={child.id} 
                            node={child} 
                            depth={depth + 1} 
                            on_select={on_select}
                            selected_id={selected_id}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}, (prev, next) => {
    return (
        prev.node.status === next.node.status &&
        prev.node.end_time === next.node.end_time &&
        prev.node.children === next.node.children &&
        prev.node.attributes === next.node.attributes &&
        prev.depth === next.depth &&
        prev.selected_id === next.selected_id &&
        (prev.selected_id === prev.node.id) === (next.selected_id === next.node.id)
    );
});

/**
 * Lineage_Stream
 * A real-time hierarchical telemetry feed visualizing the swarm's activity and parent-child relationships.
 * Refactored for strict snake_case compliance and consistent prop propagation.
 */
export const Lineage_Stream: React.FC<{ is_detached_view?: boolean }> = ({ is_detached_view = false }): React.ReactElement => {
    const { active_trace_id, get_trace_tree } = use_trace_store();
    const { get_agent } = use_agent_store();
    const { is_lineage_stream_detached, toggle_lineage_stream_detachment } = use_tab_store();
    const [selected_node, set_selected_node] = useState<Trace_Node | null>(null);
    const [sidebar_width, set_sidebar_width] = useState(380);
    const stream_ref = useRef<HTMLDivElement>(null);
    const resize_handlers_ref = useRef<{ on_move: (e: MouseEvent) => void; on_up: () => void } | null>(null);

    const [is_expanded, set_is_expanded] = useState(false);
    const [original_rect, set_original_rect] = useState<{ left: number; top: number; width: number } | null>(null);
    const container_ref = useRef<HTMLDivElement>(null);
    const click_timer_ref = useRef<NodeJS.Timeout | null>(null);

    const handle_select_node = useCallback((node: Trace_Node) => {
        set_selected_node(current => (current?.id === node.id ? null : node));
    }, []);

    // SAFETY: Ensure event listeners are cleaned up if component unmounts during drag
    React.useEffect(() => {
        return () => {
            if (resize_handlers_ref.current) {
                document.removeEventListener('mousemove', resize_handlers_ref.current.on_move);
                document.removeEventListener('mouseup', resize_handlers_ref.current.on_up);
            }
            if (click_timer_ref.current) {
                clearTimeout(click_timer_ref.current);
            }
        };
    }, []);

    // Sync dimensions using ResizeObserver when expanded
    useEffect(() => {
        if (!is_expanded || is_detached_view || !container_ref.current) return;
        const update_rect = () => {
            if (container_ref.current) {
                const rect = container_ref.current.getBoundingClientRect();
                set_original_rect({ left: rect.left, top: rect.top, width: rect.width });
            }
        };
        update_rect();
        const observer = new ResizeObserver(update_rect);
        observer.observe(container_ref.current);
        return () => observer.disconnect();
    }, [is_expanded, is_detached_view]);

    const active_tree = active_trace_id ? get_trace_tree(active_trace_id) : [];

    const handle_sidebar_resize_start = (e: React.MouseEvent): void => {
        if (is_detached_view || is_expanded) return;
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
            resize_handlers_ref.current = null;
        };

        resize_handlers_ref.current = { on_move: on_mouse_move, on_up: on_mouse_up };
        document.addEventListener('mousemove', on_mouse_move);
        document.addEventListener('mouseup', on_mouse_up);
    };

    const handle_header_click = (e: React.MouseEvent) => {
        if (is_detached_view) return;

        const target = e.target as HTMLElement;
        if (target.closest('button') || target.closest('input')) {
            return;
        }

        if (e.detail === 1) {
            click_timer_ref.current = setTimeout(() => {
                if (!is_expanded && container_ref.current) {
                    const rect = container_ref.current.getBoundingClientRect();
                    set_original_rect({ left: rect.left, top: rect.top, width: rect.width });
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

    return (
        <div
            ref={container_ref}
            className={clsx(
                !is_detached_view && "flex-1 min-h-[240px] flex flex-col",
                is_detached_view && "h-full w-full"
            )}
            style={!is_detached_view ? { width: sidebar_width } : undefined}
        >
            <div
                className={clsx(
                    "flex flex-col bg-[color:var(--color-background)]/20 border-l border-[color:var(--color-surface)] overflow-hidden relative group/sidebar transition-all duration-300",
                    !is_detached_view && !is_expanded && "flex-1 sovereign-card h-full w-full",
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
                } : (is_detached_view ? { width: '100%' } : { width: '100%' })}
                ref={stream_ref}
            >
                {!is_detached_view && !is_expanded && <div className="neural-grid opacity-[0.05]" />}
                {!is_detached_view && !is_expanded && (
                    <div
                        onMouseDown={handle_sidebar_resize_start}
                        className="absolute inset-y-0 left-0 w-1 cursor-col-resize hover:bg-emerald-500/20 active:bg-emerald-500/40 transition-colors z-20"
                    />
                )}

                <Tooltip content={i18n.t('trace.tooltip')} position="left">
                    <div 
                        onClick={handle_header_click}
                        className="relative z-10 p-3 border-b border-[color:var(--color-border)] bg-[color:var(--color-background)] flex items-center justify-between transition-colors cursor-help select-none"
                    >
                        <h3 className="sovereign-header-text flex items-center gap-2">
                            <Activity size={12} className="text-emerald-500" />
                            {i18n.t('trace.stream_title')}
                            {active_trace_id && (
                                <span className="text-[9px] font-mono px-2 py-0.5 rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)] text-zinc-500 truncate max-w-[120px] ml-2 normal-case tracking-normal">
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
                                onClick={(e) => {
                                    e.stopPropagation();
                                    toggle_lineage_stream_detachment();
                                }}
                                className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                                title={is_lineage_stream_detached ? i18n.t('trace_stream.recall_tooltip') : i18n.t('trace_stream.detach_tooltip')}
                            >
                                {is_lineage_stream_detached ? <Minimize2 size={14} /> : <ExternalLink size={14} />}
                            </button>
                        </div>
                    </div>
                </Tooltip>

                <LocalErrorBoundary fallbackMessage="OTel Trace Stream Degraded: Tree rendering failure.">
                    <div className="flex-1 overflow-y-auto p-4 space-y-2 custom-scrollbar relative">
                        <AnimatePresence>
                            {(active_tree || []).map((root_node): React.ReactElement => (
                                <motion.div
                                    key={root_node.id}
                                    initial={{ opacity: 0, scale: 0.95 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    exit={{ opacity: 0 }}
                                >
                                    <Trace_Tree_Node 
                                        node={root_node} 
                                        depth={0} 
                                        on_select={handle_select_node}
                                        selected_id={selected_node?.id || null}
                                    />
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
                </LocalErrorBoundary>

                {/* Node Inspection Drawer / Overlay */}
                {selected_node && (
                    <div
                        data-testid="lineage-detail-overlay"
                        className="absolute inset-0 z-30 bg-zinc-950/95 backdrop-blur-md flex flex-col animate-in slide-in-from-right duration-200"
                    >
                        <LocalErrorBoundary fallbackMessage="Trace Detail Degraded: Failed to render span metadata.">
                            <Trace_Detail_Panel
                                span={selected_node}
                                agent_name={get_agent(selected_node.agent_id)?.name || selected_node.agent_id}
                                is_detached={false}
                                on_close={() => set_selected_node(null)}
                            />
                        </LocalErrorBoundary>
                    </div>
                )}
            </div>
        </div>
    );
};
