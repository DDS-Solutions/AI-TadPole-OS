/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the UI component visibility and dropdown states. 
 */

/**
 * @file dropdown_store.test.ts
 * @description Suite for the Centralized Dropdown State (dropdown_store).
 * @module Stores/DropdownStore
 * @testedBehavior
 * - Mutual Exclusion: Only one dropdown can be open at a time.
 * - Toggle Logic: Opening and closing behavior for specific IDs and types.
 * - Reset: Ensuring close_dropdown clears all state.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity (open_id, open_type, toggle_dropdown, close_dropdown, is_open).
 * - Verified 154 tests sweep continuation.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { use_dropdown_store } from './dropdown_store';

describe('use_dropdown_store', () => {
    beforeEach(() => {
        use_dropdown_store.getState().close_dropdown();
    });

    it('starts with no dropdown open', () => {
        const state = use_dropdown_store.getState();
        expect(state.open_id).toBeNull();
        expect(state.open_type).toBeNull();
    });

    it('toggle_dropdown() opens a dropdown', () => {
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'role');
        const state = use_dropdown_store.getState();
        expect(state.open_id).toBe('agent-1');
        expect(state.open_type).toBe('role');
    });

    it('toggle_dropdown() same ID+type closes it', () => {
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'role');
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'role');
        const state = use_dropdown_store.getState();
        expect(state.open_id).toBeNull();
    });

    it('toggle_dropdown() different ID auto-closes the previous (mutual exclusion)', () => {
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'role');
        use_dropdown_store.getState().toggle_dropdown('agent-2', 'role');
        const state = use_dropdown_store.getState();
        expect(state.open_id).toBe('agent-2');
    });

    it('toggle_dropdown() different type on same ID auto-closes the previous', () => {
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'role');
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'skill');
        const state = use_dropdown_store.getState();
        expect(state.open_type).toBe('skill');
    });

    it('close_dropdown() resets all state', () => {
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'role');
        use_dropdown_store.getState().close_dropdown();
        const state = use_dropdown_store.getState();
        expect(state.open_id).toBeNull();
    });

    it('is_open() returns true for matching IDs', () => {
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'role');
        expect(use_dropdown_store.getState().is_open('agent-1', 'role')).toBe(true);
        expect(use_dropdown_store.getState().is_open('agent-2', 'role')).toBe(false);
    });
});

