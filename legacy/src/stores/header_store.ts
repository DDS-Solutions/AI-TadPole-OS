/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Contextual header and breadcrumb orchestrator. 
 * Manages the propagation of page titles, operational statuses, and dynamic action groups to the root layout shell.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Breadcrumb trail desync during rapid navigation, or action group "ghosting" after page transition.
 * - **Telemetry Link**: Search for `[HeaderStore]` or `set_page_context` in UI tracing.
 */

import { create } from 'zustand';
import React from 'react';

export interface Header_State {
    actions: React.ReactNode | null;
    set_header_actions: (actions: React.ReactNode | null) => void;
    clear_header_actions: () => void;
}

/**
 * use_header_store
 * Manages the dynamic actions area of the application's top navigation bar.
 * Allows components to inject context-specific buttons or controls into the shell.
 */
export const use_header_store = create<Header_State>((set) => ({
    actions: null,
    set_header_actions: (actions) => set({ actions }),
    clear_header_actions: () => set({ actions: null }),
}));

