import { create } from 'zustand';

// OTel Span structure representing a unit of work
export interface TraceSpan {
    id: string;             // Span ID (16 hex chars)
    traceId: string;        // Trace ID (32 hex chars)
    parentId?: string;      // Parent Span ID if this is a child
    name: string;           // e.g., "agent_run" or "execute_tool: fetch_url"
    agentId: string;        // The agent executing this span
    missionId: string;      // The mission context
    startTime: number;      // Unix epoch MS
    endTime?: number;       // Unix epoch MS (undefined if running)
    status: 'running' | 'success' | 'error';
    attributes: Record<string, string | number | boolean>;
}

// Tree structure for UI rendering
export interface TraceNode extends TraceSpan {
    children: TraceNode[];
}

interface TraceStoreState {
    spans: Record<string, TraceSpan>;
    activeTraceId: string | null;

    addSpan: (span: TraceSpan) => void;
    updateSpan: (id: string, updates: Partial<TraceSpan>) => void;
    setActiveTrace: (traceId: string) => void;

    // Selectors
    getTraceTree: (traceId: string) => TraceNode[];
    clearTrace: (traceId: string) => void;
    clearAll: () => void;
}

export const useTraceStore = create<TraceStoreState>((set, get) => ({
    spans: {},
    activeTraceId: null,

    addSpan: (span) => set((state) => ({
        spans: {
            ...state.spans,
            [span.id]: span
        }
    })),

    updateSpan: (id, updates) => set((state) => {
        const existing = state.spans[id];
        if (!existing) return state;
        return {
            spans: {
                ...state.spans,
                [id]: { ...existing, ...updates }
            }
        };
    }),

    setActiveTrace: (traceId) => set({ activeTraceId: traceId }),

    getTraceTree: (traceId) => {
        const { spans } = get();
        const traceSpans = Object.values(spans).filter(s => s.traceId === traceId);

        // Build an O(1) lookup map
        const nodeMap: Record<string, TraceNode> = {};
        traceSpans.forEach(s => {
            nodeMap[s.id] = { ...s, children: [] };
        });

        const rootNodes: TraceNode[] = [];

        // Build the tree
        Object.values(nodeMap).forEach(node => {
            if (node.parentId && nodeMap[node.parentId]) {
                nodeMap[node.parentId].children.push(node);
            } else {
                rootNodes.push(node);
            }
        });

        return rootNodes;
    },

    clearTrace: (traceId) => set((state) => {
        const newSpans = { ...state.spans };
        Object.keys(newSpans).forEach(id => {
            if (newSpans[id].traceId === traceId) {
                delete newSpans[id];
            }
        });
        return {
            spans: newSpans,
            activeTraceId: state.activeTraceId === traceId ? null : state.activeTraceId
        };
    }),

    clearAll: () => set({ spans: {}, activeTraceId: null })
}));
