/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification Suite**: System Validation Engine. 
 * Validates the strict enforcement of system identifiers, numeric governance limits, and network endpoint security (protocol check). 
 * Ensures malformed or malicious inputs are intercepted before reaching the Rust engine.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Unexpected sanitization of valid characters or incorrect limit boundary checks (off-by-one).
 * - **Telemetry Link**: Run `npm run test` or check `[validation_utils.test]` in Vitest results.
 */

import { describe, it, expect } from 'vitest';
import { ValidationUtils } from './validation_utils';

describe('ValidationUtils', () => {
    describe('sanitize_id', () => {
        it('normalizes strings to lowercase kebab-case', () => {
            expect(ValidationUtils.sanitize_id('My Name 123!')).toBe('my-name-123');
        });

        it('removes leading and trailing hyphens', () => {
            expect(ValidationUtils.sanitize_id('---test---')).toBe('test');
        });

        it('replaces multiple hyphens with a single one', () => {
            expect(ValidationUtils.sanitize_id('test   name')).toBe('test-name');
        });
    });

    describe('is_valid_name', () => {
        it('validates length constraints (2-64 chars)', () => {
            expect(ValidationUtils.is_valid_name('A')).toBe(false);
            expect(ValidationUtils.is_valid_name('Valid')).toBe(true);
            expect(ValidationUtils.is_valid_name(' '.repeat(10))).toBe(false);
        });
    });

    describe('is_valid_limit', () => {
        it('enforces numeric boundaries', () => {
            expect(ValidationUtils.is_valid_limit(50, 0, 100)).toBe(true);
            expect(ValidationUtils.is_valid_limit(150, 0, 100)).toBe(false);
            expect(ValidationUtils.is_valid_limit(undefined)).toBe(true);
        });
    });

    describe('is_valid_temperature', () => {
        it('enforces 0.0 to 2.0 range', () => {
            expect(ValidationUtils.is_valid_temperature(0.7)).toBe(true);
            expect(ValidationUtils.is_valid_temperature(2.1)).toBe(false);
            expect(ValidationUtils.is_valid_temperature(-0.1)).toBe(false);
        });
    });

    describe('is_valid_url', () => {
        it('validates http and https protocols', () => {
            expect(ValidationUtils.is_valid_url('https://api.openai.com')).toBe(true);
            expect(ValidationUtils.is_valid_url('http://localhost:8080')).toBe(true);
            expect(ValidationUtils.is_valid_url('ftp://server.com')).toBe(false);
            expect(ValidationUtils.is_valid_url('not-a-url')).toBe(false);
            expect(ValidationUtils.is_valid_url(undefined)).toBe(true);
        });
    });
});

// Metadata: [validation_utils_test]

// Metadata: [validation_utils_test]
