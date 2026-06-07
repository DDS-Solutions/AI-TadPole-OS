/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * Handles pathfinder BFS computation and interactive step-by-step connection lists.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Shortest path not found due to broken ID mapping or unconnected subgraphs.
 * - **Telemetry Link**: Search `[PathFinderModal]` in observability traces.
 */

import React, { useState, useMemo } from 'react';
import { X, Search, GitCommit, Route } from 'lucide-react';
import type { ExtendedGraphNode } from './types';
import { 
    useUIStateContext, 
    useGraphDataContext, 
    useSelectionContext 
} from './graph_context_hooks';

export const PathFinderModal: React.FC = () => {
    const {
        isPathFinderOpen,
        setIsPathFinderOpen,
        setHighlightedPathNodeIds
    } = useUIStateContext();
    const { graphData } = useGraphDataContext();
    const { selectNode } = useSelectionContext();

    const isOpen = isPathFinderOpen;
    const onClose = () => setIsPathFinderOpen(false);
    const nodes = graphData.nodes;
    const links = graphData.links;
    const nodeMap = graphData.nodeMap;
    const onHighlightPath = setHighlightedPathNodeIds;
    const onSelectNode = (node: ExtendedGraphNode) => selectNode(node, true);

    const [startId, setStartId] = useState<string>('');
    const [endId, setEndId] = useState<string>('');
    const [searchStart, setSearchStart] = useState<string>('');
    const [searchEnd, setSearchEnd] = useState<string>('');
    const [calculatedPath, setCalculatedPath] = useState<ExtendedGraphNode[] | null>(null);
    const [searched, setSearched] = useState<boolean>(false);

    // Filter nodes alphabetically
    const sortedNodes = useMemo(() => {
        return [...nodes].sort((a, b) => a.name.localeCompare(b.name));
    }, [nodes]);

    // Search filter lists
    const filteredStartNodes = useMemo(() => {
        const query = searchStart.toLowerCase().trim();
        if (!query) return sortedNodes.slice(0, 100); // Limit lists for performance
        return sortedNodes.filter(n => 
            n.name.toLowerCase().includes(query) || 
            n.path.toLowerCase().includes(query)
        ).slice(0, 100);
    }, [sortedNodes, searchStart]);

    const filteredEndNodes = useMemo(() => {
        const query = searchEnd.toLowerCase().trim();
        if (!query) return sortedNodes.slice(0, 100);
        return sortedNodes.filter(n => 
            n.name.toLowerCase().includes(query) || 
            n.path.toLowerCase().includes(query)
        ).slice(0, 100);
    }, [sortedNodes, searchEnd]);

    // Precompute adjacency list to prevent O(E) reconstruction on every calculation/UI interaction
    const adjList = useMemo(() => {
        const adj: Record<string, string[]> = {};
        for (const node of nodes) {
            adj[node.id] = [];
        }
        for (const link of links) {
            const u = typeof link.source === 'string' 
                ? link.source 
                : (link.source as ExtendedGraphNode).id;
            const v = typeof link.target === 'string' 
                ? link.target 
                : (link.target as ExtendedGraphNode).id;
            
            if (u && v && adj[u] && adj[v]) {
                adj[u].push(v);
                adj[v].push(u); // Undirected/Bidirectional
            }
        }
        return adj;
    }, [nodes, links]);

    // BFS Pathfinding Logic
    const handleCalculatePath = () => {
        setSearched(true);
        if (!startId || !endId) {
            setCalculatedPath(null);
            onHighlightPath(null);
            return;
        }

        if (startId === endId) {
            const node = nodeMap.get(startId);
            const pathList = node ? [node] : [];
            setCalculatedPath(pathList);
            onHighlightPath(node ? [node.id] : null);
            return;
        }

        // Standard BFS
        const queue: string[] = [startId];
        const visited = new Set<string>([startId]);
        const parent: Record<string, string> = {};

        let found = false;
        while (queue.length > 0) {
            const curr = queue.shift()!;
            if (curr === endId) {
                found = true;
                break;
            }

            for (const neighbor of adjList[curr] || []) {
                if (!visited.has(neighbor)) {
                    visited.add(neighbor);
                    parent[neighbor] = curr;
                    queue.push(neighbor);
                }
            }
        }

        if (found) {
            const pathIds: string[] = [];
            let temp = endId;
            while (temp !== startId) {
                pathIds.push(temp);
                temp = parent[temp];
            }
            pathIds.push(startId);
            pathIds.reverse();

            // Resolve to full node objects
            const pathNodes = pathIds
                .map(id => nodeMap.get(id))
                .filter((n): n is ExtendedGraphNode => !!n);

            setCalculatedPath(pathNodes);
            onHighlightPath(pathIds);
        } else {
            setCalculatedPath([]);
            onHighlightPath(null);
        }
    };

    const handleClear = () => {
        setStartId('');
        setEndId('');
        setSearchStart('');
        setSearchEnd('');
        setCalculatedPath(null);
        setSearched(false);
        onHighlightPath(null);
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[100] p-4 animate-in fade-in duration-200">
            <div className="w-full max-w-2xl bg-zinc-950 border border-zinc-800 rounded-2xl flex flex-col max-h-[90vh] shadow-2xl overflow-hidden">
                
                {/* Header */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-900 bg-zinc-900/50">
                    <div className="flex items-center gap-2.5">
                        <Route size={16} className="text-cyan-400" />
                        <h3 className="text-sm font-black text-white uppercase tracking-widest font-mono">
                            Dependency Pathfinder
                        </h3>
                    </div>
                    <button 
                        onClick={onClose}
                        className="text-zinc-500 hover:text-white transition-colors cursor-pointer"
                    >
                        <X size={16} />
                    </button>
                </div>

                {/* Content */}
                <div className="flex-1 overflow-y-auto p-6 flex flex-col gap-6 custom-scrollbar">
                    
                    {/* Setup Dropdowns */}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        
                        {/* Start Symbol Selection */}
                        <div className="flex flex-col gap-2">
                            <label className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider font-mono">
                                Start Symbol
                            </label>
                            <div className="relative">
                                <Search size={12} className="absolute left-3 top-3 text-zinc-500" />
                                <input
                                    type="text"
                                    placeholder="Search start symbol..."
                                    value={searchStart}
                                    onChange={(e) => setSearchStart(e.target.value)}
                                    className="w-full pl-9 pr-3 py-2 bg-zinc-900 border border-zinc-850 hover:border-zinc-800 focus:border-cyan-500 rounded-xl text-xs font-mono text-zinc-350 focus:outline-none placeholder-zinc-600 transition-colors animate-none"
                                />
                            </div>
                            <select
                                size={4}
                                value={startId}
                                onChange={(e) => setStartId(e.target.value)}
                                className="w-full p-2 bg-zinc-900/50 border border-zinc-850 hover:border-zinc-800 rounded-xl text-xs font-mono text-zinc-350 focus:outline-none custom-scrollbar transition-colors h-28"
                            >
                                {filteredStartNodes.map((n) => (
                                    <option key={n.id} value={n.id} className="p-1 hover:bg-zinc-800 rounded">
                                        {n.name}
                                    </option>
                                ))}
                                {filteredStartNodes.length === 0 && (
                                    <option disabled className="text-zinc-600 text-center py-4">
                                        No symbols found
                                    </option>
                                )}
                            </select>
                        </div>

                        {/* End Symbol Selection */}
                        <div className="flex flex-col gap-2">
                            <label className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider font-mono">
                                End Symbol
                            </label>
                            <div className="relative">
                                <Search size={12} className="absolute left-3 top-3 text-zinc-500" />
                                <input
                                    type="text"
                                    placeholder="Search end symbol..."
                                    value={searchEnd}
                                    onChange={(e) => setSearchEnd(e.target.value)}
                                    className="w-full pl-9 pr-3 py-2 bg-zinc-900 border border-zinc-850 hover:border-zinc-800 focus:border-cyan-500 rounded-xl text-xs font-mono text-zinc-350 focus:outline-none placeholder-zinc-600 transition-colors animate-none"
                                />
                            </div>
                            <select
                                size={4}
                                value={endId}
                                onChange={(e) => setEndId(e.target.value)}
                                className="w-full p-2 bg-zinc-900/50 border border-zinc-850 hover:border-zinc-800 rounded-xl text-xs font-mono text-zinc-350 focus:outline-none custom-scrollbar transition-colors h-28"
                            >
                                {filteredEndNodes.map((n) => (
                                    <option key={n.id} value={n.id} className="p-1 hover:bg-zinc-800 rounded">
                                        {n.name}
                                    </option>
                                ))}
                                {filteredEndNodes.length === 0 && (
                                    <option disabled className="text-zinc-600 text-center py-4">
                                        No symbols found
                                    </option>
                                )}
                            </select>
                        </div>

                    </div>

                    {/* Actions */}
                    <div className="flex items-center gap-3">
                        <button
                            onClick={handleCalculatePath}
                            disabled={!startId || !endId}
                            className="flex-1 py-2.5 px-4 bg-cyan-500 hover:bg-cyan-400 disabled:bg-zinc-800 disabled:text-zinc-600 text-black text-xs font-bold uppercase tracking-widest rounded-xl transition-colors cursor-pointer disabled:cursor-not-allowed font-mono flex items-center justify-center gap-2"
                        >
                            <GitCommit size={14} />
                            Calculate Shortest Path
                        </button>
                        <button
                            onClick={handleClear}
                            className="px-4 py-2.5 bg-zinc-900 hover:bg-zinc-850 border border-zinc-800 hover:border-zinc-700 text-zinc-400 hover:text-white text-xs font-bold uppercase tracking-widest rounded-xl transition-all cursor-pointer font-mono"
                        >
                            Reset
                        </button>
                    </div>

                    {/* Path Results */}
                    {searched && (
                        <div className="flex flex-col gap-3 border-t border-zinc-900 pt-5">
                            <h4 className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider font-mono">
                                Path Trace Result
                            </h4>

                            {calculatedPath && calculatedPath.length > 0 ? (
                                <div className="flex flex-col gap-2">
                                    <div className="flex items-center gap-2 bg-emerald-500/10 border border-emerald-500/30 p-3 rounded-xl">
                                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                        <span className="text-[10px] text-emerald-400 font-bold font-mono">
                                            Path found: {calculatedPath.length} nodes involved ({calculatedPath.length - 1} edges/steps)
                                        </span>
                                    </div>

                                    {/* List steps */}
                                    <div className="flex flex-col bg-zinc-950 border border-zinc-900 rounded-xl overflow-hidden divide-y divide-zinc-900">
                                        {calculatedPath.map((node, index) => {
                                            const isStart = index === 0;
                                            const isEnd = index === calculatedPath.length - 1;

                                            return (
                                                <div 
                                                    key={node.id}
                                                    onClick={() => {
                                                        onSelectNode(node);
                                                    }}
                                                    className="flex items-start gap-3 p-3 hover:bg-zinc-900/60 cursor-pointer group transition-colors"
                                                >
                                                    <div className="flex flex-col items-center mt-0.5">
                                                        <span className="text-[9px] font-bold text-zinc-500 font-mono w-4 text-center">
                                                            {index + 1}
                                                        </span>
                                                        {!isEnd && (
                                                            <div className="w-[1px] h-6 bg-zinc-800 my-1 group-hover:bg-cyan-500 transition-colors" />
                                                        )}
                                                    </div>

                                                    <div className="flex-1 min-w-0 flex flex-col gap-0.5">
                                                        <div className="flex items-center gap-2">
                                                            <span className="text-[11px] font-bold text-white font-mono truncate group-hover:text-cyan-400 transition-colors">
                                                                 {node.name}
                                                            </span>
                                                            {isStart && (
                                                                <span className="text-[8px] font-black text-cyan-400 bg-cyan-400/15 border border-cyan-400/35 px-1 rounded-md uppercase font-mono scale-90">
                                                                    START
                                                                </span>
                                                            )}
                                                            {isEnd && (
                                                                <span className="text-[8px] font-black text-amber-500 bg-amber-500/15 border border-amber-500/35 px-1 rounded-md uppercase font-mono scale-90">
                                                                    END
                                                                </span>
                                                            )}
                                                            {!isStart && !isEnd && (
                                                                <span className="text-[8px] font-bold text-zinc-500 bg-zinc-900 border border-zinc-850 px-1 rounded-md uppercase font-mono scale-90">
                                                                    {node.kind}
                                                                </span>
                                                            )}
                                                        </div>
                                                        <span className="text-[9px] text-zinc-500 font-mono truncate">
                                                            {node.path}
                                                        </span>
                                                    </div>
                                                </div>
                                            );
                                        })}
                                    </div>
                                </div>
                            ) : (
                                <div className="flex items-center gap-2 bg-rose-500/10 border border-rose-500/30 p-3 rounded-xl">
                                    <div className="w-1.5 h-1.5 rounded-full bg-rose-500 animate-pulse" />
                                    <span className="text-[10px] text-rose-400 font-bold font-mono">
                                        No connections or path found between selected symbols.
                                    </span>
                                </div>
                            )}
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div className="px-6 py-4 border-t border-zinc-900 bg-zinc-900/20 flex items-center justify-between text-[9px] text-zinc-500 font-mono select-none">
                    <span>Click any step symbol to center/focus on the active canvas.</span>
                </div>

            </div>
        </div>
    );
};

// Metadata: [PathFinderModal]
