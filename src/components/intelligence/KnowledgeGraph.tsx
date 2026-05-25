/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[KnowledgeGraph]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Knowledge Graph Component**: Coordinates codebase dependency visualization.
 * Decomposed into specialized sub-modules (GraphView, CognitionSidebar, AnomalyPanel)
 * to separate concerns, prevent render cycles, and ensure type safety.
 */

import React, { useEffect, useMemo, useRef, useState } from 'react';
import type { ForceGraphMethods } from 'react-force-graph-2d';
import { Target, Zap, RefreshCw, ArrowLeft, ArrowRight, Download, Route } from 'lucide-react';
import { intelligence_api_service, type CodeGraphResponse } from '../../services/intelligence_api_service';
import type { ExtendedGraphNode, ForceGraphLink } from './knowledge_graph/types';
import { GraphView } from './knowledge_graph/GraphView';
import { CognitionSidebar } from './knowledge_graph/CognitionSidebar';
import { AnomalyPanel } from './knowledge_graph/AnomalyPanel';
import { sanitize_graph_data } from './knowledge_graph/graph_sanitizer';
import { PathFinderModal } from './knowledge_graph/PathFinderModal';

export const KnowledgeGraph: React.FC = () => {
    const fg_ref = useRef<ForceGraphMethods<ExtendedGraphNode, ForceGraphLink> | undefined>(undefined);
    const [data, set_data] = useState<CodeGraphResponse | null>(null);
    const [loading, set_loading] = useState(true);
    const [selected_node, set_selected_node] = useState<ExtendedGraphNode | null>(null);
    const [affected_nodes, set_affected_nodes] = useState<Set<string>>(new Set());
    const [hover_node, set_hover_node] = useState<ExtendedGraphNode | null>(null);
    const [active_info_tab, set_active_info_tab] = useState<'info' | 'memory'>('info');

    // History and Pathfinder states
    const [nodeHistory, setNodeHistory] = useState<ExtendedGraphNode[]>([]);
    const [historyIndex, setHistoryIndex] = useState<number>(-1);
    const [isPathFinderOpen, setIsPathFinderOpen] = useState(false);
    const [highlightedPathNodeIds, setHighlightedPathNodeIds] = useState<string[] | null>(null);

    const is_memory_node = useMemo(() => {
        if (!selected_node) return false;
        const name = selected_node.name.toLowerCase();
        const path = selected_node.path.toLowerCase();
        return name.includes('memory') || path.includes('memory');
    }, [selected_node]);

    useEffect(() => {
        let active = true;
        const load = async () => {
            try {
                const graph = await intelligence_api_service.get_code_graph();
                const sanitized = sanitize_graph_data(graph);
                if (active) {
                    set_data(sanitized.data);
                }
            } catch (err) {
                console.error('[KnowledgeGraph] Failed to fetch graph:', err);
            } finally {
                if (active) {
                    set_loading(false);
                }
            }
        };
        load();
        return () => {
            active = false;
        };
    }, []);

    // Transform data for force-graph
    const graph_data = useMemo(() => {
        if (!data) return { nodes: [], links: [] };

        const pathNodeSet = new Set(highlightedPathNodeIds || []);
        
        // Build map for bidirectional path link trace matching
        const pathLinkSet = new Set<string>();
        if (highlightedPathNodeIds && highlightedPathNodeIds.length > 1) {
            for (let i = 0; i < highlightedPathNodeIds.length - 1; i++) {
                const u = highlightedPathNodeIds[i];
                const v = highlightedPathNodeIds[i + 1];
                pathLinkSet.add(`${u}->${v}`);
                pathLinkSet.add(`${v}->${u}`);
            }
        }

        const nodes: ExtendedGraphNode[] = data.nodes.map(node => {
            const id = `${node.path}:${node.name}`;
            return {
                ...node,
                id,
                is_affected: affected_nodes.has(id),
                is_path_highlighted: pathNodeSet.has(id)
            };
        });

        const links: ForceGraphLink[] = data.links.map(link => {
            const u = typeof link.source === 'string' 
                ? link.source 
                : (link.source as ExtendedGraphNode).id || `${(link.source as ExtendedGraphNode).path}:${(link.source as ExtendedGraphNode).name}`;
            const v = typeof link.target === 'string' 
                ? link.target 
                : (link.target as ExtendedGraphNode).id || `${(link.target as ExtendedGraphNode).path}:${(link.target as ExtendedGraphNode).name}`;
            const is_path_highlighted = pathLinkSet.has(`${u}->${v}`) || pathLinkSet.has(`${v}->${u}`);
            return {
                source: link.source,
                target: link.target,
                is_path_highlighted
            };
        });

        return { nodes, links };
    }, [data, affected_nodes, highlightedPathNodeIds]);

    const selectNode = async (node: ExtendedGraphNode, pushHistory = true) => {
        set_selected_node(node);
        try {
            const affected = await intelligence_api_service.get_blast_radius(node.name, node.path);
            const affected_ids = new Set(affected.map(n => `${n.path}:${n.name}`));
            set_affected_nodes(affected_ids);

            if (pushHistory) {
                setNodeHistory(prev => {
                    const truncated = prev.slice(0, historyIndex + 1);
                    if (truncated.length > 0 && truncated[truncated.length - 1].id === node.id) {
                        return truncated;
                    }
                    const updated = [...truncated, node];
                    setHistoryIndex(updated.length - 1);
                    return updated;
                });
            }

            // Center and zoom
            if (fg_ref.current) {
                if (typeof node.x === 'number' && typeof node.y === 'number') {
                    fg_ref.current.centerAt(node.x, node.y, 1000);
                    fg_ref.current.zoom(2.5, 1000);
                }
            }
        } catch (err) {
            console.error('[KnowledgeGraph] Blast radius failed:', err);
        }
    };

    const handle_node_click = (node: ExtendedGraphNode) => {
        selectNode(node, true);
    };

    const handleBack = () => {
        if (historyIndex > 0) {
            const prevIndex = historyIndex - 1;
            setHistoryIndex(prevIndex);
            selectNode(nodeHistory[prevIndex], false);
        }
    };

    const handleForward = () => {
        if (historyIndex < nodeHistory.length - 1) {
            const nextIndex = historyIndex + 1;
            setHistoryIndex(nextIndex);
            selectNode(nodeHistory[nextIndex], false);
        }
    };

    const handleExportPNG = () => {
        const canvas = (fg_ref.current as unknown as { canvasElement?: () => HTMLCanvasElement })?.canvasElement?.();
        if (!canvas) {
            console.error('[KnowledgeGraph] Canvas element not found');
            return;
        }
        
        // Export high resolution png
        const link = document.createElement('a');
        link.download = `tadpole_knowledge_graph_${new Date().toISOString().slice(0, 10)}.png`;
        link.href = canvas.toDataURL('image/png');
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    };

    const handle_close_sidebar = () => {
        set_selected_node(null);
        set_affected_nodes(new Set());
    };

    return (
        <div className="w-full h-full relative bg-zinc-950 rounded-2xl border border-zinc-900 overflow-hidden flex flex-col items-stretch min-h-[500px]">
            {loading ? (
                <div className="absolute inset-0 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm z-50">
                    <div className="flex flex-col items-center gap-4">
                        <RefreshCw className="w-8 h-8 text-cyan-500 animate-spin" />
                        <p className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.3em]">Synthesizing Symbol Graph...</p>
                    </div>
                </div>
            ) : null}

            {!loading && (
                <div className="flex-1 min-h-0 w-full relative">
                    <GraphView
                        graph_data={graph_data}
                        selected_node={selected_node}
                        hover_node={hover_node}
                        set_hover_node={set_hover_node}
                        affected_nodes={affected_nodes}
                        on_node_click={handle_node_click}
                        fg_ref={fg_ref}
                    />
                </div>
            )}

            {/* Header HUD */}
            <div className="absolute top-6 left-6 pointer-events-none select-none z-30">
                <div className="flex flex-col gap-2">
                    <div className="flex items-center gap-3">
                        <div className="w-2.5 h-2.5 rounded-full bg-cyan-500 shadow-[0_0_15px_#22d3ee]" />
                        <h2 className="text-xs font-black text-white uppercase tracking-[0.4em]">Knowledge Graph</h2>
                    </div>
                    <div className="flex items-center gap-4 ml-6">
                        <div className="flex items-center gap-2">
                            <Target size={10} className="text-zinc-500" />
                            <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">{data?.nodes.length || 0} Symbols</span>
                        </div>
                        <div className="w-px h-2 bg-zinc-800" />
                        <div className="flex items-center gap-2">
                            <Zap size={10} className="text-zinc-500" />
                            <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">{data?.links.length || 0} Edges</span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Navigation & Utilities HUD panel */}
            {!loading && (
                <div className="absolute top-20 left-6 flex items-center gap-2 z-35 pointer-events-auto select-none">
                    <button
                        disabled={historyIndex <= 0}
                        onClick={handleBack}
                        className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 disabled:hover:border-zinc-900 rounded-lg text-zinc-400 hover:text-white disabled:text-zinc-650 disabled:border-zinc-900 transition-all cursor-pointer disabled:cursor-not-allowed"
                        title="Go Back (History)"
                    >
                        <ArrowLeft size={12} />
                    </button>
                    <button
                        disabled={historyIndex >= nodeHistory.length - 1}
                        onClick={handleForward}
                        className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 disabled:hover:border-zinc-900 rounded-lg text-zinc-400 hover:text-white disabled:text-zinc-650 disabled:border-zinc-900 transition-all cursor-pointer disabled:cursor-not-allowed"
                        title="Go Forward (History)"
                    >
                        <ArrowRight size={12} />
                    </button>
                    <div className="w-px h-4 bg-zinc-800 mx-1" />
                    <button
                        onClick={() => setIsPathFinderOpen(true)}
                        className="flex items-center gap-1.5 px-3 py-1.5 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-cyan-500/50 hover:bg-zinc-900 rounded-lg text-[10px] font-bold text-zinc-300 hover:text-cyan-400 transition-all cursor-pointer font-mono"
                        title="Open Dependency Pathfinder"
                    >
                        <Route size={12} />
                        <span>Find Path</span>
                    </button>
                    <button
                        onClick={handleExportPNG}
                        className="flex items-center gap-1.5 px-3 py-1.5 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-emerald-500/50 hover:bg-zinc-900 rounded-lg text-[10px] font-bold text-zinc-300 hover:text-emerald-400 transition-all cursor-pointer font-mono"
                        title="Export Canvas to PNG"
                    >
                        <Download size={12} />
                        <span>Export PNG</span>
                    </button>
                </div>
            )}

            {/* Floating Info Panel */}
            {selected_node && (
                <CognitionSidebar
                    selected_node={selected_node}
                    is_memory_node={is_memory_node}
                    active_info_tab={active_info_tab}
                    set_active_info_tab={set_active_info_tab}
                    affected_nodes={affected_nodes}
                    total_nodes_count={data?.nodes.length || 0}
                    on_close={handle_close_sidebar}
                />
            )}

            {/* Legend */}
            <div className="absolute top-6 right-6 flex flex-col gap-2 bg-zinc-950/40 backdrop-blur-md p-3 rounded-xl border border-zinc-900/50 select-none pointer-events-none z-30">
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-cyan-450" style={{ backgroundColor: '#22d3ee' }} />
                    <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Function / Method</span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                    <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Struct / Class</span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-amber-400" />
                    <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Trait / Interface</span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-[#a855f7]" />
                    <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Enum</span>
                </div>
            </div>

            {/* Code Anomalies Panel */}
            {data?.anomalies && data.anomalies.length > 0 && (
                <AnomalyPanel
                    anomalies={data.anomalies}
                    nodes={graph_data.nodes}
                    selected_node={selected_node}
                    on_anomaly_click={handle_node_click}
                />
            )}

            {/* Pathfinder Modal Dialog Overlay */}
            <PathFinderModal
                isOpen={isPathFinderOpen}
                onClose={() => setIsPathFinderOpen(false)}
                nodes={graph_data.nodes}
                links={graph_data.links}
                onHighlightPath={(pathNodeIds) => setHighlightedPathNodeIds(pathNodeIds)}
                onSelectNode={(node) => selectNode(node, true)}
            />
        </div>
    );
};

// Metadata: [KnowledgeGraph]
