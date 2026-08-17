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
import { sanitize_object } from './sanitizer';

describe('sanitize_object', () => {
    it('returns primitives and null as-is', () => {
        expect(sanitize_object(null)).toBeNull();
        expect(sanitize_object(undefined)).toBeUndefined();
        expect(sanitize_object(42)).toBe(42);
        expect(sanitize_object('hello')).toBe('hello');
        expect(sanitize_object(true)).toBe(true);
    });

    it('recursively sanitizes arrays', () => {
        const input = [1, 'two', { safe: true, constructor: 'bad' }];
        const sanitized = sanitize_object(input) as unknown[];
        expect(Array.isArray(sanitized)).toBe(true);
        expect(sanitized[0]).toBe(1);
        expect(sanitized[1]).toBe('two');
        expect(sanitized[2]).toEqual({ safe: true });
    });

    it('strips prototype pollution keys (__proto__, prototype, constructor)', () => {
        const malicious = JSON.parse('{"valid": 1, "__proto__": {"polluted": true}, "constructor": "fn", "prototype": "obj"}');
        const clean = sanitize_object(malicious) as Record<string, unknown>;

        expect(clean.valid).toBe(1);
        expect('__proto__' in clean).toBe(false);
        expect('constructor' in clean).toBe(false);
        expect('prototype' in clean).toBe(false);
    });

    it('recursively cleans nested objects', () => {
        const nested = {
            level1: {
                safeKey: 'ok',
                __proto__: { bad: true },
                level2: {
                    nestedSafe: 123,
                    constructor: 'danger'
                }
            }
        };

        const clean = sanitize_object(nested) as any;
        expect(clean.level1.safeKey).toBe('ok');
        expect(clean.level1.level2.nestedSafe).toBe(123);
        expect('__proto__' in clean.level1).toBe(false);
        expect('constructor' in clean.level1.level2).toBe(false);
    });
});
