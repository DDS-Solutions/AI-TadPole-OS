import { describe, it, expect, beforeEach } from 'vitest';
import { useDropdownStore } from './dropdownStore';

describe('useDropdownStore', () => {
    beforeEach(() => {
        useDropdownStore.getState().close();
    });

    it('starts with no dropdown open', () => {
        const state = useDropdownStore.getState();
        expect(state.openId).toBeNull();
        expect(state.openType).toBeNull();
    });

    it('toggle() opens a dropdown', () => {
        useDropdownStore.getState().toggle('agent-1', 'role');
        const state = useDropdownStore.getState();
        expect(state.openId).toBe('agent-1');
        expect(state.openType).toBe('role');
    });

    it('toggle() same ID+type closes it', () => {
        useDropdownStore.getState().toggle('agent-1', 'role');
        useDropdownStore.getState().toggle('agent-1', 'role');
        const state = useDropdownStore.getState();
        expect(state.openId).toBeNull();
    });

    it('toggle() different ID auto-closes the previous (mutual exclusion)', () => {
        useDropdownStore.getState().toggle('agent-1', 'role');
        useDropdownStore.getState().toggle('agent-2', 'role');
        const state = useDropdownStore.getState();
        expect(state.openId).toBe('agent-2');
    });

    it('toggle() different type on same ID auto-closes the previous', () => {
        useDropdownStore.getState().toggle('agent-1', 'role');
        useDropdownStore.getState().toggle('agent-1', 'skill');
        const state = useDropdownStore.getState();
        expect(state.openType).toBe('skill');
    });

    it('close() resets all state', () => {
        useDropdownStore.getState().toggle('agent-1', 'role');
        useDropdownStore.getState().close();
        const state = useDropdownStore.getState();
        expect(state.openId).toBeNull();
    });

    it('isOpen() returns true for matching IDs', () => {
        useDropdownStore.getState().toggle('agent-1', 'role');
        expect(useDropdownStore.getState().isOpen('agent-1', 'role')).toBe(true);
        expect(useDropdownStore.getState().isOpen('agent-2', 'role')).toBe(false);
    });
});
