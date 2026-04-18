/**
 * @docs ARCHITECTURE:Stores
 * 
 * ### AI Assist Note
 * **Core functional element for the Tadpole OS engine.**
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path: Runtime logic error or state corruption.**
 * - **Telemetry Link**: Search `[dropdown_store.ts]` in tracing logs.
 */

import { create } from 'zustand';

/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Centralized dropdown state for the Hierarchy.
 * Replaces 15+ props that were threaded from Org_Chart → Hierarchy_Node.
 * Only one dropdown can be open at a time.
 */

/** Identifies which dropdown category is active. */
export type Dropdown_Type = 'skill' | 'model' | 'model_2' | 'model_3' | 'role';

export interface Dropdown_State {
    /** ID of the agent whose dropdown is currently open, or null. */
    open_id: string | null;
    /** Category of the currently open dropdown, or null. */
    open_type: Dropdown_Type | null;
    /** Opens a dropdown if closed, closes it if already open. Only one can be open at a time. */
    toggle_dropdown: (id: string, type: Dropdown_Type) => void;
    /** Closes whatever dropdown is currently open. */
    close_dropdown: () => void;
    /** Returns true if the specified agent+category dropdown is the one currently open. */
    is_open: (id: string, type: Dropdown_Type) => boolean;
}

/**
 * Centralized dropdown state for the Hierarchy.
 */
export const use_dropdown_store = create<Dropdown_State>((set, get) => ({
    open_id: null,
    open_type: null,

    toggle_dropdown: (id: string, type: Dropdown_Type) => set((state: Dropdown_State) =>
        state.open_id === id && state.open_type === type
            ? { open_id: null, open_type: null }
            : { open_id: id, open_type: type }
    ),

    close_dropdown: () => set({ open_id: null, open_type: null }),

    is_open: (id: string, type: Dropdown_Type) => {
        const s = get();
        return s.open_id === id && s.open_type === type;
    }
}));
