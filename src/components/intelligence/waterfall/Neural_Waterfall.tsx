/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / Neural_Waterfall
 * - **Primary Entrypoints**: `Neural_Waterfall`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React, { useMemo, useState, useLayoutEffect, useRef } from 'react';
import { Network, ExternalLink, Minimize2 } from 'lucide-react';
import { use_trace_store } from '../../../stores/trace_store';
import { type Trace_Node } from '../../../types';
import { use_agent_store } from '../../../stores/agent_store';
import { use_tab_store } from '../../../stores/tab_store';
import { i18n } from '../../../i18n';
import clsx from 'clsx';
import { Tooltip, Portal_Window } from '../../ui';

import {
    TickerContext,
    TICK_INTERVAL_MS,
    ROW_HEIGHT_PX,
    VIEWPORT_PADDING_PX,
} from './WaterfallTicker';
import { Waterfall_Row } from './WaterfallRow';
import { Trace_Detail_Panel, LocalErrorBoundary } from './TraceDetailPanel';

export const Neural_Waterfall: React.FC<{ is_detached_view?: boolean }> = ({ is_detached_view = false }) => {
    const { active_trace_id, get_trace_tree } = use_trace_store();
    const { get_agent } = use_agent_store();
    const { is_trace_stream_detached, toggle_trace_stream_detachment } = use_tab_store();

    const [zoom_multiplier, set_zoom_multiplier] = useState(1);
    const [selected_span_id, set_selected_span_id] = useState<string | null>(null);
    const [is_details_detached, set_is_details_detached] = useState(false);

    const container_ref = useRef<HTMLDivElement>(null);
    const wrapper_ref = useRef<HTMLDivElement>(null);
    const [container_width, set_container_width] = useState(800);
    const [scroll_top, set_scroll_top] = useState(0);
    const [viewport_height, set_viewport_height] = useState(400);
    const [render_start_time] = useState(() => Date.now());

    const [is_expanded, set_is_expanded] = useState(false);
    const [original_rect, set_original_rect] = useState<{ left: number; top: number; width: number } | null>(null);
    const click_timer_ref = useRef<NodeJS.Timeout | null>(null);

    React.useEffect(() => {
        return () => {
            if (click_timer_ref.current) {
                clearTimeout(click_timer_ref.current);
            }
        };
    }, []);

    React.useEffect(() => {
        if (!is_expanded || is_detached_view || !wrapper_ref.current) return;
        const update_rect = () => {
            if (wrapper_ref.current) {
                const rect = wrapper_ref.current.getBoundingClientRect();
                set_original_rect({ left: rect.left, top: rect.top, width: rect.width });
            }
        };
        update_rect();
        const observer = new ResizeObserver(update_rect);
        observer.observe(wrapper_ref.current);
        return () => observer.disconnect();
    }, [is_expanded, is_detached_view]);

    const handle_header_click = (e: React.MouseEvent) => {
        if (is_detached_view) return;

        const target = e.target as HTMLElement;
        if (target.closest('button') || target.closest('input')) {
            return;
        }

        if (e.detail === 1) {
            click_timer_ref.current = setTimeout(() => {
                if (!is_expanded && wrapper_ref.current) {
                    const rect = wrapper_ref.current.getBoundingClientRect();
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

    // Shared Ticker Registry (NW-008)
    const ticker = useMemo(() => {
        const listeners = new Set<(now: number) => void>();
        const state = { interval: null as NodeJS.Timeout | null };
        return {
            subscribe(listener: (now: number) => void) {
                listeners.add(listener);
                if (listeners.size === 1) {
                    state.interval = setInterval(() => {
                        const now = Date.now();
                        listeners.forEach(l => l(now));
                    }, TICK_INTERVAL_MS);
                }
                return () => {
                    listeners.delete(listener);
                    if (listeners.size === 0 && state.interval) {
                        clearInterval(state.interval);
                        state.interval = null;
                    }
                };
            }
        };
    }, []);

    // Flatten tree and calculate timeline metrics
    const timeline_spans = useMemo(() => {
        if (!active_trace_id) return [];

        const raw_tree = get_trace_tree(active_trace_id);
        const flat: (Trace_Node & { depth: number })[] = [];

        // SAFETY: Iterative DFS to avoid stack overflow on deep traces
        const stack: { nodes: Trace_Node[]; depth: number; index: number }[] = [
            { nodes: raw_tree, depth: 0, index: 0 }
        ];

        while (stack.length > 0) {
            const current = stack[stack.length - 1];
            if (current.index < current.nodes.length) {
                const node = current.nodes[current.index];
                flat.push({ ...node, depth: current.depth });
                current.index++;
                if (node.children?.length) {
                    stack.push({ nodes: node.children, depth: current.depth + 1, index: 0 });
                }
            } else {
                stack.pop();
            }
        }

        return flat;
    }, [active_trace_id, get_trace_tree]);

    // Ticker-free boundary calculations
    const { min_time, total_duration } = useMemo(() => {
        if (timeline_spans.length === 0) return { min_time: 0, total_duration: 0 };

        const min = Math.min(...timeline_spans.map(s => s.start_time));
        const max = Math.max(...timeline_spans.map(s => s.end_time || s.start_time), render_start_time);
        const duration = Math.max(1, max - min);

        return { min_time: min, total_duration: duration };
    }, [timeline_spans, render_start_time]);

    // Measure viewport layout & track resizes (NW-002 / NW-010)
    useLayoutEffect(() => {
        const container = container_ref.current;
        if (!container) return;

        const handle_resize = () => {
            requestAnimationFrame(() => {
                if (!container) return;
                set_container_width(container.getBoundingClientRect().width);
                set_viewport_height(container.clientHeight);
            });
        };

        handle_resize();

        const observer = new ResizeObserver(handle_resize);
        observer.observe(container);

        return () => observer.disconnect();
    }, []);

    // Fit-to-width default zoom calculation
    const default_zoom = useMemo(() => {
        if (total_duration <= 0) return 1;
        return Math.max(0.0001, (container_width - VIEWPORT_PADDING_PX) / total_duration);
    }, [total_duration, container_width]);

    const zoom_factor = useMemo(() => {
        return default_zoom * zoom_multiplier;
    }, [default_zoom, zoom_multiplier]);

    const timeline_width = useMemo(() => {
        return Math.max(container_width - VIEWPORT_PADDING_PX, total_duration * zoom_factor);
    }, [container_width, total_duration, zoom_factor]);

    const selected_span = useMemo(() => {
        if (!selected_span_id) return null;
        return timeline_spans.find(s => s.id === selected_span_id) || null;
    }, [timeline_spans, selected_span_id]);

    // Viewport Virtualization calculations (NW-002)
    const { visible_spans, total_list_height } = useMemo(() => {
        const total_height = timeline_spans.length * ROW_HEIGHT_PX;
        if (timeline_spans.length === 0) return { visible_spans: [], total_list_height: 0 };

        // Add 5 rows buffer above/below viewport to prevent flickering on fast scrolls
        const start_idx = Math.max(0, Math.floor(scroll_top / ROW_HEIGHT_PX) - 5);
        const end_idx = Math.min(timeline_spans.length, Math.ceil((scroll_top + viewport_height) / ROW_HEIGHT_PX) + 5);

        const sliced = timeline_spans.slice(start_idx, end_idx).map((span, idx) => ({
            span,
            top_px: (start_idx + idx) * ROW_HEIGHT_PX
        }));

        return { visible_spans: sliced, total_list_height: total_height };
    }, [timeline_spans, scroll_top, viewport_height]);

    const handle_scroll = (e: React.UIEvent<HTMLDivElement>) => {
        set_scroll_top(e.currentTarget.scrollTop);
    };

    return (
        <TickerContext.Provider value={ticker}>
            <div
                ref={wrapper_ref}
                className={clsx(
                    !is_detached_view && "h-64 border-t border-[color:var(--color-surface)] shrink-0 relative",
                    is_detached_view && "h-full w-full"
                )}
            >
                <div
                    className={clsx(
                        "flex-grow flex overflow-hidden relative group transition-all duration-300 w-full h-full",
                        !is_detached_view && !is_expanded && "sovereign-card overflow-hidden border-t border-[color:var(--color-surface)] shrink-0",
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

                    <div className="flex-1 flex flex-col overflow-hidden">
                        <Tooltip content={i18n.t('trace_stream.tooltip')} position="left">
                            <div 
                                onClick={handle_header_click}
                                className="relative z-10 p-3 border-b border-[color:var(--color-border)] bg-[color:var(--color-background)] flex items-center justify-between transition-colors cursor-help select-none"
                            >
                                <h3 className="sovereign-header-text flex items-center gap-2">
                                    <Network size={12} strokeWidth={1.5} className="text-cyan-500" />
                                    {i18n.t('trace_stream.title')}
                                    {total_duration > 0 && (
                                        <span className="text-[9px] font-mono text-zinc-600 ml-2 normal-case tracking-normal">
                                            {total_duration}ms Total
                                        </span>
                                    )}
                                </h3>

                                <div className="flex items-center gap-2">
                                    {/* Zoom Slider */}
                                    {active_trace_id && timeline_spans.length > 0 && (
                                        <div className="flex items-center gap-2 mr-3 bg-zinc-900 border border-zinc-800 rounded-lg px-2.5 py-1">
                                            <span className="text-[8px] font-mono text-zinc-500 uppercase tracking-widest">Zoom</span>
                                            <input
                                                aria-label="Timeline Zoom"
                                                type="range"
                                                min="0.2"
                                                max="5"
                                                step="0.1"
                                                value={zoom_multiplier}
                                                onChange={e => set_zoom_multiplier(parseFloat(e.target.value))}
                                                onClick={(e) => e.stopPropagation()}
                                                className="w-16 accent-cyan-500 bg-zinc-800 rounded-lg cursor-pointer h-1"
                                            />
                                            <span className="text-[8px] font-mono text-zinc-400 font-bold">{Math.round(zoom_multiplier * 100)}%</span>
                                        </div>
                                    )}

                                    <div className="flex gap-1.5 mr-2">
                                        <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                                        <div className="w-2.5 h-2.5 rounded-full bg-zinc-800 border border-zinc-700"></div>
                                    </div>
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            toggle_trace_stream_detachment();
                                        }}
                                        className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                                        title={is_trace_stream_detached ? i18n.t('trace_stream.recall_tooltip') : i18n.t('trace_stream.detach_tooltip')}
                                    >
                                        {is_trace_stream_detached ? <Minimize2 size={14} strokeWidth={1.5} /> : <ExternalLink size={14} strokeWidth={1.5} />}
                                    </button>
                                </div>
                            </div>
                        </Tooltip>

                    <div
                        ref={container_ref}
                        onScroll={handle_scroll}
                        className="flex-grow overflow-x-auto overflow-y-auto p-4 custom-scrollbar relative z-10"
                    >
                        {!active_trace_id || timeline_spans.length === 0 ? (
                            <div className="flex flex-col items-center justify-center h-full opacity-30 text-center px-6">
                                <Network size={24} strokeWidth={1.5} className="mb-3 text-cyan-500/50" />
                                <p className="sovereign-header-text !text-zinc-500">
                                     LINK READY :: AWAITING TELEMETRY
                                </p>
                            </div>
                        ) : (
                            <LocalErrorBoundary>
                                <div style={{ width: `${128 + 32 + timeline_width}px`, height: `${total_list_height}px` }} className="relative pr-8">
                                    {/* Unified timeline grid background */}
                                    <div
                                        className="absolute inset-y-0 bg-[linear-gradient(to_right,#ffffff03_1px,transparent_1px)] pointer-events-none"
                                        style={{
                                            left: `128px`,
                                            width: `${timeline_width}px`,
                                            backgroundSize: `${200 * zoom_factor}px 100%`
                                        }}
                                    />
                                    {visible_spans.map(({ span, top_px }) => (
                                        <Waterfall_Row
                                            key={span.id}
                                            span={span}
                                            top_px={top_px}
                                            min_time={min_time}
                                            total_duration={total_duration}
                                            zoom_factor={zoom_factor}
                                            agent_name={get_agent(span.agent_id)?.name || span.agent_id}
                                            on_select={set_selected_span_id}
                                            is_selected={selected_span_id === span.id}
                                        />
                                    ))}
                                </div>
                            </LocalErrorBoundary>
                        )}
                    </div>
                </div>

                {/* Inline Detail Flyout */}
                {selected_span && !is_details_detached && (
                    <Trace_Detail_Panel
                        span={selected_span}
                        agent_name={get_agent(selected_span.agent_id)?.name || selected_span.agent_id}
                        is_detached={false}
                        on_close={() => set_selected_span_id(null)}
                        on_detach={() => set_is_details_detached(true)}
                    />
                )}

                {/* Detached Window Overlay Placeholder in Main Panel */}
                {selected_span && is_details_detached && (
                    <div
                        data-testid="detached-overlay-placeholder"
                        className="w-80 border-l border-zinc-800 bg-zinc-950/80 backdrop-blur-sm p-6 flex flex-col items-center justify-center text-center relative select-none animate-in slide-in-from-right duration-200"
                    >
                        <div className="space-y-4 relative z-10 flex flex-col items-center justify-center">
                            <div className="relative inline-block">
                                <ExternalLink size={24} strokeWidth={1.5} className="text-zinc-600 animate-pulse" />
                                <div className="absolute inset-0 bg-cyan-500/10 blur-xl rounded-full" />
                            </div>
                            <div className="space-y-1">
                                <h4 className="text-[11px] font-bold tracking-[0.15em] text-zinc-300 uppercase">
                                    {i18n.t('layout.sector_detached') || 'SECTOR DETACHED'}
                                </h4>
                                <p className="text-[8px] text-zinc-500 font-mono uppercase tracking-widest">
                                    LINK ESTABLISHED :: DETAILS_DETACHED
                                </p>
                            </div>
                            <button
                                onClick={() => set_is_details_detached(false)}
                                className="px-4 py-2 bg-zinc-850 hover:bg-zinc-800 border border-zinc-700 text-zinc-200 text-[9px] font-black uppercase tracking-[0.15em] rounded-md transition-all active:scale-95 cursor-pointer"
                            >
                                {i18n.t('layout.recall_sector') || 'RECALL SECTOR'}
                            </button>
                        </div>
                    </div>
                )}

                {/* Detached Window for details */}
                {selected_span && is_details_detached && (
                    <Portal_Window
                        id={`span-detail-${selected_span.id}`}
                        title={`Trace Detail: ${selected_span.name}`}
                        on_close={() => set_is_details_detached(false)}
                        width={500}
                        height={600}
                    >
                        <Trace_Detail_Panel
                            span={selected_span}
                            agent_name={get_agent(selected_span.agent_id)?.name || selected_span.agent_id}
                            is_detached={true}
                            on_close={() => {
                                set_is_details_detached(false);
                                set_selected_span_id(null);
                            }}
                        />
                    </Portal_Window>
                )}
                </div>
            </div>
        </TickerContext.Provider>
    );
};
