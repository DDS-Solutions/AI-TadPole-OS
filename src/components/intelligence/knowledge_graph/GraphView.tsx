/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[GraphView]` in observability traces.
 */

import React, { useMemo } from 'react';
import ForceGraph2D from 'react-force-graph-2d';
import { THEME_COLORS, GRAPH_THEME } from '../../../constants/theme';
import type { ExtendedGraphNode, ForceGraphLink, GraphLinkObject, GraphNodeObject } from './types';
import { 
    useGraphDataContext, 
    useSelectionContext, 
    useViewportContext 
} from './graph_context_hooks';
import {
    COMMUNITY_COLORS,
    get_link_source_id,
    get_link_target_id,
    get_node_community,
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
        drawNode(node, ctx, global_scale, selectedNodeId, hoverNodeId);
    }, [selectedNodeId, hoverNodeId]);

    // Memoize link styling accessors to prevent inline recreation and react-force-graph refresh thrashing
    const linkColor = useMemo(() => (link: GraphLinkObject) => {
        if (link.is_path_highlighted) {
            return '#eab308'; // Amber for pathfinder
        }
        const source_id = get_link_source_id(link);
        const target_id = get_link_target_id(link);
        const is_source_affected = affectedNodes.has(source_id);
        const is_target_affected = affectedNodes.has(target_id);
        if (is_source_affected && is_target_affected) {
            return THEME_COLORS.ERROR;
        }

        const source_community = get_node_community(link.source, nodeMap);
        const target_community = get_node_community(link.target, nodeMap);
        if (source_community !== undefined && target_community !== undefined && source_community === target_community) {
            return COMMUNITY_COLORS[source_community % COMMUNITY_COLORS.length];
        }

        return THEME_COLORS.NEURAL_GRID;
    }, [nodeMap, affectedNodes]);

    const linkWidth = useMemo(() => (link: GraphLinkObject) => {
        if (link.is_path_highlighted) {
            return 2.5; // Thicker link on active path
        }
        const source_id = get_link_source_id(link);
        const target_id = get_link_target_id(link);
        const is_source_affected = affectedNodes.has(source_id);
        const is_target_affected = affectedNodes.has(target_id);
        if (is_source_affected && is_target_affected) {
            return 2;
        }

        const source_community = get_node_community(link.source, nodeMap);
        const target_community = get_node_community(link.target, nodeMap);
        if (source_community !== undefined && target_community !== undefined && source_community === target_community) {
            return 1.5;
        }

        return 0.75;
    }, [nodeMap, affectedNodes]);

    const linkDirectionalParticles = useMemo(() => (link: GraphLinkObject) => {
        if (link.is_path_highlighted) return 4;
        const source_id = get_link_source_id(link);
        const target_id = get_link_target_id(link);
        const is_source_affected = affectedNodes.has(source_id);
        const is_target_affected = affectedNodes.has(target_id);
        if (is_source_affected && is_target_affected) return 3;

        const source_community = get_node_community(link.source, nodeMap);
        const target_community = get_node_community(link.target, nodeMap);
        if (source_community !== undefined && target_community !== undefined && source_community === target_community) {
            return 2;
        }
        return 0; // Prevent noise by only flowing particles intra-community or on highlighted paths
    }, [nodeMap, affectedNodes]);

    const linkDirectionalParticleSpeed = useMemo(() => (link: GraphLinkObject) => {
        return link.is_path_highlighted ? GRAPH_THEME.PARTICLE_SPEED * 2 : GRAPH_THEME.PARTICLE_SPEED;
    }, []);

    const linkDirectionalParticleColor = useMemo(() => (link: GraphLinkObject) => {
        if (link.is_path_highlighted) {
            return '#eab308';
        }
        const source_id = get_link_source_id(link);
        const target_id = get_link_target_id(link);
        const is_source_affected = affectedNodes.has(source_id);
        const is_target_affected = affectedNodes.has(target_id);
        if (is_source_affected && is_target_affected) {
            return THEME_COLORS.ERROR;
        }

        const source_community = get_node_community(link.source, nodeMap);
        const target_community = get_node_community(link.target, nodeMap);
        if (source_community !== undefined && target_community !== undefined && source_community === target_community) {
            return COMMUNITY_COLORS[source_community % COMMUNITY_COLORS.length];
        }
        return '#38bdf8';
    }, [nodeMap, affectedNodes]);

    const linkDirectionalParticleWidth = useMemo(() => (link: GraphLinkObject) => {
        if (link.is_path_highlighted) {
            return 3.5; // Glowing flowing particle
        }
        const source_id = get_link_source_id(link);
        const target_id = get_link_target_id(link);
        const is_source_affected = affectedNodes.has(source_id);
        const is_target_affected = affectedNodes.has(target_id);
        if (is_source_affected && is_target_affected) {
            return 3;
        }

        const source_community = get_node_community(link.source, nodeMap);
        const target_community = get_node_community(link.target, nodeMap);
        if (source_community !== undefined && target_community !== undefined && source_community === target_community) {
            return 2;
        }
        return 0;
    }, [nodeMap, affectedNodes]);

    const nodePointerAreaPaint = useMemo(() => (node: GraphNodeObject, color: string, ctx: CanvasRenderingContext2D) => {
        ctx.fillStyle = color;
        ctx.beginPath(); 
        ctx.arc(node.x ?? 0, node.y ?? 0, 8, 0, 2 * Math.PI, false); 
        ctx.fill();
    }, []);

    return (
        <ForceGraph2D<ExtendedGraphNode, ForceGraphLink>
            ref={fgRef}
            graphData={optimizedData}
            nodeCanvasObject={node_canvas_object}
            nodePointerAreaPaint={nodePointerAreaPaint}
            linkColor={linkColor}
            linkWidth={linkWidth}
            linkDirectionalParticles={linkDirectionalParticles}
            linkDirectionalParticleSpeed={linkDirectionalParticleSpeed}
            linkDirectionalParticleColor={linkDirectionalParticleColor}
            linkDirectionalParticleWidth={linkDirectionalParticleWidth}
            backgroundColor="rgba(0,0,0,0)"
            d3AlphaDecay={0.01}
            d3VelocityDecay={0.3}
            onNodeClick={(node) => selectNode(node)}
            onNodeHover={(node) => setHoverNode(node)}
            minZoom={0.1}
            maxZoom={10}
        />
    );
};

// Metadata: [GraphView]

