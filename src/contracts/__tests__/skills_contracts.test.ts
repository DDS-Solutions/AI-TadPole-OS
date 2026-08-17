/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Skills Contract Suite**: Validates static contracts, type predicates, and system registries for skills and workflows.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing slash-command prefixes or registry drift.
 * - **Trace Scope**: Vitest skills contract assertions.
 */

import { describe, it, expect } from 'vitest';
import {
    SYSTEM_SKILLS,
    SYSTEM_WORKFLOWS,
    is_valid_skill,
    is_valid_workflow,
    get_all_system_skills,
    get_all_system_workflows
} from '../skills';

describe('Skills Contracts & Registry Validation', () => {
    it('should validate known system skills with O(1) predicate', () => {
        expect(SYSTEM_SKILLS.length).toBeGreaterThan(10);
        expect(is_valid_skill(SYSTEM_SKILLS[0])).toBe(true);
        expect(is_valid_skill('non_existent_skill_xyz')).toBe(false);
    });

    it('should validate known system workflows with slash-command prefix', () => {
        expect(SYSTEM_WORKFLOWS.length).toBeGreaterThan(10);
        expect(is_valid_workflow('/test')).toBe(true);
        expect(is_valid_workflow('invalid_workflow_without_slash')).toBe(false);
    });

    it('should return sorted arrays of system skills and workflows', () => {
        const skills = get_all_system_skills();
        const workflows = get_all_system_workflows();

        expect(skills.length).toBe(SYSTEM_SKILLS.length);
        expect(workflows.length).toBe(SYSTEM_WORKFLOWS.length);

        // Verify sorted order
        const sorted_skills_copy = [...skills].sort();
        expect(skills).toEqual(sorted_skills_copy);

        const sorted_workflows_copy = [...workflows].sort();
        expect(workflows).toEqual(sorted_workflows_copy);
    });
});
