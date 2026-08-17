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
import { is_allowed_origin } from './origin_guard';

describe('is_allowed_origin', () => {
    it('allows loopback hostnames (localhost, 127.0.0.1, [::1])', () => {
        expect(is_allowed_origin('http://localhost:8000')).toBe(true);
        expect(is_allowed_origin('ws://127.0.0.1:8000')).toBe(true);
        expect(is_allowed_origin('http://[::1]:8000')).toBe(true);
    });

    it('rejects unlisted external origins', () => {
        expect(is_allowed_origin('http://evil.com')).toBe(false);
        expect(is_allowed_origin('https://attacker.org:8080')).toBe(false);
    });

    it('allows explicit runtime allowed origins', () => {
        const allowed = ['https://trusted.domain.com', 'api.internal.net'];
        expect(is_allowed_origin('https://trusted.domain.com/ws', allowed)).toBe(true);
        expect(is_allowed_origin('http://api.internal.net:3000', allowed)).toBe(true);
        expect(is_allowed_origin('http://other.domain.com', allowed)).toBe(false);
    });

    it('handles malformed URLs safely by returning false', () => {
        expect(is_allowed_origin('not-a-valid-url')).toBe(false);
        expect(is_allowed_origin('')).toBe(false);
    });
});
