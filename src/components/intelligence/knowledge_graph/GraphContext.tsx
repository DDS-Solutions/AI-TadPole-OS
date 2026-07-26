/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * Orchestrates the graph provider tree by composing `useGraphData`
 * (data fetching + projection) and `useNodeSelection` (interaction +
 * navigation + viewport) into five granular React contexts.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[GraphContext]` in observability traces.
 */

import React, { useMemo } from 'react';
import {
    GraphDataContext,
    SelectionContext,
    NavigationContext,
    ViewportContext,
    UIStateContext,
} from './graph_context_defs';
import type {
    GraphDataContextType,
    SelectionContextType,
    NavigationContextType,
    ViewportContextType,
    UIStateContextType,
} from './graph_context_defs';

import { useGraphData } from './useGraphData';
import { useNodeSelection } from './useNodeSelection';

// Re-export types for backwards compatibility with external consumers
export type {
    GraphDataContextType,
    SelectionContextType,
    NavigationContextType,
    ViewportContextType,
    UIStateContextType,
};

// ==========================================
// Sub-provider components for Context Tree Separation
// ==========================================

export const GraphDataProvider: React.FC<{ children: React.ReactNode; value: GraphDataContextType }> = ({ children, value }) => {
    return <GraphDataContext.Provider value={value}>{children}</GraphDataContext.Provider>;
};

export const SelectionProvider: React.FC<{ children: React.ReactNode; value: SelectionContextType }> = ({ children, value }) => {
    return <SelectionContext.Provider value={value}>{children}</SelectionContext.Provider>;
};

export const NavigationProvider: React.FC<{ children: React.ReactNode; value: NavigationContextType }> = ({ children, value }) => {
    return <NavigationContext.Provider value={value}>{children}</NavigationContext.Provider>;
};

export const ViewportProvider: React.FC<{ children: React.ReactNode; value: ViewportContextType }> = ({ children, value }) => {
    return <ViewportContext.Provider value={value}>{children}</ViewportContext.Provider>;
};

export const UIStateProvider: React.FC<{ children: React.ReactNode; value: UIStateContextType }> = ({ children, value }) => {
    return <UIStateContext.Provider value={value}>{children}</UIStateContext.Provider>;
};

// ==========================================
// Master Provider Implementation
// ==========================================

export const GraphProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    // Hook 1: Data fetching, projection, and selection state
    const graphDataHook = useGraphData();

    // Hook 2: Interaction, navigation, and viewport controls
    const nodeSelectionHook = useNodeSelection({
        selectedNode: graphDataHook.selectedNode,
        setSelectedNode: graphDataHook.setSelectedNode,
        setAffectedNodesList: graphDataHook.setAffectedNodesList,
        setBlastRadiusLoading: graphDataHook.setBlastRadiusLoading,
        setBlastRadiusError: graphDataHook.setBlastRadiusError,
        viewMode: graphDataHook.viewMode,
    });

    // ==========================================
    // Memoized Provider Values
    // ==========================================

    const dataValue = useMemo<GraphDataContextType>(() => ({
        data: graphDataHook.data,
        loading: graphDataHook.loading,
        error: graphDataHook.error,
        graphData: graphDataHook.graphData,
    }), [graphDataHook.data, graphDataHook.loading, graphDataHook.error, graphDataHook.graphData]);

    const selectionValue = useMemo<SelectionContextType>(() => ({
        selectedNode: graphDataHook.selectedNode,
        hoverNode: graphDataHook.hoverNode,
        affectedNodes: graphDataHook.affectedNodes,
        blastRadiusLoading: graphDataHook.blastRadiusLoading,
        blastRadiusError: graphDataHook.blastRadiusError,
        setHoverNode: graphDataHook.setHoverNode,
        selectNode: nodeSelectionHook.selectNode,
        resetSelection: graphDataHook.resetSelection,
    }), [
        graphDataHook.selectedNode, graphDataHook.hoverNode, graphDataHook.affectedNodes,
        graphDataHook.blastRadiusLoading, graphDataHook.blastRadiusError, graphDataHook.setHoverNode,
        nodeSelectionHook.selectNode, graphDataHook.resetSelection,
    ]);

    const navigationValue = useMemo<NavigationContextType>(() => ({
        nodeHistory: nodeSelectionHook.historyState.history,
        historyIndex: nodeSelectionHook.historyState.index,
        goBack: nodeSelectionHook.goBack,
        goForward: nodeSelectionHook.goForward,
    }), [nodeSelectionHook.historyState.history, nodeSelectionHook.historyState.index, nodeSelectionHook.goBack, nodeSelectionHook.goForward]);

    const viewportValue = useMemo<ViewportContextType>(() => ({
        zoomIn: nodeSelectionHook.zoomIn,
        zoomOut: nodeSelectionHook.zoomOut,
        zoomFit: nodeSelectionHook.zoomFit,
        exportPNG: nodeSelectionHook.exportPNG,
        fgRef: nodeSelectionHook.fgRef,
    }), [nodeSelectionHook.zoomIn, nodeSelectionHook.zoomOut, nodeSelectionHook.zoomFit, nodeSelectionHook.exportPNG, nodeSelectionHook.fgRef]);

    const uiStateValue = useMemo<UIStateContextType>(() => ({
        isPathFinderOpen: nodeSelectionHook.isPathFinderOpen,
        setIsPathFinderOpen: nodeSelectionHook.setIsPathFinderOpen,
        highlightedPathNodeIds: graphDataHook.highlightedPathNodeIds,
        setHighlightedPathNodeIds: graphDataHook.setHighlightedPathNodeIds,
        activeInfoTab: nodeSelectionHook.activeInfoTab,
        setActiveInfoTab: nodeSelectionHook.setActiveInfoTab,
        isMemoryNode: graphDataHook.isMemoryNode,
        viewMode: graphDataHook.viewMode,
        setViewMode: graphDataHook.setViewMode,
    }), [
        nodeSelectionHook.isPathFinderOpen, nodeSelectionHook.setIsPathFinderOpen,
        graphDataHook.highlightedPathNodeIds, graphDataHook.setHighlightedPathNodeIds,
        nodeSelectionHook.activeInfoTab, nodeSelectionHook.setActiveInfoTab,
        graphDataHook.isMemoryNode, graphDataHook.viewMode, graphDataHook.setViewMode,
    ]);

    return (
        <GraphDataProvider value={dataValue}>
            <SelectionProvider value={selectionValue}>
                <NavigationProvider value={navigationValue}>
                    <ViewportProvider value={viewportValue}>
                        <UIStateProvider value={uiStateValue}>
                            {children}
                        </UIStateProvider>
                    </ViewportProvider>
                </NavigationProvider>
            </SelectionProvider>
        </GraphDataProvider>
    );
};

// Metadata: [GraphContext]
