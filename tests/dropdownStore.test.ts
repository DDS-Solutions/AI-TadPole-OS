import { describe, it, expect, beforeEach } from 'vitest';
import { useDropdownStore } from '../src/stores/dropdownStore';

describe('dropdownStore', () => {
    beforeEach(() => {
        useDropdownStore.getState().close();
    });

    it('starts with no dropdown open', () => {
        const { openId, openType } = useDropdownStore.getState();
        expect(openId).toBeNull();
        expect(openType).toBeNull();
    });

    it('toggle() opens a dropdown', () => {
        useDropdownStore.getState().toggle('agent-1', 'skill');

        const { openId, openType } = useDropdownStore.getState();
        expect(openId).toBe('agent-1');
        expect(openType).toBe('skill');
    });

    it('toggle() same ID+type closes it', () => {
        const { toggle } = useDropdownStore.getState();

        toggle('agent-1', 'model');
        toggle('agent-1', 'model');

        const { openId, openType } = useDropdownStore.getState();
        expect(openId).toBeNull();
        expect(openType).toBeNull();
    });

    it('toggle() different ID auto-closes the previous (mutual exclusion)', () => {
        const { toggle, isOpen } = useDropdownStore.getState();

        toggle('agent-1', 'skill');
        expect(isOpen('agent-1', 'skill')).toBe(true);

        toggle('agent-2', 'model');
        expect(isOpen('agent-1', 'skill')).toBe(false);
        expect(isOpen('agent-2', 'model')).toBe(true);
    });

    it('toggle() different type on same ID auto-closes the previous', () => {
        const { toggle, isOpen } = useDropdownStore.getState();

        toggle('agent-1', 'skill');
        toggle('agent-1', 'role');

        expect(isOpen('agent-1', 'skill')).toBe(false);
        expect(isOpen('agent-1', 'role')).toBe(true);
    });

    it('close() resets all state', () => {
        const { toggle, close } = useDropdownStore.getState();

        toggle('agent-1', 'model2');
        close();

        const { openId, openType } = useDropdownStore.getState();
        expect(openId).toBeNull();
        expect(openType).toBeNull();
    });

    it('isOpen() returns false for non-matching IDs', () => {
        const { toggle, isOpen } = useDropdownStore.getState();

        toggle('agent-1', 'skill');

        expect(isOpen('agent-1', 'skill')).toBe(true);
        expect(isOpen('agent-2', 'skill')).toBe(false);
        expect(isOpen('agent-1', 'model')).toBe(false);
    });
});
