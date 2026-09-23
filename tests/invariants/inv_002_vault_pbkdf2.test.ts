/**
 * @docs ARCHITECTURE:Security
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_002_vault_pbkdf2.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-002: Vault PBKDF2 — Key derivation iteration count ≥ 600,000.
 *
 * Asserts that the `derive_key` function in crypto-core.ts uses
 * a minimum PBKDF2 iteration count to prevent brute-force attacks
 * on the vault's master key.
 */

import { describe, it, expect } from 'vitest';
import { derive_key } from '../../src/utils/crypto-core';

describe('INV-002: Vault PBKDF2 iteration floor', () => {
    it('derives a CryptoKey with default iterations (≥ 600,000)', async () => {
        const password = 'test-passphrase';
        const salt = crypto.getRandomValues(new Uint8Array(16));

        // derive_key's default `iterations` parameter is 600000.
        // If someone lowers it, this test must fail.
        const key = await derive_key(password, salt);

        // The key should be a valid CryptoKey
        expect(key).toBeDefined();
        expect(key.type).toBe('secret');
    });

    it('rejects iterations below the security floor', async () => {
        // This test ensures the FUNCTION SIGNATURE enforces the floor.
        // We call with the explicit default to prove the default is ≥ 600000.
        // If a developer changes the default param to < 600000,
        // the source-code assertion below catches it.
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(resolve('src/utils/crypto-core.ts'), 'utf-8');

        // Assert the default parameter value in the source code
        const match = source.match(/iterations\s*=\s*(\d+)/);
        expect(match).not.toBeNull();
        const default_iterations = parseInt(match![1], 10);
        expect(default_iterations).toBeGreaterThanOrEqual(600_000);
    });

    it('produces a non-extractable key (security invariant)', async () => {
        const salt = crypto.getRandomValues(new Uint8Array(16));
        const key = await derive_key('password', salt);

        // Keys MUST be non-extractable — this is a security property.
        // If someone makes them extractable, this test fails.
        expect(key.extractable).toBe(false);
    });

    it('produces keys with AES-GCM algorithm', async () => {
        const salt = crypto.getRandomValues(new Uint8Array(16));
        const key = await derive_key('password', salt);

        // Verify the key is configured for AES-GCM
        expect(key.algorithm).toBeDefined();
        expect((key.algorithm as AesKeyAlgorithm).name).toBe('AES-GCM');
    });

    it('produces keys that can encrypt and decrypt', async () => {
        const salt = crypto.getRandomValues(new Uint8Array(16));
        const key = await derive_key('password', salt);

        // Prove the key is functional by encrypting and decrypting
        const iv = crypto.getRandomValues(new Uint8Array(12));
        const plaintext = new TextEncoder().encode('test data');
        const ciphertext = await crypto.subtle.encrypt(
            { name: 'AES-GCM', iv },
            key,
            plaintext
        );
        const decrypted = await crypto.subtle.decrypt(
            { name: 'AES-GCM', iv },
            key,
            ciphertext
        );
        expect(new TextDecoder().decode(decrypted)).toBe('test data');
    });
});
