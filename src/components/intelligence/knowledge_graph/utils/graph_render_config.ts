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
const drawDatabaseIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    const w = size;
    const h = size * 0.8;
    const rx = w / 2;
    const ry = h / 4;
    
    // Top face
    ctx.beginPath();
    ctx.ellipse(x, y - h/3, rx, ry, 0, 0, 2 * Math.PI);
    ctx.stroke();
    
    // Middle section
    ctx.beginPath();
    ctx.ellipse(x, y, rx, ry, 0, 0, Math.PI);
    ctx.moveTo(x - rx, y - h/3);
    ctx.lineTo(x - rx, y);
    ctx.moveTo(x + rx, y - h/3);
    ctx.lineTo(x + rx, y);
    ctx.stroke();

    // Bottom section
    ctx.beginPath();
    ctx.ellipse(x, y + h/3, rx, ry, 0, 0, Math.PI);
    ctx.moveTo(x - rx, y);
    ctx.lineTo(x - rx, y + h/3);
    ctx.moveTo(x + rx, y);
    ctx.lineTo(x + rx, y + h/3);
    ctx.stroke();
};

const drawGlobeIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    const r = size / 2;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.stroke();
    
    // Equator
    ctx.beginPath();
    ctx.moveTo(x - r, y);
    ctx.lineTo(x + r, y);
    ctx.stroke();

    // Vertical meridian
    ctx.beginPath();
    ctx.moveTo(x, y - r);
    ctx.lineTo(x, y + r);
    ctx.stroke();

    // Curved meridians
    ctx.beginPath();
    ctx.ellipse(x, y, r / 2, r, 0, 0, 2 * Math.PI);
    ctx.stroke();
};

const drawFileTextIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    const w = size * 0.7;
    const h = size * 0.9;
    const left = x - w/2;
    const right = x + w/2;
    const top = y - h/2;
    const bottom = y + h/2;
    const fold = size * 0.25;

    ctx.beginPath();
    ctx.moveTo(left, top);
    ctx.lineTo(right - fold, top);
    ctx.lineTo(right, top + fold);
    ctx.lineTo(right, bottom);
    ctx.lineTo(left, bottom);
    ctx.closePath();
    ctx.stroke();

    // Fold line
    ctx.beginPath();
    ctx.moveTo(right - fold, top);
    ctx.lineTo(right - fold, top + fold);
    ctx.lineTo(right, top + fold);
    ctx.stroke();

    // Lines of text
    ctx.beginPath();
    ctx.moveTo(left + w * 0.2, y);
    ctx.lineTo(right - w * 0.2, y);
    ctx.moveTo(left + w * 0.2, y + h * 0.2);
    ctx.lineTo(right - w * 0.2, y + h * 0.2);
    ctx.stroke();
};

const drawActivityIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    const w = size;
    const left = x - w/2;
    ctx.beginPath();
    ctx.moveTo(left, y);
    ctx.lineTo(left + w * 0.25, y);
    ctx.lineTo(left + w * 0.4, y - size * 0.35);
    ctx.lineTo(left + w * 0.55, y + size * 0.35);
    ctx.lineTo(left + w * 0.7, y - size * 0.15);
    ctx.lineTo(left + w * 0.8, y);
    ctx.lineTo(left + w, y);
    ctx.stroke();
};

const drawBookOpenIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    const w = size * 0.9;
    const h = size * 0.6;
    const midX = x;
    const left = x - w/2;
    const right = x + w/2;
    const top = y - h/2;
    const bottom = y + h/2;

    ctx.beginPath();
    // Left page top curve
    ctx.moveTo(midX, bottom);
    ctx.quadraticCurveTo(midX - w*0.25, top + h*0.2, left, top);
    ctx.lineTo(left, bottom - h*0.2);
    ctx.quadraticCurveTo(midX - w*0.25, bottom, midX, bottom - h*0.1);

    // Right page top curve
    ctx.quadraticCurveTo(midX + w*0.25, bottom, right, bottom - h*0.2);
    ctx.lineTo(right, top);
    ctx.quadraticCurveTo(midX + w*0.25, top + h*0.2, midX, bottom);
    ctx.stroke();
    
    // Center binding line
    ctx.beginPath();
    ctx.moveTo(midX, bottom);
    ctx.lineTo(midX, top + h*0.2);
    ctx.stroke();
};

/**
 * Pure Canvas-based drawer for nodes.
 * Used inside ForceGraph2D's nodeCanvasObject.
 * 
 * @param node The node object to draw on the canvas
 * @param ctx The Canvas 2D rendering context
 * @param global_scale The current viewport scale factor
 * @param selectedNodeId The primitive ID string of the currently selected node, or null
 * @param hoverNodeId The primitive ID string of the currently hovered node, or null
 * @param viewMode The visual view mode ('symbols' or 'okf')
 */
