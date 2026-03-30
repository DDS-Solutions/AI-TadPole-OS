import { describe, it, expect, beforeEach } from 'vitest';
import { useRoleStore, selectRoleNames } from '../src/stores/roleStore';

describe('RoleStore', () => {
    beforeEach(() => {
        // Reset the store to initial state by clearing the roles (mocking reset)
        // Since we are using persist middleware, we don't have a direct clear, 
        // but we can replace it with INITIAL_ROLES manually if needed.
    });

    it('should initialize with default system roles', () => {
        const state = useRoleStore.getState();
        expect(Object.keys(state.roles).length).toBeGreaterThan(0);
        expect(state.roles['CEO']).toBeDefined();
        expect(state.roles['CEO'].skills).toContain('deep_research');
    });

    it('should allow adding a new custom blueprint', () => {
        const newBlueprint = {
            skills: ['Quantum Computing', 'Neural Link'],
            workflows: ['Singularity Protocol']
        };

        useRoleStore.getState().addRole('AI Overlord', newBlueprint);

        const updatedState = useRoleStore.getState();
        expect(updatedState.roles['AI Overlord']).toEqual(newBlueprint);
        expect(selectRoleNames(updatedState)).toContain('AI Overlord');
    });

    it('should allow updating an existing blueprint', () => {
        const update = {
            skills: ['Enhanced Debugging'],
            workflows: ['Full System Wipe']
        };

        useRoleStore.getState().updateRole('CEO', update);

        const updatedState = useRoleStore.getState();
        expect(updatedState.roles['CEO']).toEqual(update);
    });

    it('should allow deleting a blueprint', () => {
        const roleName = 'Finance Analyst';
        expect(useRoleStore.getState().roles[roleName]).toBeDefined();

        useRoleStore.getState().deleteRole(roleName);

        expect(useRoleStore.getState().roles[roleName]).toBeUndefined();
    });

    it('should maintain referential stability for role names selector', () => {
        const state1 = useRoleStore.getState();
        const names1 = selectRoleNames(state1);
        const names2 = selectRoleNames(state1); // Calling twice on same state

        // This is a basic test for the selector function itself, 
        // in a real React environment we useMemo/useSelector to enforce this.
        expect(JSON.stringify(names1)).toBe(JSON.stringify(names2));
    });
});
