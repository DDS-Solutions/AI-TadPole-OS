/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Neural Vault and Secure Key management store.** 
 * Verifies the handling of session tokens, master key state (In-Memory only, no sessionStorage/SEC-02), and the orchestration of multi-tab vault synchronization via `BroadcastChannel`. 
 * Mocks `crypto_service` and `BroadcastChannel` to isolate security primitives from environment-specific side-effects and ensuring consistent encryption/decryption logic.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Unauthorized access if a vault segment is unsealed without a valid Merkle proof or failure to rotate session keys during a security breach.
 * - **Telemetry Link**: Search `[vault_store.test]` in tracing logs.
 */


/**
 * @file vault_store.test.ts
 * @description Suite for the Secure Credential Vault and API Key Management.
 * @module Stores/VaultStore
 * @testedBehavior
 * - Master Key Lifecycle: Verification of lock/unlock mechanisms.
 * - Credential Security: Validation that master keys are NEVER persisted to sessionStorage (SEC-02).
 * - Multi-Tab Sync: Orchestration of vault state across browser tabs via BroadcastChannel.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks crypto_service for encryption/decryption side-effects.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';

vi.hoisted(() => {
    const post_message = vi.fn();
    (globalThis as any).bc_post_message = post_message;
    (globalThis as any).BroadcastChannel = class {
        onmessage = null;
        postMessage(data: any) {
            (globalThis as any).bc_post_message(data);
        }
        close() {}
    };
});

const bc_post_message = (globalThis as any).bc_post_message;

import { use_vault_store } from './vault_store';

// Mock Crypto_Service via its dependency on crypto_utils or similar
vi.mock('../services/crypto_service', () => ({
    Crypto_Service: {
        encrypt_data: vi.fn().mockImplementation((val) => `encrypted_${val}`),
        decrypt_data: vi.fn().mockImplementation((val) => val.replace('encrypted_', '')),
        verify_master_key: vi.fn().mockResolvedValue(true),
        generate_id: vi.fn().mockReturnValue('test-id')
    }
}));

import { Crypto_Service } from '../services/crypto_service';

describe('use_vault_store', () => {
    const SESSION_KEY = 'tadpole-vault-master-key';

    beforeEach(() => {
        bc_post_message.mockClear();
        // Clear sessionStorage
        sessionStorage.clear();

        use_vault_store.setState({
            is_locked: true,
            master_key: null,
            encrypted_configs: {},
            inactivity_timeout: 300000,
        });
        vi.clearAllMocks();
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
        vi.unstubAllGlobals();
    });

    describe('Lock/Unlock Mechanism with Sync', () => {
        it('unlocks and updates BroadcastChannel (No sessionStorage)', async () => {
            const store = use_vault_store.getState();
            await store.unlock('my-password');

            // SEC-02: Verify NO persistence in sessionStorage
            expect(sessionStorage.getItem(SESSION_KEY)).toBeNull();
            expect(bc_post_message).toHaveBeenCalledWith(expect.objectContaining({ type: 'UNLOCK', payload: 'my-password' }));
            expect(use_vault_store.getState().is_locked).toBe(false);
        });

        it('locks and clears BroadcastChannel', () => {
            use_vault_store.setState({ is_locked: false, master_key: 'secret' });
            const store = use_vault_store.getState();

            store.lock();

            expect(sessionStorage.getItem(SESSION_KEY)).toBeNull();
            expect(bc_post_message).toHaveBeenCalledWith(expect.objectContaining({ type: 'LOCK' }));
            expect(use_vault_store.getState().is_locked).toBe(true);
        });

        it('does NOT rehydrate from sessionStorage in get_api_key (SEC-02)', async () => {
            // Store is locked in memory. We attempt to "seed" sessionStorage (simulating an old version or attack)
            sessionStorage.setItem(SESSION_KEY, 'stale-key');
            use_vault_store.setState({ 
                is_locked: true, 
                master_key: null,
                encrypted_configs: { 'test': 'encrypted_val' }
            });

            const store = use_vault_store.getState();
            const key = await store.get_api_key('test');

            // Should fail to decrypt because it no longer looks at sessionStorage
            expect(key).toBeNull();
            expect(use_vault_store.getState().is_locked).toBe(true);
            expect(use_vault_store.getState().master_key).toBeNull();
        });
    });

    describe('Config Management', () => {
        it('encrypts new api key when unlocked', async () => {
            use_vault_store.setState({ is_locked: false, master_key: 'mypass' });
            
            const store = use_vault_store.getState();
            await store.set_encrypted_config('openai', 'sk-secret123');

            const state = use_vault_store.getState();
            // Standardizes to lowercase
            expect(state.encrypted_configs['openai']).toBe('encrypted_sk-secret123');
            expect(Crypto_Service.encrypt_data).toHaveBeenCalledWith('sk-secret123', 'mypass');
        });

        it('gets decrypted API key (case-insensitive)', async () => {
            use_vault_store.setState({ 
                is_locked: false, 
                master_key: 'mypass',
                encrypted_configs: { 'inception': 'encrypted_super-secret' }
            });

            const store = use_vault_store.getState();
            // Should work with uppercase too
            const key = await store.get_api_key('INCEPTION');

            expect(key).toBe('super-secret');
            expect(Crypto_Service.decrypt_data).toHaveBeenCalledWith('encrypted_super-secret', 'mypass');
        });
    });
});


// Metadata: [vault_store_test]

// Metadata: [vault_store_test]
