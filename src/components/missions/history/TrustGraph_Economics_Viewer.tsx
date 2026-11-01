/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / TrustGraph_Economics_Viewer
 * - **Primary Entrypoints**: `TrustGraph_Economics_Viewer`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` 2PC transaction ledger visualization, entity relations, JetBrains Mono currency and hash fields.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useRef, useEffect, useMemo, useCallback } from 'react';
import { 
    Network, 
    Coins, 
    ArrowRight, 
    Bot, 
    FileText, 
    Lightbulb, 
    Flag,
    CheckCircle2,
    LayoutGrid,
    Sparkles,
    RotateCcw
} from 'lucide-react';
import ForceGraph2D from 'react-force-graph-2d';
const ForceGraph2DComponent = ForceGraph2D as unknown as React.ComponentType<Record<string, unknown>>;

import type { TrustGraphPayload, EconomicsPayload } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';

interface TrustGraphEconomicsViewerProps {
    trust_graph: TrustGraphPayload;
    economics: EconomicsPayload;
}

export const TrustGraph_Economics_Viewer: React.FC<TrustGraphEconomicsViewerProps> = ({
    trust_graph,
    economics,
}) => {
    const [sub_tab, set_sub_tab] = useState<'trustgraph' | 'ledger'>('trustgraph');
    const [graph_view_mode, set_graph_view_mode] = useState<'forensic' | 'interactive'>('forensic');
    const is_large_swarm = trust_graph.entities.length >= 25;

    const graph_container_ref = useRef<HTMLDivElement>(null);
    const [graph_dims, set_graph_dims] = useState<{ width: number; height: number }>({ width: 600, height: 400 });
    const fg_ref = useRef<{ zoomToFit?: (durationMs?: number, padding?: number) => void } | null>(null);

    useEffect(() => {
        if (!graph_container_ref.current) return;
        const ro = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                if (width > 0 && height > 0) {
                    set_graph_dims({ width: Math.floor(width), height: Math.floor(height) });
                }
            }
        });
        ro.observe(graph_container_ref.current);
        return () => ro.disconnect();
    }, [graph_view_mode, sub_tab]);

    const force_graph_data = useMemo(() => {
        const nodes = trust_graph.entities.map((e) => ({
            id: e.id,
            name: e.name,
            node_type: e.node_type,
            color: e.node_type.toUpperCase() === 'AGENT' ? '#06b6d4'
                 : e.node_type.toUpperCase() === 'FILE' ? '#a855f7'
                 : e.node_type.toUpperCase() === 'CONCEPT' ? '#f59e0b'
                 : '#10b981',
        }));
        const valid_ids = new Set(nodes.map(n => n.id));
        const links = trust_graph.relations
            .filter(r => valid_ids.has(r.source_entity_id) && valid_ids.has(r.target_entity_id))
            .map(r => ({
                source: r.source_entity_id,
                target: r.target_entity_id,
                relation: r.relation_type,
                weight: r.weight,
            }));
        return { nodes, links };
    }, [trust_graph]);

    const paint_node = useCallback((node: { id: string; name?: string; color: string; x?: number; y?: number }, ctx: CanvasRenderingContext2D, globalScale: number) => {
        const label = node.name || node.id;
        const fontSize = Math.max(9 / globalScale, 2.5);
        const radius = 6;

        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius, 0, 2 * Math.PI, false);
        ctx.fillStyle = node.color;
        ctx.fill();
        ctx.strokeStyle = '#18181b';
        ctx.lineWidth = 1.5;
        ctx.stroke();

        ctx.font = `${fontSize}px monospace`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';
        ctx.fillStyle = '#e4e4e7';
        ctx.fillText(label, node.x ?? 0, (node.y ?? 0) + radius + 2);
    }, []);

    const render_node_icon = (type: string) => {
        const t = type.toUpperCase();
        if (t === 'AGENT') return <Bot size={13} className="text-cyan-400" />;
        if (t === 'FILE') return <FileText size={13} className="text-purple-400" />;
        if (t === 'CONCEPT') return <Lightbulb size={13} className="text-amber-400" />;
        return <Flag size={13} className="text-emerald-400" />;
    };

    return (
        <div className="flex flex-col h-full bg-zinc-950 border border-zinc-800/80 rounded-xl overflow-hidden">
            {/* Header Switcher */}
            <div className="p-3 border-b border-zinc-800/80 bg-zinc-900/50 flex items-center justify-between gap-3 flex-wrap">
                <div className="flex items-center gap-1 bg-zinc-950 p-0.5 rounded-lg border border-zinc-800 text-xs">
                    <Tooltip content="Topological map of executing agents, files, and concept entities" position="bottom">
                        <button
                            onClick={() => set_sub_tab('trustgraph')}
                            className={`flex items-center gap-1.5 px-3 py-1 rounded-md font-semibold transition-colors ${
                                sub_tab === 'trustgraph'
                                    ? 'bg-zinc-800 text-cyan-300 shadow-sm'
                                    : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                        >
                            <Network size={13} />
                            <span>TrustGraph Topology ({trust_graph.entities.length} Nodes)</span>
                        </button>
                    </Tooltip>
                    <Tooltip content="Two-Phase Commit (2PC) atomic micro-credit transactions and token expenditure" position="bottom">
                        <button
                            onClick={() => set_sub_tab('ledger')}
                            className={`flex items-center gap-1.5 px-3 py-1 rounded-md font-semibold transition-colors ${
                                sub_tab === 'ledger'
                                    ? 'bg-zinc-800 text-amber-300 shadow-sm'
                                    : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                        >
                            <Coins size={13} />
                            <span>2PC Economic Ledger ({economics.transactions.length} Tx)</span>
                        </button>
                    </Tooltip>
                </div>

                {/* Right controls: layout selector for TrustGraph or burn rollup for Ledger */}
                <div className="flex items-center gap-3 text-[11px] font-mono text-zinc-400">
                    {sub_tab === 'trustgraph' ? (
                        <div className="flex items-center gap-2">
                            <span className="hidden sm:inline">{trust_graph.relations.length} relations mapped</span>
                            <div className="flex items-center gap-0.5 bg-zinc-950 p-0.5 rounded-md border border-zinc-800">
                                <Tooltip content="Deterministic forensic card grid (standard audit mode)" position="bottom">
                                    <button
                                        onClick={() => set_graph_view_mode('forensic')}
                                        className={`flex items-center gap-1 px-2 py-0.5 rounded font-sans font-semibold text-[11px] transition-colors ${
                                            graph_view_mode === 'forensic'
                                                ? 'bg-zinc-800 text-cyan-300'
                                                : 'text-zinc-500 hover:text-zinc-300'
                                        }`}
                                    >
                                        <LayoutGrid size={11} />
                                        <span>Forensic Grid</span>
                                    </button>
                                </Tooltip>
                                <Tooltip content="Opt-in force-directed physics map (recommended for large swarms >= 25 nodes)" position="bottom">
                                    <button
                                        onClick={() => set_graph_view_mode('interactive')}
                                        className={`flex items-center gap-1 px-2 py-0.5 rounded font-sans font-semibold text-[11px] transition-colors ${
                                            graph_view_mode === 'interactive'
                                                ? 'bg-zinc-800 text-cyan-300'
                                                : 'text-zinc-500 hover:text-zinc-300'
                                        }`}
                                    >
                                        <Network size={11} />
                                        <span>Interactive Map</span>
                                        {is_large_swarm && (
                                            <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-ping" />
                                        )}
                                    </button>
                                </Tooltip>
                            </div>
                        </div>
                    ) : (
                        <span>Total Burn: ${economics.total_cost_usd.toFixed(4)}</span>
                    )}
                </div>
            </div>

            {/* Sub-tab 1: TrustGraph Topology */}
            {sub_tab === 'trustgraph' && (
                <div 
                    ref={graph_container_ref}
                    className="flex-1 overflow-y-auto custom-scrollbar p-4 flex flex-col gap-4 min-h-[420px]"
                >
                    {trust_graph.entities.length === 0 ? (
                        <div className="flex flex-col items-center justify-center p-12 text-center text-zinc-500">
                            <Network size={36} className="text-zinc-600 mb-2" />
                            <span className="text-xs font-semibold text-zinc-400">No TrustGraph Nodes Indexed</span>
                            <span className="text-[11px] text-zinc-600 mt-1">Entity relationships are generated during multi-agent collaboration</span>
                        </div>
                    ) : graph_view_mode === 'interactive' ? (
                        /* Interactive Force-Directed Topology Map */
                        <div className="flex-1 flex flex-col relative rounded-xl border border-zinc-800 bg-zinc-950 overflow-hidden min-h-[400px]">
                            {/* Floating Controls & Legend */}
                            <div className="absolute top-3 left-3 z-10 flex items-center gap-2 bg-slate-950/60 backdrop-blur-md p-1.5 px-3 rounded-lg border border-zinc-800 text-[11px] text-zinc-300">
                                <span className="font-semibold text-cyan-400">Legend:</span>
                                <span className="inline-flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-cyan-400" /> Agent</span>
                                <span className="inline-flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-purple-400" /> File</span>
                                <span className="inline-flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-amber-400" /> Concept</span>
                                <span className="inline-flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-emerald-400" /> Service</span>
                            </div>

                            <div className="absolute top-3 right-3 z-10 flex items-center gap-1.5">
                                <Tooltip content="Reset simulation camera and center nodes" position="left">
                                    <button
                                        onClick={() => fg_ref.current?.zoomToFit?.(400, 20)}
                                        className="p-1.5 rounded-lg bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-700 text-zinc-300 transition-colors"
                                        aria-label="Reset zoom"
                                    >
                                        <RotateCcw size={13} />
                                    </button>
                                </Tooltip>
                                <button
                                    onClick={() => set_graph_view_mode('forensic')}
                                    className="px-2.5 py-1 rounded-lg bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-700 text-zinc-300 font-sans text-xs font-semibold transition-colors flex items-center gap-1"
                                >
                                    <LayoutGrid size={12} />
                                    <span>Back to Forensic Grid</span>
                                </button>
                            </div>

                            <div className="flex-1 w-full h-full min-h-[400px]">
                                <ForceGraph2DComponent
                                    ref={fg_ref}
                                    width={graph_dims.width - 34}
                                    height={Math.max(400, graph_dims.height - 34)}
                                    graphData={force_graph_data}
                                    nodeCanvasObject={paint_node}
                                    linkColor={() => '#3f3f46'}
                                    linkDirectionalArrowLength={4.5}
                                    linkDirectionalArrowRelPos={0.9}
                                    linkDirectionalArrowColor={() => '#a1a1aa'}
                                    linkLabel={(link: { relation?: string; weight?: number }) => `${link.relation} (w=${link.weight})`}
                                    backgroundColor="#09090b"
                                    cooldownTicks={100}
                                    d3AlphaDecay={0.04}
                                    d3VelocityDecay={0.3}
                                />
                            </div>
                        </div>
                    ) : (
                        /* Deterministic Forensic Card Grid (Default Audit Mode) */
                        <>
                            {is_large_swarm && (
                                <div className="p-3 rounded-lg bg-cyan-950/20 border border-cyan-800/40 flex items-center justify-between text-xs text-cyan-200">
                                    <div className="flex items-center gap-2">
                                        <Sparkles size={14} className="text-cyan-400 flex-shrink-0" />
                                        <span>
                                            Large Swarm Topology ({trust_graph.entities.length} nodes) detected. Interactive Topology Map is recommended for spatial clustering.
                                        </span>
                                    </div>
                                    <button
                                        onClick={() => set_graph_view_mode('interactive')}
                                        className="px-2.5 py-1 rounded bg-cyan-500/20 hover:bg-cyan-500/30 text-cyan-300 font-semibold border border-cyan-500/40 text-[11px] whitespace-nowrap ml-2"
                                    >
                                        Launch Interactive Map
                                    </button>
                                </div>
                            )}

                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                {/* Entity Nodes */}
                                <div className="flex flex-col gap-2">
                                    <span className="text-[10px] font-bold text-zinc-400 uppercase tracking-widest px-1">
                                        Entity Nodes ({trust_graph.entities.length})
                                    </span>
                                    <div className="flex flex-col gap-1.5">
                                        {trust_graph.entities.map((node) => (
                                            <div
                                                key={node.id}
                                                className="p-3 rounded-lg bg-zinc-900/40 border border-zinc-800/80 flex items-center justify-between"
                                            >
                                                <div className="flex items-center gap-2.5">
                                                    <div className="p-1.5 rounded-md bg-zinc-950 border border-zinc-800">
                                                        {render_node_icon(node.node_type)}
                                                    </div>
                                                    <div className="flex flex-col">
                                                        <span className="text-xs font-semibold text-zinc-200">
                                                            {node.name}
                                                        </span>
                                                        <span className="font-mono text-[10px] text-zinc-500">
                                                            {node.node_type} • ID: {node.id.slice(0, 8)}
                                                        </span>
                                                    </div>
                                                </div>
                                                {node.description && (
                                                    <span className="text-[11px] text-zinc-400 max-w-xs truncate text-right">
                                                        {node.description}
                                                    </span>
                                                )}
                                            </div>
                                        ))}
                                    </div>
                                </div>

                                {/* Relationship Edges */}
                                <div className="flex flex-col gap-2">
                                    <span className="text-[10px] font-bold text-zinc-400 uppercase tracking-widest px-1">
                                        Relationship Edges ({trust_graph.relations.length})
                                    </span>
                                    <div className="flex flex-col gap-1.5">
                                        {trust_graph.relations.map((rel) => (
                                            <div
                                                key={rel.id}
                                                className="p-3 rounded-lg bg-zinc-900/40 border border-zinc-800/80 flex items-center justify-between text-xs"
                                            >
                                                <div className="flex items-center gap-2 font-mono text-[11px]">
                                                    <span className="text-cyan-400">{rel.source_entity_id.slice(0, 10)}</span>
                                                    <ArrowRight size={12} className="text-zinc-600" />
                                                    <span className="text-purple-400">{rel.target_entity_id.slice(0, 10)}</span>
                                                </div>
                                                <div className="flex items-center gap-2">
                                                    <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-zinc-950 border border-zinc-800 text-zinc-300">
                                                        {rel.relation_type}
                                                    </span>
                                                    <span className="font-mono text-[10px] text-zinc-500">
                                                        w={rel.weight.toFixed(1)}
                                                    </span>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        </>
                    )}
                </div>
            )}

            {/* Sub-tab 2: 2PC Economic Ledger */}
            {sub_tab === 'ledger' && (
                <div className="flex-1 overflow-y-auto custom-scrollbar p-4 flex flex-col gap-3">
                    {economics.transactions.length === 0 ? (
                        <div className="flex flex-col items-center justify-center p-12 text-center text-zinc-500">
                            <Coins size={36} className="text-zinc-600 mb-2" />
                            <span className="text-xs font-semibold text-zinc-400">No Micro-Transactions Recorded</span>
                            <span className="text-[11px] text-zinc-600 mt-1">Autonomous 2PC atomic economic settlement events appear here</span>
                        </div>
                    ) : (
                        economics.transactions.map((tx) => (
                            <div
                                key={tx.id}
                                className="p-3.5 rounded-xl bg-zinc-900/40 border border-zinc-800/80 flex items-center justify-between text-xs"
                            >
                                <div className="flex items-center gap-3">
                                    <div className="p-2 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-400 font-bold font-mono text-[11px]">
                                        {tx.transaction_type}
                                    </div>
                                    <div className="flex flex-col">
                                        <div className="flex items-center gap-2 font-mono text-[11px] text-zinc-300">
                                            <span>{tx.buyer_address ?? 'Agent'}</span>
                                            <ArrowRight size={12} className="text-zinc-500" />
                                            <span>{tx.seller_address ?? 'Sovereign Treasury'}</span>
                                        </div>
                                        <span className="font-mono text-[10px] text-zinc-500 mt-0.5">
                                            TxID: {tx.id} • {new Date(tx.created_at * 1000).toLocaleString()}
                                        </span>
                                    </div>
                                </div>

                                <div className="flex items-center gap-3">
                                    <span className="font-mono text-sm font-bold text-amber-300">
                                        {tx.amount} micro-credits
                                    </span>
                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                        <CheckCircle2 size={10} /> {tx.status}
                                    </span>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            )}
        </div>
    );
};
