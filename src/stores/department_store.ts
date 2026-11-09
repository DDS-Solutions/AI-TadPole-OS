/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / department_store
 * - **Primary Entrypoints**: `use_department_store`, `Department_State`, `Department_Actions`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface Department_State {
    departments: string[];
}

export interface Department_Actions {
    add_department: (name: string) => void;
    edit_department: (old_name: string, new_name: string) => void;
    delete_department: (name: string) => void;
}

const DEFAULT_DEPARTMENTS = [
    'Executive',
    'Engineering',
    'Operations',
    'Product',
    'Marketing',
    'Sales',
    'Design',
    'Research',
    'Support',
    'Quality Assurance',
    'Intelligence',
    'Finance',
    'Growth',
    'Success',
    'Swarm Core'
];

export const use_department_store = create<Department_State & Department_Actions>()(
    persist(
        (set) => ({
            departments: DEFAULT_DEPARTMENTS,

            add_department: (name) => {
                set((state) => {
                    const trimmed = name.trim();
                    if (!trimmed || state.departments.some(d => d.toLowerCase() === trimmed.toLowerCase())) return state;
                    return { departments: [...state.departments, trimmed] };
                });
            },

            edit_department: (old_name, new_name) => {
                set((state) => {
                    const trimmed_old = old_name.trim();
                    const trimmed_new = new_name.trim();
                    if (!trimmed_new || state.departments.some(d => d.toLowerCase() === trimmed_new.toLowerCase() && d.toLowerCase() !== trimmed_old.toLowerCase())) return state;
                    return {
                        departments: state.departments.map(d => d.toLowerCase() === trimmed_old.toLowerCase() ? trimmed_new : d)
                    };
                });
            },

            delete_department: (name) => {
                set((state) => ({
                    departments: state.departments.filter(d => d !== name)
                }));
            }
        }),
        {
            name: 'tadpole-departments-storage-v1',
            version: 2,
            migrate: (persistedState: unknown, version: number) => {
                const state = persistedState as Department_State;
                if (version < 2 && state && Array.isArray(state.departments)) {
                    if (!state.departments.some(d => d.toLowerCase() === 'swarm core')) {
                        return {
                            ...state,
                            departments: [...state.departments, 'Swarm Core']
                        };
                    }
                }
                return state;
            }
        }
    )
);


import { register_store_for_test_reset } from '../utils/store_registry';

register_store_for_test_reset(() => {
    use_department_store.setState(use_department_store.getInitialState(), true);
});
