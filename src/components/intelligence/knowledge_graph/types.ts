/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / types
 * - **Primary Entrypoints**: `ExtendedGraphNode`, `ForceGraphLink`, `GraphMethods`, `GraphNodeObject`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
    ttl?: number | null;
    text?: string;
    is_broken?: boolean;
    is_expiring?: boolean;
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
    // graphData() is a real Kapsule getter exposed at runtime but not in the library's .d.ts
    graphData(): { nodes: ExtendedGraphNode[]; links: ForceGraphLink[] } | undefined;
}
