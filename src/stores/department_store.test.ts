/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / department_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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

    it('should allow renaming a department with casing change only', () => {
        use_department_store.getState().edit_department('Engineering', 'engineering');
        const state = use_department_store.getState();
        expect(state.departments).toContain('engineering');
        expect(state.departments).not.toContain('Engineering');
    });

    it('should reject editing to an existing different department', () => {
        use_department_store.getState().edit_department('Success', 'Engineering');
        const state = use_department_store.getState();
        expect(state.departments).toContain('Success');
        expect(state.departments.filter(d => d === 'Engineering')).toHaveLength(1);
    });

    it('should ignore empty or whitespace-only additions and edits', () => {
        use_department_store.getState().add_department('   ');
        expect(use_department_store.getState().departments.length).toBe(14);

        use_department_store.getState().edit_department('Executive', '   ');
        expect(use_department_store.getState().departments).toContain('Executive');
    });

    it('should allow deleting a department', () => {
        use_department_store.getState().delete_department('Finance');
        const state = use_department_store.getState();
        expect(state.departments).not.toContain('Finance');
        expect(state.departments.length).toBe(13);
    });

    it('should handle deleting a non-existent department safely', () => {
        use_department_store.getState().delete_department('NonExistent');
        expect(use_department_store.getState().departments.length).toBe(14);
    });
});
