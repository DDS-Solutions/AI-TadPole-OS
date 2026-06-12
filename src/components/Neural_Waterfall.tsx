/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-fidelity timeline visualization (Gantt style) of swarm operations. 
 * Renders trace spans as interactive bars with real-time "Now" tickers for running tasks.
 * Optimized for local ticking (consolidated O(1) interval), viewport virtualization, 
 * hardware-accelerated transforms, HTML sanitization, and PII masking.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Clock skew, DOM size exhaustion, or unescaped HTML characters.
 * - **Telemetry Link**: Search for `[Neural_Waterfall]` in UI tracing.
 */

import React, { useMemo, useState, useEffect, useRef, useLayoutEffect } from 'react';
import { Network, ExternalLink, Minimize2, X, Clock, Terminal, Activity, CheckCircle2, AlertTriangle } from 'lucide-react';
import { use_trace_store } from '../stores/trace_store';
import { type Trace_Node } from '../types';
import { use_agent_store } from '../stores/agent_store';
import { use_tab_store } from '../stores/tab_store';
import { i18n } from '../i18n';
import clsx from 'clsx';
import { Tooltip, Portal_Window } from './ui';

// CONSTANTS (IMR-02)
const TICK_INTERVAL_MS = 500;
const MIN_BAR_WIDTH_PX = 8;
const ROW_HEIGHT_PX = 38;
const ROW_TRACK_HEIGHT_PX = 32;
const ROW_HEADER_WIDTH_PX = 128;
const VIEWPORT_PADDING_PX = 160;

// Shared Ticker Registry Context (NW-008)
const TickerContext = React.createContext<{
    subscribe: (listener: (now: number) => void) => () => void;
} | null>(null);

// PII & Secrets Redaction (SEC-002 / SEC-004)
const SENSITIVE_KEYS = /token|api_key|secret|password|authorization|private_key|credential/i;
const SENSITIVE_VALUES_REGEX = /bearer\s+[a-zA-Z0-9_\-.]+|ey[a-zA-Z0-9_\-.]+\.ey[a-zA-Z0-9_\-.]+\.[a-zA-Z0-9_\-.]+|ghp_[a-zA-Z0-9]+|sk_live_[a-zA-Z0-9]+/i;

const redact_attributes = (attributes: Record<string, string | number | boolean>) => {
    const redacted: Record<string, string | number | boolean> = {};
    for (const [key, value] of Object.entries(attributes || {})) {
        const is_sensitive_key = SENSITIVE_KEYS.test(key);
        const is_sensitive_value = typeof value === 'string' && SENSITIVE_VALUES_REGEX.test(value);
        if (is_sensitive_key || is_sensitive_value) {
            redacted[key] = '[REDACTED]';
        } else {
            redacted[key] = value;
        }
    }
    return redacted;
};

// Failsafe Error Boundary (Reliability / Clock Skew)
class LocalErrorBoundary extends React.Component<{ children: React.ReactNode }, { hasError: boolean }> {
    constructor(props: { children: React.ReactNode }) {
        super(props);
        this.state = { hasError: false };
    }
    
    static getDerivedStateFromError() {
        return { hasError: true };
    }
    
    componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        console.error("[LocalErrorBoundary] Neural Waterfall crashed:", error, errorInfo);
    }
    
    render() {
        if (this.state.hasError) {
            return (
                <div className="p-4 bg-red-950/20 border border-red-500/20 text-red-400 font-mono text-xs rounded-lg m-4">
                    ⚠️ Observability Link Degraded: Trace rendering failure.
                </div>
            );
        }
        return this.props.children;
    }
}

interface TraceDetailPanelProps {
    span: Trace_Node;
    agent_name: string;
    is_detached: boolean;
    on_close: () => void;
    on_detach?: () => void;
}

