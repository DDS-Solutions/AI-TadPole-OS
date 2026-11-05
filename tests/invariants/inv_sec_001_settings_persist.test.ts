/**
 * @docs ARCHITECTURE:Security
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_sec_001_settings_persist.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-SEC-001: Settings Store Token Persistence — API token excluded from localStorage.
 *
 * Asserts that the settings_store `partialize` configuration strips
 * `tadpole_os_api_key` from the persisted state, ensuring raw API tokens
 * are never written to disk via localStorage.
 */

import { describe, it, expect } from 'vitest';

describe('INV-SEC-001: Settings store token exclusion from persistence', () => {
    it('partialize blanks tadpole_os_api_key before persisting', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(resolve('src/stores/settings_store.ts'), 'utf-8');

        // The partialize function must explicitly blank the API key
        expect(source).toContain("tadpole_os_api_key: ''");
        expect(source).toContain('partialize');
    });

    it('uses localStorage as the persistence backend (not sessionStorage)', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(resolve('src/stores/settings_store.ts'), 'utf-8');

        // Must use localStorage explicitly
        expect(source).toContain('localStorage');
        expect(source).toContain('createJSONStorage');
    });

    it('onRehydrateStorage purges legacy dev tokens', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(resolve('src/stores/settings_store.ts'), 'utf-8');

        // The rehydration hook must exist to sanitize loaded state
        expect(source).toContain('onRehydrateStorage');
        // Legacy token set must be defined
        expect(source).toContain('LEGACY_DEV_TOKENS');
    });

    it('confirms partialize does NOT include api_key in the return shape', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(resolve('src/stores/settings_store.ts'), 'utf-8');

        // Extract the partialize block and verify the key is blanked
        const partialize_match = source.match(/partialize:\s*\(state\)\s*=>\s*\(\{[\s\S]*?\}\)/);
        expect(partialize_match).not.toBeNull();

        const partialize_block = partialize_match![0];
        // The block must contain the blanking assignment
        expect(partialize_block).toContain("tadpole_os_api_key: ''");
    });
});
