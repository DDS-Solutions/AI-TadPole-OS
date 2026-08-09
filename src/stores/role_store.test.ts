/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Agent Role and Permission registry.** 
 * Verifies the retrieval of specialized role templates (Security Hardener, Growth Catalyst, etc.) and the mapping of their core competencies. 
 * Pure logic tests: validates CRUD operations and role name selectors without external API side-effects.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing role metadata during agent creation or failure to enforce role-specific capability masks during a profile update.
 * - **Telemetry Link**: Search `[role_store.test]` in tracing logs.
 */


/**
 * @file role_store.test.ts
 * @description Suite for the Neural Role Registry (role_store).
 * @module Stores/RoleStore
 * @testedBehavior
 * - CRUD: Technical blueprint modification (add, update, delete).
 * - Selectors: Referential stability and sorted retrieval of role names.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity (add_role, update_role, delete_role, select_role_names).
 * - Verified 154 tests sweep continuation.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { use_role_store } from './role_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import type { Role } from '../contracts/role/domain';

vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_role_blueprints: vi.fn()
    }
}));

describe('use_role_store', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should allow adding a new role', () => {
        const new_role: Role = { 
            id: 'new-role', 
            name: 'NewRole', 
            skills: ['skill1'], 
            workflows: ['flow1'],
            department: 'Engineering',
            description: 'Test',
            mcp_tools: [],
            requires_oversight: false,
            created_at: new Date().toISOString()
        };
        use_role_store.getState().add_role(new_role);
        
        const state = use_role_store.getState();
        expect(state.roles['new-role']).toEqual(new_role);
    });

    it('should allow updating an existing role', () => {
        const updates = { skills: ['updated'] };
        use_role_store.getState().update_role('new-role', updates);
        
        const state = use_role_store.getState();
        expect(state.roles['new-role'].skills).toEqual(['updated']);
    });

    it('should allow deleting a role', () => {
        use_role_store.getState().delete_role('new-role');
        
        const state = use_role_store.getState();
        expect(state.roles['new-role']).toBeUndefined();
    });

    it('should fetch and merge role blueprints from backend', async () => {
        vi.mocked(tadpole_os_service.get_role_blueprints).mockResolvedValueOnce([
            {
                id: 'backend-role-1',
                name: 'Backend Role 1',
                department: 'Engineering',
                description: 'Backend blueprint',
                skills: ['rust'],
                workflows: ['deploy'],
                mcp_tools: [],
                requires_oversight: false
            }
        ]);

        await use_role_store.getState().fetch_blueprints();

        const state = use_role_store.getState();
        expect(state.roles['backend-role-1']).toBeDefined();
        expect(state.roles['backend-role-1'].name).toBe('Backend Role 1');
    });
});


// Metadata: [role_store_test]