const Trace_Detail_Panel: React.FC<TraceDetailPanelProps> = ({
    span,
    agent_name,
    is_detached,
    on_close,
    on_detach
}) => {
    const is_running = !span.end_time;
    const [local_now, set_local_now] = useState(() => Date.now());
    const ticker = React.useContext(TickerContext);

    useEffect(() => {
        if (!is_running || !ticker) return;
        return ticker.subscribe(set_local_now);
    }, [is_running, ticker]);

    const duration = is_running ? (local_now - span.start_time) : (span.end_time! - span.start_time);
    const sanitized_attributes = useMemo(() => redact_attributes(span.attributes || {}), [span.attributes]);
    
    return (
        <div className={clsx(
            "flex flex-col h-full bg-zinc-950 font-sans border-[color:var(--color-border)] select-none",
            is_detached ? "w-full p-6" : "w-80 border-l border-zinc-800 bg-zinc-900/60 backdrop-blur-md p-4 animate-in slide-in-from-right duration-200"
        )}>
            {/* Header */}
            <div className="flex items-center justify-between border-b border-white/5 pb-3 mb-4">
                <h4 className="text-xs font-bold text-zinc-200 uppercase tracking-wider truncate flex items-center gap-1.5">
                    <Terminal size={12} strokeWidth={1.5} className="text-cyan-500" />
                    {span.name}
                </h4>
                <div className="flex items-center gap-2">
                    {on_detach && !is_detached && (
                        <button
                            onClick={on_detach}
                            className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                            title="Detach Details Window"
                        >
                            <ExternalLink size={14} strokeWidth={1.5} />
                        </button>
                    )}
                    <button
                        onClick={on_close}
                        className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                        title="Close details"
                    >
                        <X size={14} strokeWidth={1.5} />
                    </button>
                </div>
            </div>

            {/* Content Area */}
            <div className="flex-1 overflow-y-auto space-y-4 pr-1 custom-scrollbar">
                {/* Status and Duration */}
                <div className="grid grid-cols-2 gap-3">
                    <div className="bg-zinc-900 border border-white/5 rounded-lg p-2.5">
                        <span className="text-[8px] font-bold text-zinc-500 uppercase tracking-widest block mb-1">Status</span>
                        <div className="flex items-center gap-1.5">
                            {span.status === 'success' && <CheckCircle2 size={12} strokeWidth={1.5} className="text-emerald-500" />}
                            {span.status === 'error' && <AlertTriangle size={12} strokeWidth={1.5} className="text-red-500" />}
                            {span.status === 'running' && <Activity size={12} strokeWidth={1.5} className="text-cyan-500 animate-pulse" />}
                            <span className={clsx(
                                "text-[10px] font-bold uppercase tracking-wider",
                                span.status === 'success' && "text-emerald-400",
                                span.status === 'error' && "text-red-400",
                                span.status === 'running' && "text-cyan-400"
                            )}>
                                {span.status}
                            </span>
                        </div>
                    </div>

                    <div className="bg-zinc-900 border border-white/5 rounded-lg p-2.5">
                        <span className="text-[8px] font-bold text-zinc-500 uppercase tracking-widest block mb-1">Elapsed Time</span>
                        <div className="flex items-center gap-1.5 font-mono text-[10px] text-zinc-300 font-bold">
                            <Clock size={12} strokeWidth={1.5} className="text-zinc-500" />
                            {Math.max(0, duration)}ms
                        </div>
                    </div>
                </div>

                {/* Agent Assignment */}
                <div className="bg-zinc-900 border border-white/5 rounded-lg p-3">
                    <span className="text-[8px] font-bold text-zinc-500 uppercase tracking-widest block mb-2">Executing Agent</span>
                    <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-bold text-xs text-cyan-400">
                            {agent_name[0] || '?'}
                        </div>
                        <div className="flex flex-col">
                            <span className="text-xs font-bold text-zinc-200">{agent_name}</span>
                            <span className="text-[9px] font-mono text-zinc-600 uppercase tracking-tighter block mt-0.5">ID: {span.agent_id}</span>
                        </div>
                    </div>
                </div>

                {/* Attributes Details */}
                <div className="space-y-2">
                    <span className="text-[8px] font-bold text-zinc-500 uppercase tracking-widest block">Span Attributes</span>
                    {Object.keys(sanitized_attributes).length === 0 ? (
                        <span className="text-[10px] text-zinc-600 italic font-mono block pl-1">No metadata attributes recorded.</span>
                    ) : (
                        <div className="space-y-2">
                            {Object.entries(sanitized_attributes).map(([key, value]) => (
                                <div key={key} className="bg-zinc-900/50 border border-white/5 rounded-lg p-2.5 font-mono">
                                    <span className="text-[8px] text-zinc-500 font-bold block mb-1 truncate" title={key}>{key}</span>
                                    <pre className="text-[10px] text-zinc-300 font-mono whitespace-pre-wrap break-all overflow-x-auto bg-black/30 p-1.5 rounded border border-white/5">
                                        {typeof value === 'object' ? JSON.stringify(value, null, 2) : String(value)}
                                    </pre>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export const Neural_Waterfall: React.FC<{ is_detached_view?: boolean }> = ({ is_detached_view = false }) => {
    const { active_trace_id, get_trace_tree } = use_trace_store();
    const { get_agent } = use_agent_store();
    const { is_trace_stream_detached, toggle_trace_stream_detachment } = use_tab_store();
    
    const [zoom_multiplier, set_zoom_multiplier] = useState(1);
    const [selected_span_id, set_selected_span_id] = useState<string | null>(null);
    const [is_details_detached, set_is_details_detached] = useState(false);

    const container_ref = useRef<HTMLDivElement>(null);
    const [container_width, set_container_width] = useState(800);
    const [scroll_top, set_scroll_top] = useState(0);
    const [viewport_height, set_viewport_height] = useState(400);
    const [render_start_time] = useState(() => Date.now());

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
            <div className={clsx(
                "flex-grow flex overflow-hidden relative group",
                !is_detached_view && "sovereign-card overflow-hidden h-64 border-t border-[color:var(--color-surface)] shrink-0",
                is_detached_view && "h-full"
            )}>
                {!is_detached_view && <div className="neural-grid opacity-[0.05]" />}
                
                <div className="flex-1 flex flex-col overflow-hidden">
                    <Tooltip content={i18n.t('trace_stream.tooltip')} position="left">
                        <div className="relative z-10 p-3 border-b border-[color:var(--color-border)] bg-[color:var(--color-background)] flex items-center justify-between transition-colors cursor-help">
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
                                    onClick={() => toggle_trace_stream_detachment()}
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
                                <div style={{ width: `${ROW_HEADER_WIDTH_PX + 32 + timeline_width}px`, height: `${total_list_height}px` }} className="relative pr-8">
                                    {/* Unified timeline grid background */}
                                    <div 
                                        className="absolute inset-y-0 bg-[linear-gradient(to_right,#ffffff03_1px,transparent_1px)] pointer-events-none"
                                        style={{ 
                                            left: `${ROW_HEADER_WIDTH_PX}px`,
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
        </TickerContext.Provider>
    );
};

interface WaterfallRowProps {
    span: Trace_Node & { depth: number };
    top_px: number;
    min_time: number;
    total_duration: number;
    zoom_factor: number;
    agent_name: string;
    on_select: (id: string) => void;
    is_selected: boolean;
}

const Waterfall_Row = React.memo(({ 
    span, 
    top_px,
    min_time,
    total_duration, 
    zoom_factor, 
    agent_name, 
    on_select, 
    is_selected 
}: WaterfallRowProps) => {
    const is_running = !span.end_time;
    const [local_now, set_local_now] = useState(() => Date.now());
    const ticker = React.useContext(TickerContext);

    useEffect(() => {
        if (!is_running || !ticker) return;
        return ticker.subscribe(set_local_now);
    }, [is_running, ticker]);

    const duration = is_running ? (local_now - span.start_time) : (span.end_time! - span.start_time);
    
    // Transform-only calculations to prevent layout thrashing (NW-003 / NW-007)
    const left_px = Math.max(0, span.start_time - min_time) * zoom_factor;
    const width_px = Math.max(MIN_BAR_WIDTH_PX, duration * zoom_factor);

    return (
        <div 
            onClick={() => on_select(span.id)}
            onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    on_select(span.id);
                    e.preventDefault();
                }
            }}
            tabIndex={0}
            role="row"
            className={clsx(
                "absolute left-0 right-0 flex items-center group cursor-pointer hover:bg-white/5 rounded-sm p-1 transition-colors select-none",
                is_selected && "bg-white/10"
            )}
            style={{
                top: `${top_px}px`,
                height: `${ROW_TRACK_HEIGHT_PX}px`
            }}
        >
            <div className="w-32 flex-shrink-0 text-right pr-4 truncate pt-1 z-10 sticky left-0 bg-zinc-950">
                <span className="text-[9px] font-mono text-zinc-500 block uppercase tracking-wider">{agent_name}</span>
                <span className="text-[8px] font-mono text-zinc-600 block truncate">{span.name}</span>
            </div>

            <div 
                className="flex-1 relative h-6 bg-[color:var(--color-surface)]/50 rounded overflow-hidden"
                style={{ width: `${total_duration * zoom_factor}px` }}
            >
                <div className="absolute inset-0 bg-[linear-gradient(to_right,#ffffff03_1px,transparent_1px)] bg-[size:10%] pointer-events-none" />

                <div
                    className={clsx(
                        "absolute top-1 bottom-1 rounded-sm flex items-center px-1 overflow-hidden border border-black/20 will-change-transform"
                    )}
                    style={{
                        transform: `translate3d(${left_px}px, 0, 0)`,
                        width: `${width_px}px`,
                        backgroundColor: is_running ? 'rgba(6, 182, 212, 0.8)' : span.status === 'error' ? 'rgba(239, 68, 68, 0.8)' : 'rgba(34, 197, 94, 0.8)'
                    }}
                >
                    {width_px > 45 && (
                        <span className="text-[8px] font-mono text-white/90 truncate drop-shadow-md">
                            {Math.max(0, duration)}ms
                        </span>
                    )}
                </div>
            </div>
        </div>
    );
}, (prev, next) => {
    // Only re-render if selection, boundaries, status, or configuration change (NW-006)
    if (prev.is_selected !== next.is_selected) return false;
    if (prev.top_px !== next.top_px) return false;
    if (prev.min_time !== next.min_time) return false;
    
    const was_running = !prev.span.end_time;
    const is_running = !next.span.end_time;
    if (was_running !== is_running) return false;
    
    // For running spans, let the local ticker handle updates; no need to trigger full React re-render from parent props
    if (is_running) {
        return (
            prev.span.id === next.span.id &&
            prev.span.status === next.span.status &&
            prev.zoom_factor === next.zoom_factor &&
            prev.agent_name === next.agent_name
        );
    }
    
    // For completed spans, compare static attributes
    return (
        prev.span.id === next.span.id &&
        prev.span.status === next.span.status &&
        prev.span.end_time === next.span.end_time &&
        prev.total_duration === next.total_duration &&
        prev.zoom_factor === next.zoom_factor &&
        prev.agent_name === next.agent_name
    );
});

// Metadata: [Neural_Waterfall]
