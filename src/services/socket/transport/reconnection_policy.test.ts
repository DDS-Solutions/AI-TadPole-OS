/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect } from 'vitest';
import { ReconnectionPolicy } from './reconnection_policy';

describe('ReconnectionPolicy', () => {
    it('calculates exponential backoff delay within max limits', () => {
        const policy = new ReconnectionPolicy(1000, 10000, 5);
        expect(policy.get_delay(0)).toBe(1000);
        expect(policy.get_delay(1)).toBe(2000);
        expect(policy.get_delay(2)).toBe(4000);
        expect(policy.get_delay(3)).toBe(8000);
        expect(policy.get_delay(4)).toBe(10000); // capped at max_backoff
        expect(policy.get_delay(10)).toBe(10000); // capped at max_backoff
    });

    it('enforces maximum retry limits', () => {
        const policy = new ReconnectionPolicy(1000, 10000, 3);
        expect(policy.should_retry(0)).toBe(true);
        expect(policy.should_retry(1)).toBe(true);
        expect(policy.should_retry(2)).toBe(true);
        expect(policy.should_retry(3)).toBe(false);
        expect(policy.should_retry(4)).toBe(false);
    });

    it('uses default backoff configurations if omitted', () => {
        const default_policy = new ReconnectionPolicy();
        expect(default_policy.get_delay(0)).toBe(2000);
        expect(default_policy.should_retry(9)).toBe(true);
        expect(default_policy.should_retry(10)).toBe(false);
    });
});
