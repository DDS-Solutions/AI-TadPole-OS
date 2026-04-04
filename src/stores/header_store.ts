import { create } from 'zustand';
import React from 'react';

interface Header_State {
    actions: React.ReactNode | null;
    set_actions: (actions: React.ReactNode | null) => void;
    clear_actions: () => void;
}

/**
 * use_header_store
 * Manages the dynamic actions area of the application's top navigation bar.
 * Allows components to inject context-specific buttons or controls into the shell.
 */
export const use_header_store = create<Header_State>((set) => ({
    actions: null,
    set_actions: (actions) => set({ actions }),
    clear_actions: () => set({ actions: null }),
}));
