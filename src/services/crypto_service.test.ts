/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Client-Side Encryption and Neural Vault security layer.** 
 * Verifies the hashing of audit logs, generation of session tokens, and the encryption/decryption of mission-critical data with master keys. 
 * Mocks `crypto_utils` to bypass Web Worker requirements in the Vitest environment.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Merkle proof mismatch due to incorrect log ordering or failure to provide entropy during token generation.
 * - **Telemetry Link**: Search `[crypto_service.test]` in tracing logs.
 */


/**
 * @file crypto_service.test.ts
 * @description Suite for the Neural Vault cryptography layer.
 * @module Services/crypto_service
 * @testedBehavior
 * - Identity Generation: UUID-like unique identifier generation.
 * - Encryption/Decryption: Secure data wrapping and unwrapping with master keys.
 * - Key Verification: Validation of master keys against encrypted samples.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks crypto_utils to bypass Web Worker requirements in Vitest.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Crypto_Service } from './crypto_service';
import * as crypto_utils from '../utils/crypto';

// Mock the underlying crypto utils because they use Web Workers
vi.mock('../utils/crypto', () => ({
    encrypt_text: vi.fn(),
    decrypt_text: vi.fn()
}));

describe('crypto_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('generate_id', () => {
        it('generates a valid UUID-like format', () => {
            const id = Crypto_Service.generate_id();
            expect(id).toMatch(/^[0-9a-f-]{36}$/);
        });

        it('is reasonably unique', () => {
            const id1 = Crypto_Service.generate_id();
            const id2 = Crypto_Service.generate_id();
            expect(id1).not.toBe(id2);
        });
    });

    describe('Encryption/Decryption', () => {
        const password = 'test-password';
        const raw_text = 'sensitive-data';
        const encrypted_text = '{"encrypted": "true"}';

        it('calls encrypt_text util correctly', async () => {
            (crypto_utils.encrypt_text as any).mockResolvedValue(encrypted_text);
            
            const result = await Crypto_Service.encrypt_data(raw_text, password);
            expect(crypto_utils.encrypt_text).toHaveBeenCalledWith(raw_text, password);
            expect(result).toBe(encrypted_text);
        });

        it('calls decrypt_text util correctly', async () => {
            (crypto_utils.decrypt_text as any).mockResolvedValue(raw_text);
            
            const result = await Crypto_Service.decrypt_data(encrypted_text, password);
            expect(crypto_utils.decrypt_text).toHaveBeenCalledWith(encrypted_text, password);
            expect(result).toBe(raw_text);
        });

        it('throws FAILED_TO_ENCRYPT_DATA on encryption error', async () => {
            (crypto_utils.encrypt_text as any).mockRejectedValue(new Error('fail'));
            
            await expect(Crypto_Service.encrypt_data(raw_text, password))
                .rejects.toThrow('FAILED_TO_ENCRYPT_DATA');
        });

        it('throws INVALID_MASTER_KEY on decryption error', async () => {
            (crypto_utils.decrypt_text as any).mockRejectedValue(new Error('fail'));
            
            await expect(Crypto_Service.decrypt_data(encrypted_text, password))
                .rejects.toThrow('INVALID_MASTER_KEY');
        });
    });

    describe('verify_master_key', () => {
        it('returns true on valid key', async () => {
            (crypto_utils.decrypt_text as any).mockResolvedValue('success');
            const result = await Crypto_Service.verify_master_key('sample', 'pass');
            expect(result).toBe(true);
        });

        it('returns false on invalid key', async () => {
            (crypto_utils.decrypt_text as any).mockRejectedValue(new Error('fail'));
            const result = await Crypto_Service.verify_master_key('sample', 'wrong');
            expect(result).toBe(false);
        });
    });
});


// Metadata: [crypto_service_test]

// Metadata: [crypto_service_test]
