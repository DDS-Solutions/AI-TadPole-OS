/**
 * @docs ARCHITECTURE:Stores
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / dropdown_store
 * - **Primary Entrypoints**: `use_dropdown_store`, `Dropdown_State`, `Dropdown_Type`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { create } from 'zustand';

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
}

/**
 * Centralized dropdown state for the Hierarchy.
 */
export const use_dropdown_store = create<Dropdown_State>((set) => ({
    open_id: null,
    open_type: null,

    toggle_dropdown: (id: string, type: Dropdown_Type) => set((state: Dropdown_State) =>
        state.open_id === id && state.open_type === type
            ? { open_id: null, open_type: null }
            : { open_id: id, open_type: type }
    ),

    close_dropdown: () => set({ open_id: null, open_type: null })
}));
