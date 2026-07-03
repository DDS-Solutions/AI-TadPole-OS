/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[types]` in observability traces.
 */

import type { ForceGraphMethods, LinkObject, NodeObject } from 'react-force-graph-2d';
import type { SymbolNode } from '../../../contracts/generated';

export interface ExtendedGraphNode extends SymbolNode {
    id: string;
    is_affected: boolean;
    is_path_highlighted?: boolean;
    community?: number;
    x?: number;
    y?: number;
    vx?: number;
    vy?: number;
    fx?: number;
    fy?: number;

    // --- OKF Extensions ---
    concept_type?: string;
    title?: string;
    description?: string;
    resource_uri?: string;
    tags?: string;
    confidence?: number;
    human_confirmed?: boolean;
    text?: string;
}

export interface ForceGraphLink {
    source: string | ExtendedGraphNode; 
    target: string | ExtendedGraphNode;
    is_path_highlighted?: boolean;
    is_implicit?: boolean;
}

export type GraphNodeObject = NodeObject<ExtendedGraphNode>;
export type GraphLinkObject = LinkObject<ExtendedGraphNode, ForceGraphLink>;
export interface GraphMethods extends ForceGraphMethods<GraphNodeObject, GraphLinkObject> {
    canvasElement(): HTMLCanvasElement | null;
}

// Metadata: [types]
