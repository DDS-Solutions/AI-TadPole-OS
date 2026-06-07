/**
 * @docs ARCHITECTURE:Intelligence
 *
 * ### AI Assist Note
 * Granular context hooks for the Knowledge Graph provider.
 * Extracted to a separate file to satisfy `react-refresh/only-export-components`
 * which requires that files exporting React components do not also export non-component bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Hook called outside `GraphProvider` tree → throws Error with context name.
 * - **Telemetry Link**: Search `[graph_context_hooks]` in component traces.
 */

import { useContext, useMemo } from 'react';
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

export type {
    GraphDataContextType,
    SelectionContextType,
    NavigationContextType,
    ViewportContextType,
    UIStateContextType,
};

export const useGraphDataContext = () => {
    const context = useContext(GraphDataContext);
    if (!context) throw new Error('useGraphDataContext must be used within a GraphProvider');
    return context;
};

export const useSelectionContext = () => {
    const context = useContext(SelectionContext);
    if (!context) throw new Error('useSelectionContext must be used within a GraphProvider');
    return context;
};

export const useNavigationContext = () => {
    const context = useContext(NavigationContext);
    if (!context) throw new Error('useNavigationContext must be used within a GraphProvider');
    return context;
};

export const useViewportContext = () => {
    const context = useContext(ViewportContext);
    if (!context) throw new Error('useViewportContext must be used within a GraphProvider');
    return context;
};

export const useUIStateContext = () => {
    const context = useContext(UIStateContext);
    if (!context) throw new Error('useUIStateContext must be used within a GraphProvider');
    return context;
};

// Legacy single-hook fallback for backwards compatibility
export const useGraphContext = () => {
    const data = useGraphDataContext();
    const selection = useSelectionContext();
    const navigation = useNavigationContext();
    const viewport = useViewportContext();
    const uiState = useUIStateContext();
    return useMemo(() => ({
        ...data,
        ...selection,
        ...navigation,
        ...viewport,
        ...uiState
    }), [data, selection, navigation, viewport, uiState]);
};

// Metadata: [graph_context_hooks]
