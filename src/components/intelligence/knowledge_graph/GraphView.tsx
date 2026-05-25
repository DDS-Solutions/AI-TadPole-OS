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

/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **GraphView Component**: Renders the 2D Canvas force-directed graph.
 * Features customizable colors, interactive nodes, and flowing directional particles.
 */

import React, { useMemo, useEffect } from 'react';
import ForceGraph2D, { type ForceGraphMethods } from 'react-force-graph-2d';
import { THEME_COLORS, GRAPH_THEME } from '../../../constants/theme';
import type { ExtendedGraphNode, ForceGraphLink } from './types';

interface GraphViewProps {
    graph_data: { nodes: ExtendedGraphNode[]; links: ForceGraphLink[] };
    selected_node: ExtendedGraphNode | null;
    hover_node: ExtendedGraphNode | null;
    set_hover_node: (node: ExtendedGraphNode | null) => void;
    affected_nodes: Set<string>;
    on_node_click: (node: ExtendedGraphNode) => void;
    fg_ref: React.MutableRefObject<ForceGraphMethods<ExtendedGraphNode, ForceGraphLink> | undefined>;
}

export const GraphView: React.FC<GraphViewProps> = ({
    graph_data,
    selected_node,
    hover_node,
    set_hover_node,
    affected_nodes,
    on_node_click,
    fg_ref
}) => {

    const optimizedData = useMemo(() => ({
        nodes: graph_data.nodes,
        links: graph_data.links
    }), [graph_data.nodes, graph_data.links]);

    useEffect(() => {
        return () => {
            if (fg_ref.current) fg_ref.current = undefined;
        };
    }, [fg_ref]);

    const get_link_source_id = (link: ForceGraphLink): string => {
        if (typeof link.source === 'string') return link.source;
        return (link.source as any).id || '';
    };

    const get_link_target_id = (link: ForceGraphLink): string => {
        if (typeof link.target === 'string') return link.target;
        return (link.target as any).id || '';
    };

    // ### 🎨 High-Performance Rendering: Intelligence Aesthetic
    const node_canvas_object = useMemo(() => (node: ExtendedGraphNode, ctx: CanvasRenderingContext2D, global_scale: number) => {
        ctx.save();
        
        const label = node.name;
        const font_size = 10 / global_scale;
        const radius = GRAPH_THEME.NODE_RADIUS * (node.is_affected ? 1.5 : 1);
        
        const x = node.x ?? 0;
        const y = node.y ?? 0;

        // Color based on Symbol Kind
        let kind_color = THEME_COLORS.IDLE;
        const normalized_kind = (node.kind || '').toLowerCase();
        if (normalized_kind === 'func' || normalized_kind === 'function' || normalized_kind === 'method') kind_color = THEME_COLORS.BUSY;
        if (normalized_kind === 'struct' || normalized_kind === 'class') kind_color = THEME_COLORS.SUCCESS;
        if (normalized_kind === 'trait' || normalized_kind === 'interface') kind_color = THEME_COLORS.DEGRADED;
        if (normalized_kind === 'enum') kind_color = '#a855f7'; // Purple

        // Override if affected
        if (node.is_affected) kind_color = THEME_COLORS.ERROR;

        // 1. Glow Halo
        if (node.is_affected || (selected_node && selected_node.id === node.id) || node.is_path_highlighted) {
            ctx.beginPath();
            ctx.arc(x, y, radius * 1.8, 0, 2 * Math.PI, false);
            if (node.is_affected) {
                ctx.fillStyle = THEME_COLORS.GLOW_ROSE;
            } else if (node.is_path_highlighted) {
                ctx.fillStyle = 'rgba(234, 179, 8, 0.45)'; // Amber pathfinder glow
            } else {
                ctx.fillStyle = THEME_COLORS.GLOW_CYAN;
            }
            ctx.fill();
        }

        // 2. Core Node
        ctx.beginPath();
        ctx.arc(x, y, radius, 0, 2 * Math.PI, false);
        ctx.fillStyle = kind_color;
        ctx.fill();

        // 3. Highlight Border
        if ((hover_node && hover_node.id === node.id) || (selected_node && selected_node.id === node.id) || node.is_path_highlighted) {
            ctx.strokeStyle = node.is_path_highlighted ? '#fbbf24' : 'white';
            ctx.lineWidth = (node.is_path_highlighted ? 1.5 : 1) / global_scale;
            ctx.stroke();
        }

        // 4. Label (Zoom Dependent)
        if (global_scale > 1.2 || (selected_node && selected_node.id === node.id) || node.is_affected || node.is_path_highlighted) {
            ctx.font = `${font_size}px ${GRAPH_THEME.LABEL_FONT}`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'top';
            ctx.fillStyle = node.is_affected 
                ? '#fda4af' 
                : (node.is_path_highlighted ? '#fef08a' : 'white');
            ctx.fillText(label, x, y + radius + 2);
        }

        ctx.restore();
    }, [selected_node, hover_node]);

    return (
        <ForceGraph2D
            ref={fg_ref as any}
            graphData={optimizedData}
            nodeCanvasObject={node_canvas_object}
            nodePointerAreaPaint={(node: ExtendedGraphNode, color, ctx) => {
                ctx.fillStyle = color;
                ctx.beginPath(); 
                ctx.arc(node.x ?? 0, node.y ?? 0, 8, 0, 2 * Math.PI, false); 
                ctx.fill();
            }}
            linkColor={(link: ForceGraphLink) => {
                if (link.is_path_highlighted) {
                    return '#eab308'; // Amber for pathfinder
                }
                const source_id = get_link_source_id(link);
                const target_id = get_link_target_id(link);
                const is_source_affected = affected_nodes.has(source_id);
                const is_target_affected = affected_nodes.has(target_id);
                return is_source_affected && is_target_affected ? THEME_COLORS.ERROR : THEME_COLORS.NEURAL_GRID;
            }}
            linkWidth={(link: ForceGraphLink) => {
                if (link.is_path_highlighted) {
                    return 2.5; // Thicker link on active path
                }
                const source_id = get_link_source_id(link);
                return affected_nodes.has(source_id) ? 2 : 1;
            }}
            linkDirectionalParticles={2}
            linkDirectionalParticleSpeed={(link: ForceGraphLink) => {
                return link.is_path_highlighted ? GRAPH_THEME.PARTICLE_SPEED * 2 : GRAPH_THEME.PARTICLE_SPEED;
            }}
            linkDirectionalParticleWidth={(link: ForceGraphLink) => {
                if (link.is_path_highlighted) {
                    return 3.5; // Glowing flowing particle
                }
                const source_id = get_link_source_id(link);
                return affected_nodes.has(source_id) ? 3 : 0;
            }}
            backgroundColor="rgba(0,0,0,0)"
            d3AlphaDecay={0.01}
            d3VelocityDecay={0.3}
            onNodeClick={on_node_click}
            onNodeHover={(node: ExtendedGraphNode | null) => set_hover_node(node)}
        />
    );
};

// Metadata: [GraphView]
