/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / trace_store
 * - **Primary Entrypoints**: `use_trace_store`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { create } from 'zustand';
import { event_bus } from '../services/event_bus';
import type { Trace_Span, Trace_Node } from '../types';

interface Trace_Store_State {
    spans: Record<string, Trace_Span>;
    active_trace_id: string | null;

    // Actions
    add_span: (span: Trace_Span) => void;
    update_span: (id: string, updates: Partial<Trace_Span>) => void;
    touch_activity: (id: string) => void;
    reap_stale_spans: (now?: number) => number;
    set_active_trace: (trace_id: string) => void;

    // Selectors
    get_trace_tree: (trace_id: string) => Trace_Node[];
    clear_trace: (trace_id: string) => void;
    clear_all: () => void;
}

/**
 * use_trace_store
 * Centralized observability store for system traces and agent execution telemetry.
 */
export const use_trace_store = create<Trace_Store_State>((set, get) => ({
    spans: {},
    active_trace_id: null,

    add_span: (span: Trace_Span): void => {
        const enriched: Trace_Span = {
            ...span,
            last_activity_at: span.last_activity_at || span.start_time || Date.now()
        };
        set((state) => ({
            spans: {
                ...state.spans,
                [span.id]: enriched
            }
        }));
        event_bus.emit_trace(enriched);
    },

    update_span: (id: string, updates: Partial<Trace_Span>): void => {
        set((state) => {
            const existing = state.spans[id];
            if (!existing) return state;
            const updated = {
                ...existing,
                ...updates,
                last_activity_at: updates.last_activity_at || Date.now()
            };
            event_bus.emit_trace(updated);
            return {
                spans: {
                    ...state.spans,
                    [id]: updated
                }
            };
        });
    },

    touch_activity: (id: string): void => {
        set((state) => {
            const existing = state.spans[id];
            if (!existing || existing.status !== 'running') return state;
            return {
                spans: {
                    ...state.spans,
                    [id]: { ...existing, last_activity_at: Date.now() }
                }
            };
        });
    },

    reap_stale_spans: (now: number = Date.now()): number => {
        let reaped_count = 0;
        set((state) => {
            const new_spans = { ...state.spans };
            let has_mutations = false;

            Object.keys(new_spans).forEach((id) => {
                const s = new_spans[id];
                if (s.status === 'running') {
                    const ttl_ms = (s.timeout_seconds || 60) * 1000;
                    const last_act = s.last_activity_at || s.start_time;
                    if (now - last_act >= ttl_ms) {
                        new_spans[id] = {
                            ...s,
                            status: 'error',
                            end_time: now,
                            attributes: {
                                ...s.attributes,
                                error: 'SPAN_TIMEOUT_REAPED',
                                'error.reason': `Span exceeded ${s.timeout_seconds || 60}s inactivity threshold without closure`
                            }
                        };
                        has_mutations = true;
                        reaped_count += 1;
                        event_bus.emit_trace(new_spans[id]);
                    }
                }
            });

            return has_mutations ? { spans: new_spans } : state;
        });
        return reaped_count;
    },

    set_active_trace: (trace_id: string): void => { set({ active_trace_id: trace_id }); },

    get_trace_tree: (trace_id: string): Trace_Node[] => {
        const { spans } = get();
        const trace_spans = Object.values(spans).filter(s => s.trace_id === trace_id);

        // Build an O(1) lookup map
        const node_map: Record<string, Trace_Node> = {};
        trace_spans.forEach(s => {
            node_map[s.id] = { ...s, children: [] };
        });

        const root_nodes: Trace_Node[] = [];

        // Build the tree
        Object.values(node_map).forEach(node => {
            if (node.parent_id && node_map[node.parent_id]) {
                node_map[node.parent_id].children.push(node);
            } else {
                root_nodes.push(node);
            }
        });

        return root_nodes;
    },

    clear_trace: (trace_id: string): void => {
        set((state) => {
            const new_spans = { ...state.spans };
            Object.keys(new_spans).forEach(id => {
                if (new_spans[id].trace_id === trace_id) {
                    delete new_spans[id];
                }
            });
            return {
                spans: new_spans,
                active_trace_id: state.active_trace_id === trace_id ? null : state.active_trace_id
            };
        });
    },

    clear_all: (): void => { set({ spans: {}, active_trace_id: null }); }
}));

// Initialize Hub Synchronization
event_bus.subscribe_traces?.((span: unknown) => {
    const trace_span = span as Trace_Span;
    const state = use_trace_store.getState();
    const existing = state.spans[trace_span.id];
    
    // Simple conflict resolution: Only update if the span is new or has changed.
    // If the incoming trace is a partial update, merge it with existing span properties.
    const merged = existing ? { ...existing, ...trace_span } : trace_span;
    if (!existing || JSON.stringify(existing) !== JSON.stringify(merged)) {
        use_trace_store.setState((state) => ({
            spans: {
                ...state.spans,
                [trace_span.id]: merged
            }
        }));
    }
});
