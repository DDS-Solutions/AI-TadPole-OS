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
 * Behaviorally exercises `use_settings_store` to prove that:
 * 1. Saving a real API key keeps it in memory while blanking `tadpole_os_api_key` in localStorage.
 * 2. Non-secret preferences persist to localStorage intact.
 * 3. Legacy dev tokens are rejected by `is_valid_api_key` and stripped on save/rehydration.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { use_settings_store, is_valid_api_key } from '../../src/stores/settings_store';

describe('INV-SEC-001: Settings store token exclusion from persistence', () => {
    beforeEach(() => {
        localStorage.clear();
        use_settings_store.setState(use_settings_store.getInitialState(), true);
    });

    it('blanks tadpole_os_api_key and tadpole_os_admin_key in localStorage while preserving non-secret settings', () => {
        const secretToken = 'sk-sovereign-runtime-secret-token-9876543210';
        const adminSecret = 'sk-sovereign-admin-stepup-secret-9876543210';
        const current = use_settings_store.getState().settings;
        const err = use_settings_store.getState().save_settings({
            ...current,
            tadpole_os_api_key: secretToken,
            tadpole_os_admin_key: adminSecret,
            theme: 'emerald',
        });

        expect(err).toBeNull();
        // Both tokens remain available in memory for the live session
        expect(use_settings_store.getState().settings.tadpole_os_api_key).toBe(secretToken);
        expect(use_settings_store.getState().settings.tadpole_os_admin_key).toBe(adminSecret);

        // Persisted localStorage payload MUST NEVER contain either raw token
        const rawPersisted = localStorage.getItem('tadpole_settings');
        expect(rawPersisted).not.toBeNull();
        expect(rawPersisted!).not.toContain(secretToken);
        expect(rawPersisted!).not.toContain(adminSecret);

        const parsed = JSON.parse(rawPersisted!) as {
            state: { settings: { tadpole_os_api_key: string; tadpole_os_admin_key: string; theme: string } };
        };
        expect(parsed.state.settings.tadpole_os_api_key).toBe('');
        expect(parsed.state.settings.tadpole_os_admin_key).toBe('');
        expect(parsed.state.settings.theme).toBe('emerald');
    });

    it('rejects legacy development tokens and strips them on rehydration', async () => {
        expect(is_valid_api_key('my-secure-token-123')).toBe(false);
        expect(is_valid_api_key('short')).toBe(false);
        expect(is_valid_api_key('valid-sovereign-admin-token-2026')).toBe(true);

        // Seed poisoned legacy token in localStorage and trigger Zustand rehydration
        localStorage.setItem(
            'tadpole_settings',
            JSON.stringify({
                state: {
                    settings: {
                        ...use_settings_store.getState().settings,
                        tadpole_os_api_key: 'my-secure-token-123',
                    },
                },
                version: 0,
            })
        );

        await use_settings_store.persist.rehydrate();
        expect(use_settings_store.getState().settings.tadpole_os_api_key).toBe('');
    });
});
