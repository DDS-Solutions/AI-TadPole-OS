import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface Tab {
    id: string;
    title: string;
    path: string;
    icon?: string;
    is_detached?: boolean;
}

interface Tab_State {
    tabs: Tab[];
    active_tab_id: string | null;

    // Actions
    open_tab: (tab: Omit<Tab, 'id' | 'is_detached'>) => void;
    close_tab: (id: string) => void;
    set_active_tab: (id: string) => void;
    update_tab_title: (path: string, title: string) => void;
    toggle_tab_detachment: (id: string) => void;
    is_system_log_detached: boolean;
    toggle_system_log_detachment: () => void;
    is_trace_stream_detached: boolean;
    toggle_trace_stream_detachment: () => void;
    is_lineage_stream_detached: boolean;
    toggle_lineage_stream_detachment: () => void;
}

// Helper to normalize paths for comparison
const normalize_path = (path: string) => path === '/' ? '/' : path.replace(/\/$/, '');

/**
 * use_tab_store
 * Orchestrates the multi-tab layout and persistent workspace state.
 * Features cross-window synchronization for a seamless multi-monitor experience.
 */
export const use_tab_store = create<Tab_State>()(
    persist(
        (set, get) => ({
            tabs: [
                { id: 'initial-ops', title: 'Operations', path: '/dashboard', icon: 'LayoutDashboard' }
            ],
            active_tab_id: 'initial-ops',

            open_tab: (tab_data) => {
                const { tabs } = get();
                const normalized_goal = normalize_path(tab_data.path);
                
                // Check if tab already exists for this path
                const existing_tab = tabs.find(t => normalize_path(t.path) === normalized_goal);
                
                if (existing_tab) {
                    if (existing_tab.title !== tab_data.title || existing_tab.icon !== tab_data.icon) {
                        set({ 
                            tabs: tabs.map(t => t.id === existing_tab.id ? { ...t, title: tab_data.title, icon: tab_data.icon } : t),
                            active_tab_id: existing_tab.id 
                        });
                    } else {
                        set({ active_tab_id: existing_tab.id });
                    }
                    return;
                }

                // Add new tab
                const new_id = `tab-${Math.random().toString(36).substring(2, 11)}`;
                const new_tab: Tab = { ...tab_data, id: new_id };
                
                set({
                    tabs: [...tabs, new_tab],
                    active_tab_id: new_id
                });
            },

            close_tab: (id) => {
                const { tabs, active_tab_id } = get();
                
                // Don't close the last tab
                if (tabs.length <= 1) return;

                const filtered_tabs = tabs.filter(t => t.id !== id);
                
                let next_active_id = active_tab_id;
                if (active_tab_id === id) {
                    // Switch to the tab to the left, or the first one available
                    const index = tabs.findIndex(t => t.id === id);
                    const next_index = Math.max(0, index - 1);
                    next_active_id = filtered_tabs[next_index]?.id || filtered_tabs[0]?.id;
                }

                set({
                    tabs: filtered_tabs,
                    active_tab_id: next_active_id
                });
            },

            set_active_tab: (id) => {
                const tab = get().tabs.find(t => t.id === id);
                if (tab) {
                    set({ active_tab_id: id });
                }
            },

            update_tab_title: (path, title) => {
                const normalized_goal = normalize_path(path);
                set(state => ({
                    tabs: state.tabs.map(t => 
                        normalize_path(t.path) === normalized_goal ? { ...t, title } : t
                    )
                }));
            },

            toggle_tab_detachment: (id: string) => {
                const { tabs } = get();
                set({
                    tabs: tabs.map(t => t.id === id ? { ...t, is_detached: !t.is_detached } : t)
                });
            },

            is_system_log_detached: false,

            toggle_system_log_detachment: () => {
                set({ is_system_log_detached: !get().is_system_log_detached });
            },

            is_trace_stream_detached: false,
            toggle_trace_stream_detachment: () => {
                set({ is_trace_stream_detached: !get().is_trace_stream_detached });
            },
            is_lineage_stream_detached: false,
            toggle_lineage_stream_detachment: () => {
                set({ is_lineage_stream_detached: !get().is_lineage_stream_detached });
            }
        }),
        {
            name: 'tadpole-tabs-storage',
            version: 2,
            migrate: (persisted: any, version: number) => {
                if (version === 1) {
                    // Handle legacy field names if necessary
                    return {
                        ...persisted,
                        active_tab_id: persisted.activeTabId || persisted.active_tab_id,
                        is_system_log_detached: persisted.isSystemLogDetached || persisted.is_system_log_detached,
                        is_trace_stream_detached: persisted.isTraceStreamDetached || persisted.is_trace_stream_detached,
                        is_lineage_stream_detached: persisted.isLineageStreamDetached || persisted.is_lineage_stream_detached
                    };
                }
                return persisted;
            }
        }
    )
);

// TAB_STORE_SYNC: High-fidelity cross-tab layout parity
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel('tab_store_sync') : null;

if (sync_channel) {
    // Fingerprint based on specific synced properties only
    const get_sync_fingerprint = (state: Tab_State) => JSON.stringify({
        tabs: state.tabs,
        active_tab_id: state.active_tab_id
    });

    let last_broadcast = get_sync_fingerprint(use_tab_store.getState());

    sync_channel.onmessage = (event) => {
        if (event.data.type === 'SYNC_STATE_TABS') {
            const { tabs, active_tab_id } = event.data.payload;
            const current_fingerprint = get_sync_fingerprint(use_tab_store.getState());
            const next_fingerprint = JSON.stringify({ tabs, active_tab_id });

            if (current_fingerprint !== next_fingerprint) {
                last_broadcast = next_fingerprint; // Update fingerprint BEFORE setting state
                use_tab_store.setState({ tabs, active_tab_id }); 
            }
        }
    };

    use_tab_store.subscribe((state) => {
        const current = get_sync_fingerprint(state);
        if (current !== last_broadcast) {
            last_broadcast = current;
            sync_channel.postMessage({ 
                type: 'SYNC_STATE_TABS', 
                payload: { tabs: state.tabs, active_tab_id: state.active_tab_id } 
            });
        }
    });
}
