/**
 * @docs ARCHITECTURE:Intelligence
 *
 * ### AI Assist Note
 * **Node Selection, Navigation & Viewport Hook**: Manages the interactive
 * behaviors of the graph — node selection with blast radius fetching,
 * browser-style back/forward navigation, viewport zoom controls,
 * and PNG export.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: AbortController race conditions on rapid node clicks.
 * - **Telemetry Link**: Search `[useNodeSelection]` in component traces.
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { intelligence_api_service } from '../../../services/intelligence_api_service';
import { i18n } from '../../../i18n';
import type { ExtendedGraphNode, GraphMethods } from './types';

export interface UseNodeSelectionParams {
    selectedNode: ExtendedGraphNode | null;
    setSelectedNode: React.Dispatch<React.SetStateAction<ExtendedGraphNode | null>>;
    setAffectedNodesList: React.Dispatch<React.SetStateAction<string[]>>;
    setBlastRadiusLoading: React.Dispatch<React.SetStateAction<boolean>>;
    setBlastRadiusError: React.Dispatch<React.SetStateAction<string | null>>;
    viewMode: 'symbols' | 'okf';
}

export interface UseNodeSelectionResult {
    fgRef: React.MutableRefObject<GraphMethods | undefined>;
    selectNode: (node: ExtendedGraphNode, pushHistory?: boolean) => Promise<void>;
    historyState: { history: ExtendedGraphNode[]; index: number };
    goBack: () => void;
    goForward: () => void;
    zoomIn: () => void;
    zoomOut: () => void;
    zoomFit: () => void;
    exportPNG: () => void;
    isPathFinderOpen: boolean;
    setIsPathFinderOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isAnomalyPanelOpen: boolean;
    setIsAnomalyPanelOpen: React.Dispatch<React.SetStateAction<boolean>>;
    activeInfoTab: 'info' | 'memory';
    setActiveInfoTab: React.Dispatch<React.SetStateAction<'info' | 'memory'>>;
}

export function useNodeSelection({
    selectedNode,
    setSelectedNode,
    setAffectedNodesList,
    setBlastRadiusLoading,
    setBlastRadiusError,
    viewMode,
}: UseNodeSelectionParams): UseNodeSelectionResult {
    const fgRef = useRef<GraphMethods | undefined>(undefined);
    const abortControllerRef = useRef<AbortController | null>(null);

    // Ref to hold selectedNode to break useCallback dependency chains
    const selectedNodeRef = useRef(selectedNode);
    useEffect(() => {
        selectedNodeRef.current = selectedNode;
    }, [selectedNode]);

    // Navigation State (Atomic History)
    const [historyState, setHistoryState] = useState<{ history: ExtendedGraphNode[]; index: number }>({
        history: [],
        index: -1,
    });

    // UI State
    const [isPathFinderOpen, setIsPathFinderOpen] = useState(false);
    const [isAnomalyPanelOpen, setIsAnomalyPanelOpen] = useState(false);
    const [activeInfoTab, setActiveInfoTab] = useState<'info' | 'memory'>('info');

    // Cleanup abort controller on unmount
    useEffect(() => {
        return () => {
            if (abortControllerRef.current) {
                abortControllerRef.current.abort();
            }
        };
    }, []);

    // Camera movement side-effect synchronized with React's commit cycle
    useEffect(() => {
        if (selectedNode && fgRef.current && typeof selectedNode.x === 'number' && typeof selectedNode.y === 'number') {
            fgRef.current.centerAt(selectedNode.x, selectedNode.y, 1000);
            fgRef.current.zoom(2.5, 1000);
        }
    }, [selectedNode]);

    // Node Selection with AbortController race-condition guard and stable callback reference
    const selectNode = useCallback(async (node: ExtendedGraphNode, pushHistory = true) => {
        if (selectedNodeRef.current?.id === node.id) {
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
            console.error(`[useNodeSelection] Fetching ${viewMode === 'okf' ? 'peers' : 'blast radius'} failed:`, err);
            if (!controller.signal.aborted) {
                setBlastRadiusError(viewMode === 'okf' ? i18n.t('knowledge_graph.error_peer_links') : i18n.t('knowledge_graph.error_blast_radius'));
            }
        } finally {
            if (!controller.signal.aborted) {
                setBlastRadiusLoading(false);
            }
        }
    }, [setSelectedNode, setAffectedNodesList, setBlastRadiusLoading, setBlastRadiusError, setHistoryState, viewMode]);

    // Navigation with functional state updates (stable callback references)
    const goBack = useCallback(() => {
        setHistoryState(prev => {
            if (prev.index <= 0) return prev;
            const nextIndex = prev.index - 1;
            const targetNode = prev.history[nextIndex];
            selectNode(targetNode, false);
            return { ...prev, index: nextIndex };
        });
    }, [selectNode]);

    const goForward = useCallback(() => {
        setHistoryState(prev => {
            if (prev.index >= prev.history.length - 1) return prev;
            const nextIndex = prev.index + 1;
            const targetNode = prev.history[nextIndex];
            selectNode(targetNode, false);
            return { ...prev, index: nextIndex };
        });
    }, [selectNode]);

    // Viewport controls
    const zoomIn = useCallback(() => {
        if (fgRef.current) {
            fgRef.current.zoom(fgRef.current.zoom() * 1.4, 400);
        }
    }, []);

    const zoomOut = useCallback(() => {
        if (fgRef.current) {
            fgRef.current.zoom(fgRef.current.zoom() / 1.4, 400);
        }
    }, []);

    const zoomFit = useCallback(() => {
        if (fgRef.current) {
            const nodes = fgRef.current.graphData()?.nodes;
            if (nodes && nodes.length > 0) {
                let sumX = 0;
                let sumY = 0;
                let validCount = 0;
                for (let i = 0; i < nodes.length; i++) {
                    const nx = nodes[i].x;
                    const ny = nodes[i].y;
                    if (typeof nx === 'number' && typeof ny === 'number' && !isNaN(nx) && !isNaN(ny)) {
                        sumX += nx;
                        sumY += ny;
                        validCount++;
                    }
                }
                if (validCount > 0 && typeof fgRef.current.centerAt === 'function') {
                    fgRef.current.centerAt(sumX / validCount, sumY / validCount, 400);
                }
            }
            if (typeof fgRef.current.zoom === 'function') {
                fgRef.current.zoom(0.1, 400);
            }
        }
    }, []);

    const exportPNG = useCallback(() => {
        try {
            // react-force-graph-2d renders a .force-graph-container div with a <canvas> child.
            // The library does NOT expose a canvasElement() method on its ref, so we
            // query the DOM directly to find the rendered canvas.
            const container = document.querySelector('.force-graph-container');
            const canvas = container?.querySelector('canvas') as HTMLCanvasElement | null;
            if (!canvas) {
                console.error('[useNodeSelection] Canvas element not found in .force-graph-container');
                return;
            }

            // Size guard: limit to 16M pixels (e.g. 4096 x 4096) to prevent OOM
            if (canvas.width * canvas.height > 16 * 1024 * 1024) {
                console.warn('[useNodeSelection] Canvas too large for export');
                return;
            }

            // Create offscreen canvas to composite dark background behind transparent graph nodes
            const offscreen = document.createElement('canvas');
            offscreen.width = canvas.width;
            offscreen.height = canvas.height;
            const ctx = offscreen.getContext('2d');

            if (ctx) {
                ctx.fillStyle = '#090d16'; // Tadpole OS Obsidian theme background
                ctx.fillRect(0, 0, offscreen.width, offscreen.height);
                ctx.drawImage(canvas, 0, 0);
            }

            // Synchronous toDataURL ensures link.click() runs inside user event gesture tick
            const dataUrl = (ctx ? offscreen : canvas).toDataURL('image/png');
            if (!dataUrl || dataUrl === 'data:,') {
                console.error('[useNodeSelection] Failed to generate canvas data URL');
                return;
            }

            const link = document.createElement('a');
            link.download = `tadpole_knowledge_graph_${new Date().toISOString().slice(0, 10)}.png`;
            link.href = dataUrl;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
        } catch (err) {
            console.error('[useNodeSelection] Export PNG failed:', err);
        }
    }, []);

    return {
        fgRef,
        selectNode,
        historyState,
        goBack,
        goForward,
        zoomIn,
        zoomOut,
        zoomFit,
        exportPNG,
        isPathFinderOpen,
        setIsPathFinderOpen,
        isAnomalyPanelOpen,
        setIsAnomalyPanelOpen,
        activeInfoTab,
        setActiveInfoTab,
    };
}

// Metadata: [useNodeSelection]
