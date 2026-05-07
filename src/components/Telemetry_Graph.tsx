/**
 * @docs ARCHITECTURE:Telemetry
 * @docs OPERATIONS_MANUAL:Telemetry
 * 
 * ### AI Assist Note
 * **UI Component**: DAG-based execution flow visualizer for agent swarm missions. 
 * Employs `reactflow` and `dagre` for hierarchical node ranking and real-time status pulses (Running/Success/Error).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Layout collision if `dagre` fails to rank complex cyclic traces, node starvation due to missing parent spans, or edge resolution 404 during rapid purges.
 * - **Telemetry Link**: Search for `[Telemetry_Graph]` or `layout_compute` in UI tracing.
 */

import React, { useEffect, useMemo } from 'react';
import { motion } from 'framer-motion';
import ReactFlow, { 
    Background, 
    Controls, 
    type Node, 
    type Edge, 
    Position,
    MarkerType,
    useNodesState,
    useEdgesState,
    Handle,
    Panel
} from 'reactflow';
import dagre from 'dagre';
import { use_trace_store, type Trace_Span } from '../stores/trace_store';
import { use_agent_store } from '../stores/agent_store';
import 'reactflow/dist/style.css';
import { THEME_COLORS } from '../constants/theme';
import { i18n } from '../i18n';

const dagre_graph = new dagre.graphlib.Graph();
dagre_graph.setDefaultEdgeLabel(() => ({}));

const node_width = 220;
const node_height = 80;

// Custom Node Component for a more "Premium" look
const Telemetry_Node = ({ data }: { data: Trace_Span & { agent_name?: string } }) => {
    const is_error = data.status === 'error';
    const is_running = data.status === 'running';
    const is_tool = data.name.startsWith('execute_tool');

    return (
        <div className={`
            relative px-4 py-3 rounded-2xl border min-w-[200px]
            ${is_error ? 'bg-rose-950/20 border-rose-500/50' : 'bg-zinc-900/90 border-zinc-800'}
            ${is_running ? 'shadow-[0_0_20px_rgba(6,182,212,0.15)] ring-1 ring-cyan-500/30' : ''}
            backdrop-blur-xl transition-all duration-500
        `}>
            <Handle type="target" position={Position.Top} className="opacity-0" />
            
            <div className="flex flex-col gap-1">
                <div className="flex items-center justify-between gap-4">
                    <span className="text-[10px] font-black text-zinc-500 uppercase tracking-[2px] truncate">
                        {data.agent_name || 'NEURAL NODE'}
                    </span>
                    {is_running && (
                        <motion.div 
                            animate={{ opacity: [0.3, 1, 0.3] }}
                            transition={{ duration: 1.5, repeat: Infinity }}
                            className="w-1.5 h-1.5 rounded-full bg-cyan-400 shadow-[0_0_8px_#22d3ee]" 
                        />
                    )}
                </div>
                
                <h3 className="text-[12px] font-semibold text-zinc-100 truncate leading-tight">
                    {data.name.includes(':') ? data.name.split(':')[1] : data.name}
                </h3>

                <div className="flex items-center gap-2 mt-1">
                    <div className={`
                        text-[8px] font-bold px-1.5 py-0.5 rounded-md uppercase tracking-wider
                        ${is_tool ? 'bg-amber-500/10 text-amber-400' : 'bg-zinc-800 text-zinc-400'}
                    `}>
                        {is_tool ? 'Tool' : 'Phase'}
                    </div>
                    {data.end_time && (
                         <span className="text-[8px] text-zinc-600 font-mono">
                            {((data.end_time - data.start_time) / 1000).toFixed(2)}s
                         </span>
                    )}
                </div>
            </div>

            {is_running && (
                <motion.div 
                    layoutId="glow"
                    className="absolute inset-0 rounded-2xl border border-cyan-500/50 shadow-[0_0_15px_rgba(6,182,212,0.2)]"
                    animate={{ opacity: [0.1, 0.4, 0.1] }}
                    transition={{ duration: 2, repeat: Infinity }}
                />
            )}

            <Handle type="source" position={Position.Bottom} className="opacity-0" />
        </div>
    );
};

const node_types = {
    telemetry: Telemetry_Node,
};

const get_layouted_elements = (nodes: Node[], edges: Edge[], direction = 'TB') => {
    dagre_graph.setGraph({ rankdir: direction, nodesep: 50, ranksep: 70 });

    nodes.forEach((node) => {
        dagre_graph.setNode(node.id, { width: node_width, height: node_height });
    });

    edges.forEach((edge) => {
        dagre_graph.setEdge(edge.source, edge.target);
    });

    dagre.layout(dagre_graph);

    const layouted_nodes = nodes.map((node) => {
        const node_with_position = dagre_graph.node(node.id);
        
        // Return cloned objects to maintain React state immutability
        return {
            ...node,
            targetPosition: Position.Top,
            sourcePosition: Position.Bottom,
            position: {
                x: node_with_position.x - node_width / 2,
                y: node_with_position.y - node_height / 2,
            },
        };
    });

    return { nodes: layouted_nodes, edges };
};

