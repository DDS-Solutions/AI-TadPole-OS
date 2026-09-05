/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / dropdown_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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

    it('open_id and open_type state matches matching IDs', () => {
        use_dropdown_store.getState().toggle_dropdown('agent-1', 'role');
        const state = use_dropdown_store.getState();
        expect(state.open_id === 'agent-1' && state.open_type === 'role').toBe(true);
        expect(state.open_id === 'agent-2' && state.open_type === 'role').toBe(false);
    });
});
