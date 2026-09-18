/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / aaak_decoder.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect } from 'vitest';
import { decodeAAAK, isAAAK } from './aaak_decoder';

describe('AAAK Decoder', () => {
    describe('decodeAAAK', () => {
        it('expands all standard AAAK patterns', () => {
            const input = "*ok* *err* RES: FND: SRC: LOC: GOAL: *done* *busy*";
            const output = decodeAAAK(input);
            
            expect(output).toContain('✅ Status: Success');
            expect(output).toContain('❌ Status: Failed');
            expect(output).toContain('🔍 Result:');
            expect(output).toContain('💡 Finding:');
            expect(output).toContain('🌐 Source:');
            expect(output).toContain('📍 Location:');
            expect(output).toContain('🎯 Primary Goal:');
            expect(output).toContain('🏁 Mission Complete');
            expect(output).toContain('🐝 Task in progress');
        });

        it('handles weather and unit data', () => {
            const input = "WTR| 25 deg temp 30";
            const output = decodeAAAK(input);
            
            expect(output).toBe('🌤️ Weather Data:  25 degrees temperature 30');
        });

        it('returns empty string for null/undefined/empty input', () => {
            expect(decodeAAAK("")).toBe("");
            expect(decodeAAAK(null as any)).toBe("");
        });

        it('leaves non-AAAK text untouched', () => {
            const input = "Normal system message without markers.";
            expect(decodeAAAK(input)).toBe(input);
        });

        it('does not corrupt whole words like temperature or degrees', () => {
            const input = "The temperature is 25 degrees in California.";
            expect(decodeAAAK(input)).toBe("The temperature is 25 degrees in California.");
        });
    });

    describe('isAAAK', () => {
        it('detects AAAK strings correctly', () => {
            expect(isAAAK("*ok*")).toBe(true);
            expect(isAAAK("RES: something")).toBe(true);
            expect(isAAAK("GOAL: focus")).toBe(true);
            expect(isAAAK("LOC: server-1")).toBe(true);
            expect(isAAAK("WTR| high")).toBe(true);
        });

        it('returns false for non-AAAK strings', () => {
            expect(isAAAK("Hello world")).toBe(false);
            expect(isAAAK("Status: OK")).toBe(false); // No asterisk
        });
    });
});
