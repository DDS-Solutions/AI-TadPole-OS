/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[dropdownStore_test]` in observability traces.
 */

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
        const { toggle_dropdown } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'skill');
        let state = use_dropdown_store.getState();
        expect(state.open_id === 'agent-1' && state.open_type === 'skill').toBe(true);

        toggle_dropdown('agent-2', 'model');
        state = use_dropdown_store.getState();
        expect(state.open_id === 'agent-1' && state.open_type === 'skill').toBe(false);
        expect(state.open_id === 'agent-2' && state.open_type === 'model').toBe(true);
    });

    it('toggle_dropdown() different type on same ID auto-closes the previous', () => {
        const { toggle_dropdown } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'skill');
        toggle_dropdown('agent-1', 'role');

        const state = use_dropdown_store.getState();
        expect(state.open_id === 'agent-1' && state.open_type === 'skill').toBe(false);
        expect(state.open_id === 'agent-1' && state.open_type === 'role').toBe(true);
    });

    it('close_dropdown() resets all state', () => {
        const { toggle_dropdown, close_dropdown } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'model_2');
        close_dropdown();

        const { open_id, open_type } = use_dropdown_store.getState();
        expect(open_id).toBeNull();
        expect(open_type).toBeNull();
    });

    it('state matches for matching IDs and types', () => {
        const { toggle_dropdown } = use_dropdown_store.getState();

        toggle_dropdown('agent-1', 'skill');

        const state = use_dropdown_store.getState();
        expect(state.open_id === 'agent-1' && state.open_type === 'skill').toBe(true);
        expect(state.open_id === 'agent-2' && state.open_type === 'skill').toBe(false);
        expect(state.open_id === 'agent-1' && state.open_type === 'model').toBe(false);
    });
});

// Metadata: [dropdownStore_test]
