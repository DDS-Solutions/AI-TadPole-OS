/**
 * @docs ARCHITECTURE:Intelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / graph_context_hooks
 * - **Primary Entrypoints**: `useGraphDataContext`, `useSelectionContext`, `useNavigationContext`, `useViewportContext`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useContext } from 'react';
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
