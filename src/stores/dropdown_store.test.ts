import { describe, it, expect, beforeEach } from 'vitest';
import { use_dropdown_store } from './dropdown_store';

describe('use_dropdown_store', () => {
    beforeEach(() => {
        use_dropdown_store.getState().close();
    });

    it('starts with no dropdown open', () => {
        const state = use_dropdown_store.getState();
        expect(state.openId).toBeNull();
        expect(state.openType).toBeNull();
    });

    it('toggle() opens a dropdown', () => {
        use_dropdown_store.getState().toggle('agent-1', 'role');
        const state = use_dropdown_store.getState();
        expect(state.openId).toBe('agent-1');
        expect(state.openType).toBe('role');
    });

    it('toggle() same ID+type closes it', () => {
        use_dropdown_store.getState().toggle('agent-1', 'role');
        use_dropdown_store.getState().toggle('agent-1', 'role');
        const state = use_dropdown_store.getState();
        expect(state.openId).toBeNull();
    });

    it('toggle() different ID auto-closes the previous (mutual exclusion)', () => {
        use_dropdown_store.getState().toggle('agent-1', 'role');
        use_dropdown_store.getState().toggle('agent-2', 'role');
        const state = use_dropdown_store.getState();
        expect(state.openId).toBe('agent-2');
    });

    it('toggle() different type on same ID auto-closes the previous', () => {
        use_dropdown_store.getState().toggle('agent-1', 'role');
        use_dropdown_store.getState().toggle('agent-1', 'skill');
        const state = use_dropdown_store.getState();
        expect(state.openType).toBe('skill');
    });

    it('close() resets all state', () => {
        use_dropdown_store.getState().toggle('agent-1', 'role');
        use_dropdown_store.getState().close();
        const state = use_dropdown_store.getState();
        expect(state.openId).toBeNull();
    });

    it('isOpen() returns true for matching IDs', () => {
        use_dropdown_store.getState().toggle('agent-1', 'role');
        expect(use_dropdown_store.getState().isOpen('agent-1', 'role')).toBe(true);
        expect(use_dropdown_store.getState().isOpen('agent-2', 'role')).toBe(false);
    });
});

