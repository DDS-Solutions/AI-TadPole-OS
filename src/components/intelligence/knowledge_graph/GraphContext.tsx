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
import { i18n } from '../../../i18n';
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

// ==========================================
// Module-level Helper Functions
// ==========================================

// Helper to extract links from text matching concept IDs in OKF descriptions
const extractLinks = (text: string, nodeIds: Set<string>): string[] => {
    if (!text || typeof text !== 'string') return [];
    const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
    const targets: string[] = [];
    let match;
    while ((match = linkRegex.exec(text)) !== null) {
        const targetUrl = match[2];
        const cleanTarget = targetUrl.replace(/^\//, '').replace(/\.md$/, '').trim();
        if (nodeIds.has(cleanTarget)) {
            targets.push(cleanTarget);
        } else {
            for (const id of nodeIds) {
                if (id.endsWith(cleanTarget) || cleanTarget.endsWith(id)) {
                    targets.push(id);
                    break;
                }
            }
        }
    }
    return targets;
};

// Module-level cache to hold node coordinate coordinates referentially across renders.
// Cleared on viewMode transitions and unmount via hook.
const nodesCache = new Map<string, ExtendedGraphNode>();

// ==========================================
// Sub-provider components for Context Tree Separation
// ==========================================

export const GraphDataProvider: React.FC<{ children: React.ReactNode; value: GraphDataContextType }> = ({ children, value }) => {
    return <GraphDataContext.Provider value={value}>{children}</GraphDataContext.Provider>;
};

export const SelectionProvider: React.FC<{ children: React.ReactNode; value: SelectionContextType }> = ({ children, value }) => {
    return <SelectionContext.Provider value={value}>{children}</SelectionContext.Provider>;
};

export const NavigationProvider: React.FC<{ children: React.ReactNode; value: NavigationContextType }> = ({ children, value }) => {
    return <NavigationContext.Provider value={value}>{children}</NavigationContext.Provider>;
};

export const ViewportProvider: React.FC<{ children: React.ReactNode; value: ViewportContextType }> = ({ children, value }) => {
    return <ViewportContext.Provider value={value}>{children}</ViewportContext.Provider>;
};

export const UIStateProvider: React.FC<{ children: React.ReactNode; value: UIStateContextType }> = ({ children, value }) => {
    return <UIStateContext.Provider value={value}>{children}</UIStateContext.Provider>;
};

// ==========================================
// Master Provider Implementation
// ==========================================

export const GraphProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const fgRef = useRef<GraphMethods | undefined>(undefined);
    const abortControllerRef = useRef<AbortController | null>(null);

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

    // Context 5: UI State
    const [isPathFinderOpen, setIsPathFinderOpen] = useState(false);
    const [highlightedPathNodeIds, setHighlightedPathNodeIds] = useState<string[] | null>(null);
    const [activeInfoTab, setActiveInfoTab] = useState<'info' | 'memory'>('info');

    // --- OKF Extensions ---
    const [viewMode, setViewModeState] = useState<'symbols' | 'okf'>('symbols');

    const setViewMode = useCallback((mode: 'symbols' | 'okf') => {
        setViewModeState(mode);
        nodesCache.clear();
        setHighlightedPathNodeIds(null);
    }, [setHighlightedPathNodeIds]);

    // Clear coordinate cache on unmount to avoid memory leaks/pollution
    useEffect(() => {
        return () => {
            nodesCache.clear();
        };
    }, []);

    const resetSelection = useCallback(() => {
        setSelectedNode(null);
        setAffectedNodesList([]);
        setBlastRadiusError(null);
    }, [setSelectedNode, setAffectedNodesList, setBlastRadiusError]);

    // Derivations
    const affectedNodes = useMemo(() => new Set(affectedNodesList), [affectedNodesList]);

    const isMemoryNode = useMemo(() => {
        if (!selectedNode) return false;
        const name = (selectedNode.name || '').toLowerCase();
        const path = (selectedNode.path || '').toLowerCase();
        const memoryWordRegex = /\b(memory|rag|iks|knowledge_store|lance)\b/;
        return memoryWordRegex.test(name) || memoryWordRegex.test(path);
    }, [selectedNode]);

    // Cleanup reference controller on unmount
    useEffect(() => {
        return () => {
            if (abortControllerRef.current) {
                abortControllerRef.current.abort();
            }
        };
    }, []);

    // Load data based on viewMode
    useEffect(() => {
        const controller = new AbortController();
        Promise.resolve().then(() => {
            resetSelection();
        });
        
        const load = async () => {
            setLoading(true);
            setError(null);
            try {
                if (viewMode === 'symbols') {
                    const graph = await intelligence_api_service.get_code_graph();
                    const sanitized = sanitize_graph_data(graph);
                    if (!controller.signal.aborted) {
                        setData(sanitized.data);
                    }
                } else {
                    // Fetch OKF knowledge entries from IKS (limit 200 for graph display)
                    const entries = await intelligence_api_service.get_knowledge({ limit: 200 });
                    if (controller.signal.aborted) return;

                    const nodeIds = new Set(entries.map(e => e.id));
                    const nodes: ExtendedGraphNode[] = entries.map(e => ({
                        id: e.id,
                        name: e.title || e.id,
                        path: e.topic,
                        kind: e.concept_type,
                        signature: e.resource_uri || '',
                        start_line: 0,
                        end_line: 0,
                        is_affected: false,
                        concept_type: e.concept_type,
                        title: e.title || undefined,
                        description: e.description || undefined,
                        resource_uri: e.resource_uri || undefined,
                        tags: e.tags || undefined,
                        confidence: e.confidence,
                        human_confirmed: e.human_confirmed,
                        text: e.text,
                    }));

                    const links: ForceGraphLink[] = [];
                    const processedLinks = new Set<string>();

                    for (const entry of entries) {
                        const targets = extractLinks(entry.text, nodeIds);
                        for (const target of targets) {
                            const linkKey = `${entry.id}->${target}`;
                            if (!processedLinks.has(linkKey)) {
                                links.push({
                                    source: entry.id,
                                    target: target,
                                    is_path_highlighted: false,
                                });
                                processedLinks.add(linkKey);
                            }
                        }
                    }

                    if (!controller.signal.aborted) {
                        setData({
                            nodes,
                            links: links as ForceGraphLink[],
                            anomalies: [],
                        });
                    }
                }
            } catch (err) {
                if (err instanceof Error && err.name === 'AbortError') return;
                console.error(`[GraphContext] Failed to fetch graph (${viewMode}):`, err);
                if (!controller.signal.aborted) {
                    setError(viewMode === 'symbols' ? i18n.t('knowledge_graph.error_fetch_symbols') : i18n.t('knowledge_graph.error_fetch_okf'));
                }
            } finally {
                if (!controller.signal.aborted) {
                    setLoading(false);
                }
            }
        };
        load();
        
        return () => {
            controller.abort();
        };
    }, [viewMode, resetSelection]);

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

        // Eviction safety limit: clear cache if it grows past 5,000 entries to prevent OOM
        if (nodesCache.size > 5000) {
            nodesCache.clear();
        }

        const nodes: ExtendedGraphNode[] = data.nodes.map(node => {
            const id = (node as ExtendedGraphNode).id || `${node.path}:${node.name}`;
            const is_affected = affectedNodes.has(id);
            const is_path_highlighted = pathNodeSet.has(id);

            const optNode = node as ExtendedGraphNode;
            const cached = nodesCache.get(id);
            if (cached && 
                cached.is_affected === is_affected && 
                cached.is_path_highlighted === is_path_highlighted &&
                cached.name === optNode.name &&
                cached.path === optNode.path &&
                cached.kind === optNode.kind &&
                cached.community === optNode.community &&
                cached.confidence === optNode.confidence &&
                cached.human_confirmed === optNode.human_confirmed &&
                cached.text === optNode.text &&
                cached.title === optNode.title &&
                cached.description === optNode.description &&
                cached.resource_uri === optNode.resource_uri &&
                cached.tags === optNode.tags
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
                is_path_highlighted,
                is_implicit: false
            };
        });

        // Inject dynamic implicit semantic links for peer nodes in OKF mode
        if (viewMode === 'okf' && selectedNode) {
            affectedNodes.forEach(peerId => {
                if (peerId !== selectedNode.id) {
                    links.push({
                        source: selectedNode.id,
                        target: peerId,
                        is_implicit: true
                    });
                }
            });
        }

        // Build O(1) lookup map from the same nodes array (no extra iteration)
        const nodeMap = new Map<string, ExtendedGraphNode>();
        for (const node of nodes) {
            nodeMap.set(node.id, node);
        }

        return { nodes, links, nodeMap };
    }, [data, affectedNodes, highlightedPathNodeIds, selectedNode, viewMode]);

    // Context 2 Actions: Node Selection with AbortController race-condition guard
    const selectNode = useCallback(async (node: ExtendedGraphNode, pushHistory = true) => {
        if (selectedNode?.id === node.id) {
            return;
        }

        setSelectedNode(node);

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

        // Focus/Zoom imperative call with rAF to ensure React renders first
        requestAnimationFrame(() => {
            if (fgRef.current && typeof node.x === 'number' && typeof node.y === 'number') {
                fgRef.current.centerAt(node.x, node.y, 1000);
                fgRef.current.zoom(2.5, 1000);
            }
        });

        if (abortControllerRef.current) {
            abortControllerRef.current.abort();
        }

        const controller = new AbortController();
        abortControllerRef.current = controller;

        setBlastRadiusLoading(true);
        setBlastRadiusError(null);

        try {
            if (viewMode === 'okf') {
                const peers = await intelligence_api_service.get_knowledge_peers(node.id, 5, controller.signal);
                if (!controller.signal.aborted) {
                    const peer_ids = peers.map(p => p.id).sort();
                    setAffectedNodesList(peer_ids);
                }
            } else {
                const affected = await intelligence_api_service.get_blast_radius(node.name || '', node.path || '', controller.signal);
                if (!controller.signal.aborted) {
                    const affected_ids = affected.map(n => `${n.path}:${n.name}`).sort();
                    setAffectedNodesList(affected_ids);
                }
            }
        } catch (err) {
            if (err instanceof Error && err.name === 'AbortError') {
                return; // Request was aborted, ignore
            }
            console.error(`[GraphContext] Fetching ${viewMode === 'okf' ? 'peers' : 'blast radius'} failed:`, err);
            if (!controller.signal.aborted) {
                setBlastRadiusError(viewMode === 'okf' ? i18n.t('knowledge_graph.error_peer_links') : i18n.t('knowledge_graph.error_blast_radius'));
            }
        } finally {
            if (!controller.signal.aborted) {
                setBlastRadiusLoading(false);
            }
        }
    }, [fgRef, selectedNode, setSelectedNode, setBlastRadiusLoading, setBlastRadiusError, setAffectedNodesList, setHistoryState, viewMode]);

    // Context 3 Actions: Navigation (stale-closure safe via historyState dependency and direct selectNode calls)
    const goBack = useCallback(() => {
        if (historyState.index > 0) {
            const nextIndex = historyState.index - 1;
            const targetNode = historyState.history[nextIndex];
            setHistoryState(prev => ({ ...prev, index: nextIndex }));
            selectNode(targetNode, false);
        }
    }, [historyState, selectNode]);

    const goForward = useCallback(() => {
        if (historyState.index < historyState.history.length - 1) {
            const nextIndex = historyState.index + 1;
            const targetNode = historyState.history[nextIndex];
            setHistoryState(prev => ({ ...prev, index: nextIndex }));
            selectNode(targetNode, false);
        }
    }, [historyState, selectNode]);

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
        try {
            const canvas = fgRef.current?.canvasElement();
            if (!canvas) {
                console.error('[GraphContext] Canvas element not found');
                return;
            }

            // Size guard: limit to 16M pixels (e.g. 4096 x 4096) to prevent OOM
            if (canvas.width * canvas.height > 16 * 1024 * 1024) {
                console.warn('[GraphContext] Canvas too large for export');
                return;
            }

            canvas.toBlob((blob) => {
                if (!blob) {
                    console.error('[GraphContext] Failed to generate image blob');
                    return;
                }
                const url = URL.createObjectURL(blob);
                const link = document.createElement('a');
                link.download = `tadpole_knowledge_graph_${new Date().toISOString().slice(0, 10)}.png`;
                link.href = url;
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
                URL.revokeObjectURL(url);
            }, 'image/png');
        } catch (err) {
            console.error('[GraphContext] Export PNG failed:', err);
        }
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
        isMemoryNode,
        viewMode,
        setViewMode
    }), [isPathFinderOpen, setIsPathFinderOpen, highlightedPathNodeIds, setHighlightedPathNodeIds, activeInfoTab, setActiveInfoTab, isMemoryNode, viewMode, setViewMode]);

    return (
        <GraphDataProvider value={dataValue}>
            <SelectionProvider value={selectionValue}>
                <NavigationProvider value={navigationValue}>
                    <ViewportProvider value={viewportValue}>
                        <UIStateProvider value={uiStateValue}>
                            {children}
                        </UIStateProvider>
                    </ViewportProvider>
                </NavigationProvider>
            </SelectionProvider>
        </GraphDataProvider>
    );
};

// Metadata: [GraphContext]
