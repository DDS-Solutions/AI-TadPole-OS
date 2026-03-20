import { describe, it, expect, vi, beforeEach } from 'vitest';
import { deriveKey, encryptRaw, decryptRaw } from './crypto-core';

// Fully mock global browser crypto API
const subtleMock = {
    importKey: vi.fn(),
    deriveKey: vi.fn(),
    encrypt: vi.fn(),
    decrypt: vi.fn()
};

const getRandomValuesMock = vi.fn((arr: Uint8Array) => arr); // just return the zero'd array for deterministic testing

describe('crypto-core', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Setup global crypto
        vi.stubGlobal('crypto', {
            subtle: subtleMock,
            getRandomValues: getRandomValuesMock
        });
    });

    describe('deriveKey', () => {
        it('uses PBKDF2 to derive an AES-GCM key', async () => {
            const mockBaseKey = {};
            const mockDerivedKey = {};
            
            subtleMock.importKey.mockResolvedValue(mockBaseKey);
            subtleMock.deriveKey.mockResolvedValue(mockDerivedKey);

            const result = await deriveKey('password123', new Uint8Array([1, 2, 3]));

            expect(result).toBe(mockDerivedKey);
            expect(subtleMock.importKey).toHaveBeenCalledWith(
                'raw',
                expect.anything(), // password buffer
                'PBKDF2',
                false,
                ['deriveBits', 'deriveKey']
            );
            expect(subtleMock.deriveKey).toHaveBeenCalledWith(
                expect.objectContaining({ name: 'PBKDF2', hash: 'SHA-256' }),
                mockBaseKey,
                { name: 'AES-GCM', length: 256 },
                false,
                ['encrypt', 'decrypt']
            );
        });

        it('throws an error if crypto.subtle is undefined', async () => {
            vi.stubGlobal('crypto', {}); // Remove subtle
            
            await expect(deriveKey('pass', new Uint8Array([1, 2, 3])))
                .rejects.toThrow('Neural Secure Context');
        });
    });

    describe('encryptRaw', () => {
        it('encrypts returning json string of salt, iv, and data', async () => {
            const mockKey = {};
            subtleMock.importKey.mockResolvedValue({});
            subtleMock.deriveKey.mockResolvedValue(mockKey);
            
            const fakeEncryptedArrayBuffer = new Uint8Array([10, 20, 30]).buffer;
            subtleMock.encrypt.mockResolvedValue(fakeEncryptedArrayBuffer);

            const jsonStr = await encryptRaw('secret text', 'pass');
            const data = JSON.parse(jsonStr);

            expect(data).toHaveProperty('salt');
            expect(data).toHaveProperty('iv');
            expect(data).toHaveProperty('data', '0a141e'); // hex representations of 10,20,30
            expect(subtleMock.encrypt).toHaveBeenCalledWith(
                expect.objectContaining({ name: 'AES-GCM' }),
                mockKey,
                expect.anything() // target text buffer
            );
        });

        it('throws an error if crypto.subtle is undefined', async () => {
            vi.stubGlobal('crypto', {}); // Remove subtle
            
            await expect(encryptRaw('test', 'pass'))
                .rejects.toThrow('Neural Secure Context');
        });
    });

    describe('decryptRaw', () => {
        it('decrypts returning original string', async () => {
             const mockKey = {};
             subtleMock.importKey.mockResolvedValue({});
             subtleMock.deriveKey.mockResolvedValue(mockKey);

             // Provide hex representations to decryptRaw
             const payload = JSON.parse('{"salt": "0000", "iv": "000000", "data": "74657374"}'); // test
             
             // Decrypt expects unencoded buffer result which textDecoder reads
             const fakeDecryptedBuffer = new TextEncoder().encode('hello world').buffer;
             subtleMock.decrypt.mockResolvedValue(fakeDecryptedBuffer);

             const result = await decryptRaw(JSON.stringify(payload), 'my-pass');
             
             expect(result).toBe('hello world');
             expect(subtleMock.decrypt).toHaveBeenCalledWith(
                 expect.objectContaining({ name: 'AES-GCM' }),
                 mockKey,
                 expect.any(Uint8Array)
             );
        });

        it('throws friendly error on decryption failure', async () => {
             subtleMock.importKey.mockResolvedValue({});
             subtleMock.deriveKey.mockResolvedValue({});
             subtleMock.decrypt.mockRejectedValue(new Error('native throw'));

             const payload = JSON.parse('{"salt": "00", "iv": "00", "data": "00"}');
             
             await expect(decryptRaw(JSON.stringify(payload), 'wrong-pass'))
                 .rejects.toThrow('Decryption failed');
        });
    });
});
