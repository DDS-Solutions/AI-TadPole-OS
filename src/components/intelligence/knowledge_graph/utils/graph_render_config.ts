/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * Pure Canvas rendering utility functions for Tadpole OS Knowledge Graph.
 * Handles modular node rendering (OKF vs Symbols), isolated icon sub-drawers with canvas state purity,
 * and canonical link classification.
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

export type LinkStatus = 'PATH' | 'IMPLICIT' | 'AFFECTED' | 'COMMUNITY' | 'DEFAULT';

/**
 * Returns canonical classification for graph links to eliminate logic duplication across styling accessors.
 */
export const get_link_status = (
    link: GraphLinkObject,
    nodeMap: Map<string, ExtendedGraphNode>,
    affectedNodes: Set<string>
): LinkStatus => {
    if (link.is_path_highlighted) return 'PATH';
    if (link.is_implicit) return 'IMPLICIT';
    const source_id = get_link_source_id(link);
    const target_id = get_link_target_id(link);
    if (affectedNodes.has(source_id) && affectedNodes.has(target_id)) return 'AFFECTED';
    const source_community = get_node_community(link.source, nodeMap);
    const target_community = get_node_community(link.target, nodeMap);
    if (source_community !== undefined && target_community !== undefined && source_community === target_community) {
        return 'COMMUNITY';
    }
    return 'DEFAULT';
};

/**
 * Returns a user-friendly display name for a symbol kind.
 */
export const get_kind_display = (kind: string): string => {
    const k = (kind || '').toLowerCase();
    switch (k) {
        case 'func': return 'Function';
        case 'struct': return 'Struct';
        case 'class': return 'Class';
        case 'trait': return 'Trait';
        case 'interface': return 'Interface';
        case 'enum': return 'Enum';
        case 'impl': return 'Implementation';
        case 'type': return 'Type';
        case 'method': return 'Method';
        default: return k ? k.charAt(0).toUpperCase() + k.slice(1) : '';
    }
};

/**
 * Pure Canvas icon drawers with internal save/restore state protection.
 */
const setupIconContext = (ctx: CanvasRenderingContext2D) => {
    ctx.save();
    ctx.strokeStyle = THEME_COLORS.SECONDARY;
    ctx.lineWidth = 1.2;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
};

const drawDatabaseIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    setupIconContext(ctx);
    const w = size;
    const h = size * 0.8;
    const rx = w / 2;
    const ry = h / 4;
    
    ctx.beginPath();
    ctx.ellipse(x, y - h/3, rx, ry, 0, 0, 2 * Math.PI);
    ctx.stroke();
    
    ctx.beginPath();
    ctx.ellipse(x, y, rx, ry, 0, 0, Math.PI);
    ctx.moveTo(x - rx, y - h/3);
    ctx.lineTo(x - rx, y);
    ctx.moveTo(x + rx, y - h/3);
    ctx.lineTo(x + rx, y);
    ctx.stroke();

    ctx.beginPath();
    ctx.ellipse(x, y + h/3, rx, ry, 0, 0, Math.PI);
    ctx.moveTo(x - rx, y);
    ctx.lineTo(x - rx, y + h/3);
    ctx.moveTo(x + rx, y);
    ctx.lineTo(x + rx, y + h/3);
    ctx.stroke();
    ctx.restore();
};

const drawGlobeIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    setupIconContext(ctx);
    const r = size / 2;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.stroke();
    
    ctx.beginPath();
    ctx.moveTo(x - r, y);
    ctx.lineTo(x + r, y);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(x, y - r);
    ctx.lineTo(x, y + r);
    ctx.stroke();

    ctx.beginPath();
    ctx.ellipse(x, y, r / 2, r, 0, 0, 2 * Math.PI);
    ctx.stroke();
    ctx.restore();
};

const drawFileTextIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    setupIconContext(ctx);
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

    ctx.beginPath();
    ctx.moveTo(right - fold, top);
    ctx.lineTo(right - fold, top + fold);
    ctx.lineTo(right, top + fold);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(left + w * 0.2, y);
    ctx.lineTo(right - w * 0.2, y);
    ctx.moveTo(left + w * 0.2, y + h * 0.2);
    ctx.lineTo(right - w * 0.2, y + h * 0.2);
    ctx.stroke();
    ctx.restore();
};

const drawActivityIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    setupIconContext(ctx);
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
    ctx.restore();
};

const drawBookOpenIcon = (ctx: CanvasRenderingContext2D, x: number, y: number, size: number) => {
    setupIconContext(ctx);
    const w = size * 0.9;
    const h = size * 0.6;
    const midX = x;
    const left = x - w/2;
    const right = x + w/2;
    const top = y - h/2;
    const bottom = y + h/2;

    ctx.beginPath();
    ctx.moveTo(midX, bottom);
    ctx.quadraticCurveTo(midX - w*0.25, top + h*0.2, left, top);
    ctx.lineTo(left, bottom - h*0.2);
    ctx.quadraticCurveTo(midX - w*0.25, bottom, midX, bottom - h*0.1);

    ctx.quadraticCurveTo(midX + w*0.25, bottom, right, bottom - h*0.2);
    ctx.lineTo(right, top);
    ctx.quadraticCurveTo(midX + w*0.25, top + h*0.2, midX, bottom);
    ctx.stroke();
    
    ctx.beginPath();
    ctx.moveTo(midX, bottom);
    ctx.lineTo(midX, top + h*0.2);
    ctx.stroke();
    ctx.restore();
};

/**
 * Dedicated drawer for OKF (Ontology / Knowledge Framework) nodes.
 */
const drawNodeOKF = (
    node: GraphNodeObject,
    ctx: CanvasRenderingContext2D,
    global_scale: number,
    selectedNodeId: string | null,
    hoverNodeId: string | null
): void => {
    const label = node.name;
    const x = node.x ?? 0;
    const y = node.y ?? 0;
    const radius = 10;
    
    const isSelected = !!selectedNodeId && selectedNodeId === node.id;
    const isHovered = !!hoverNodeId && hoverNodeId === node.id;
    
    const isConfirmed = node.human_confirmed === true;
    const tagsStr = Array.isArray(node.tags)
        ? (node.tags as string[]).join(' ').toLowerCase()
        : (typeof node.tags === 'string' ? node.tags.toLowerCase() : '');

    const isBroken = node.is_broken ?? (
        (node.resource_uri && node.resource_uri.toLowerCase().startsWith('broken')) || 
        tagsStr.includes('broken')
    );
    const isExpiring = node.is_expiring ?? (
        (node.ttl !== null && node.ttl !== undefined && !isConfirmed) || 
        tagsStr.includes('expiring')
    );

    // 1. Glow Halo / Pulsing borders for Selected Node
    if (isSelected || isHovered) {
        ctx.beginPath();
        ctx.arc(x, y, radius * 1.5, 0, 2 * Math.PI, false);
        ctx.fillStyle = isSelected ? 'rgba(228, 228, 231, 0.25)' : 'rgba(228, 228, 231, 0.1)';
        ctx.fill();

        if (isSelected) {
            ctx.beginPath();
            ctx.arc(x, y, radius * 1.8, 0, 2 * Math.PI, false);
            ctx.strokeStyle = THEME_COLORS.FOCUS_RING;
            ctx.lineWidth = 1.5 / global_scale;
            ctx.stroke();
        }
    }

    // 2. Core Node Surface
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, 2 * Math.PI, false);
    ctx.fillStyle = THEME_COLORS.DARK_SURFACE;
    ctx.fill();

    // 3. Highlight Border / Status Border
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, 2 * Math.PI, false);
    if (isBroken) {
        ctx.strokeStyle = THEME_COLORS.CYBER_RED;
        ctx.lineWidth = 2 / global_scale;
    } else if (isSelected) {
        ctx.strokeStyle = THEME_COLORS.SECONDARY;
        ctx.lineWidth = 1.5 / global_scale;
    } else if (isHovered) {
        ctx.strokeStyle = THEME_COLORS.SECONDARY;
        ctx.lineWidth = 1.5 / global_scale;
    } else {
        ctx.strokeStyle = THEME_COLORS.NEURAL_GRID;
        ctx.lineWidth = 1.5 / global_scale;
    }
    ctx.stroke();

    // 4. Draw Type Indicator Icon
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

    // 5. Draw State Condition Dot/Badge Highlights
    if (isConfirmed) {
        ctx.beginPath();
        ctx.arc(x + radius * 0.7, y - radius * 0.7, 2.5, 0, 2 * Math.PI, false);
        ctx.fillStyle = THEME_COLORS.CYBER_GREEN;
        ctx.fill();
        ctx.strokeStyle = THEME_COLORS.DARK_SURFACE;
        ctx.lineWidth = 0.5;
        ctx.stroke();
    } else if (isExpiring) {
        ctx.beginPath();
        ctx.arc(x + radius * 0.7, y - radius * 0.7, 2.5, 0, 2 * Math.PI, false);
        ctx.fillStyle = THEME_COLORS.WARNING;
        ctx.fill();
        ctx.strokeStyle = THEME_COLORS.DARK_SURFACE;
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
            ? THEME_COLORS.ROSE_LIGHT 
            : (isExpiring ? THEME_COLORS.AMBER_LIGHT_TEXT : 'white');
        ctx.fillText(label, x, y + radius + 3);
    }
};

