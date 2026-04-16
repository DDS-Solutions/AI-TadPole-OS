/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Personas and Role-Based Access Control (RBAC) definitions. 
 * Orchestrates agent behavioral templates, system-prompt defaults, and sectoral access scopes.
 * 
 * ### 🔍 Debugging & Observability
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ROLE_ACTIONS as INITIAL_ROLES } from '../data/mock_agents';

/**
 * Role_Definition
 * Represents the technical and operational blueprint for an agent role.
 */
export interface Role_Definition {
    /** List of specialized AI skills assigned to this role */
    skills: string[];
    /** List of high-level operational workflows this role can execute */
    workflows: string[];
}

/**
 * Role_State
 * State structure for the neural role registry.
 */
export interface Role_State {
    /** Map of role names to their corresponding technical definitions */
    roles: Record<string, Role_Definition>;
}

/**
 * Role_Actions
 * Operations available to modify the system-level role library.
 */
export interface Role_Actions {
    /** 
     * Adds a new role to the system library. 
     * @param name - The unique identifier/name for the role.
     * @param definition - The skill/workflow blueprint for the role.
     */
    add_role: (name: string, definition: Role_Definition) => void;
    /** 
     * Updates an existing role definition.
     * @param name - The name of the role to update.
     * @param definition - The new blueprint data.
     */
    update_role: (name: string, definition: Role_Definition) => void;
    /** 
     * Permanently removes a role from the system library.
     * @param name - The name of the role to delete.
     */
    delete_role: (name: string) => void;
}

/**
 * use_role_store
 * 
 * A reactive, persistent store for managing agent "Blueprints."
 * This store serves as the organizational governance layer, allowing users
 * to define, promote, and customize technical roles for the agent swarm.
 * 
 * Pattern: NeuralRegistry (State Persistence)
 */
export const use_role_store = create<Role_State & Role_Actions>()(
    persist(
        (set) => ({
            roles: INITIAL_ROLES,

            add_role: (name, definition) => {
                set((state) => ({
                    roles: { ...state.roles, [name]: definition }
                }));
            },

            update_role: (name, definition) => {
                set((state) => ({
                    roles: { ...state.roles, [name]: definition }
                }));
            },

            delete_role: (name) => {
                set((state) => {
                    const new_roles = { ...state.roles };
                    delete new_roles[name];
                    return { roles: new_roles };
                });
            },
        }),
        {
            name: 'tadpole-roles-storage',
        }
    )
);

/**
 * select_role_names
 * Selector to retrieve a sorted list of unique role identifiers.
 */
export const select_role_names = (state: Role_State) => Object.keys(state.roles || {}).sort();

/**
 * select_roles
 * Selector to retrieve the entire role registry map.
 */
export const select_roles = (state: Role_State) => state.roles;
