import { create } from 'zustand';
import React from 'react';

interface HeaderState {
    actions: React.ReactNode | null;
    setActions: (actions: React.ReactNode | null) => void;
    clearActions: () => void;
}

export const useHeaderStore = create<HeaderState>((set) => ({
    actions: null,
    setActions: (actions) => set({ actions }),
    clearActions: () => set({ actions: null }),
}));
