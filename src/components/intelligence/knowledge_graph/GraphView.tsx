/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * High-performance 2D Force-Directed Graph renderer for Tadpole OS.
 * Features memoized styling accessors, canonical link classification, theme tokens,
 * and debounced auto-centering viewport bounds.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[GraphView]` in observability traces.
 */

import React, { useMemo, useEffect, useRef, useCallback, useState } from 'react';
import ForceGraph2D from 'react-force-graph-2d';
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const ForceGraph2DComponent = ForceGraph2D as unknown as React.ComponentType<any>;

import { THEME_COLORS, GRAPH_THEME } from '../../../constants/theme';
import type { ExtendedGraphNode, GraphLinkObject, GraphNodeObject } from './types';
import {
    useGraphDataContext,
    useSelectionContext,
    useViewportContext,
    useUIStateContext
} from './graph_context_hooks';
import {
    COMMUNITY_COLORS,
    get_link_source_id,
    get_link_target_id,
    get_node_community,
    get_link_status,
    drawNode
} from './utils/graph_render_config';

export const GraphView: React.FC = () => {
    const { graphData } = useGraphDataContext();
    const {
        selectedNode,
        hoverNode,
        setHoverNode,
        affectedNodes,
        selectNode
    } = useSelectionContext();
    const { fgRef } = useViewportContext();
    const { viewMode } = useUIStateContext();

    // --- Container dimension tracking ---
    // ForceGraph2D defaults to window size if width/height are not provided.
    // We measure the actual parent container and pass explicit dimensions
    // so the canvas is constrained to the card, not the browser window.
    const containerRef = useRef<HTMLDivElement>(null);
    const [dimensions, setDimensions] = useState<{ width: number; height: number }>({ width: 0, height: 0 });

    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;

        const observer = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                setDimensions((prev) => {
                    if (prev.width === Math.round(width) && prev.height === Math.round(height)) return prev;
                    return { width: Math.round(width), height: Math.round(height) };
                });
            }
        });
        observer.observe(el);

        // Initial measurement
        const rect = el.getBoundingClientRect();
        setDimensions({ width: Math.round(rect.width), height: Math.round(rect.height) });

        return () => observer.disconnect();
    }, []);

    const optimizedData = useMemo(() => ({
        nodes: graphData.nodes,
        links: graphData.links
    }), [graphData.nodes, graphData.links]);

    // Build a map of node IDs for fast O(1) community lookups on links
    const nodeMap = useMemo(() => {
        const map = new Map<string, ExtendedGraphNode>();
        graphData.nodes.forEach(node => {
            map.set(node.id, node);
        });
        return map;
    }, [graphData.nodes]);

    // Optimize canvas drawing object by only depending on primitive node IDs to prevent unnecessary memo updates
    const selectedNodeId = selectedNode?.id || null;
    const hoverNodeId = hoverNode?.id || null;

    const node_canvas_object = useMemo(() => (node: GraphNodeObject, ctx: CanvasRenderingContext2D, global_scale: number) => {
        drawNode(node, ctx, global_scale, selectedNodeId, hoverNodeId, viewMode);
    }, [selectedNodeId, hoverNodeId, viewMode]);

    // Memoize link styling accessors using canonical get_link_status classification
    const linkColor = useMemo(() => (link: GraphLinkObject) => {
        const status = get_link_status(link, nodeMap, affectedNodes);
        if (status === 'PATH') return THEME_COLORS.AMBER;
        if (status === 'IMPLICIT') return THEME_COLORS.AMBER_GLOW;
        if (viewMode === 'okf') return 'rgba(113, 113, 122, 0.35)';
        if (status === 'AFFECTED') return THEME_COLORS.ERROR;
        if (status === 'COMMUNITY') {
            const comm = get_node_community(link.source, nodeMap);
            return comm !== undefined ? COMMUNITY_COLORS[comm % COMMUNITY_COLORS.length] : THEME_COLORS.NEURAL_GRID;
        }
        return THEME_COLORS.NEURAL_GRID;
    }, [nodeMap, affectedNodes, viewMode]);

    const linkWidth = useMemo(() => (link: GraphLinkObject) => {
        const status = get_link_status(link, nodeMap, affectedNodes);
        if (status === 'PATH') return 2.5;
        if (status === 'IMPLICIT') return 1;
        if (status === 'AFFECTED') return 2;
        if (status === 'COMMUNITY') return 1.5;
        return 0.75;
    }, [nodeMap, affectedNodes]);

    const linkDirectionalParticles = useMemo(() => (link: GraphLinkObject) => {
        const status = get_link_status(link, nodeMap, affectedNodes);
        if (status === 'PATH') return 4;
        if (status === 'AFFECTED') return 3;
        if (status === 'COMMUNITY') return 2;
        return 0;
    }, [nodeMap, affectedNodes]);

    const linkDirectionalParticleSpeed = useMemo(() => (link: GraphLinkObject) => {
        return link.is_path_highlighted ? GRAPH_THEME.PARTICLE_SPEED * 2 : GRAPH_THEME.PARTICLE_SPEED;
    }, []);

    const linkDirectionalParticleColor = useMemo(() => (link: GraphLinkObject) => {
        const status = get_link_status(link, nodeMap, affectedNodes);
        if (status === 'PATH') return THEME_COLORS.AMBER;
        if (status === 'AFFECTED') return THEME_COLORS.ERROR;
        if (status === 'COMMUNITY') {
            const comm = get_node_community(link.source, nodeMap);
            return comm !== undefined ? COMMUNITY_COLORS[comm % COMMUNITY_COLORS.length] : THEME_COLORS.SKY;
        }
        return THEME_COLORS.SKY;
    }, [nodeMap, affectedNodes]);

    const linkDirectionalParticleWidth = useMemo(() => (link: GraphLinkObject) => {
        const status = get_link_status(link, nodeMap, affectedNodes);
        if (status === 'PATH') return 3.5;
        if (status === 'AFFECTED') return 3;
        if (status === 'COMMUNITY') return 2;
        return 0;
    }, [nodeMap, affectedNodes]);

    const linkDirectionalArrowLength = useMemo(() => (link: GraphLinkObject) => {
        const source_id = get_link_source_id(link);
        const target_id = get_link_target_id(link);

        const is_connected =
            (selectedNodeId !== null && (source_id === selectedNodeId || target_id === selectedNodeId)) ||
            (hoverNodeId !== null && (source_id === hoverNodeId || target_id === hoverNodeId)) ||
            link.is_path_highlighted;

        if (is_connected) {
            return link.is_path_highlighted ? 5 : 3.5;
        }
        return 0;
    }, [selectedNodeId, hoverNodeId]);

    const linkDirectionalArrowColor = useMemo(() => (link: GraphLinkObject) => {
        return linkColor(link);
    }, [linkColor]);

    const nodePointerAreaPaint = useMemo(() => (node: GraphNodeObject, color: string, ctx: CanvasRenderingContext2D) => {
        ctx.fillStyle = color;
        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, 8, 0, 2 * Math.PI, false);
        ctx.fill();
    }, []);

    // Single-execution auto-centering guard to prevent camera jitter between useEffect and onEngineStop
    const initialCenteredRef = useRef<Record<string, boolean>>({});

    useEffect(() => {
        initialCenteredRef.current = {};
    }, [graphData.nodes.length]);

    const handleAutoCenter = useCallback(() => {
        const modeKey = viewMode;
        if (initialCenteredRef.current[modeKey]) return;
        if (fgRef.current) {
            initialCenteredRef.current[modeKey] = true;

            // Calculate actual center of mass of nodes to center graph in card container
            const nodes = graphData.nodes;
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

            // Set max zoomed out level (0.1)
            if (typeof fgRef.current.zoom === 'function') {
                fgRef.current.zoom(0.1, 400);
            }
        }
    }, [viewMode, graphData.nodes, fgRef]);

    useEffect(() => {
        if (!graphData.nodes || graphData.nodes.length === 0) return;
        const timer = setTimeout(handleAutoCenter, 150);
        return () => clearTimeout(timer);
    }, [viewMode, graphData.nodes, handleAutoCenter]);

    return (
        <div ref={containerRef} style={{ width: '100%', height: '100%', position: 'absolute', inset: 0 }}>
            {dimensions.width > 0 && dimensions.height > 0 && (
                <ForceGraph2DComponent
                    ref={fgRef}
                    width={dimensions.width}
                    height={dimensions.height}
                    graphData={optimizedData}
                    nodeCanvasObject={node_canvas_object}
                    nodePointerAreaPaint={nodePointerAreaPaint}
                    linkColor={linkColor}
                    linkWidth={linkWidth}
                    linkDirectionalParticles={linkDirectionalParticles}
                    linkDirectionalParticleSpeed={linkDirectionalParticleSpeed}
                    linkDirectionalParticleColor={linkDirectionalParticleColor}
                    linkDirectionalParticleWidth={linkDirectionalParticleWidth}
                    linkDirectionalArrowLength={linkDirectionalArrowLength}
                    linkDirectionalArrowColor={linkDirectionalArrowColor}
                    linkDirectionalArrowRelPos={1}
                    linkDashArray={(link: GraphLinkObject) => link.is_implicit ? [3, 3] : null}
                    backgroundColor="rgba(0,0,0,0)"
                    d3AlphaDecay={0.02}
                    d3VelocityDecay={0.3}
                    onEngineStop={handleAutoCenter}
                    onNodeClick={(node: GraphNodeObject) => selectNode(node)}
                    onNodeHover={(node: GraphNodeObject | null) => setHoverNode(node)}
                    minZoom={0.1}
                    maxZoom={10}
                />
            )}
        </div>
    );
};

// Metadata: [GraphView]
