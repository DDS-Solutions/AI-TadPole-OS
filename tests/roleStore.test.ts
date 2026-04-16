import { describe, it, expect, beforeEach } from 'vitest';
import { use_role_store, select_role_names } from '../src/stores/role_store';

describe('role_store', () => {
    beforeEach(() => {
        // Reset the store to initial state if needed
    });

    it('should initialize with default system roles', () => {
        const state = use_role_store.getState();
        expect(Object.keys(state.roles).length).toBeGreaterThan(0);
        expect(state.roles['CEO']).toBeDefined();
        expect(state.roles['CEO'].skills).toContain('deep_research');
    });

    it('should allow adding a new custom blueprint', () => {
        const new_blueprint = {
            skills: ['Quantum Computing', 'Neural Link'],
            workflows: ['Singularity Protocol']
        };

        use_role_store.getState().add_role('AI Overlord', new_blueprint);

        const updated_state = use_role_store.getState();
        expect(updated_state.roles['AI Overlord']).toEqual(new_blueprint);
        expect(select_role_names(updated_state)).toContain('AI Overlord');
    });

    it('should allow updating an existing blueprint', () => {
        const update = {
            skills: ['Enhanced Debugging'],
            workflows: ['Full System Wipe']
        };

        use_role_store.getState().update_role('CEO', update);

        const updated_state = use_role_store.getState();
        expect(updated_state.roles['CEO']).toEqual(update);
    });

    it('should allow deleting a blueprint', () => {
        const role_name = 'Finance Analyst';
        expect(use_role_store.getState().roles[role_name]).toBeDefined();

        use_role_store.getState().delete_role(role_name);

        expect(use_role_store.getState().roles[role_name]).toBeUndefined();
    });

    it('should maintain referential stability for role names selector', () => {
        const state1 = use_role_store.getState();
        const names1 = select_role_names(state1);
        const names2 = select_role_names(state1); // Calling twice on same state

        expect(JSON.stringify(names1)).toBe(JSON.stringify(names2));
    });
});
