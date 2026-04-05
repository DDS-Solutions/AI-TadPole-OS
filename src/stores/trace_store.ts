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

/**
 * Trace_Span
 * Represents an OpenTelemetry-compatible span for granular activity tracking.
 * Features strict snake_case naming for backend parity and serialization.
 */
export interface Trace_Span {
    id: string;             // Span ID (16 hex chars)
    trace_id: string;        // Trace ID (32 hex chars)
    parent_id?: string;      // Parent Span ID if this is a child
    name: string;           // e.g., "agent_run" or "execute_tool: fetch_url"
    agent_id: string;        // The agent executing this span
    mission_id: string;      // The mission context
    start_time: number;      // Unix epoch MS
    end_time?: number;       // Unix epoch MS (undefined if running)
    status: 'running' | 'success' | 'error';
    attributes: Record<string, string | number | boolean>;
}

/**
 * Trace_Node
 * Hierarchical tree structure for UI rendering and Gantt chart visualization.
 */
export interface Trace_Node extends Trace_Span {
    children: Trace_Node[];
}

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

    add_span: (span) => set((state) => ({
        spans: {
            ...state.spans,
            [span.id]: span
        }
    })),

    update_span: (id, updates) => set((state) => {
        const existing = state.spans[id];
        if (!existing) return state;
        return {
            spans: {
                ...state.spans,
                [id]: { ...existing, ...updates }
            }
        };
    }),

    set_active_trace: (trace_id) => set({ active_trace_id: trace_id }),

    get_trace_tree: (trace_id) => {
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

    clear_trace: (trace_id) => set((state) => {
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
    }),

    clear_all: () => set({ spans: {}, active_trace_id: null })
}));

