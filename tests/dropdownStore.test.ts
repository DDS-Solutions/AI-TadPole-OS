import { describe, it, expect, beforeEach } from 'vitest';
import { use_dropdown_store } from '../src/stores/dropdown_store';

describe('dropdown_store', () => {
    beforeEach(() => {
        use_dropdown_store.getState().close_dropdown();
    });

    it('starts with no dropdown open', () => {
        const { open_id, open_type } = use_dropdown_store.getState();
        expect(open_id).toBeNull();
        expect(open_type).toBeNull();
    });

    it('toggle_dropdown() opens a dropdown', () => {
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'skill');

        const { open_id, open_type } = use_dropdown_store.getState();
        expect(open_id).toBe('agent-1');
        expect(open_type).toBe('skill');
    });

    it('toggle_dropdown() same ID+type closes it', () => {
        const { toggle_dropdown } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'model');
        toggle_dropdown('agent-1', 'model');

        const { open_id, open_type } = use_dropdown_store.getState();
        expect(open_id).toBeNull();
        expect(open_type).toBeNull();
    });

    it('toggle_dropdown() different ID auto-closes the previous (mutual exclusion)', () => {
        const { toggle_dropdown, is_open } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'skill');
        expect(is_open('agent-1', 'skill')).toBe(true);

        toggle_dropdown('agent-2', 'model');
        expect(is_open('agent-1', 'skill')).toBe(false);
        expect(is_open('agent-2', 'model')).toBe(true);
    });

    it('toggle_dropdown() different type on same ID auto-closes the previous', () => {
        const { toggle_dropdown, is_open } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'skill');
        toggle_dropdown('agent-1', 'role');

        expect(is_open('agent-1', 'skill')).toBe(false);
        expect(is_open('agent-1', 'role')).toBe(true);
    });

    it('close_dropdown() resets all state', () => {
        const { toggle_dropdown, close_dropdown } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'model2');
        close_dropdown();

        const { open_id, open_type } = use_dropdown_store.getState();
        expect(open_id).toBeNull();
        expect(open_type).toBeNull();
    });

    it('is_open() returns false for non-matching IDs', () => {
        const { toggle_dropdown, is_open } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'skill');

        expect(is_open('agent-1', 'skill')).toBe(true);
        expect(is_open('agent-2', 'skill')).toBe(false);
        expect(is_open('agent-1', 'model')).toBe(false);
    });
});
