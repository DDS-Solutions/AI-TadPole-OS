import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ROLE_ACTIONS as INITIAL_ROLES } from '../data/mockAgents';

/**
 * RoleDefinition
 * Represents the technical and operational blueprint for an agent role.
 */
export interface RoleDefinition {
    /** List of specialized AI capabilities assigned to this role */
    capabilities: string[];
    /** List of high-level operational workflows this role can execute */
    workflows: string[];
}

/**
 * RoleState
 * State structure for the neural role registry.
 */
interface RoleState {
    /** Map of role names to their corresponding technical definitions */
    roles: Record<string, RoleDefinition>;
}

/**
 * RoleActions
 * Operations available to modify the system-level role library.
 */
interface RoleActions {
    /** 
     * Adds a new role to the system library. 
     * @param name - The unique identifier/name for the role.
     * @param definition - The skill/workflow blueprint for the role.
     */
    addRole: (name: string, definition: RoleDefinition) => void;
    /** 
     * Updates an existing role definition.
     * @param name - The name of the role to update.
     * @param definition - The new blueprint data.
     */
    updateRole: (name: string, definition: RoleDefinition) => void;
    /** 
     * Permanently removes a role from the system library.
     * @param name - The name of the role to delete.
     */
    deleteRole: (name: string) => void;
}

/**
 * useRoleStore
 * 
 * A reactive, persistent store for managing agent "Blueprints."
 * This store serves as the organizational governance layer, allowing users
 * to define, promote, and customize technical roles for the agent swarm.
 * 
 * Pattern: NeuralRegistry (State Persistence)
 */
export const useRoleStore = create<RoleState & RoleActions>()(
    persist(
        (set) => ({
            roles: INITIAL_ROLES,

            addRole: (name, definition) => {
                set((state) => ({
                    roles: { ...state.roles, [name]: definition }
                }));
            },

            updateRole: (name, definition) => {
                set((state) => ({
                    roles: { ...state.roles, [name]: definition }
                }));
            },

            deleteRole: (name) => {
                set((state) => {
                    const newRoles = { ...state.roles };
                    delete newRoles[name];
                    return { roles: newRoles };
                });
            },
        }),
        {
            name: 'tadpole-roles-storage',
        }
    )
);

/**
 * selectRoleNames
 * Selector to retrieve a sorted list of unique role identifiers.
 */
export const selectRoleNames = (state: RoleState) => Object.keys(state.roles).sort();

/**
 * selectRoles
 * Selector to retrieve the entire role registry map.
 */
export const selectRoles = (state: RoleState) => state.roles;
