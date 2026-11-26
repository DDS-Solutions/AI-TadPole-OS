/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / tab_store
 * - **Primary Entrypoints**: `use_tab_store`, `Tab`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

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
    is_swarm_pulse_detached: boolean;
    toggle_swarm_pulse_detachment: () => void;
    is_agent_grid_detached: boolean;
    toggle_agent_grid_detachment: () => void;
    detached_op_tab_ids: string[];
    detach_op_tab: (id: string) => void;
    reattach_op_tab: (id: string) => void;
    reattach_all_op_tabs: () => void;
    reattach_observability_sectors: () => void;
}

// Helper to normalize paths for comparison
const normalize_path = (path: string) => path === '/' ? '/' : path.replace(/\/$/, '');

type Persisted_Tab_State = Pick<Tab_State, 'tabs' | 'active_tab_id'>;

const create_detachment_slice = (
  set: (partial: Partial<Tab_State> | ((state: Tab_State) => Partial<Tab_State>)) => void,
  get: () => Tab_State
) => ({
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
  },
  is_swarm_pulse_detached: false,
  toggle_swarm_pulse_detachment: () => {
    set({ is_swarm_pulse_detached: !get().is_swarm_pulse_detached });
  },
  is_agent_grid_detached: false,
  toggle_agent_grid_detachment: () => {
    set({ is_agent_grid_detached: !get().is_agent_grid_detached });
  },
  detached_op_tab_ids: [],
  detach_op_tab: (id: string) => {
    const current = get().detached_op_tab_ids || [];
    if (!current.includes(id)) {
      set({ detached_op_tab_ids: [...current, id] });
    }
  },
  reattach_op_tab: (id: string) => {
    const current = get().detached_op_tab_ids || [];
    set({ detached_op_tab_ids: current.filter(tid => tid !== id) });
  },
  reattach_all_op_tabs: () => {
    set({ detached_op_tab_ids: [] });
  },
  reattach_observability_sectors: () => {
    set({
      is_system_log_detached: false,
      is_trace_stream_detached: false,
      is_lineage_stream_detached: false
    });
  }
});

const handle_open_tab = (
  tab_data: Omit<Tab, 'id' | 'is_detached'>,
  set: (partial: Partial<Tab_State> | ((state: Tab_State) => Partial<Tab_State>)) => void,
  get: () => Tab_State
) => {
  const tabs = get().tabs || [];
  const normalized_goal = normalize_path(tab_data.path);
  const existing_tab = tabs.find(t => normalize_path(t.path) === normalized_goal);
  
  if (existing_tab) {
    const has_changes = existing_tab.title !== tab_data.title || existing_tab.icon !== tab_data.icon;
    const next_tabs = has_changes
      ? tabs.map(t => t.id === existing_tab.id ? { ...t, title: tab_data.title, icon: tab_data.icon } : t)
      : tabs;
    set({ tabs: next_tabs, active_tab_id: existing_tab.id });
    return;
  }

  const new_id = (typeof crypto !== 'undefined' && crypto.randomUUID) 
    ? crypto.randomUUID() 
    : `tab-${Date.now().toString(36)}-${(typeof performance !== 'undefined' ? Math.floor(performance.now() * 1000) : 0).toString(36)}`;
  set({
    tabs: [...tabs, { ...tab_data, id: new_id }],
    active_tab_id: new_id
  });
};

const handle_close_tab = (
  id: string,
  set: (partial: Partial<Tab_State> | ((state: Tab_State) => Partial<Tab_State>)) => void,
  get: () => Tab_State
) => {
  const { tabs, active_tab_id } = get();
  if (tabs.length <= 1) return;

  const filtered_tabs = (tabs || []).filter(t => t.id !== id);
  if (active_tab_id !== id) {
    set({ tabs: filtered_tabs });
    return;
  }

  const index = (tabs || []).findIndex(t => t.id === id);
  const next_index = Math.max(0, index - 1);
  const next_active_id = filtered_tabs[next_index]?.id || filtered_tabs[0]?.id;

  set({
    tabs: filtered_tabs,
    active_tab_id: next_active_id
  });
};

