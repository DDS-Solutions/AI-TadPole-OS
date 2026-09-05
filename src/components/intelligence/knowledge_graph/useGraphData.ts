/**
 * @docs ARCHITECTURE:Intelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / useGraphData
 * - **Primary Entrypoints**: `useGraphData`, `UseGraphDataResult`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[useGraphData]`
 * - **Witness Tests**: none declared
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { intelligence_api_service } from '../../../services/intelligence_api_service';
import { i18n } from '../../../i18n';
import { sanitize_graph_data } from './graph_sanitizer';
import type { SanitizedGraphResult } from './graph_sanitizer';
import type { ExtendedGraphNode, ForceGraphLink } from './types';

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

export interface UseGraphDataResult {
    data: SanitizedGraphResult['data'] | null;
    loading: boolean;
    error: string | null;
    graphData: {
        nodes: ExtendedGraphNode[];
        links: ForceGraphLink[];
        nodeMap: Map<string, ExtendedGraphNode>;
    };
    viewMode: 'symbols' | 'okf';
    setViewMode: (mode: 'symbols' | 'okf') => void;
    resetSelection: () => void;
    // Selection state passed through for graph projection coupling
    selectedNode: ExtendedGraphNode | null;
    setSelectedNode: React.Dispatch<React.SetStateAction<ExtendedGraphNode | null>>;
    hoverNode: ExtendedGraphNode | null;
    setHoverNode: React.Dispatch<React.SetStateAction<ExtendedGraphNode | null>>;
    affectedNodes: Set<string>;
    affectedNodesList: string[];
    setAffectedNodesList: React.Dispatch<React.SetStateAction<string[]>>;
    blastRadiusLoading: boolean;
    setBlastRadiusLoading: React.Dispatch<React.SetStateAction<boolean>>;
    blastRadiusError: string | null;
    setBlastRadiusError: React.Dispatch<React.SetStateAction<string | null>>;
    highlightedPathNodeIds: string[] | null;
    setHighlightedPathNodeIds: React.Dispatch<React.SetStateAction<string[] | null>>;
    isMemoryNode: boolean;
}

export function useGraphData(): UseGraphDataResult {
    // UI State
    const [viewMode, setViewModeState] = useState<'symbols' | 'okf'>('symbols');

    // Data State
    const [data, setData] = useState<SanitizedGraphResult['data'] | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Selection State (co-located here because graphData projection depends on it)
    const [selectedNode, setSelectedNode] = useState<ExtendedGraphNode | null>(null);
    const [hoverNode, setHoverNode] = useState<ExtendedGraphNode | null>(null);
    const [affectedNodesList, setAffectedNodesList] = useState<string[]>([]);
    const [blastRadiusLoading, setBlastRadiusLoading] = useState(false);
    const [blastRadiusError, setBlastRadiusError] = useState<string | null>(null);

    const [highlightedPathNodeIds, setHighlightedPathNodeIds] = useState<string[] | null>(null);

    const resetSelection = useCallback(() => {
        setSelectedNode(null);
        setAffectedNodesList([]);
        setBlastRadiusError(null);
    }, [setSelectedNode, setAffectedNodesList, setBlastRadiusError]);

    const setViewMode = useCallback((mode: 'symbols' | 'okf') => {
        setViewModeState(mode);
        setHighlightedPathNodeIds(null);
        resetSelection();
    }, [setHighlightedPathNodeIds, resetSelection]);

    // Derivations
    const affectedNodes = useMemo(() => new Set(affectedNodesList), [affectedNodesList]);

    const isMemoryNode = useMemo(() => {
        if (!selectedNode) return false;
        const name = (selectedNode.name || '').toLowerCase();
        const path = (selectedNode.path || '').toLowerCase();
        const memoryWordRegex = /\b(memory|rag|iks|knowledge_store|lance)\b/;
        return memoryWordRegex.test(name) || memoryWordRegex.test(path);
    }, [selectedNode]);

    // Load data based on viewMode
    useEffect(() => {
        const controller = new AbortController();
        
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
                        docstring: null,
                        docstring_range: null,
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
                console.error(`[useGraphData] Failed to fetch graph (${viewMode}):`, err);
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
    }, [viewMode]);

    // Pure functional memoized graph data projection
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
            const rawNode = node as ExtendedGraphNode;
            const id = rawNode.id || `${rawNode.path}:${rawNode.name}`;
            const is_affected = affectedNodes.has(id);
            const is_path_highlighted = pathNodeSet.has(id);

            const resourceUriLower = (rawNode.resource_uri || '').toLowerCase();
            const tagsLower = (rawNode.tags || '').toLowerCase();

            const is_broken = resourceUriLower.startsWith('broken') || tagsLower.includes('broken');
            const isConfirmed = rawNode.human_confirmed === true;
            const is_expiring = (rawNode.ttl !== null && rawNode.ttl !== undefined && !isConfirmed) || tagsLower.includes('expiring');

            return {
                ...rawNode,
                id,
                is_affected,
                is_path_highlighted,
                is_broken,
                is_expiring,
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

        // Build O(1) lookup map from the same nodes array
        const nodeMap = new Map<string, ExtendedGraphNode>();
        for (const node of nodes) {
            nodeMap.set(node.id, node);
        }

        return { nodes, links, nodeMap };
    }, [data, affectedNodes, highlightedPathNodeIds, selectedNode, viewMode]);

    return {
        data,
        loading,
        error,
        graphData,
        viewMode,
        setViewMode,
        resetSelection,
        selectedNode,
        setSelectedNode,
        hoverNode,
        setHoverNode,
        affectedNodes,
        affectedNodesList,
        setAffectedNodesList,
        blastRadiusLoading,
        setBlastRadiusLoading,
        blastRadiusError,
        setBlastRadiusError,
        highlightedPathNodeIds,
        setHighlightedPathNodeIds,
        isMemoryNode,
    };
}
