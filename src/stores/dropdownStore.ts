import { create } from 'zustand';

/** Identifies which dropdown category is active. */
type DropdownType = 'skill' | 'model' | 'model2' | 'model3' | 'role';

interface DropdownState {
    /** ID of the agent whose dropdown is currently open, or null. */
    openId: string | null;
    /** Category of the currently open dropdown, or null. */
    openType: DropdownType | null;
    /** Opens a dropdown if closed, closes it if already open. Only one can be open at a time. */
    toggle: (id: string, type: DropdownType) => void;
    /** Closes whatever dropdown is currently open. */
    close: () => void;
    /** Returns true if the specified agent+category dropdown is the one currently open. */
    isOpen: (id: string, type: DropdownType) => boolean;
}

/**
 * Centralized dropdown state for the Hierarchy.
 * Replaces 15+ props that were threaded from OrgChart → HierarchyNode.
 * Only one dropdown can be open at a time.
 */
export const useDropdownStore = create<DropdownState>((set, get) => ({
    openId: null,
    openType: null,

    toggle: (id, type) => set(state =>
        state.openId === id && state.openType === type
            ? { openId: null, openType: null }
            : { openId: id, openType: type }
    ),

    close: () => set({ openId: null, openType: null }),

    isOpen: (id, type) => {
        const s = get();
        return s.openId === id && s.openType === type;
    }
}));
