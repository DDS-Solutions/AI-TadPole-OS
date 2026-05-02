/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification Suite**: AAAK Swarm Dialect Decoder. 
 * Ensures 100% accuracy in translating high-efficiency swarm markers into human-readable UI strings. 
 * Validates weather data parsing and fallback logic for untagged text streams.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Regex collision with user-generated text or unmapped AAAK codes causing translation drift.
 * - **Telemetry Link**: Run `npm run test` or check Vitest results for `aaak_decoder.test`.
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
    });

    describe('isAAAK', () => {
        it('detects AAAK strings correctly', () => {
            expect(isAAAK("*ok*")).toBe(true);
            expect(isAAAK("RES: something")).toBe(true);
            expect(isAAAK("GOAL: focus")).toBe(true);
            expect(isAAAK("WTR| high")).toBe(true);
        });

        it('returns false for non-AAAK strings', () => {
            expect(isAAAK("Hello world")).toBe(false);
            expect(isAAAK("Status: OK")).toBe(false); // No asterisk
        });
    });
});

// Metadata: [aaak_decoder_test]

// Metadata: [aaak_decoder_test]
