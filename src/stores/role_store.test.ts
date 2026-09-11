/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / role_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
    const mock_role: Role = {
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

    beforeEach(() => {
        vi.clearAllMocks();
        use_role_store.setState({ roles: {} });
    });

    it('should allow adding a new role', () => {
        use_role_store.getState().add_role(mock_role);
        
        const state = use_role_store.getState();
        expect(state.roles['new-role']).toEqual(mock_role);
    });

    it('should allow updating an existing role', () => {
        use_role_store.setState({ roles: { 'new-role': { ...mock_role } } });
        
        const updates = { skills: ['updated'] };
        use_role_store.getState().update_role('new-role', updates);
        
        const state = use_role_store.getState();
        expect(state.roles['new-role'].skills).toEqual(['updated']);
    });

    it('should allow deleting a role', () => {
        use_role_store.setState({ roles: { 'new-role': { ...mock_role } } });
        
        use_role_store.getState().delete_role('new-role');
        
        const state = use_role_store.getState();
        expect(state.roles['new-role']).toBeUndefined();
    });

    it('should replace entire role dictionary on set_roles', () => {
        const role_a = { ...mock_role, id: 'role-a', name: 'Role A' };
        const role_b = { ...mock_role, id: 'role-b', name: 'Role B' };

        use_role_store.getState().set_roles([role_a, role_b]);

        const state = use_role_store.getState();
        expect(Object.keys(state.roles)).toEqual(['role-a', 'role-b']);
        expect(state.roles['role-a'].name).toBe('Role A');
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

    it('migrates v1 roles state to v2 role objects', () => {
        const migrate = (use_role_store as any).persist?.getOptions?.()?.migrate;
        if (typeof migrate === 'function') {
            const v1_state = {
                roles: {
                    'Lead Architect': { skills: ['arch'], workflows: ['review'] }
                }
            };
            const migrated = migrate(v1_state, 1);
            expect(migrated.roles['lead-architect']).toBeDefined();
            expect(migrated.roles['lead-architect'].name).toBe('Lead Architect');
            expect(migrated.roles['lead-architect'].department).toBe('Operations');
        }
    });
});