/**
 * Dedicated drawer for Codebase Symbols nodes.
 */
const drawNodeSymbols = (
    node: GraphNodeObject,
    ctx: CanvasRenderingContext2D,
    global_scale: number,
    selectedNodeId: string | null,
    hoverNodeId: string | null
): void => {
    const label = node.name;
    const x = node.x ?? 0;
    const y = node.y ?? 0;
    const radius = GRAPH_THEME.NODE_RADIUS * (node.is_affected ? 1.5 : 1);
    
    let kind_color = THEME_COLORS.IDLE;
    const normalized_kind = (node.kind || '').toLowerCase();
    if (normalized_kind === 'func' || normalized_kind === 'function' || normalized_kind === 'method') kind_color = THEME_COLORS.BUSY;
    if (normalized_kind === 'struct' || normalized_kind === 'class') kind_color = THEME_COLORS.SUCCESS;
    if (normalized_kind === 'trait' || normalized_kind === 'interface') kind_color = THEME_COLORS.DEGRADED;
    if (normalized_kind === 'enum') kind_color = THEME_COLORS.RUNNING;

    if (node.is_affected) kind_color = THEME_COLORS.ERROR;

    // 1. Glow Halo
    if (node.is_affected || (selectedNodeId && selectedNodeId === node.id) || node.is_path_highlighted) {
        ctx.beginPath();
        ctx.arc(x, y, radius * 1.8, 0, 2 * Math.PI, false);
        if (node.is_affected) {
            ctx.fillStyle = THEME_COLORS.GLOW_ROSE;
        } else if (node.is_path_highlighted) {
            ctx.fillStyle = THEME_COLORS.AMBER_GLOW;
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
        ctx.strokeStyle = node.is_path_highlighted ? THEME_COLORS.AMBER_LIGHT : 'white';
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
            ? THEME_COLORS.ROSE_LIGHT 
            : (node.is_path_highlighted ? THEME_COLORS.AMBER_LIGHT_TEXT : 'white');
        ctx.fillText(label, x, y + radius + 2);
    }
};

/**
 * Pure Canvas-based dispatcher drawer for nodes.
 * Used inside ForceGraph2D's nodeCanvasObject.
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
    if (viewMode === 'okf') {
        drawNodeOKF(node, ctx, global_scale, selectedNodeId, hoverNodeId);
    } else {
        drawNodeSymbols(node, ctx, global_scale, selectedNodeId, hoverNodeId);
    }
    ctx.restore();
};

// Metadata: [graph_render_config]
