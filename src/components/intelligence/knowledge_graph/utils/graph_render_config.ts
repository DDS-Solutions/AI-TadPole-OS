/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Returns a user-friendly display name for a symbol kind.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[graph_render_config]` in observability traces.
 */

import { THEME_COLORS, GRAPH_THEME } from '../../../../constants/theme';
import type { ExtendedGraphNode, ForceGraphLink, GraphLinkObject, GraphNodeObject } from '../types';

// Sleek Tailwind CSS HSL tailored color palette for communities
export const COMMUNITY_COLORS = [
    '#38bdf8', // Sky
    '#34d399', // Emerald
    '#f472b6', // Pink
    '#a78bfa', // Purple
    '#fbbf24', // Amber
    '#f87171', // Red
    '#60a5fa', // Blue
    '#fb923c'  // Orange
];

export const get_link_source_id = (link: ForceGraphLink | GraphLinkObject): string => {
    if (typeof link.source === 'string') return link.source;
    return link.source?.id?.toString() || '';
};

export const get_link_target_id = (link: ForceGraphLink | GraphLinkObject): string => {
    if (typeof link.target === 'string') return link.target;
    return link.target?.id?.toString() || '';
};

export const get_node_community = (
    nodeOrId: string | ExtendedGraphNode,
    nodeMap: Map<string, ExtendedGraphNode>
): number | undefined => {
    if (typeof nodeOrId === 'string') {
        return nodeMap.get(nodeOrId)?.community;
    }
    return nodeOrId?.community;
};

/**
 * Returns a user-friendly display name for a symbol kind.
 */
export const get_kind_display = (kind: string): string => {
    switch ((kind || '').toLowerCase()) {
        case 'func': return 'Function';
        case 'struct': return 'Struct';
        case 'class': return 'Class';
        case 'trait': return 'Trait';
        case 'interface': return 'Interface';
        case 'enum': return 'Enum';
        case 'impl': return 'Implementation';
        case 'type': return 'Type';
        case 'method': return 'Method';
        default: return (kind || '').charAt(0).toUpperCase() + (kind || '').slice(1);
    }
};

/**
 * Pure Canvas-based drawer for nodes.
 * Used inside ForceGraph2D's nodeCanvasObject.
 * 
 * @param node The node object to draw on the canvas
 * @param ctx The Canvas 2D rendering context
 * @param global_scale The current viewport scale factor. Labels are only rendered when global_scale > 1.2, or if the node is selected, affected, or on a highlighted path to keep the viewport clean.
 * @param selectedNodeId The primitive ID string of the currently selected node, or null
 * @param hoverNodeId The primitive ID string of the currently hovered node, or null
 */
export const drawNode = (
    node: GraphNodeObject,
    ctx: CanvasRenderingContext2D,
    global_scale: number,
    selectedNodeId: string | null,
    hoverNodeId: string | null
): void => {
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
    if (normalized_kind === 'enum') kind_color = '#06b6d4'; // Teal

    // Override if affected
    if (node.is_affected) kind_color = THEME_COLORS.ERROR;

    // 1. Glow Halo
    if (node.is_affected || (selectedNodeId && selectedNodeId === node.id) || node.is_path_highlighted) {
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

    // 3. Highlight Border / Community Border
    if ((hoverNodeId && hoverNodeId === node.id) || (selectedNodeId && selectedNodeId === node.id) || node.is_path_highlighted) {
        ctx.strokeStyle = node.is_path_highlighted ? '#fbbf24' : 'white';
        ctx.lineWidth = (node.is_path_highlighted ? 1.5 : 1) / global_scale;
        ctx.stroke();
    } else if (node.community !== undefined) {
        ctx.strokeStyle = COMMUNITY_COLORS[node.community % COMMUNITY_COLORS.length];
        ctx.lineWidth = 1.5 / global_scale;
        ctx.stroke();
    }

    // 4. Label (Zoom Dependent)
    if (global_scale > 1.2 || (selectedNodeId && selectedNodeId === node.id) || node.is_affected || node.is_path_highlighted) {
        ctx.font = `${font_size}px ${GRAPH_THEME.LABEL_FONT}`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';
        ctx.fillStyle = node.is_affected 
            ? '#fda4af' 
            : (node.is_path_highlighted ? '#fef08a' : 'white');
        ctx.fillText(label, x, y + radius + 2);
    }

    ctx.restore();
};

// Metadata: [graph_render_config]
