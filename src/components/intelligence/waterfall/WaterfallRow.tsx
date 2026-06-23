/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **Waterfall Row**: Virtualised, memoised row component for the Neural Waterfall Gantt view.
 * Uses a custom memo comparator (NW-006) to prevent unnecessary re-renders — running spans
 * get updates only from the local TickerContext, not from parent prop changes.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Bar renders at 0px width → `MIN_BAR_WIDTH_PX` guard should prevent this.
 * - **Telemetry Link**: Search `[Neural_Waterfall]` in UI tracing.
 */

import React, { useState, useEffect } from 'react';
import clsx from 'clsx';
import { type Trace_Node } from '../../../types';
import { TickerContext, MIN_BAR_WIDTH_PX, ROW_TRACK_HEIGHT_PX, ROW_HEADER_WIDTH_PX } from './WaterfallTicker';

export interface WaterfallRowProps {
    span: Trace_Node & { depth: number };
    top_px: number;
    min_time: number;
    total_duration: number;
    zoom_factor: number;
    agent_name: string;
    on_select: (id: string) => void;
    is_selected: boolean;
}

export const Waterfall_Row = React.memo(({
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
            <div className="flex-shrink-0 text-right pr-4 truncate pt-1 z-10 sticky left-0 bg-zinc-950" style={{ width: `${ROW_HEADER_WIDTH_PX}px` }}>
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

Waterfall_Row.displayName = 'Waterfall_Row';

// Metadata: [WaterfallRow]
