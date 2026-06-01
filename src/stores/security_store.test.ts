/**
 * @docs ARCHITECTURE:UI-Stores
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[security_store_test]` in observability traces.
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
});

// Metadata: [security_store_test]
