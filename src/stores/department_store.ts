/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Departments configuration.
 * Orchestrates dynamic department definitions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Failed store hydration or duplicate department naming.
 * - **Telemetry Link**: Search `[DepartmentStore]` in UI traces.
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
    'Success'
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
                    const trimmed_new = new_name.trim();
                    if (!trimmed_new || state.departments.some(d => d.toLowerCase() === trimmed_new.toLowerCase() && d !== old_name)) return state;
                    return {
                        departments: state.departments.map(d => d === old_name ? trimmed_new : d)
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
            version: 1
        }
    )
);

// Metadata: [DepartmentStore]
