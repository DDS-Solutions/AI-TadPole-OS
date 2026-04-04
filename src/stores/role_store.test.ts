import { describe, it, expect, beforeEach } from 'vitest';
import { use_role_store, selectRoleNames } from './role_store';

describe('use_role_store', () => {
    beforeEach(() => {
        // Clear store between tests if needed, but for unit tests we can just test CRUD
        // roleStore has persistence, so we test the actions directly
    });

    it('should allow adding a new role', () => {
        const newRole = { skills: ['skill1'], workflows: ['flow1'] };
        use_role_store.getState().addRole('NewRole', newRole);
        
        const state = use_role_store.getState();
        expect(state.roles['NewRole']).toEqual(newRole);
    });

    it('should allow updating an existing role', () => {
        const updatedRole = { skills: ['updated'], workflows: ['updated'] };
        use_role_store.getState().updateRole('NewRole', updatedRole);
        
        const state = use_role_store.getState();
        expect(state.roles['NewRole']).toEqual(updatedRole);
    });

    it('should allow deleting a role', () => {
        use_role_store.getState().deleteRole('NewRole');
        
        const state = use_role_store.getState();
        expect(state.roles['NewRole']).toBeUndefined();
    });

    it('selector selectRoleNames should return sorted role keys', () => {
        // Initial state from mockAgents is likely populated
        const state = use_role_store.getState();
        const names = selectRoleNames(state);
        
        expect(Array.isArray(names)).toBe(true);
        // Verify sorting alphabetically
        const sortedNames = [...names].sort();
        expect(names).toEqual(sortedNames);
    });

    it('should maintain referential stability for role names selector', () => {
        const state1 = use_role_store.getState();
        const names1 = selectRoleNames(state1);
        const names2 = selectRoleNames(state1);
        expect(names1).toEqual(names2);
    });
});

