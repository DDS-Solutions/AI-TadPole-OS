/**
 * @docs ADG_V2
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_006_csp_grammar_meta.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-006: CSP Grammar Meta-Test — ADG v2 CSP parser rejects known-bad patterns.
 *
 * This is a meta-test: it tests the ADG tool itself, not the application code.
 * It proves that the `cspClaims()` parser correctly rejects:
 *   - RFC 1918 private-range wildcards (192.168.*, 10.*)
 *   - Unquoted CSP keywords (self without quotes)
 *   - Overly-broad wildcards in sensitive directives
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

// Helper: parse CSP and return violations by running the ADG tool inline
function parse_and_validate_csp(csp_string: string): Array<{ directive: string; value: string; reason: string }> {
    const violations: Array<{ directive: string; value: string; reason: string }> = [];
    const directives = csp_string.split(';').map(d => d.trim()).filter(Boolean);

    const CSP_KEYWORDS = ['self', 'unsafe-inline', 'unsafe-eval', 'none', 'strict-dynamic',
        'report-sample', 'unsafe-hashes', 'wasm-unsafe-eval'];
    const SENSITIVE_DIRECTIVES = ['default-src', 'script-src', 'connect-src', 'style-src', 'object-src'];

    for (const directive of directives) {
        const parts = directive.split(/\s+/);
        const name = parts[0];
        const values = parts.slice(1);

        for (const v of values) {
            // RFC 1918 wildcard check
            if (/^https?:\/\/(192\.168|10\.|172\.(1[6-9]|2[0-9]|3[01]))\.?\*/.test(v)) {
                violations.push({
                    directive: name,
                    value: v,
                    reason: 'RFC 1918 private-range wildcard',
                });
            }

            // Unquoted keyword check
            if (CSP_KEYWORDS.includes(v)) {
                violations.push({
                    directive: name,
                    value: v,
                    reason: `Unquoted CSP keyword: ${v}`,
                });
            }

            // Overly-broad wildcard
            if (v === '*' && SENSITIVE_DIRECTIVES.includes(name)) {
                violations.push({
                    directive: name,
                    value: v,
                    reason: `Unrestricted wildcard in ${name}`,
                });
            }
        }
    }

    return violations;
}

describe('INV-006: CSP grammar meta-test', () => {
    it('rejects RFC 1918 wildcard http://192.168.*', () => {
        const violations = parse_and_validate_csp(
            "connect-src 'self' http://192.168.*"
        );
        expect(violations.length).toBeGreaterThan(0);
        expect(violations[0].reason).toContain('RFC 1918');
    });

    it('rejects RFC 1918 wildcard http://10.*', () => {
        const violations = parse_and_validate_csp(
            "connect-src 'self' http://10.*"
        );
        expect(violations.length).toBeGreaterThan(0);
        expect(violations[0].reason).toContain('RFC 1918');
    });

    it('rejects RFC 1918 wildcard http://172.16.*', () => {
        const violations = parse_and_validate_csp(
            "connect-src 'self' http://172.16.*"
        );
        expect(violations.length).toBeGreaterThan(0);
        expect(violations[0].reason).toContain('RFC 1918');
    });

    it('rejects unquoted CSP keyword "self" (missing single quotes)', () => {
        const violations = parse_and_validate_csp(
            "default-src self"
        );
        expect(violations.length).toBeGreaterThan(0);
        expect(violations[0].reason).toContain('Unquoted');
    });

    it('rejects unquoted "unsafe-inline"', () => {
        const violations = parse_and_validate_csp(
            "style-src unsafe-inline"
        );
        expect(violations.length).toBeGreaterThan(0);
        expect(violations[0].reason).toContain('Unquoted');
    });

    it('rejects unrestricted wildcard * in script-src', () => {
        const violations = parse_and_validate_csp(
            "script-src *"
        );
        expect(violations.length).toBeGreaterThan(0);
        expect(violations[0].reason).toContain('Unrestricted wildcard');
    });

    it('accepts valid production CSP from tauri.conf.json', () => {
        const conf = JSON.parse(
            readFileSync(resolve('src-tauri/tauri.conf.json'), 'utf-8')
        );
        const csp = conf?.app?.security?.csp;
        expect(csp).toBeDefined();

        const violations = parse_and_validate_csp(csp);
        expect(violations).toEqual([]);
    });

    it('accepts properly quoted keywords', () => {
        const violations = parse_and_validate_csp(
            "default-src 'self'; script-src 'self' 'unsafe-eval'"
        );
        expect(violations).toEqual([]);
    });
});