const create_tab_slice = (
  set: (partial: Partial<Tab_State> | ((state: Tab_State) => Partial<Tab_State>)) => void,
  get: () => Tab_State
): Tab_State => ({
  tabs: [
    { id: 'initial-ops', title: 'Operations', path: '/dashboard', icon: 'LayoutDashboard' }
  ],
  active_tab_id: 'initial-ops',
  open_tab: (tab_data) => handle_open_tab(tab_data, set, get),
  close_tab: (id) => handle_close_tab(id, set, get),
  set_active_tab: (id) => {
    if (get().active_tab_id === id) return;
    const tab = (get().tabs || []).find(t => t.id === id);
    if (tab) set({ active_tab_id: id });
  },
  update_tab_title: (path, title) => {
    const normalized_goal = normalize_path(path);
    set(state => ({
      tabs: (state.tabs || []).map(t => 
        normalize_path(t.path) === normalized_goal ? { ...t, title } : t
      )
    }));
  },
  toggle_tab_detachment: (id: string) => {
    set({
      tabs: (get().tabs || []).map(t => t.id === id ? { ...t, is_detached: !t.is_detached } : t)
    });
  },
  ...create_detachment_slice(set, get)
});

/**
 * use_tab_store
 * Orchestrates the multi-tab layout and persistent workspace state.
 * Features cross-window synchronization for a seamless multi-monitor experience.
 */
export const use_tab_store = create<Tab_State>()(
  persist(
    (set, get) => create_tab_slice(set, get),
    {
      name: 'tadpole-tabs-storage',
      version: 6,
      partialize: (state): Persisted_Tab_State => ({
        tabs: (state.tabs || []).map(t => ({ ...t, is_detached: false })),
        active_tab_id: state.active_tab_id,
      }),
      migrate: (persisted_state: unknown): Persisted_Tab_State => {
        const state = (persisted_state || {}) as Record<string, unknown>;
        const clean_tabs = Array.isArray(state.tabs)
          ? (state.tabs as Tab[]).map(t => ({ ...t, is_detached: false }))
          : [{ id: 'initial-ops', title: 'Operations', path: '/dashboard', icon: 'LayoutDashboard' }];

        return {
          tabs: clean_tabs,
          active_tab_id: typeof state.active_tab_id === 'string' ? state.active_tab_id : 'initial-ops'
        };
      }
    }
  )
);

function handle_recall_sector(payload?: { type?: string; id?: string; tab_id?: string | null }) {
    const { type, id, tab_id } = payload || {};
    if (type === 'system-log') {
        use_tab_store.setState({ is_system_log_detached: false });
    } else if (type === 'trace-stream') {
        use_tab_store.setState({ is_trace_stream_detached: false });
    } else if (type === 'lineage-stream') {
        use_tab_store.setState({ is_lineage_stream_detached: false });
    } else if (type === 'swarm-pulse') {
        use_tab_store.setState({ is_swarm_pulse_detached: false });
    } else if (type === 'agent-status') {
        if (tab_id) {
            use_tab_store.getState().reattach_op_tab(tab_id);
        } else {
            use_tab_store.setState({ is_agent_grid_detached: false });
        }
    } else if (type === 'tab' && id) {
        const tabs = use_tab_store.getState().tabs || [];
        use_tab_store.setState({
            tabs: tabs.map(t => t.id === id ? { ...t, is_detached: false } : t)
        });
    }
}

// TAB_STORE_SYNC: High-fidelity cross-tab layout parity
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel('tab_store_sync') : null;

if (sync_channel) {
    // Fingerprint based on specific synced properties only
    const get_sync_fingerprint = (state: Tab_State) => JSON.stringify({
        tabs: state.tabs || [],
        active_tab_id: state.active_tab_id
    });

    let last_broadcast = get_sync_fingerprint(use_tab_store.getState());

    sync_channel.onmessage = (event) => {
        if (event.data?.type === 'SYNC_STATE_TABS') {
            const { tabs, active_tab_id } = event.data.payload;
            const current_fingerprint = get_sync_fingerprint(use_tab_store.getState());
            const next_fingerprint = JSON.stringify({ tabs, active_tab_id });

            if (current_fingerprint !== next_fingerprint && tabs && Array.isArray(tabs)) {
                last_broadcast = next_fingerprint; // Update fingerprint BEFORE setting state
                use_tab_store.setState({ tabs, active_tab_id }); 
            }
        } else if (event.data?.type === 'RECALL_SECTOR') {
            handle_recall_sector(event.data.payload);
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


import { register_store_for_test_reset } from '../utils/store_registry';

register_store_for_test_reset(() => {
    use_tab_store.setState(use_tab_store.getInitialState(), true);
});