export const drawNode = (
    node: GraphNodeObject,
    ctx: CanvasRenderingContext2D,
    global_scale: number,
    selectedNodeId: string | null,
    hoverNodeId: string | null,
    viewMode: 'symbols' | 'okf' = 'symbols'
): void => {
    ctx.save();
    
    const label = node.name;
    const x = node.x ?? 0;
    const y = node.y ?? 0;

    if (viewMode === 'okf') {
        const radius = 10; // slightly larger for icons
        
        // 1. Glow Halo / Pulsing borders for Selected Node
        const isSelected = selectedNodeId && selectedNodeId === node.id;
        const isHovered = hoverNodeId && hoverNodeId === node.id;
        
        // Determine operational states
        const isConfirmed = node.human_confirmed === true;
        const isBroken = (node.resource_uri && node.resource_uri.toLowerCase().startsWith('broken')) || 
                         (node.tags && node.tags.toLowerCase().includes('broken'));
        // Expiring: has TTL and not confirmed
        const isExpiring = (node.ttl !== null && !isConfirmed) || 
                           (node.tags && node.tags.toLowerCase().includes('expiring'));

        if (isSelected || isHovered) {
            ctx.beginPath();
            ctx.arc(x, y, radius * 1.5, 0, 2 * Math.PI, false);
            // active / selected receives neural-pulse border (#e4e4e7) and focus-ring (#10b981) outline highlights
            ctx.fillStyle = isSelected ? 'rgba(228, 228, 231, 0.25)' : 'rgba(228, 228, 231, 0.1)';
            ctx.fill();

            if (isSelected) {
                // Draw focus-ring outer outline
                ctx.beginPath();
                ctx.arc(x, y, radius * 1.8, 0, 2 * Math.PI, false);
                ctx.strokeStyle = '#10b981'; // focus-ring
                ctx.lineWidth = 1.5 / global_scale;
                ctx.stroke();
            }
        }

        // 2. Core Node
        ctx.beginPath();
        ctx.arc(x, y, radius, 0, 2 * Math.PI, false);
        ctx.fillStyle = '#18181b'; // Zinc-900 surface
        ctx.fill();

        // 3. Highlight Border / Status Border
        // default border: monochromatic zinc border (rgba(39, 39, 42, 0.4))
        ctx.beginPath();
        ctx.arc(x, y, radius, 0, 2 * Math.PI, false);
        if (isBroken) {
            ctx.strokeStyle = '#ef4444'; // cyber-red
            ctx.lineWidth = 2 / global_scale;
        } else if (isSelected) {
            ctx.strokeStyle = '#e4e4e7'; // neural-pulse
            ctx.lineWidth = 1.5 / global_scale;
        } else if (isHovered) {
            ctx.strokeStyle = '#71717a'; // zinc-500
            ctx.lineWidth = 1.5 / global_scale;
        } else {
            ctx.strokeStyle = 'rgba(39, 39, 42, 0.4)'; // zinc-800 with transparency
            ctx.lineWidth = 1.5 / global_scale;
        }
        ctx.stroke();

        // 4. Draw Type Indicator Icon (thin 1.5px stroke Zinc-500 #71717a)
        ctx.save();
        ctx.strokeStyle = '#71717a';
        ctx.lineWidth = 1.2;
        ctx.lineCap = 'round';
        ctx.lineJoin = 'round';
        
        const conceptType = (node.concept_type || '').toLowerCase();
        const iconSize = 9;
        
        if (conceptType.includes('bigquery') || conceptType.includes('table') || conceptType.includes('dataset') || conceptType.includes('db') || conceptType.includes('database')) {
            drawDatabaseIcon(ctx, x, y, iconSize);
        } else if (conceptType.includes('api') || conceptType.includes('endpoint') || conceptType.includes('service') || conceptType.includes('route') || conceptType.includes('globe') || conceptType.includes('network')) {
            drawGlobeIcon(ctx, x, y, iconSize);
        } else if (conceptType.includes('playbook') || conceptType.includes('sop') || conceptType.includes('file') || conceptType.includes('document') || conceptType.includes('doc') || conceptType.includes('text')) {
            drawFileTextIcon(ctx, x, y, iconSize);
        } else if (conceptType.includes('metric') || conceptType.includes('kpi') || conceptType.includes('activity') || conceptType.includes('pulse') || conceptType.includes('telemetry')) {
            drawActivityIcon(ctx, x, y, iconSize);
        } else {
            drawBookOpenIcon(ctx, x, y, iconSize);
        }
        ctx.restore();

        // 5. Draw State Condition Dot/Badge Highlights
        if (isConfirmed) {
            // Verified/Confirmed State: cyber-green (#22c55e) dot highlight
            ctx.beginPath();
            ctx.arc(x + radius * 0.7, y - radius * 0.7, 2.5, 0, 2 * Math.PI, false);
            ctx.fillStyle = '#22c55e'; // cyber-green
            ctx.fill();
            ctx.strokeStyle = '#18181b';
            ctx.lineWidth = 0.5;
            ctx.stroke();
        } else if (isExpiring) {
            // Expiring / Pending Review: cyber-amber (#f59e0b) dot highlight
            ctx.beginPath();
            ctx.arc(x + radius * 0.7, y - radius * 0.7, 2.5, 0, 2 * Math.PI, false);
            ctx.fillStyle = '#f59e0b'; // cyber-amber
            ctx.fill();
            ctx.strokeStyle = '#18181b';
            ctx.lineWidth = 0.5;
            ctx.stroke();
        }

        // 6. Label (Zoom Dependent or active status)
        if (global_scale > 1.2 || isSelected || isHovered) {
            const font_size = 10 / global_scale;
            ctx.font = `${font_size}px ${GRAPH_THEME.LABEL_FONT}`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'top';
            ctx.fillStyle = isBroken 
                ? '#fda4af' // Rose-300
                : (isExpiring ? '#fef08a' : 'white');
            ctx.fillText(label, x, y + radius + 3);
        }
    } else {
        // Fallback to Codebase Symbols representation
        const radius = GRAPH_THEME.NODE_RADIUS * (node.is_affected ? 1.5 : 1);
        
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
            const font_size = 10 / global_scale;
            ctx.font = `${font_size}px ${GRAPH_THEME.LABEL_FONT}`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'top';
            ctx.fillStyle = node.is_affected 
                ? '#fda4af' 
                : (node.is_path_highlighted ? '#fef08a' : 'white');
            ctx.fillText(label, x, y + radius + 2);
        }
    }
    
    ctx.restore();
};

// Metadata: [graph_render_config]
