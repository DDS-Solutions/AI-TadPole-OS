/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Lineage and execution observability buffer. 
 * Orchestrates the parent-child relationship tracking for distributed agent tasks and swarm mission histories.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Trace ID parentage loss during async handoffs, or recursive trace loops in circular mission flows.
 * - **Telemetry Link**: Search for `[TraceStore]` or `TRACE_PULSE` in service logs.
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
        set((state) => ({
            spans: {
                ...state.spans,
                [span.id]: span
            }
        }));
        event_bus.emit_trace(span);
    },

    update_span: (id: string, updates: Partial<Trace_Span>): void => {
        set((state) => {
            const existing = state.spans[id];
            if (!existing) return state;
            const updated = { ...existing, ...updates };
            event_bus.emit_trace(updated);
            return {
                spans: {
                    ...state.spans,
                    [id]: updated
                }
            };
        });
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
event_bus.subscribe_traces((span: unknown) => {
    const trace_span = span as Trace_Span;
    const state = use_trace_store.getState();
    const existing = state.spans[trace_span.id];
    
    // Simple conflict resolution: Only update if the span is new or has changed
    if (!existing || JSON.stringify(existing) !== JSON.stringify(trace_span)) {
        use_trace_store.setState((state) => ({
            spans: {
                ...state.spans,
                [trace_span.id]: trace_span
            }
        }));
    }
});

// Metadata: [trace_store]
