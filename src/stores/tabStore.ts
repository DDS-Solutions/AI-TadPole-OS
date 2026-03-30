import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface Tab {
    id: string;
    title: string;
    path: string;
    icon?: string;
    isDetached?: boolean;
}

interface TabState {
    tabs: Tab[];
    activeTabId: string | null;

    // Actions
    openTab: (tab: Omit<Tab, 'id' | 'isDetached'>) => void;
    closeTab: (id: string) => void;
    setActiveTab: (id: string) => void;
    updateTabTitle: (path: string, title: string) => void;
    toggleTabDetachment: (id: string) => void;
    isSystemLogDetached: boolean;
    toggleSystemLogDetachment: () => void;
    isTraceStreamDetached: boolean;
    toggleTraceStreamDetachment: () => void;
    isLineageStreamDetached: boolean;
    toggleLineageStreamDetachment: () => void;
}

// Helper to normalize paths for comparison
const normalizePath = (path: string) => path === '/' ? '/' : path.replace(/\/$/, '');

export const useTabStore = create<TabState>()(
    persist(
        (set, get) => ({
            tabs: [
                { id: 'initial-ops', title: 'Operations', path: '/dashboard', icon: 'LayoutDashboard' }
            ],
            activeTabId: 'initial-ops',

            openTab: (tabData) => {
                const { tabs } = get();
                const normalizedPath = normalizePath(tabData.path);
                
                // Check if tab already exists for this path
                const existingTab = tabs.find(t => normalizePath(t.path) === normalizedPath);
                
                if (existingTab) {
                    if (existingTab.title !== tabData.title || existingTab.icon !== tabData.icon) {
                        set({ 
                            tabs: tabs.map(t => t.id === existingTab.id ? { ...t, title: tabData.title, icon: tabData.icon } : t),
                            activeTabId: existingTab.id 
                        });
                    } else {
                        set({ activeTabId: existingTab.id });
                    }
                    return;
                }

                // Add new tab
                const newId = `tab-${Math.random().toString(36).substr(2, 9)}`;
                const newTab: Tab = { ...tabData, id: newId };
                
                set({
                    tabs: [...tabs, newTab],
                    activeTabId: newId
                });
            },

            closeTab: (id) => {
                const { tabs, activeTabId } = get();
                
                // Don't close the last tab
                if (tabs.length <= 1) return;

                const filteredTabs = tabs.filter(t => t.id !== id);
                
                let newActiveId = activeTabId;
                if (activeTabId === id) {
                    // Switch to the tab to the left, or the first one available
                    const index = tabs.findIndex(t => t.id === id);
                    const nextIndex = Math.max(0, index - 1);
                    newActiveId = filteredTabs[nextIndex]?.id || filteredTabs[0]?.id;
                }

                set({
                    tabs: filteredTabs,
                    activeTabId: newActiveId
                });
            },

            setActiveTab: (id) => {
                const tab = get().tabs.find(t => t.id === id);
                if (tab) {
                    set({ activeTabId: id });
                }
            },

            updateTabTitle: (path, title) => {
                const normalizedGoal = normalizePath(path);
                set(state => ({
                    tabs: state.tabs.map(t => 
                        normalizePath(t.path) === normalizedGoal ? { ...t, title } : t
                    )
                }));
            },

            toggleTabDetachment: (id: string) => {
                const { tabs } = get();
                set({
                    tabs: tabs.map(t => t.id === id ? { ...t, isDetached: !t.isDetached } : t)
                });
            },

            isSystemLogDetached: false,

            toggleSystemLogDetachment: () => {
                set({ isSystemLogDetached: !get().isSystemLogDetached });
            },

            isTraceStreamDetached: false,
            toggleTraceStreamDetachment: () => {
                set({ isTraceStreamDetached: !get().isTraceStreamDetached });
            },
            isLineageStreamDetached: false,
            toggleLineageStreamDetachment: () => {
                set({ isLineageStreamDetached: !get().isLineageStreamDetached });
            }
        }),
        {
            name: 'tadpole-tabs-storage',
            version: 2
        }
    )
);

// TAB_STORE_SYNC: High-fidelity cross-tab layout parity
const bc = typeof window !== 'undefined' ? new BroadcastChannel('tab_store_sync') : null;

if (bc) {
    // Fingerprint based on specific synced properties only
    const getSyncFingerprint = (state: TabState) => JSON.stringify({
        tabs: state.tabs,
        activeTabId: state.activeTabId
    });

    let lastBroadcast = getSyncFingerprint(useTabStore.getState());

    bc.onmessage = (event) => {
        if (event.data.type === 'SYNC_STATE_TABS') {
            const { tabs, activeTabId } = event.data.payload;
            const currentFingerprint = getSyncFingerprint(useTabStore.getState());
            const nextFingerprint = JSON.stringify({ tabs, activeTabId });

            if (currentFingerprint !== nextFingerprint) {
                lastBroadcast = nextFingerprint; // Update fingerprint BEFORE setting state
                useTabStore.setState({ tabs, activeTabId }); // Partial update, no 'true' replacement
            }
        }
    };

    useTabStore.subscribe((state) => {
        const current = getSyncFingerprint(state);
        if (current !== lastBroadcast) {
            lastBroadcast = current;
            bc.postMessage({ 
                type: 'SYNC_STATE_TABS', 
                payload: { tabs: state.tabs, activeTabId: state.activeTabId } 
            });
        }
    });
}
