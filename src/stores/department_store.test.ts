/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Swarm Departments registry.** 
 * Verifies dynamic department configuration, addition, modification, and deletion.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Duplicate department additions or incorrect rename mappings.
 * - **Telemetry Link**: Search `[department_store.test]` in tracing logs.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { use_department_store } from './department_store';

describe('use_department_store', () => {
    beforeEach(() => {
        // Reset to initial departments
        use_department_store.setState({
            departments: [
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
            ]
        });
    });

    it('should initialize with default departments', () => {
        const state = use_department_store.getState();
        expect(state.departments).toContain('Engineering');
        expect(state.departments).toContain('Executive');
        expect(state.departments.length).toBe(14);
    });

    it('should allow adding a new department', () => {
        use_department_store.getState().add_department('Legal');
        const state = use_department_store.getState();
        expect(state.departments).toContain('Legal');
        expect(state.departments.length).toBe(15);
    });

    it('should reject duplicate departments case-insensitively', () => {
        use_department_store.getState().add_department('engineering');
        const state = use_department_store.getState();
        expect(state.departments.length).toBe(14);
    });

    it('should allow editing a department name', () => {
        use_department_store.getState().edit_department('Success', 'Customer Success');
        const state = use_department_store.getState();
        expect(state.departments).toContain('Customer Success');
        expect(state.departments).not.toContain('Success');
    });

    it('should allow deleting a department', () => {
        use_department_store.getState().delete_department('Finance');
        const state = use_department_store.getState();
        expect(state.departments).not.toContain('Finance');
        expect(state.departments.length).toBe(13);
    });
});

// Metadata: [department_store_test]
