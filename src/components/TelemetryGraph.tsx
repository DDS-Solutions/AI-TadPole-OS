import React, { useEffect, useMemo } from 'react';
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
import { useTraceStore, type TraceSpan } from '../stores/traceStore';
import { useAgentStore } from '../stores/agentStore';
import { motion } from 'framer-motion';
import 'reactflow/dist/style.css';

const dagreGraph = new dagre.graphlib.Graph();
dagreGraph.setDefaultEdgeLabel(() => ({}));

const nodeWidth = 220;
const nodeHeight = 80;

// Custom Node Component for a more "Premium" look
const TelemetryNode = ({ data }: { data: TraceSpan & { agentName?: string } }) => {
    const isError = data.status === 'error';
    const isRunning = data.status === 'running';
    const isTool = data.name.startsWith('execute_tool');

    return (
        <div className={`
            relative px-4 py-3 rounded-2xl border min-w-[200px]
            ${isError ? 'bg-rose-950/20 border-rose-500/50' : 'bg-zinc-900/90 border-zinc-800'}
            ${isRunning ? 'shadow-[0_0_20px_rgba(6,182,212,0.15)] ring-1 ring-cyan-500/30' : ''}
            backdrop-blur-xl transition-all duration-500
        `}>
            <Handle type="target" position={Position.Top} className="opacity-0" />
            
            <div className="flex flex-col gap-1">
                <div className="flex items-center justify-between gap-4">
                    <span className="text-[10px] font-black text-zinc-500 uppercase tracking-[2px] truncate">
                        {data.agentName || 'NEURAL NODE'}
                    </span>
                    {isRunning && (
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
                        ${isTool ? 'bg-amber-500/10 text-amber-400' : 'bg-zinc-800 text-zinc-400'}
                    `}>
                        {isTool ? 'Tool' : 'Phase'}
                    </div>
                    {data.endTime && (
                         <span className="text-[8px] text-zinc-600 font-mono">
                            {((data.endTime - data.startTime) / 1000).toFixed(2)}s
                         </span>
                    )}
                </div>
            </div>

            {isRunning && (
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

const nodeTypes = {
    telemetry: TelemetryNode,
};

const getLayoutedElements = (nodes: Node[], edges: Edge[], direction = 'TB') => {
    dagreGraph.setGraph({ rankdir: direction, nodesep: 50, ranksep: 70 });

    nodes.forEach((node) => {
        dagreGraph.setNode(node.id, { width: nodeWidth, height: nodeHeight });
    });

    edges.forEach((edge) => {
        dagreGraph.setEdge(edge.source, edge.target);
    });

    dagre.layout(dagreGraph);

    nodes.forEach((node) => {
        const nodeWithPosition = dagreGraph.node(node.id);
        node.targetPosition = Position.Top;
        node.sourcePosition = Position.Bottom;

        node.position = {
            x: nodeWithPosition.x - nodeWidth / 2,
            y: nodeWithPosition.y - nodeHeight / 2,
        };

        return node;
    });

    return { nodes, edges };
};

export const TelemetryGraph: React.FC<{ initialMissionId?: string }> = ({ initialMissionId }) => {
    const { spans, clearAll } = useTraceStore();
    const { agents } = useAgentStore();
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState([]);
    const [filterMissionId, setFilterMissionId] = React.useState<string | undefined>(initialMissionId);

    const availableMissions = useMemo(() => {
        const missions = new Set<string>();
        Object.values(spans).forEach(s => {
            if (s.missionId && s.missionId !== 'unknown') {
                missions.add(s.missionId);
            }
        });
        return Array.from(missions);
    }, [spans]);

    // PERF: Throttle layout updates. Real-time layout on every span is too expensive.
    const [layouted, setLayouted] = React.useState<{nodes: Node[], edges: Edge[]}>({ nodes: [], edges: [] });
    
    useEffect(() => {
        const spanList = Object.values(spans);
        const filteredSpans = filterMissionId 
            ? spanList.filter(s => s.missionId === filterMissionId)
            : spanList;

        const newNodes: Node[] = filteredSpans.map(span => {
            const agent = agents.find(a => String(a.id) === String(span.agentId));
            return {
                id: span.id,
                type: 'telemetry',
                data: { ...span, agentName: agent?.name },
                position: { x: 0, y: 0 },
            };
        });

        const newEdges: Edge[] = [];
        filteredSpans.forEach(span => {
            if (span.parentId && spans[span.parentId]) {
                newEdges.push({
                    id: `e-${span.parentId}-${span.id}`,
                    source: span.parentId,
                    target: span.id,
                    animated: span.status === 'running',
                    style: { 
                        stroke: span.status === 'running' ? '#06b6d4' : '#27272a', 
                        strokeWidth: 2,
                        opacity: span.status === 'running' ? 1 : 0.6
                    },
                    markerEnd: {
                        type: MarkerType.ArrowClosed,
                        color: span.status === 'running' ? '#06b6d4' : '#27272a',
                    },
                });
            }
        });

        // Only re-layout if structure changed (nodes/edges count) or status changed significantly
        // For simplicity, we'll just debounce the layout call
        const timer = setTimeout(() => {
            const result = getLayoutedElements(newNodes, newEdges);
            setLayouted(result);
        }, 300); // 300ms debounce for layout

        return () => clearTimeout(timer);
    }, [spans, filterMissionId, agents]);

    useEffect(() => {
        setNodes(layouted.nodes);
        setEdges(layouted.edges);
    }, [layouted, setNodes, setEdges]);

    return (
        <div className="w-full h-full bg-zinc-950 rounded-[2rem] border border-zinc-900 overflow-hidden relative group">
            <ReactFlow
                nodes={nodes}
                edges={edges}
                nodeTypes={nodeTypes}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
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
                            <h2 className="text-sm font-black text-white uppercase tracking-[4px]">Swarm Engine</h2>
                        </div>
                        <p className="text-[10px] text-zinc-500 font-bold uppercase tracking-wider ml-6">Overlord View • Live Telemetry Graph</p>
                    </div>

                    <div className="flex gap-2 items-center">
                        <select 
                            value={filterMissionId || ''} 
                            onChange={(e) => setFilterMissionId(e.target.value || undefined)}
                            className="bg-zinc-900/60 backdrop-blur-xl border border-zinc-800 rounded-xl px-3 py-2 text-[10px] font-bold text-zinc-300 uppercase tracking-widest outline-none focus:ring-2 focus:ring-cyan-500/20 transition-all hover:bg-zinc-900"
                        >
                            <option value="">Global Swarm</option>
                            {availableMissions.map(m => (
                                <option key={m} value={m}>Mission: {m.substring(0, 8)}</option>
                            ))}
                        </select>
                        
                        <button 
                            onClick={clearAll}
                            className="bg-zinc-900/60 backdrop-blur-xl border border-zinc-800 rounded-xl px-4 py-2 text-[10px] font-bold text-zinc-500 uppercase tracking-widest hover:text-rose-400 hover:border-rose-900/50 hover:bg-zinc-900 transition-all"
                        >
                            Purge Trace
                        </button>
                    </div>
                </Panel>

                <Panel position="bottom-right" className="m-6">
                    <div className="px-4 py-2 bg-zinc-950/80 backdrop-blur-sm border border-zinc-900 rounded-xl flex items-center gap-4">
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-cyan-500" />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Active</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Success</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-rose-500" />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Error</span>
                        </div>
                    </div>
                </Panel>
            </ReactFlow>
        </div>
    );
};
