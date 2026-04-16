/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Infrastructure**: Manages the crypto-core tests. 
 * Part of the Tadpole-OS core infrastructure.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Unit test regression (failure in Vitest runner) or pinned crypto algorithm mismatch.
 * - **Telemetry Link**: Run `npm run test` or check Vitest dashboard.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { derive_key, encrypt_raw, decrypt_raw } from './crypto-core';

// Fully mock global browser crypto API
const subtle_mock = {
    importKey: vi.fn(),
    deriveKey: vi.fn(),
    encrypt: vi.fn(),
    decrypt: vi.fn()
};

const get_random_values_mock = vi.fn((arr: Uint8Array) => arr); // just return the zero'd array for deterministic testing

describe('crypto-core', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Setup global crypto
        vi.stubGlobal('crypto', {
            subtle: subtle_mock,
            getRandomValues: get_random_values_mock
        });
    });

    describe('derive_key', () => {
        it('uses PBKDF2 to derive an AES-GCM key', async () => {
            const mock_base_key = {};
            const mock_derived_key = {};
            
            subtle_mock.importKey.mockResolvedValue(mock_base_key);
            subtle_mock.deriveKey.mockResolvedValue(mock_derived_key);

            const result = await derive_key('password123', new Uint8Array([1, 2, 3]));

            expect(result).toBe(mock_derived_key);
            expect(subtle_mock.importKey).toHaveBeenCalledWith(
                'raw',
                expect.anything(), // password buffer
                'PBKDF2',
                false,
                ['deriveBits', 'deriveKey']
            );
            expect(subtle_mock.deriveKey).toHaveBeenCalledWith(
                expect.objectContaining({ name: 'PBKDF2', hash: 'SHA-256' }),
                mock_base_key,
                { name: 'AES-GCM', length: 256 },
                false,
                ['encrypt', 'decrypt']
            );
        });

        it('throws an error if crypto.subtle is undefined', async () => {
            vi.stubGlobal('crypto', {}); // Remove subtle
            
            await expect(derive_key('pass', new Uint8Array([1, 2, 3])))
                .rejects.toThrow('Neural Secure Context');
        });
    });

    describe('encrypt_raw', () => {
        it('encrypts returning json string of salt, iv, and data', async () => {
            const mock_key = {};
            subtle_mock.importKey.mockResolvedValue({});
            subtle_mock.deriveKey.mockResolvedValue(mock_key);
            
            const fake_encrypted_buffer = new Uint8Array([10, 20, 30]).buffer;
            subtle_mock.encrypt.mockResolvedValue(fake_encrypted_buffer);

            const json_str = await encrypt_raw('secret text', 'pass');
            const data = JSON.parse(json_str);

            expect(data).toHaveProperty('salt');
            expect(data).toHaveProperty('iv');
            expect(data).toHaveProperty('data', '0a141e'); // hex representations of 10,20,30
            expect(subtle_mock.encrypt).toHaveBeenCalledWith(
                expect.objectContaining({ name: 'AES-GCM' }),
                mock_key,
                expect.anything() // target text buffer
            );
        });

        it('throws an error if crypto.subtle is undefined', async () => {
            vi.stubGlobal('crypto', {}); // Remove subtle
            
            await expect(encrypt_raw('test', 'pass'))
                .rejects.toThrow('Neural Secure Context');
        });
    });

    describe('decrypt_raw', () => {
        it('decrypts returning original string', async () => {
             const mock_key = {};
             subtle_mock.importKey.mockResolvedValue({});
             subtle_mock.deriveKey.mockResolvedValue(mock_key);

             // Provide hex representations to decrypt_raw
             const payload = JSON.parse('{"salt": "0000", "iv": "000000", "data": "74657374"}'); // test
             
             // Decrypt expects unencoded buffer result which textDecoder reads
             const fake_decrypted_buffer = new TextEncoder().encode('hello world').buffer;
             subtle_mock.decrypt.mockResolvedValue(fake_decrypted_buffer);

             const result = await decrypt_raw(JSON.stringify(payload), 'my-pass');
             
             expect(result).toBe('hello world');
             expect(subtle_mock.decrypt).toHaveBeenCalledWith(
                 expect.objectContaining({ name: 'AES-GCM' }),
                 mock_key,
                 expect.any(Uint8Array)
             );
        });

        it('throws friendly error on decryption failure', async () => {
             subtle_mock.importKey.mockResolvedValue({});
             subtle_mock.deriveKey.mockResolvedValue({});
             subtle_mock.decrypt.mockRejectedValue(new Error('native throw'));

             const payload = JSON.parse('{"salt": "00", "iv": "00", "data": "00"}');
             
             await expect(decrypt_raw(JSON.stringify(payload), 'wrong-pass'))
                 .rejects.toThrow('Decryption failed');
        });
    });
});


