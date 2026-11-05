/**
 * @docs ARCHITECTURE:Security
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_sec_002_session_storage.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-SEC-002: Session Storage — No plain-text auth tokens in sessionStorage.
 *
 * Asserts that the codebase does not store raw authentication tokens
 * (API keys, JWTs, Bearer tokens) in sessionStorage, which is accessible
 * to any JavaScript running in the same origin.
 */

import { describe, it, expect } from 'vitest';
import { resolve } from 'node:path';

describe('INV-SEC-002: No plain-text auth tokens in sessionStorage', () => {
    it('no source file writes auth tokens to sessionStorage', async () => {
        const { readFileSync } = await import('node:fs');
        const { globSync } = await import('node:fs');

        const source_files = globSync('src/**/*.{ts,tsx}');
        const violations: string[] = [];

        const SESSION_STORAGE_TOKEN_PATTERN =
            /sessionStorage\.(setItem|set)\s*\(\s*['"`].*(?:token|key|secret|auth|jwt|bearer)/i;

        for (const file of source_files) {
            const content = readFileSync(resolve(file), 'utf-8');
            if (SESSION_STORAGE_TOKEN_PATTERN.test(content)) {
                violations.push(file);
            }
        }

        expect(violations).toEqual([]);
    });

    it('no source file reads auth tokens from sessionStorage for credential storage', async () => {
        const { readFileSync } = await import('node:fs');
        const { globSync } = await import('node:fs');

        const source_files = globSync('src/**/*.{ts,tsx}')
            .filter((f: string) => !f.includes('.test.') && !f.includes('.spec.'));
        const session_storage_uses: string[] = [];

        for (const file of source_files) {
            const content = readFileSync(resolve(file), 'utf-8');
            // Check for sessionStorage usage patterns that suggest token storage
            if (content.includes('sessionStorage') && /(?:token|api_key|auth|jwt|bearer)/i.test(content)) {
                const lines = content.split('\n');
                for (let i = 0; i < lines.length; i++) {
                    const line = lines[i].trim();
                    // Skip comment lines — they explain WHY sessionStorage is NOT used
                    if (line.startsWith('//') || line.startsWith('*') || line.startsWith('/*')) continue;

                    if (line.includes('sessionStorage') &&
                        /(?:token|api_key|auth|jwt|bearer)/i.test(line)) {
                        session_storage_uses.push(`${file}:${i + 1}`);
                    }
                }
            }
        }

        expect(session_storage_uses).toEqual([]);
    });
});
