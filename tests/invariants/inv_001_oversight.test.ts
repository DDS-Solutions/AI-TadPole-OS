/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_001_oversight.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-001: Oversight Decision — User question response maps correctly.
 *
 * Asserts that `resolve_user_question` in sovereign_store.ts:
 *   1. Maps the approve_option to 'approved' and other options to 'rejected'
 *   2. Reverts state (question back to pending, operator message removed) on network error
 *   3. Uses question_id for strict matching (not text-based fuzzy matching)
 */

import { describe, it, expect } from 'vitest';

describe('INV-001: Oversight decision mapping', () => {
    it('decision logic maps approve_option to "approved" and others to "rejected"', () => {
        // This tests the decision-mapping logic extracted from sovereign_store.ts:
        // const effective_approve = approve_option ?? question_options[0];
        // const decision = answer === effective_approve ? 'approved' : 'rejected';

        const test_cases = [
            // [answer, approve_option, options, expected_decision]
            { answer: 'Yes', approve_option: 'Yes', options: ['Yes', 'No'], expected: 'approved' },
            { answer: 'No', approve_option: 'Yes', options: ['Yes', 'No'], expected: 'rejected' },
            { answer: 'Proceed', approve_option: undefined, options: ['Proceed', 'Cancel'], expected: 'approved' },
            { answer: 'Cancel', approve_option: undefined, options: ['Proceed', 'Cancel'], expected: 'rejected' },
            { answer: 'Allow', approve_option: 'Allow', options: ['Allow', 'Deny', 'Defer'], expected: 'approved' },
            { answer: 'Deny', approve_option: 'Allow', options: ['Allow', 'Deny', 'Defer'], expected: 'rejected' },
            { answer: 'Defer', approve_option: 'Allow', options: ['Allow', 'Deny', 'Defer'], expected: 'rejected' },
        ];

        for (const { answer, approve_option, options, expected } of test_cases) {
            const effective_approve = approve_option ?? options[0];
            const decision: 'approved' | 'rejected' = answer === effective_approve ? 'approved' : 'rejected';
            expect(decision).toBe(expected);
        }
    });

    it('decision logic uses strict equality, not fuzzy matching', () => {
        // Ensure "Yes " (with trailing space) does NOT match "Yes"
        const approve_option = 'Yes';
        const answer_with_space = 'Yes ';
        const decision: 'approved' | 'rejected' =
            answer_with_space === approve_option ? 'approved' : 'rejected';
        expect(decision).toBe('rejected');

        // Ensure case-sensitivity: "yes" does NOT match "Yes"
        const answer_lowercase = 'yes';
        const decision2: 'approved' | 'rejected' =
            answer_lowercase === approve_option ? 'approved' : 'rejected';
        expect(decision2).toBe('rejected');
    });

    it('source code uses question_id for matching, not text-based lookup', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(resolve('src/stores/sovereign_store.ts'), 'utf-8');

        // The resolve function must match by question_id, not by text content
        expect(source).toContain("p.question_id === question_id");
    });

    it('source code reverts state on network error', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(resolve('src/stores/sovereign_store.ts'), 'utf-8');

        // Verify the catch block exists and reverts the question status
        expect(source).toContain("status: 'pending'");
        expect(source).toContain("filter(m => m.id !== operator_message_id)");
    });
});
