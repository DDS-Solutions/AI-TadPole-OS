/**
 * @docs ARCHITECTURE:UI-Stores
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / security_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

describe('security_store', () => {
    beforeEach(async () => {
        vi.resetModules();
        vi.clearAllMocks();
    });

    it('generates Ed25519 keys if needed', async () => {
        const { use_security_store } = await import('./security_store');
        
        // Before generation, key should be null
        expect(use_security_store.getState().publicKey).toBeNull();
        expect(use_security_store.getState().privateKey).toBeNull();

        // Trigger generation
        await use_security_store.getState().generate_keys_if_needed();

        const pub = use_security_store.getState().publicKey;
        const priv = use_security_store.getState().privateKey;

        expect(pub).not.toBeNull();
        expect(priv).not.toBeNull();
        expect(typeof pub).toBe('string');
        expect(typeof priv).toBe('string');
        expect(pub!.length).toBeGreaterThan(0);
        expect(priv!.length).toBeGreaterThan(0);
    });

    it('signs human-in-the-loop decisions correctly', async () => {
        const { use_security_store } = await import('./security_store');
        
        await use_security_store.getState().generate_keys_if_needed();
        const res = await use_security_store.getState().sign_oversight('entry-123', 'approved');

        expect(res.signature).toBeDefined();
        expect(res.verifying_key).toBe(use_security_store.getState().publicKey!);
        expect(res.signature.length).toBe(128); // Hex-encoded 64-byte signature
    });

    it('does not persist private key in localStorage', async () => {
        const { use_security_store } = await import('./security_store');
        await use_security_store.getState().generate_keys_if_needed();

        const stored_raw = localStorage.getItem('tadpole_security_keys');
        if (stored_raw) {
            const parsed = JSON.parse(stored_raw);
            expect(parsed.state.privateKey).toBeUndefined();
            expect(parsed.state.publicKey).toBeDefined();
        }
    });
});
