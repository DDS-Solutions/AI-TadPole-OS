/**
 * @docs ARCHITECTURE:Intelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / graph_context_defs
 * - **Primary Entrypoints**: `GraphDataContext`, `SelectionContext`, `NavigationContext`, `ViewportContext`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { createContext } from 'react';
import type React from 'react';
import type { SanitizedGraphResult } from './graph_sanitizer';
import type { ExtendedGraphNode, ForceGraphLink, GraphMethods } from './types';

// ==========================================
// 1. Context Interface Definitions
// ==========================================

export interface GraphDataContextType {
    data: SanitizedGraphResult['data'] | null;
    loading: boolean;
    error: string | null;
    graphData: { nodes: ExtendedGraphNode[]; links: ForceGraphLink[]; nodeMap: Map<string, ExtendedGraphNode> };
}

export interface SelectionContextType {
    selectedNode: ExtendedGraphNode | null;
    hoverNode: ExtendedGraphNode | null;
    affectedNodes: Set<string>;
    blastRadiusLoading: boolean;
    blastRadiusError: string | null;
    setHoverNode: (node: ExtendedGraphNode | null) => void;
    selectNode: (node: ExtendedGraphNode, pushHistory?: boolean) => Promise<void>;
    resetSelection: () => void;
}

export interface NavigationContextType {
    nodeHistory: ExtendedGraphNode[];
    historyIndex: number;
    goBack: () => void;
    goForward: () => void;
}

export interface ViewportContextType {
    zoomIn: () => void;
    zoomOut: () => void;
    zoomFit: () => void;
    exportPNG: () => void;
    fgRef: React.MutableRefObject<GraphMethods | undefined>;
}

export interface UIStateContextType {
    isPathFinderOpen: boolean;
    setIsPathFinderOpen: (open: boolean) => void;
    isAnomalyPanelOpen: boolean;
    setIsAnomalyPanelOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
    highlightedPathNodeIds: string[] | null;
    setHighlightedPathNodeIds: (pathNodeIds: string[] | null) => void;
    activeInfoTab: 'info' | 'memory';
    setActiveInfoTab: (tab: 'info' | 'memory') => void;
    isMemoryNode: boolean;
    // --- OKF Extensions ---
    viewMode: 'symbols' | 'okf';
    setViewMode: (mode: 'symbols' | 'okf') => void;
}

// ==========================================
// 2. React Context Instances
// ==========================================

export const GraphDataContext = createContext<GraphDataContextType | undefined>(undefined);
export const SelectionContext = createContext<SelectionContextType | undefined>(undefined);
export const NavigationContext = createContext<NavigationContextType | undefined>(undefined);
export const ViewportContext = createContext<ViewportContextType | undefined>(undefined);
export const UIStateContext = createContext<UIStateContextType | undefined>(undefined);
