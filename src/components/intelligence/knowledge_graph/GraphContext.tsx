/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[GraphContext]` in observability traces.
 */

import React, { useState, useEffect, useMemo, useRef, useCallback } from 'react';
import { intelligence_api_service } from '../../../services/intelligence_api_service';
import { sanitize_graph_data } from './graph_sanitizer';
import type { SanitizedGraphResult } from './graph_sanitizer';
import type { ExtendedGraphNode, ForceGraphLink, GraphMethods } from './types';
import {
    GraphDataContext,
    SelectionContext,
    NavigationContext,
    ViewportContext,
    UIStateContext,
} from './graph_context_defs';
import type {
    GraphDataContextType,
    SelectionContextType,
    NavigationContextType,
    ViewportContextType,
    UIStateContextType,
} from './graph_context_defs';

// Re-export types for backwards compatibility with external consumers
export type {
    GraphDataContextType,
    SelectionContextType,
    NavigationContextType,
    ViewportContextType,
    UIStateContextType,
};

// Module-level coordinate cache — preserves force-graph x/y positions across
// useMemo recomputations without requiring a ref read during render.
const nodesCache = new Map<string, ExtendedGraphNode>();

// ==========================================
// Provider Implementation
// ==========================================

export const GraphProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const fgRef = useRef<GraphMethods | undefined>(undefined);
    const abortControllerRef = useRef<AbortController | null>(null);
    const isMountedRef = useRef<boolean>(true);


    // Context 1: Data State
    const [data, setData] = useState<SanitizedGraphResult['data'] | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Context 2: Selection State
    const [selectedNode, setSelectedNode] = useState<ExtendedGraphNode | null>(null);
    const [hoverNode, setHoverNode] = useState<ExtendedGraphNode | null>(null);
    const [affectedNodesList, setAffectedNodesList] = useState<string[]>([]);
    const [blastRadiusLoading, setBlastRadiusLoading] = useState(false);
    const [blastRadiusError, setBlastRadiusError] = useState<string | null>(null);

    // Context 3: Navigation State (Atomic History)
    const [historyState, setHistoryState] = useState<{ history: ExtendedGraphNode[]; index: number }>({
        history: [],
        index: -1,
    });
    const [navigateTo, setNavigateTo] = useState<ExtendedGraphNode | null>(null);

    // Context 5: UI State
    const [isPathFinderOpen, setIsPathFinderOpen] = useState(false);
    const [highlightedPathNodeIds, setHighlightedPathNodeIds] = useState<string[] | null>(null);
    const [activeInfoTab, setActiveInfoTab] = useState<'info' | 'memory'>('info');

    // Derivations
    const affectedNodes = useMemo(() => new Set(affectedNodesList), [affectedNodesList]);

    const isMemoryNode = useMemo(() => {
        if (!selectedNode) return false;
        const name = selectedNode.name.toLowerCase();
        const path = selectedNode.path.toLowerCase();
        return name.includes('memory') || path.includes('memory');
    }, [selectedNode]);

    // Cleanup reference controller on unmount
    useEffect(() => {
        isMountedRef.current = true;
        return () => {
            isMountedRef.current = false;
            if (abortControllerRef.current) {
                abortControllerRef.current.abort();
            }
        };
    }, []);

    // Initial load
    useEffect(() => {
        const load = async () => {
            try {
                const graph = await intelligence_api_service.get_code_graph();
                const sanitized = sanitize_graph_data(graph);
                if (isMountedRef.current) {
                    setData(sanitized.data);
                }
            } catch (err) {
                console.error('[KnowledgeGraph] Failed to fetch graph:', err);
                if (isMountedRef.current) {
                    setError('Failed to fetch graph data.');
                }
            } finally {
                if (isMountedRef.current) {
                    setLoading(false);
                }
            }
        };
        load();
    }, []);

    // Memoized graph data projection with referential coordinate preservation caching
    const graphData = useMemo(() => {
        if (!data) return { nodes: [], links: [], nodeMap: new Map<string, ExtendedGraphNode>() };

        const pathNodeSet = new Set(highlightedPathNodeIds || []);
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
            const is_affected = affectedNodes.has(id);
            const is_path_highlighted = pathNodeSet.has(id);

            const cached = nodesCache.get(id);
            if (cached && 
                cached.is_affected === is_affected && 
                cached.is_path_highlighted === is_path_highlighted &&
                cached.name === node.name &&
                cached.path === node.path &&
                cached.kind === node.kind
            ) {
                return cached;
            }

            const newNode: ExtendedGraphNode = {
                ...node,
                id,
                is_affected,
                is_path_highlighted,
                x: cached?.x,
                y: cached?.y,
                vx: cached?.vx,
                vy: cached?.vy,
                fx: cached?.fx,
                fy: cached?.fy,
                community: node.community
            };
            nodesCache.set(id, newNode);
            return newNode;
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

        // Build O(1) lookup map from the same nodes array (no extra iteration)
        const nodeMap = new Map<string, ExtendedGraphNode>();
        for (const node of nodes) {
            nodeMap.set(node.id, node);
        }

        return { nodes, links, nodeMap };
    }, [data, affectedNodes, highlightedPathNodeIds]);

    // Context 2 Actions: Node Selection with AbortController race-condition guard
    const selectNode = useCallback(async (node: ExtendedGraphNode, pushHistory = true) => {
        if (abortControllerRef.current) {
            abortControllerRef.current.abort();
        }

        const controller = new AbortController();
        abortControllerRef.current = controller;

        setSelectedNode(node);
        setBlastRadiusLoading(true);
        setBlastRadiusError(null);

        try {
            const affected = await intelligence_api_service.get_blast_radius(node.name, node.path, controller.signal);
            
            if (isMountedRef.current && !controller.signal.aborted) {
                const affected_ids = affected.map(n => `${n.path}:${n.name}`).sort();
                setAffectedNodesList(affected_ids);

                if (pushHistory) {
                    setHistoryState(prev => {
                        const truncated = prev.history.slice(0, prev.index + 1);
                        if (truncated.length > 0 && truncated[truncated.length - 1].id === node.id) {
                            return prev;
                        }
                        const updated = [...truncated, node];
                        return {
                            history: updated,
                            index: updated.length - 1
                        };
                    });
                }

                // Focus/Zoom imperative call
                if (fgRef.current && typeof node.x === 'number' && typeof node.y === 'number') {
                    fgRef.current.centerAt(node.x, node.y, 1000);
                    fgRef.current.zoom(2.5, 1000);
                }
            }
        } catch (err) {
            if (err instanceof Error && err.name === 'AbortError') {
                return; // Request was aborted, ignore
            }
            console.error('[KnowledgeGraph] Blast radius failed:', err);
            if (isMountedRef.current && !controller.signal.aborted) {
                setBlastRadiusError('Failed to fetch blast radius.');
            }
        } finally {
            if (isMountedRef.current && !controller.signal.aborted) {
                setBlastRadiusLoading(false);
            }
        }
    }, [fgRef, setSelectedNode, setBlastRadiusLoading, setBlastRadiusError, setAffectedNodesList, setHistoryState]);

    const resetSelection = useCallback(() => {
        setSelectedNode(null);
        setAffectedNodesList([]);
        setBlastRadiusError(null);
    }, [setSelectedNode, setAffectedNodesList, setBlastRadiusError]);

    // Context 3 Actions: Navigation (stale-closure safe via functional updater and navigateTo side effect)
    const goBack = useCallback(() => {
        setHistoryState(prev => {
            if (prev.index > 0) {
                const nextIndex = prev.index - 1;
                setNavigateTo(prev.history[nextIndex]);
                return { ...prev, index: nextIndex };
            }
            return prev;
        });
    }, [setHistoryState, setNavigateTo]);

    const goForward = useCallback(() => {
        setHistoryState(prev => {
            if (prev.index < prev.history.length - 1) {
                const nextIndex = prev.index + 1;
                setNavigateTo(prev.history[nextIndex]);
                return { ...prev, index: nextIndex };
            }
            return prev;
        });
    }, [setHistoryState, setNavigateTo]);

    // Safely trigger node selection when navigating via history.
    // Uses queueMicrotask to avoid synchronous cascading setState inside the effect body
    // (satisfies react-hooks/set-state-in-effect).
    useEffect(() => {
        if (navigateTo) {
            const target = navigateTo;
            queueMicrotask(() => {
                setNavigateTo(null);
                selectNode(target, false);
            });
        }
    }, [navigateTo, selectNode, setNavigateTo]);

    // Context 4 Actions: Viewport controls
    const zoomIn = useCallback(() => {
        if (fgRef.current) {
            fgRef.current.zoom(fgRef.current.zoom() * 1.4, 400);
        }
    }, [fgRef]);

    const zoomOut = useCallback(() => {
        if (fgRef.current) {
            fgRef.current.zoom(fgRef.current.zoom() / 1.4, 400);
        }
    }, [fgRef]);

    const zoomFit = useCallback(() => {
        if (fgRef.current) {
            fgRef.current.zoomToFit(800, 50);
        }
    }, [fgRef]);

    const exportPNG = useCallback(() => {
        const canvas = fgRef.current?.canvasElement();
        if (!canvas) {
            console.error('[KnowledgeGraph] Canvas element not found');
            return;
        }

        const link = document.createElement('a');
        link.download = `tadpole_knowledge_graph_${new Date().toISOString().slice(0, 10)}.png`;
        link.href = canvas.toDataURL('image/png');
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    }, [fgRef]);

    // ==========================================
    // 4. Memoized Provider Values
    // ==========================================

    const dataValue = useMemo<GraphDataContextType>(() => ({
        data,
        loading,
        error,
        graphData
    }), [data, loading, error, graphData]);

    const selectionValue = useMemo<SelectionContextType>(() => ({
        selectedNode,
        hoverNode,
        affectedNodes,
        blastRadiusLoading,
        blastRadiusError,
        setHoverNode,
        selectNode,
        resetSelection
    }), [selectedNode, hoverNode, affectedNodes, blastRadiusLoading, blastRadiusError, setHoverNode, selectNode, resetSelection]);

    const navigationValue = useMemo<NavigationContextType>(() => ({
        nodeHistory: historyState.history,
        historyIndex: historyState.index,
        goBack,
        goForward
    }), [historyState.history, historyState.index, goBack, goForward]);

    const viewportValue = useMemo<ViewportContextType>(() => ({
        zoomIn,
        zoomOut,
        zoomFit,
        exportPNG,
        fgRef
    }), [zoomIn, zoomOut, zoomFit, exportPNG, fgRef]);

    const uiStateValue = useMemo<UIStateContextType>(() => ({
        isPathFinderOpen,
        setIsPathFinderOpen,
        highlightedPathNodeIds,
        setHighlightedPathNodeIds,
        activeInfoTab,
        setActiveInfoTab,
        isMemoryNode
    }), [isPathFinderOpen, setIsPathFinderOpen, highlightedPathNodeIds, setHighlightedPathNodeIds, activeInfoTab, setActiveInfoTab, isMemoryNode]);

    return (
        <GraphDataContext.Provider value={dataValue}>
            <SelectionContext.Provider value={selectionValue}>
                <NavigationContext.Provider value={navigationValue}>
                    <ViewportContext.Provider value={viewportValue}>
                        <UIStateContext.Provider value={uiStateValue}>
                            {children}
                        </UIStateContext.Provider>
                    </ViewportContext.Provider>
                </NavigationContext.Provider>
            </SelectionContext.Provider>
        </GraphDataContext.Provider>
    );
};

// Metadata: [GraphContext]
