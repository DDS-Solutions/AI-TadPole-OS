/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Personas and Role-Based Access Control (RBAC) definitions. 
 * Orchestrates agent behavioral templates, system-prompt defaults, and sectoral access scopes.
 * Aligned with the Tadpole OS `RoleBlueprint` backend model.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Failed store hydration (empty roles), role deletion mismatch, or migration errors across store versions.
 * - **Telemetry Link**: Search `[RoleStore]` in UI traces.
 */

console.debug("[RoleStore] Module loaded");

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ROLE_ACTIONS as INITIAL_ROLES_RAW } from '../data/mock_agents';

import type { Role } from '../contracts/role/domain';


/**
 * Role_State
 * State structure for the neural role registry.
 */
export interface Role_State {
    /** Map of role IDs (slugs) to their corresponding technical definitions */
    roles: Record<string, Role>;
}

/**
 * Role_Actions
 * Operations available to modify the system-level role library.
 */
export interface Role_Actions {
    add_role: (definition: Role) => void;
    update_role: (id: string, definition: Partial<Role>) => void;
    delete_role: (id: string) => void;
    set_roles: (roles: Role[]) => void;
}

// Helper to convert old mock data to new shape during initialization
const normalize_initial_roles = (): Record<string, Role> => {
    const normalized: Record<string, Role> = {};
    Object.entries(INITIAL_ROLES_RAW).forEach(([name, raw]) => {
        const id = name.toLowerCase().replace(/\s+/g, '-');
        normalized[id] = {
            id,
            name,
            department: 'Operations',
            description: `Standard ${name} role.`,
            skills: raw.skills || [],
            workflows: raw.workflows || [],
            mcp_tools: [],
            requires_oversight: false,
            created_at: new Date().toISOString()
        };
    });
    return normalized;
};

/**
 * use_role_store
 * 
 * A reactive, persistent store for managing agent "Blueprints."
 * This store serves as the organizational governance layer, allowing users
 * to define, promote, and customize technical roles for the agent swarm.
 */
export const use_role_store = create<Role_State & Role_Actions>()(
    persist(
        (set) => ({
            roles: normalize_initial_roles(),

            add_role: (definition) => {
                set((state) => ({
                    roles: { ...state.roles, [definition.id]: definition }
                }));
            },

            update_role: (id, updates) => {
                set((state) => {
                    const existing = state.roles[id];
                    if (!existing) return state;
                    return {
                        roles: { 
                            ...state.roles, 
                            [id]: { ...existing, ...updates } 
                        }
                    };
                });
            },

            delete_role: (id) => {
                set((state) => {
                    const new_roles = { ...state.roles };
                    delete new_roles[id];
                    return { roles: new_roles };
                });
            },

            set_roles: (roles_list) => {
                const new_roles: Record<string, Role> = {};
                roles_list.forEach(r => {
                    new_roles[r.id] = r;
                });
                set({ roles: new_roles });
            }
        }),
        {
            name: 'tadpole-roles-storage-v2', // Versioned to trigger fresh hydration or use migrate
            version: 2,
            migrate: (persistedState: unknown, version: number) => {
                if (version === 1) {
                    // Migration from v1 (Record<string, {skills, workflows}>) to v2
                    const old_state = persistedState as { roles?: Record<string, { id?: string; name?: string; skills?: string[]; workflows?: string[] }> };
                    const old_roles = old_state?.roles || {};
                    const new_roles: Record<string, Role> = {};
                    
                    Object.entries(old_roles).forEach(([name, raw]: [string, { skills?: string[]; workflows?: string[] }]) => {
                        const id = name.toLowerCase().replace(/\s+/g, '-');
                        new_roles[id] = {
                            id,
                            name,
                            department: 'Operations',
                            description: `Migrated ${name} role.`,
                            skills: raw.skills || [],
                            workflows: raw.workflows || [],
                            mcp_tools: [],
                            requires_oversight: false,
                            created_at: new Date().toISOString()
                        };
                    });
                    return { roles: new_roles };
                }
                return persistedState;
            }
        }
    )
);

export const select_role_names = (state: Role_State) => 
    Object.values(state.roles || {}).map(r => r.name).sort();

export const select_role_list = (state: Role_State) => 
    Object.values(state.roles || {}).sort((a, b) => a.name.localeCompare(b.name));

// Metadata: [role_store]

// Metadata: [role_store]