export const Telemetry_Graph: React.FC<{ initial_mission_id?: string }> = ({ initial_mission_id }) => {
    const { spans, clear_all } = use_trace_store();
    const { agents } = use_agent_store();
    const [nodes, set_nodes, on_nodes_change] = useNodesState([]);
    const [edges, set_edges, on_edges_change] = useEdgesState([]);
    const [filter_mission_id, set_filter_mission_id] = React.useState<string | undefined>(initial_mission_id);

    const available_missions = useMemo(() => {
        const missions = new Set<string>();
        Object.values(spans).forEach(s => {
            if (s.mission_id && s.mission_id !== 'unknown') {
                missions.add(s.mission_id);
            }
        });
        return Array.from(missions);
    }, [spans]);

    // PERF: Memoize node and edge generation to avoid expensive calculations on every span update.
    const { new_nodes, new_edges } = React.useMemo(() => {
        const span_list = Object.values(spans);
        const filtered_spans = filter_mission_id 
            ? span_list.filter(s => s.mission_id === filter_mission_id)
            : span_list;

        const nodes: Node[] = filtered_spans.map(span => {
            const agent = agents.find(a => a.id === span.agent_id);
            return {
                id: span.id,
                type: 'telemetry',
                data: { ...span, agent_name: agent?.name },
                position: { x: 0, y: 0 },
            };
        });

        const edges: Edge[] = [];
        filtered_spans.forEach(span => {
            if (span.parent_id && spans[span.parent_id]) {
                edges.push({
                    id: `e-${span.parent_id}-${span.id}`,
                    source: span.parent_id,
                    target: span.id,
                    animated: span.status === 'running',
                    style: { 
                        stroke: span.status === 'running' ? THEME_COLORS.RUNNING : THEME_COLORS.BORDER_DIM, 
                        strokeWidth: 2,
                        opacity: span.status === 'running' ? 1 : 0.6
                    },
                    markerEnd: {
                        type: MarkerType.ArrowClosed,
                        color: span.status === 'running' ? THEME_COLORS.RUNNING : THEME_COLORS.BORDER_DIM,
                    },
                });
            }
        });

        return { new_nodes: nodes, new_edges: edges };
    }, [spans, filter_mission_id, agents]);

    // PERF: Throttle layout updates. Real-time layout on every span is too expensive.
    const [layouted, set_layouted] = React.useState<{nodes: Node[], edges: Edge[]}>({ nodes: [], edges: [] });
    
    useEffect(() => {
        let is_mounted = true;

        // Only re-layout if structure changed (nodes/edges count) or status changed significantly
        // For simplicity, we'll just debounce the layout call
        const timer = setTimeout(() => {
            if (!is_mounted) return;
            const result = get_layouted_elements(new_nodes, new_edges);
            set_layouted(result);
        }, 300); // 300ms debounce for layout

        return () => {
            is_mounted = false;
            clearTimeout(timer);
        };
    }, [new_nodes, new_edges]);

    useEffect(() => {
        set_nodes(layouted.nodes);
        set_edges(layouted.edges);
    }, [layouted, set_nodes, set_edges]);

    return (
        <div className="w-full h-full bg-zinc-950 rounded-[2rem] border border-zinc-900 overflow-hidden relative group">
            <ReactFlow
                nodes={nodes}
                edges={edges}
                nodeTypes={node_types}
                onNodesChange={on_nodes_change}
                onEdgesChange={on_edges_change}
                fitView
                fitViewOptions={{ padding: 0.2 }}
                minZoom={0.1}
                maxZoom={2}
            >
                <Background color="#18181b" gap={24} size={1} />
                <Controls className="bg-zinc-900/80 backdrop-blur-md border-zinc-800 rounded-xl overflow-hidden fill-white" />
                
                <Panel position="top-left" className="m-6 flex flex-col gap-3">
                    <div className="flex flex-col gap-1">
                        <div className="flex items-center gap-3">
                            <div className="w-2.5 h-2.5 rounded-full bg-cyan-500 animate-pulse shadow-[0_0_12px_#06b6d4]" />
                            <h2 className="text-sm font-black text-white uppercase tracking-[4px]">{i18n.t('telemetry_graph.title')}</h2>
                        </div>
                        <p className="text-[10px] text-zinc-500 font-bold uppercase tracking-wider ml-6">{i18n.t('telemetry_graph.subtitle')}</p>
                    </div>

                    <div className="flex gap-2 items-center">
                        <select 
                            value={filter_mission_id || ''} 
                            onChange={(e) => set_filter_mission_id(e.target.value || undefined)}
                            className="bg-zinc-900/60 backdrop-blur-xl border border-zinc-800 rounded-xl px-3 py-2 text-[10px] font-bold text-zinc-300 uppercase tracking-widest outline-none focus:ring-2 focus:ring-cyan-500/20 transition-all hover:bg-zinc-900"
                            aria-label={i18n.t('telemetry_graph.aria_filter_mission', { defaultValue: 'Filter by Mission ID' })}
                        >
                            <option value="">{i18n.t('telemetry_graph.global_swarm')}</option>
                            {(available_missions || []).map(m => (
                                <option key={m} value={m}>{i18n.t('telemetry_graph.filter_mission', { id: m.substring(0, 8) })}</option>
                            ))}
                        </select>
                        
                        <button 
                            onClick={clear_all}
                            className="bg-zinc-900/60 backdrop-blur-xl border border-zinc-800 rounded-xl px-4 py-2 text-[10px] font-bold text-zinc-500 uppercase tracking-widest hover:text-rose-400 hover:border-rose-900/50 hover:bg-zinc-900 transition-all"
                            aria-label={i18n.t('telemetry_graph.aria_purge_trace', { defaultValue: 'Purge Telemetry Trace' })}
                        >
                            {i18n.t('telemetry_graph.purge_trace')}
                        </button>
                    </div>
                </Panel>

                <Panel position="bottom-right" className="m-6">
                     <div className="px-4 py-2 bg-zinc-950/80 backdrop-blur-sm border border-zinc-900 rounded-xl flex items-center gap-4">
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-cyan-500" />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('telemetry_graph.legend_active')}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('telemetry_graph.legend_success')}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-rose-500" />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('telemetry_graph.legend_error')}</span>
                        </div>
                    </div>
                </Panel>
            </ReactFlow>
        </div>
    );
};


// Metadata: [Telemetry_Graph]
